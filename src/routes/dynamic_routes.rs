//! 动态路由配置
//!
//! 配置动态路由的分发器和优先级

use actix_web::web;
use crate::handlers::dynamic_route_handlers;

/// 配置动态路由
///
/// 将动态路由作为最后的兜底路由，确保静态路由优先匹配
pub fn configure_dynamic_routes(cfg: &mut web::ServiceConfig) {
    // 健康检查端点 - 必须在通配符路由之前配置
    cfg.service(
        web::resource("/health/dynamic-route")
            .route(web::get().to(dynamic_route_handlers::health_check))
    );

    // 动态路由分发器 - 作为兜底路由
    // 捕获所有路径，在处理器中检查动态路由表
    cfg.service(
        web::resource("/{path:.*}")
            .route(web::get().to(dynamic_route_handlers::handle_dynamic_route))
            .route(web::post().to(dynamic_route_handlers::handle_dynamic_route))
            .route(web::put().to(dynamic_route_handlers::handle_dynamic_route))
            .route(web::delete().to(dynamic_route_handlers::handle_dynamic_route))
    );
}