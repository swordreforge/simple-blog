use actix_web::web;
use crate::handlers::api_handlers;

/// 配置 API 路由
/// 单职责：仅负责 API 接口的路由配置
pub fn configure_api_routes(cfg: &mut web::ServiceConfig) {
    // 认证相关 API
    cfg.service(
        web::resource("/api/login")
            .route(web::post().to(api_handlers::auth::login))
    ).service(
        web::resource("/api/register")
            .route(web::post().to(api_handlers::auth::register))
    ).service(
        web::resource("/api/logout")
            .route(web::post().to(api_handlers::auth::logout))
    ).service(
        web::resource("/api/check")
            .route(web::get().to(api_handlers::auth::check))
    );

    // 设置相关 API
    cfg.service(
        web::resource("/api/settings")
            .route(web::get().to(api_handlers::settings::get))
            .route(web::post().to(api_handlers::settings::update))
    ).service(
        web::resource("/api/settings/appearance")
            .route(web::get().to(api_handlers::settings::get_appearance))
            .route(web::post().to(api_handlers::settings::update_appearance))
    ).service(
        web::resource("/api/settings/music")
            .route(web::get().to(api_handlers::settings::get_music))
            .route(web::put().to(api_handlers::settings::update_music))
            .route(web::patch().to(api_handlers::settings::update_music_partial))
    ).service(
        web::resource("/api/settings/template")
            .route(web::get().to(api_handlers::settings::get_template))
            .route(web::patch().to(api_handlers::settings::update_template))
    ).service(
        web::resource("/api/settings/all")
            .route(web::get().to(api_handlers::settings::get_all))
    );

    // 音乐相关 API
    cfg.service(
        web::resource("/api/music/list")
            .route(web::get().to(api_handlers::music::list))
    ).service(
        web::resource("/api/music/playlist")
            .route(web::get().to(api_handlers::music::playlist))
    ).service(
        web::resource("/api/music/upload")
            .route(web::post().to(api_handlers::music::upload))
    ).service(
        web::resource("/api/music/play/{id}")
            .route(web::get().to(api_handlers::music::play))
    ).service(
        web::resource("/api/music/{id}")
            .route(web::put().to(api_handlers::music::update))
            .route(web::delete().to(api_handlers::music::delete))
    );

    // 附件相关 API
    cfg.service(
        web::resource("/api/attachments/upload")
            .route(web::post().to(api_handlers::attachments::upload))
    ).service(
        web::resource("/api/attachments/list")
            .route(web::get().to(api_handlers::attachments::list))
    ).service(
        web::resource("/api/attachments/{id}")
            .route(web::delete().to(api_handlers::attachments::delete))
    );

    // 管理员 API - 附件
    cfg.service(
        web::resource("/api/admin/attachments")
            .route(web::get().to(api_handlers::attachments::list))
            .route(web::post().to(api_handlers::attachments::upload))
    ).service(
        web::resource("/api/admin/attachments/{id}")
            .route(web::get().to(api_handlers::attachments::list))
            .route(web::put().to(api_handlers::attachments::update))
            .route(web::delete().to(api_handlers::attachments::delete))
            .route(web::patch().to(api_handlers::attachments::update))
    );

    // 评论 API
    cfg.service(
        web::resource("/api/comments")
            .route(web::get().to(api_handlers::comment::list))
            .route(web::post().to(api_handlers::comment::create))
            .route(web::delete().to(api_handlers::comment::delete))
    );

    // 管理员 API - 评论
    cfg.service(
        web::resource("/api/admin/comments")
            .route(web::get().to(api_handlers::comment::list))
            .route(web::post().to(api_handlers::comment::create))
    ).service(
        web::resource("/api/admin/comments/{id}")
            .route(web::delete().to(api_handlers::comment::delete))
    );

    // 管理员 API - 统计
    cfg.service(
        web::resource("/api/admin/stats")
            .route(web::get().to(api_handlers::stats::get_stats))
    );

    // 管理员 API - 分析
    cfg.service(
        web::resource("/api/admin/analytics")
            .route(web::get().to(api_handlers::analytics::most_viewed))
    ).service(
        web::resource("/api/admin/analytics/most-viewed")
            .route(web::get().to(api_handlers::analytics::most_viewed))
    ).service(
        web::resource("/api/admin/analytics/view-sources")
            .route(web::get().to(api_handlers::analytics::view_sources))
    ).service(
        web::resource("/api/admin/analytics/view-trend")
            .route(web::get().to(api_handlers::analytics::view_trend))
    ).service(
        web::resource("/api/admin/analytics/article-stats")
            .route(web::get().to(api_handlers::analytics::article_stats))
    ).service(
        web::resource("/api/admin/analytics/view-by-city")
            .route(web::get().to(api_handlers::analytics::view_by_city))
    ).service(
        web::resource("/api/admin/analytics/view-by-ip")
            .route(web::get().to(api_handlers::analytics::view_by_ip))
    );

    // 关于页面 API
    cfg.service(
        web::resource("/api/about")
            .route(web::get().to(api_handlers::about::get))
            .route(web::post().to(api_handlers::about::update))
    ).service(
        web::resource("/api/about/main-cards/admin")
            .route(web::get().to(api_handlers::about::get_main_cards_admin))
    ).service(
        web::resource("/api/about/sub-cards/admin")
            .route(web::get().to(api_handlers::about::get_sub_cards_admin))
    ).service(
        web::resource("/api/about/main-cards")
            .route(web::post().to(api_handlers::about::create_main_card))
    ).service(
        web::resource("/api/about/main-cards/update")
            .route(web::put().to(api_handlers::about::update_main_card))
    ).service(
        web::resource("/api/about/main-cards/delete")
            .route(web::delete().to(api_handlers::about::delete_main_card))
    ).service(
        web::resource("/api/about/sub-cards")
            .route(web::post().to(api_handlers::about::create_sub_card))
    ).service(
        web::resource("/api/about/sub-cards/update")
            .route(web::put().to(api_handlers::about::update_sub_card))
    ).service(
        web::resource("/api/about/sub-cards/delete")
            .route(web::delete().to(api_handlers::about::delete_sub_card))
    );

    // 用户信息 API
    cfg.service(
        web::resource("/api/user/info")
            .route(web::get().to(api_handlers::user::info))
    );

    // 管理员 API - 用户
    cfg.service(
        web::resource("/api/admin/users")
            .route(web::get().to(api_handlers::user::admin_list))
            .route(web::post().to(api_handlers::user::create))
    ).service(
        web::resource("/api/admin/users/{id}")
            .route(web::get().to(api_handlers::user::get))
            .route(web::put().to(api_handlers::user::update))
            .route(web::delete().to(api_handlers::user::delete))
    );

    // ECC 加密 API
    cfg.service(
        web::resource("/api/crypto/public-key")
            .route(web::get().to(api_handlers::crypto::get_public_key))
    ).service(
        web::resource("/api/crypto/decrypt")
            .route(web::post().to(api_handlers::crypto::decrypt_data))
    );

    // Markdown 编辑器 API
    cfg.service(
        web::resource("/api/markdown-editor/save")
            .route(web::post().to(api_handlers::markdown_editor::save))
    );

    // 分析 API
    cfg.service(
        web::resource("/api/analytics/most-viewed")
            .route(web::get().to(api_handlers::analytics::most_viewed))
    ).service(
        web::resource("/api/analytics/view-sources")
            .route(web::get().to(api_handlers::analytics::view_sources))
    ).service(
        web::resource("/api/analytics/view-trend")
            .route(web::get().to(api_handlers::analytics::view_trend))
    ).service(
        web::resource("/api/analytics/article-stats")
            .route(web::get().to(api_handlers::analytics::article_stats))
    ).service(
        web::resource("/api/analytics/view-by-city")
            .route(web::get().to(api_handlers::analytics::view_by_city))
    ).service(
        web::resource("/api/analytics/view-by-ip")
            .route(web::get().to(api_handlers::analytics::view_by_ip))
    );

    // 文件管理 API
    cfg.service(
        web::resource("/api/files")
            .route(web::get().to(api_handlers::filemanager::list))
    ).service(
        web::resource("/api/files/download")
            .route(web::get().to(api_handlers::filemanager::download))
    ).service(
        web::resource("/api/files/create-dir")
            .route(web::post().to(api_handlers::filemanager::create_dir))
    );

    // 文章相关 API
    cfg.service(
        web::resource("/api/passage/list")
            .route(web::get().to(api_handlers::passage::list))
    ).service(
        web::resource("/api/passage/{id}")
            .route(web::get().to(api_handlers::passage::get))
            .route(web::put().to(api_handlers::passage::update))
            .route(web::delete().to(api_handlers::passage::delete))
    ).service(
        web::resource("/api/passage")
            .route(web::post().to(api_handlers::passage::create))
    );

    // 管理员 API - 文章
    cfg.service(
        web::resource("/api/admin/passages")
            .route(web::get().to(api_handlers::passage::admin_list))
            .route(web::post().to(api_handlers::passage::create))
    ).service(
        web::resource("/api/admin/passages/{id}")
            .route(web::get().to(api_handlers::passage::get))
            .route(web::put().to(api_handlers::passage::update))
            .route(web::delete().to(api_handlers::passage::delete))
    );

    // 兼容 Go 版本的路由
    cfg.service(
        web::resource("/api/passages")
            .route(web::get().to(api_handlers::passage::list))
            .route(web::post().to(api_handlers::passage::create))
    ).service(
        web::resource("/api/passages/{id}")
            .route(web::get().to(api_handlers::passage::get))
    ).service(
        web::resource("/api/tags")
            .route(web::get().to(api_handlers::tags::list))
    ).service(
        web::resource("/api/categories")
            .route(web::get().to(api_handlers::categories::list))
    ).service(
        web::resource("/api/archive")
            .route(web::get().to(api_handlers::archive::list))
    ).service(
        web::resource("/api/upload")
            .route(web::post().to(api_handlers::upload::upload))
    ).service(
        web::resource("/api/sync")
            .route(web::post().to(api_handlers::sync::sync))
    );

    // 管理员 API - 分类
    cfg.service(
        web::resource("/api/admin/categories")
            .route(web::get().to(api_handlers::categories::admin_list))
            .route(web::post().to(api_handlers::categories::create))
    ).service(
        web::resource("/api/admin/categories/{id}")
            .route(web::get().to(api_handlers::categories::get))
            .route(web::put().to(api_handlers::categories::update))
            .route(web::delete().to(api_handlers::categories::delete))
    );

    // 管理员 API - 标签
    cfg.service(
        web::resource("/api/admin/tags")
            .route(web::get().to(api_handlers::tags::admin_list))
            .route(web::post().to(api_handlers::tags::create))
    ).service(
        web::resource("/api/admin/tags/{id}")
            .route(web::get().to(api_handlers::tags::get))
            .route(web::put().to(api_handlers::tags::update))
            .route(web::delete().to(api_handlers::tags::delete))
    );
}