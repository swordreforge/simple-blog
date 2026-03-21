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
    let _admin_info = match check_admin_auth(&req) {
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
            // 如果路由不是数据库类型，也从数据库中删除记录（如果有的话）
            // 这样可以避免"幽灵路由"问题：删除内存/文件路由后，数据库中仍有记录
            if old_route.route_type != crate::db::models::RouteType::Database {
                let _ = repo.delete(id).await;
            }

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