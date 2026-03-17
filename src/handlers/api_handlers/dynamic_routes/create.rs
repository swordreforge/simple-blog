use actix_web::{web, HttpResponse};
use crate::app_state::AppState;
use crate::middleware::auth::check_admin_auth;
use crate::db::models::{DynamicRoute, CreateRouteRequest, HandlerType};

/// 创建路由
pub async fn create_route(
    req: actix_web::HttpRequest,
    route_data: web::Json<CreateRouteRequest>,
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

    let route = route_data.into_inner();

    // 验证路由配置
    if let Err(e) = validate_route_config(&route) {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "message": format!("配置验证失败: {}", e)
        }));
    }

    // 检查路径冲突
    let repo = state.dynamic_route_repository();
    if let Ok(Some(_)) = repo.get_by_path(&route.path).await {
        return HttpResponse::Conflict().json(serde_json::json!({
            "success": false,
            "message": "路径已存在"
        }));
    }

    // 创建路由
    let dynamic_route = DynamicRoute {
        id: None,
        route_type: route.route_type,
        path: route.path.clone(),
        handler_type: route.handler_type,
        handler_config: route.handler_config.clone(),
        enabled: route.enabled.unwrap_or(true),
        priority: route.priority.unwrap_or(0),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        created_by: Some(admin_info.1.clone()),
        metadata: route.metadata.clone(),
    };

    match repo.create(&dynamic_route).await {
        Ok(id) => {
            // 记录操作日志
            log_route_operation(&repo, id, "create", None, &dynamic_route, &admin_info.1);

            HttpResponse::Created().json(serde_json::json!({
                "success": true,
                "message": "路由创建成功",
                "data": {
                    "id": id,
                    "path": dynamic_route.path
                }
            }))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": format!("创建失败: {}", e)
            }))
        }
    }
}

/// 验证路由配置
fn validate_route_config(route: &CreateRouteRequest) -> Result<(), String> {
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

    // 根据处理器类型验证必需字段
    match route.handler_type {
        HandlerType::Redirect => {
            if route.handler_config.get("target").is_none() {
                return Err("重定向处理器需要target字段".to_string());
            }
        }
        HandlerType::Static => {
            if route.handler_config.get("content").is_none() {
                return Err("静态内容处理器需要content字段".to_string());
            }
        }
        HandlerType::Template => {
            if route.handler_config.get("template_name").is_none() {
                return Err("模板处理器需要template_name字段".to_string());
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