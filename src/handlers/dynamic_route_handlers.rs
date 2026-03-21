//! 动态路由处理器
//!
//! 负责处理动态路由的请求分发

use crate::app_state::AppState;
use actix_web::{HttpRequest, HttpResponse, Result, web};

/// 动态路由分发器
///
/// 作为静态路由的 fallback，当静态路由未匹配时，尝试在动态路由表中查找
pub async fn handle_dynamic_route(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let mut path_str = path.into_inner();

    // URL解码：浏览器会对特殊字符进行编码（如 @ -> %40）
    // 我们需要解码以匹配数据库中存储的原始路径
    match urlencoding::decode(&path_str) {
        Ok(decoded) => {
            path_str = decoded.into_owned();
        }
        Err(e) => {
            tracing::warn!("URL解码失败: {}, 使用原始路径", e);
        }
    }

    // 规范化路径：移除多余的前导斜杠
    while path_str.starts_with("//") {
        path_str = path_str.replacen("//", "/", 1);
    }

    // 确保路径以 / 开头，以匹配数据库中存储的路径格式
    if !path_str.starts_with('/') {
        path_str = format!("/{}", path_str);
    }

    // 排除API路径，确保API请求由API路由处理器处理
    if path_str.starts_with("/api/") {
        tracing::warn!("跳过API路径: {}", path_str);
        return Ok(crate::handlers::page_handlers::render_status_page(404).await);
    }

    tracing::info!("尝试匹配动态路由: {}", path_str);

    // 在路由表中查找路由
    if let Some(route_entry) = state.route_table.get_arc(&path_str) {
        tracing::info!("动态路由匹配成功: {}", path_str);

        // 使用路由条目的 handle 方法处理请求
        // 注意：我们已经在异步上下文中，可以直接调用异步函数
        let response = route_entry.handle(&req).await;
        return Ok(response);
    }

    // 路由未匹配，返回 404 页面
    tracing::warn!("动态路由未匹配: {}", path_str);
    Ok(crate::handlers::page_handlers::render_status_page(404).await)
}

/// 动态路由健康检查
pub async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "dynamic-route"
    })))
}
