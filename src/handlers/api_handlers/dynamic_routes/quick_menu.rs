use actix_web::{web, HttpResponse};
use serde::Serialize;
use crate::app_state::AppState;

/// 快捷菜单路由项
#[derive(Debug, Serialize)]
pub struct QuickMenuRoute {
    pub id: Option<i64>,
    pub name: String,
    pub path: String,
    pub icon: String,
    pub order: i32,
    pub group_id: Option<String>,
    pub group_name: Option<String>,
}

/// 获取快捷菜单路由列表
///
/// 只返回应该在快捷菜单中显示的路由：
/// - 如果路由有 group_id，只返回 is_primary_entry=true 的路由
/// - 如果路由没有 group_id，返回 metadata.show_in_quick_menu=true 的路由
pub async fn get_quick_menu_routes(
    state: web::Data<AppState>,
) -> HttpResponse {
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

    let mut quick_menu_routes = Vec::new();

    for route in routes.into_iter().filter(|r| r.enabled) {
        // 直接访问 group_id 和 is_primary_entry 字段
        let should_show = if let Some(ref _group_id) = route.group_id {
            // 如果有路由组，只显示主要入口
            route.is_primary_entry.unwrap_or(false)
        } else {
            // 如果没有路由组，使用原有的 metadata.show_in_quick_menu 字段
            route.metadata
                .as_ref()
                .and_then(|m| m.get("show_in_quick_menu"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
        };

        if should_show {
            // 从 metadata 中提取辅助信息
            let metadata = route.metadata.as_ref();
            let icon = metadata
                .and_then(|m| m.get("menu_icon"))
                .and_then(|v| v.as_str())
                .unwrap_or("link")
                .to_string();
            let order = metadata
                .and_then(|m| m.get("menu_order"))
                .and_then(|v| v.as_i64())
                .unwrap_or(100) as i32;
            let group_name = metadata
                .and_then(|m| m.get("group_name"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            quick_menu_routes.push(QuickMenuRoute {
                id: route.id,
                name: route.route_name.unwrap_or_else(|| route.path.clone()),
                path: route.path,
                icon,
                order,
                group_id: route.group_id,
                group_name,
            });
        }
    }

    // 按 order 排序
    quick_menu_routes.sort_by_key(|r| r.order);

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": {
            "routes": quick_menu_routes
        }
    }))
}