use actix_web::{web, HttpResponse};
use crate::app_state::AppState;
use crate::middleware::auth::check_admin_auth;

/// 获取路由统计信息
pub async fn get_route_stats(
    req: actix_web::HttpRequest,
    path: web::Path<i64>,
    state: web::Data<AppState>,
) -> HttpResponse {
    // 权限检查
    if check_admin_auth(&req).is_none() {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "success": false,
            "message": "需要管理员权限"
        }));
    }

    let id = path.into_inner();
    let repo = state.dynamic_route_repository();

    match repo.get_stats(id).await {
        Ok(Some(stats)) => {
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "data": stats
            }))
        }
        Ok(None) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "message": "路由统计不存在"
            }))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": format!("查询失败: {}", e)
            }))
        }
    }
}