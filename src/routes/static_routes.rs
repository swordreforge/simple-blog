use actix_web::{web, HttpRequest, HttpResponse};
use actix_files::{Files, NamedFile};

/// 配置静态文件路由
/// 单职责：仅负责静态文件服务的路由配置
pub fn configure_static_routes(cfg: &mut web::ServiceConfig) {
    // CSS 文件
    cfg.service(
        Files::new("/css", "templates/css")
            .show_files_listing()
            .use_etag(true)
            .use_last_modified(true)
            .prefer_utf8(true)
            .index_file("index.html")
    );

    // JavaScript 文件
    cfg.service(
        Files::new("/js", "templates/js")
            .show_files_listing()
            .use_etag(true)
            .use_last_modified(true)
            .prefer_utf8(true)
            .index_file("index.html")
    );

    // 图片文件
    cfg.service(
        Files::new("/img", "img")
            .show_files_listing()
            .use_etag(true)
            .use_last_modified(true)
            .prefer_utf8(true)
    );

    // 音乐文件
    cfg.service(
        Files::new("/music", "music")
            .show_files_listing()
            .use_etag(true)
            .use_last_modified(true)
            .prefer_utf8(true)
    );

    // 附件文件
    cfg.service(
        Files::new("/attachments", "attachments")
            .show_files_listing()
            .use_etag(true)
            .use_last_modified(true)
            .prefer_utf8(true)
    );

    // Markdown 文件
    cfg.service(
        Files::new("/markdown", "markdown")
            .show_files_listing()
            .use_etag(true)
            .use_last_modified(true)
            .prefer_utf8(true)
    );
}