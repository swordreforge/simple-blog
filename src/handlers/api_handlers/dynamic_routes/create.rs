use actix_web::{web, HttpResponse};
use crate::app_state::AppState;
use crate::middleware::auth::check_admin_auth;
use crate::db::models::{DynamicRoute, CreateRouteRequest, HandlerType, RouteType};
use crate::routes::conflicts_with_static_route;
use actix_web::web::Bytes;

/// 创建路由
pub async fn create_route(
    req: actix_web::HttpRequest,
    route_data: Bytes,
    state: web::Data<AppState>,
) -> HttpResponse {
    // 权限检查
    let admin_info = match check_admin_auth(&req) {
        Some(info) => info,
        None => return HttpResponse::Forbidden().json(serde_json::json!({
            "success": false,
            "message": "需要管理员权限"
        })),
    };

    // 使用允许控制字符的serde_json配置来解析JSON
    let route: CreateRouteRequest = match serde_json::from_slice(&route_data) {
        Ok(data) => data,
        Err(e) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "message": format!("无效的请求数据: {}", e)
            }));
        }
    };

    // 验证路由配置
    if let Err(e) = validate_route_config(&route) {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "message": format!("配置验证失败: {}", e)
        }));
    }

    // 检查是否与预定义静态路由冲突
    if conflicts_with_static_route(&route.path) {
        return HttpResponse::Conflict().json(serde_json::json!({
            "success": false,
            "message": format!("路径 '{}' 与预定义静态路由冲突，请使用其他路径", route.path)
        }));
    }

    // 检查路径是否已存在于动态路由表中
    let repo = state.dynamic_route_repository();
    if let Ok(Some(_)) = repo.get_by_path(&route.path).await {
        return HttpResponse::Conflict().json(serde_json::json!({
            "success": false,
            "message": "路径已存在"
        }));
    }

    // 创建路由
    let route_type = route.route_type.unwrap_or(RouteType::Database);
    let dynamic_route = DynamicRoute {
        id: None,
        route_name: route.route_name,
        route_type,
        path: route.path.clone(),
        handler_type: route.handler_type,
        handler_config: route.handler_config.clone(),
        inline_template: route.inline_template,
        template_path: route.template_path,
        content_type_hint: route.content_type_hint,
        enabled: route.enabled.unwrap_or(true),
        priority: route.priority.unwrap_or(0),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        created_by: Some(admin_info.1.clone()),
        metadata: route.metadata.clone(),
    };

    // 根据路由类型选择存储后端
    let id = if let Some(manager) = state.route_type_manager() {
        // 先在数据库中创建记录（用于管理和日志记录）
        let db_id = match repo.create(&dynamic_route).await {
            Ok(id) => id,
            Err(e) => {
                let error_msg = e.to_string();

                // 检查是否是 UNIQUE 约束错误（路径已存在）
                if error_msg.contains("UNIQUE constraint failed") && error_msg.contains("path") {
                    tracing::warn!("路径冲突: path={}, error={}", dynamic_route.path, error_msg);
                    return HttpResponse::Conflict().json(serde_json::json!({
                        "success": false,
                        "message": "路径已存在"
                    }));
                }

                // 检查是否是数据库锁错误
                if error_msg.contains("database is locked") {
                    tracing::error!("数据库锁: path={}, error={}", dynamic_route.path, error_msg);
                    return HttpResponse::InternalServerError().json(serde_json::json!({
                        "success": false,
                        "message": "数据库繁忙，请稍后重试"
                    }));
                }

                tracing::error!("创建路由失败（数据库）: path={}, error={}", dynamic_route.path, error_msg);
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "success": false,
                    "message": format!("创建失败: {}", e)
                }));
            }
        };

        // 如果不是数据库类型，还需要在对应的存储后端中保存
        if route_type != crate::db::models::RouteType::Database {
            let storage = manager.get_storage(&route_type);
            match storage.save_route(&dynamic_route).await {
                Ok(_) => {
                    tracing::info!("路由创建成功: id={}, type={}, path={}", db_id, route_type, dynamic_route.path);
                }
                Err(e) => {
                    tracing::error!("创建路由失败（存储后端）: path={}, type={}, error={}", dynamic_route.path, route_type, e);
                    // 删除数据库记录
                    let _ = repo.delete(db_id).await;
                    return HttpResponse::InternalServerError().json(serde_json::json!({
                        "success": false,
                        "message": format!("创建失败: {}", e)
                    }));
                }
            }
        } else {
            tracing::info!("路由创建成功（数据库）: id={}, path={}", db_id, dynamic_route.path);
        }

        db_id
    } else {
        // 兼容性：如果没有 RouteTypeManager，只使用数据库
        match repo.create(&dynamic_route).await {
            Ok(id) => {
                tracing::info!("路由创建成功（数据库）: id={}, path={}", id, dynamic_route.path);
                id
            }
            Err(e) => {
                let error_msg = e.to_string();

                // 检查是否是 UNIQUE 约束错误（路径已存在）
                if error_msg.contains("UNIQUE constraint failed") && error_msg.contains("path") {
                    tracing::warn!("路径冲突: path={}, error={}", dynamic_route.path, error_msg);
                    return HttpResponse::Conflict().json(serde_json::json!({
                        "success": false,
                        "message": "路径已存在"
                    }));
                }

                // 检查是否是数据库锁错误
                if error_msg.contains("database is locked") {
                    tracing::error!("数据库锁: path={}, error={}", dynamic_route.path, error_msg);
                    return HttpResponse::InternalServerError().json(serde_json::json!({
                        "success": false,
                        "message": "数据库繁忙，请稍后重试"
                    }));
                }

                tracing::error!("创建路由失败: path={}, error={}", dynamic_route.path, error_msg);
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "success": false,
                    "message": format!("创建失败: {}", e)
                }));
            }
        }
    };

    // 记录操作日志
    log_route_operation(&repo, id, "create", None, &dynamic_route, &admin_info.1);

    // 如果路由启用，热更新到路由表
    if dynamic_route.enabled {
        if let Err(e) = state.dynamic_route_service().reload_route(id).await {
            tracing::warn!("路由热更新失败: id={}, error={}", id, e);
        }
    }

    HttpResponse::Created().json(serde_json::json!({
        "success": true,
        "message": "路由创建成功",
        "data": {
            "id": id,
            "path": dynamic_route.path,
            "route_type": route_type
        }
    }))
}

/// 检查字符串是否包含控制字符
fn contains_control_chars(s: &str) -> bool {
    s.chars().any(|c| {
        let code = c as u32;
        // 控制字符范围：0-31, 127（DEL）
        // 排除常见的空白字符：\t (9), \n (10), \r (13)
        code < 32 || code == 127
    })
}

/// 验证路由配置
fn validate_route_config(route: &CreateRouteRequest) -> Result<(), String> {
    // 验证控制字符
    if let Some(ref route_name) = route.route_name {
        if contains_control_chars(route_name) {
            return Err("路由名称不能包含控制字符".to_string());
        }
    }

    if contains_control_chars(&route.path) {
        return Err("路由路径不能包含控制字符".to_string());
    }

    if let Some(ref template_path) = route.template_path {
        if contains_control_chars(template_path) {
            return Err("模板路径不能包含控制字符".to_string());
        }
    }

    if let Some(ref metadata) = route.metadata {
        if contains_control_chars(&metadata.to_string()) {
            return Err("扩展元数据不能包含控制字符".to_string());
        }
    }

    // 验证路径格式
    if !route.path.starts_with('/') {
        return Err("路径必须以 / 开头".to_string());
    }

    // 验证路径不包含空格
    if route.path.contains(' ') {
        return Err("路径不能包含空格".to_string());
    }

    // 验证处理器配置
    if route.handler_config.is_null() || !route.handler_config.is_object() {
        return Err("处理器配置必须是有效的JSON对象".to_string());
    }

    // 根据 route_type 验证字段组合
    let route_type = route.route_type.unwrap_or(RouteType::Database);
    match route_type {
        RouteType::Database | RouteType::Memory => {
            // database/memory 类型禁止使用 template_path
            if route.template_path.is_some() {
                return Err("database/memory 类型路由不支持 template_path 字段".to_string());
            }
        }
        RouteType::File => {
            // file 类型禁止使用 inline_template
            if route.inline_template.is_some() {
                return Err("file 类型路由不支持 inline_template 字段".to_string());
            }
            // file 类型必须提供 template_path
            if route.template_path.is_none() {
                return Err("file 类型路由必须提供 template_path 字段".to_string());
            }
        }
    }

    // 根据处理器类型验证必需字段
    match route.handler_type {
        HandlerType::Redirect => {
            if route.handler_config.get("target").is_none() {
                return Err("重定向处理器需要target字段".to_string());
            }
        }
        HandlerType::Static => {
            // 静态内容处理器需要 inline_template 或 handler_config.content
            // 对于 file 类型的路由，template_path 也可以替代 inline_template
            if route.handler_config.get("content").is_none() {
                let has_inline = route.inline_template.is_some() && route.inline_template.as_ref().is_some_and(|s| !s.is_empty());
                let has_template_path = route.template_path.is_some() && route.template_path.as_ref().is_some_and(|s| !s.is_empty());
                
                if !has_inline && !has_template_path {
                    return Err("静态内容处理器需要 inline_template 字段、template_path 字段或 handler_config.content 字段".to_string());
                }
            }
        }
        HandlerType::Proxy => {
            if route.handler_config.get("target").is_none() {
                return Err("代理处理器需要target字段".to_string());
            }
        }
        HandlerType::Custom => {
            if route.handler_config.get("script").is_none() && route.handler_config.get("source").is_none() {
                return Err("自定义处理器需要script或source字段".to_string());
            }
        }
    }

    Ok(())
}

/// 记录路由操作日志
fn log_route_operation(
    repo: &crate::db::repositories::DynamicRouteRepository,
    route_id: i64,
    action: &str,
    old_config: Option<&DynamicRoute>,
    new_config: &DynamicRoute,
    username: &str,
) {
    use serde_json::to_string;

    let old_config_str = old_config.and_then(|c| to_string(c).ok());
    let new_config_str = to_string(new_config).ok();

    // 记录日志（忽略错误）
    let _ = repo.log_operation(
        Some(route_id),
        action,
        old_config_str,
        new_config_str,
        Some(username.to_string()),
        None,
        None,
    );
}