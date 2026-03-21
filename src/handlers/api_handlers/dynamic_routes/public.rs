use crate::app_state::AppState;
use crate::db::models::HandlerType;
use actix_web::{HttpResponse, web};
use serde::Deserialize;

/// 查询参数
#[derive(Debug, Deserialize)]
pub struct PublicListQuery {
    page: Option<i64>,
    limit: Option<i64>,
    handler_type: Option<String>,
    group_id: Option<String>,
}

/// 获取公开路由列表（无需管理员权限）
/// 只返回已启用的路由
pub async fn list_public_routes(
    query: web::Query<PublicListQuery>,
    state: web::Data<AppState>,
) -> HttpResponse {
    // 查询参数
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(100);

    // 默认只返回静态模板类型的路由
    let handler_type_filter = query.handler_type.as_deref().unwrap_or("static");

    // 解析handler_type参数为HandlerType枚举
    let handler_type = match handler_type_filter {
        "static" => Some(HandlerType::Static),
        "redirect" => Some(HandlerType::Redirect),
        "proxy" => Some(HandlerType::Proxy),
        "custom" => Some(HandlerType::Custom),
        _ => None,
    };

    // 使用 RouteTypeManager 获取所有存储类型的路由
    let routes = if let Some(manager) = state.route_type_manager() {
        match manager.list_all_routes().await {
            Ok(routes) => routes,
            Err(e) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "success": false,
                    "message": format!("查询失败: {}", e)
                }));
            }
        }
    } else {
        // 兼容性：如果没有 RouteTypeManager，只从数据库加载
        let repo = state.dynamic_route_repository();
        match repo.list(0, 10000, None, Some(true)).await {
            Ok((routes, _)) => routes,
            Err(e) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "success": false,
                    "message": format!("查询失败: {}", e)
                }));
            }
        }
    };

    // 过滤出已启用的路由
    let enabled_routes: Vec<_> = routes.into_iter().filter(|r| r.enabled).collect();

    // 根据handler_type参数过滤
    let filtered_routes = if let Some(ht) = handler_type {
        enabled_routes
            .into_iter()
            .filter(|r| r.handler_type == ht)
            .collect()
    } else {
        enabled_routes
    };

    // 根据group_id参数过滤（可选）
    let filtered_routes = if let Some(ref group_id) = query.group_id {
        filtered_routes
            .into_iter()
            .filter(|r| r.group_id.as_ref() == Some(group_id))
            .collect()
    } else {
        filtered_routes
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
