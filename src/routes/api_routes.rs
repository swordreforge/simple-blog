use actix_web::{web, HttpResponse};
use crate::handlers::api_handlers;

/// 配置 API 路由
/// 单职责：仅负责 API 接口的路由配置
pub fn configure_api_routes(cfg: &mut web::ServiceConfig) {
    // 认证相关 API
    cfg.service(
        web::scope("/api/auth")
            .route("/login", web::post().to(api_handlers::auth::login))
            .route("/register", web::post().to(api_handlers::auth::register))
            .route("/logout", web::post().to(api_handlers::auth::logout))
            .route("/check", web::get().to(api_handlers::auth::check))
    );

    // 文章相关 API
    cfg.service(
        web::scope("/api/passage")
            .route("/list", web::get().to(api_handlers::passage::list))
            .route("/{id}", web::get().to(api_handlers::passage::get))
            .route("", web::post().to(api_handlers::passage::create))
            .route("/{id}", web::put().to(api_handlers::passage::update))
            .route("/{id}", web::delete().to(api_handlers::passage::delete))
    );

    // 设置相关 API
    cfg.service(
        web::scope("/api/settings")
            .route("", web::get().to(api_handlers::settings::get))
            .route("", web::post().to(api_handlers::settings::update))
    );

    // 音乐相关 API
    cfg.service(
        web::scope("/api/music")
            .route("/list", web::get().to(api_handlers::music::list))
            .route("/play/{id}", web::get().to(api_handlers::music::play))
    );

    // 附件相关 API
    cfg.service(
        web::scope("/api/attachments")
            .route("/upload", web::post().to(api_handlers::attachments::upload))
            .route("/list", web::get().to(api_handlers::attachments::list))
            .route("/{id}", web::delete().to(api_handlers::attachments::delete))
    );

    // 关于页面 API
    cfg.service(
        web::scope("/api/about")
            .route("", web::get().to(api_handlers::about::get))
            .route("", web::post().to(api_handlers::about::update))
    );
}