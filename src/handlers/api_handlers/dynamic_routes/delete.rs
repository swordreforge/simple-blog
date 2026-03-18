use actix_web::{web, HttpResponse};
use crate::app_state::AppState;
use crate::middleware::auth::check_admin_auth;

/// 删除路由
pub async fn delete_route(
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

    // 获取现有路由（用于日志记录）- 使用 RouteTypeManager 从正确的存储后端加载
    let old_route = if let Some(manager) = state.route_type_manager() {
        match manager.load_route(id, None).await {
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
        }
    } else {
        // 兼容性：如果没有 RouteTypeManager，只从数据库加载
        match repo.get_by_id(id).await {
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
        }
    };

    // 删除路由 - 使用 RouteTypeManager 从正确的存储后端删除
    let delete_result = if let Some(manager) = state.route_type_manager() {
        let storage = manager.get_storage(&old_route.route_type);
        storage.delete_route(id).await
            .map_err(|e| format!("删除失败: {}", e))
    } else {
        // 兼容性：如果没有 RouteTypeManager，只使用数据库
        repo.delete(id).await
            .map_err(|e| e.to_string())
    };

    match delete_result {
        Ok(_) => {
            // 记录操作日志
            log_route_operation(&repo, id, "delete", Some(&old_route), &old_route, &admin_info.1);

            // 从路由表中移除路由
            state.dynamic_route_service().remove_route(&old_route.path);

            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "路由删除成功"
            }))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": e
            }))
        }
    }
}

/// 记录路由操作日志
fn log_route_operation(
    repo: &crate::db::repositories::DynamicRouteRepository,
    route_id: i64,
    action: &str,
    old_config: Option<&crate::db::models::DynamicRoute>,
    new_config: &crate::db::models::DynamicRoute,
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