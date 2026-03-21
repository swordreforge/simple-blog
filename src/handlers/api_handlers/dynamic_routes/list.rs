use crate::app_state::AppState;
use crate::middleware::auth::check_admin_auth;
use actix_web::{HttpResponse, web};
use serde::Deserialize;

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

    // 解析route_type参数
    let route_type = query
        .route_type
        .as_ref()
        .and_then(|s| crate::db::models::RouteType::from_str(s));

    // 使用 RouteTypeManager 获取所有存储类型的路由
    let routes = if let Some(manager) = state.route_type_manager() {
        match route_type {
            Some(rt) => {
                // 获取指定类型的路由
                match manager.list_routes_by_type(rt).await {
                    Ok(routes) => routes,
                    Err(e) => {
                        return HttpResponse::InternalServerError().json(serde_json::json!({
                            "success": false,
                            "message": format!("查询失败: {}", e)
                        }));
                    }
                }
            }
            None => {
                // 获取所有类型的路由
                match manager.list_all_routes().await {
                    Ok(routes) => routes,
                    Err(e) => {
                        return HttpResponse::InternalServerError().json(serde_json::json!({
                            "success": false,
                            "message": format!("查询失败: {}", e)
                        }));
                    }
                }
            }
        }
    } else {
        // 兼容性：如果没有 RouteTypeManager，只从数据库加载
        let repo = state.dynamic_route_repository();
        let offset = (page - 1) * limit;
        match repo.list(offset, 10000, route_type, query.enabled).await {
            Ok((routes, _)) => routes,
            Err(e) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "success": false,
                    "message": format!("查询失败: {}", e)
                }));
            }
        }
    };

    // 根据enabled参数过滤
    let filtered_routes = if let Some(enabled) = query.enabled {
        routes
            .into_iter()
            .filter(|r| r.enabled == enabled)
            .collect()
    } else {
        routes
    };

    // 计算总数
    let total = filtered_routes.len() as i64;

    // 分页
    let offset = ((page - 1) * limit) as usize;
    let paginated_routes: Vec<_> = filtered_routes
        .into_iter()
        .skip(offset)
        .take(limit as usize)
        .collect();

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": {
            "routes": paginated_routes,
            "total": total,
            "page": page,
            "limit": limit
        }
    }))
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

    // 使用 RouteTypeManager 获取路由
    if let Some(manager) = state.route_type_manager() {
        match manager.load_route(id, None).await {
            Ok(Some(route)) => HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "data": route
            })),
            Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "message": "路由不存在"
            })),
            Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": format!("查询失败: {}", e)
            })),
        }
    } else {
        // 兼容性：如果没有 RouteTypeManager，只从数据库加载
        let repo = state.dynamic_route_repository();
        match repo.get_by_id(id).await {
            Ok(Some(route)) => HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "data": route
            })),
            Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "message": "路由不存在"
            })),
            Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": format!("查询失败: {}", e)
            })),
        }
    }
}
