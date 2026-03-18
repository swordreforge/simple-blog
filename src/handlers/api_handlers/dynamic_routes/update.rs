use actix_web::{web, HttpResponse};
use crate::app_state::AppState;
use crate::middleware::auth::check_admin_auth;
use crate::db::models::{DynamicRoute, UpdateRouteRequest};
use crate::routes::conflicts_with_static_route;

/// 检查字符串是否包含控制字符
fn contains_control_chars(s: &str) -> bool {
    s.chars().any(|c| {
        let code = c as u32;
        // 控制字符范围：0-31, 127（DEL）
        code < 32 || code == 127
    })
}

/// 更新路由
pub async fn update_route(
    req: actix_web::HttpRequest,
    path: web::Path<i64>,
    route_data: web::Json<UpdateRouteRequest>,
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

    let id = path.into_inner();
    let repo = state.dynamic_route_repository();

    // 获取现有路由
    let old_route = match repo.get_by_id(id).await {
        Ok(Some(route)) => route,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "message": "路由不存在"
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": format!("查询失败: {}", e)
            }));
        }
    };

    let update_data = route_data.into_inner();

    // 验证控制字符
    if let Some(ref route_name) = update_data.route_name {
        if contains_control_chars(route_name) {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "message": "路由名称不能包含控制字符"
            }));
        }
    }

    if let Some(ref new_path) = update_data.path {
        if contains_control_chars(new_path) {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "message": "路由路径不能包含控制字符"
            }));
        }
    }

    if let Some(ref template_path) = update_data.template_path {
        if contains_control_chars(template_path) {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "message": "模板路径不能包含控制字符"
            }));
        }
    }

    if let Some(ref metadata) = update_data.metadata {
        if contains_control_chars(&metadata.to_string()) {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "message": "扩展元数据不能包含控制字符"
            }));
        }
    }

    // 检查路径冲突（如果路径被修改）
    if let Some(ref new_path) = update_data.path {
        if new_path != &old_route.path {
            // 检查是否与预定义静态路由冲突
            if conflicts_with_static_route(new_path) {
                return HttpResponse::Conflict().json(serde_json::json!({
                    "success": false,
                    "message": format!("路径 '{}' 与预定义静态路由冲突，请使用其他路径", new_path)
                }));
            }

            // 检查路径是否已存在于动态路由表中
            if let Ok(Some(_)) = repo.get_by_path(new_path).await {
                return HttpResponse::Conflict().json(serde_json::json!({
                    "success": false,
                    "message": "路径已存在"
                }));
            }
        }
    }

    // 构建更新后的路由 - 使用old_route的所有字段作为默认值
    let updated_route = DynamicRoute {
        id: old_route.id,
        route_name: update_data.route_name.or_else(|| old_route.route_name.clone()),
        route_type: update_data.route_type.unwrap_or(old_route.route_type),
        path: update_data.path.unwrap_or_else(|| old_route.path.clone()),
        handler_type: update_data.handler_type.unwrap_or(old_route.handler_type),
        handler_config: update_data.handler_config.unwrap_or_else(|| old_route.handler_config.clone()),
        inline_template: update_data.inline_template.or_else(|| old_route.inline_template.clone()),
        template_path: update_data.template_path.or_else(|| old_route.template_path.clone()),
        content_type_hint: update_data.content_type_hint.or_else(|| old_route.content_type_hint.clone()),
        enabled: update_data.enabled.unwrap_or(old_route.enabled),
        priority: update_data.priority.unwrap_or(old_route.priority),
        created_at: old_route.created_at,
        updated_at: chrono::Utc::now(),
        created_by: old_route.created_by.clone(),
        metadata: update_data.metadata.or_else(|| old_route.metadata.clone()),
    };

    // 更新路由
    match repo.update(id, &updated_route).await {
        Ok(_) => {
            // 记录操作日志
            log_route_operation(&repo, id, "update", Some(&old_route), &updated_route, &admin_info.1);

            // 热更新路由表
            if let Err(e) = state.dynamic_route_service().reload_route(id).await {
                tracing::warn!("路由热更新失败: id={}, error={}", id, e);
            }

            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "路由更新成功",
                "data": updated_route
            }))
        }
        Err(e) => {
            let error_msg = e.to_string();

            // 检查是否是 UNIQUE 约束错误（路径已存在）
            if error_msg.contains("UNIQUE constraint failed") && error_msg.contains("path") {
                tracing::warn!("路径冲突: path={}, error={}", updated_route.path, error_msg);
                return HttpResponse::Conflict().json(serde_json::json!({
                    "success": false,
                    "message": "路径已存在"
                }));
            }

            // 检查是否是数据库锁错误
            if error_msg.contains("database is locked") {
                tracing::error!("数据库锁: path={}, error={}", updated_route.path, error_msg);
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "success": false,
                    "message": "数据库繁忙，请稍后重试"
                }));
            }

            tracing::error!("更新路由失败: id={}, path={}, error={}", id, updated_route.path, error_msg);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": format!("更新失败: {}", e)
            }))
        }
    }
}

/// 部分更新路由
pub async fn patch_route(
    req: actix_web::HttpRequest,
    path: web::Path<i64>,
    route_data: web::Json<serde_json::Value>,
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

    let id = path.into_inner();
    let repo = state.dynamic_route_repository();

    // 获取现有路由
    let old_route = match repo.get_by_id(id).await {
        Ok(Some(route)) => route,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "message": "路由不存在"
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": format!("查询失败: {}", e)
            }));
        }
    };

    // 解析更新字段
    let update_data: UpdateRouteRequest = match serde_json::from_value(route_data.into_inner()) {
        Ok(data) => data,
        Err(e) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "message": format!("无效的请求数据: {}", e)
            }));
        }
    };

    // 验证控制字符
    if let Some(ref route_name) = update_data.route_name {
        if contains_control_chars(route_name) {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "message": "路由名称不能包含控制字符"
            }));
        }
    }

    if let Some(ref new_path) = update_data.path {
        if contains_control_chars(new_path) {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "message": "路由路径不能包含控制字符"
            }));
        }
    }

    if let Some(ref template_path) = update_data.template_path {
        if contains_control_chars(template_path) {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "message": "模板路径不能包含控制字符"
            }));
        }
    }

    if let Some(ref metadata) = update_data.metadata {
        if contains_control_chars(&metadata.to_string()) {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "message": "扩展元数据不能包含控制字符"
            }));
        }
    };

    // 检查路径冲突（如果路径被修改）
    if let Some(ref new_path) = update_data.path {
        if new_path != &old_route.path {
            if let Ok(Some(_)) = repo.get_by_path(new_path).await {
                return HttpResponse::Conflict().json(serde_json::json!({
                    "success": false,
                    "message": "路径已存在"
                }));
            }
        }
    }

    // 构建更新后的路由 - 使用old_route的所有字段作为默认值
    let updated_route = DynamicRoute {
        id: old_route.id,
        route_name: update_data.route_name.or_else(|| old_route.route_name.clone()),
        route_type: update_data.route_type.unwrap_or(old_route.route_type),
        path: update_data.path.unwrap_or_else(|| old_route.path.clone()),
        handler_type: update_data.handler_type.unwrap_or(old_route.handler_type),
        handler_config: update_data.handler_config.unwrap_or_else(|| old_route.handler_config.clone()),
        inline_template: update_data.inline_template.or_else(|| old_route.inline_template.clone()),
        template_path: update_data.template_path.or_else(|| old_route.template_path.clone()),
        content_type_hint: update_data.content_type_hint.or_else(|| old_route.content_type_hint.clone()),
        enabled: update_data.enabled.unwrap_or(old_route.enabled),
        priority: update_data.priority.unwrap_or(old_route.priority),
        created_at: old_route.created_at,
        updated_at: chrono::Utc::now(),
        created_by: old_route.created_by.clone(),
        metadata: update_data.metadata.or_else(|| old_route.metadata.clone()),
    };

    // 更新路由
    match repo.update(id, &updated_route).await {
        Ok(_) => {
            // 记录操作日志
            log_route_operation(&repo, id, "update", Some(&old_route), &updated_route, &admin_info.1);

            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "路由更新成功",
                "data": updated_route
            }))
        }
        Err(e) => {
            let error_msg = e.to_string();

            // 检查是否是 UNIQUE 约束错误（路径已存在）
            if error_msg.contains("UNIQUE constraint failed") && error_msg.contains("path") {
                tracing::warn!("路径冲突: path={}, error={}", updated_route.path, error_msg);
                return HttpResponse::Conflict().json(serde_json::json!({
                    "success": false,
                    "message": "路径已存在"
                }));
            }

            // 检查是否是数据库锁错误
            if error_msg.contains("database is locked") {
                tracing::error!("数据库锁: path={}, error={}", updated_route.path, error_msg);
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "success": false,
                    "message": "数据库繁忙，请稍后重试"
                }));
            }

            tracing::error!("更新路由失败: id={}, path={}, error={}", id, updated_route.path, error_msg);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": format!("更新失败: {}", e)
            }))
        }
    }
}

/// 启用路由
pub async fn enable_route(
    req: actix_web::HttpRequest,
    path: web::Path<i64>,
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

    let id = path.into_inner();
    let repo = state.dynamic_route_repository();

    // 获取现有路由
    let old_route = match repo.get_by_id(id).await {
        Ok(Some(route)) => route,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "message": "路由不存在"
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": format!("查询失败: {}", e)
            }));
        }
    };

    // 克隆old_route以避免移动问题
    let old_route_clone = old_route.clone();

    // 更新路由状态
    let updated_route = DynamicRoute {
        enabled: true,
        updated_at: chrono::Utc::now(),
        ..old_route
    };

    match repo.update(id, &updated_route).await {
        Ok(_) => {
            log_route_operation(&repo, id, "enable", Some(&old_route_clone), &updated_route, &admin_info.1);

            // 热更新路由表
            if let Err(e) = state.dynamic_route_service().reload_route(id).await {
                tracing::warn!("路由热更新失败: id={}, error={}", id, e);
            }

            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "路由已启用"
            }))
        }
        Err(e) => {
            let error_msg = e.to_string();

            // 检查是否是数据库锁错误
            if error_msg.contains("database is locked") {
                tracing::error!("数据库锁: id={}, error={}", id, error_msg);
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "success": false,
                    "message": "数据库繁忙，请稍后重试"
                }));
            }

            tracing::error!("启用路由失败: id={}, error={}", id, error_msg);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": format!("操作失败: {}", e)
            }))
        }
    }
}

/// 禁用路由
pub async fn disable_route(
    req: actix_web::HttpRequest,
    path: web::Path<i64>,
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

    let id = path.into_inner();
    let repo = state.dynamic_route_repository();

    // 获取现有路由
    let old_route = match repo.get_by_id(id).await {
        Ok(Some(route)) => route,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "message": "路由不存在"
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": format!("查询失败: {}", e)
            }));
        }
    };

    // 克隆old_route以避免移动问题
    let old_route_clone = old_route.clone();

    // 更新路由状态
    let updated_route = DynamicRoute {
        enabled: false,
        updated_at: chrono::Utc::now(),
        ..old_route
    };

    match repo.update(id, &updated_route).await {
        Ok(_) => {
            log_route_operation(&repo, id, "disable", Some(&old_route_clone), &updated_route, &admin_info.1);

            // 从路由表中移除路由
            state.dynamic_route_service().remove_route(&old_route_clone.path);

            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "路由已禁用"
            }))
        }
        Err(e) => {
            let error_msg = e.to_string();

            // 检查是否是数据库锁错误
            if error_msg.contains("database is locked") {
                tracing::error!("数据库锁: id={}, error={}", id, error_msg);
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "success": false,
                    "message": "数据库繁忙，请稍后重试"
                }));
            }

            tracing::error!("禁用路由失败: id={}, error={}", id, error_msg);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": format!("操作失败: {}", e)
            }))
        }
    }
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