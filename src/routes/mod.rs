use actix_web::web;

mod api_routes;
mod dynamic_routes;
mod page_routes;
mod static_routes;
mod static_routes_list;

pub use api_routes::configure_api_routes;
pub use dynamic_routes::configure_dynamic_routes;
pub use page_routes::configure_page_routes;
pub use static_routes::configure_static_routes;
pub use static_routes_list::conflicts_with_static_route;

/// 配置所有路由
///
/// 路由优先级（从高到低）:
/// 1. API 路由 - /api/*
/// 2. 页面路由 - /admin, /about, /friends 等
/// 3. 静态文件路由 - /static/*
/// 4. 动态路由 - 作为兜底路由
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.configure(configure_api_routes)
        .configure(configure_page_routes)
        .configure(configure_static_routes)
        .configure(configure_dynamic_routes);
}
