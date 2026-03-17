//! 动态路由处理器
//!
//! 负责处理动态路由的请求分发

use actix_web::{web, HttpRequest, HttpResponse, Result};
use crate::app_state::AppState;

/// 动态路由分发器
///
/// 作为静态路由的 fallback，当静态路由未匹配时，尝试在动态路由表中查找
pub async fn handle_dynamic_route(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let path_str = path.into_inner();
    
    tracing::debug!("尝试匹配动态路由: {}", path_str);

    // 在路由表中查找路由
    if let Some(route_entry) = state.route_table.get_arc(&path_str) {
        tracing::info!("动态路由匹配成功: {}", path_str);
        
        // 使用路由条目的 handle 方法处理请求
        return Ok(route_entry.handle(&req).await);
    }
    
    // 路由未匹配，返回 404
    tracing::debug!("动态路由未匹配: {}", path_str);
    Ok(HttpResponse::NotFound().finish())
}

/// 动态路由健康检查
pub async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "dynamic-route"
    })))
}