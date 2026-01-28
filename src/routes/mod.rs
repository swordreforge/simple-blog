use actix_web::web;

mod page_routes;
mod static_routes;
mod api_routes;

pub use page_routes::configure_page_routes;
pub use static_routes::configure_static_routes;
pub use api_routes::configure_api_routes;

/// 配置所有路由
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.configure(configure_api_routes)
       .configure(configure_page_routes)
       .configure(configure_static_routes);
}