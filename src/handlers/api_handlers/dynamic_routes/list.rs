use actix_web::{web, HttpResponse};
use serde::Deserialize;
use crate::app_state::AppState;
use crate::middleware::auth::check_admin_auth;

/// 查询参数
#[derive(Debug, Deserialize)]
pub struct ListQuery {
    page: Option<i64>,
    limit: Option<i64>,
    route_type: Option<String>,
    enabled: Option<bool>,
}

/// 获取路由列表
pub async fn list_routes(
    req: actix_web::HttpRequest,
    query: web::Query<ListQuery>,
    state: web::Data<AppState>,
) -> HttpResponse {
    // 权限检查
    if check_admin_auth(&req).is_none() {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "success": false,
            "message": "需要管理员权限"
        }));
    }

    // 查询参数
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);
    let offset = (page - 1) * limit;

    // 解析route_type参数
    let route_type = query.route_type.as_ref()
        .and_then(|s| crate::db::models::RouteType::from_str(s));

    // 从Repository获取数据
    let repo = state.dynamic_route_repository();
    match repo.list(offset, limit, route_type, query.enabled).await {
        Ok((routes, total)) => {
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "data": {
                    "routes": routes,
                    "total": total,
                    "page": page,
                    "limit": limit
                }
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

/// 获取路由详情
pub async fn get_route(
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

    match repo.get_by_id(id).await {
        Ok(Some(route)) => {
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "data": route
            }))
        }
        Ok(None) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "message": "路由不存在"
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