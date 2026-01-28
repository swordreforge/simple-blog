use actix_web::{web, HttpResponse, Result};
use actix_files::Files;
use std::path::Path;

/// 配置静态文件路由
/// 单职责：仅负责静态文件服务的路由配置
pub fn configure_static_routes(cfg: &mut web::ServiceConfig) {
    // Favicon 处理（优先级最高，避免返回 404）
    cfg.route("/favicon.ico", web::get().to(handle_favicon));

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

/// 处理 favicon 请求
async fn handle_favicon() -> Result<HttpResponse> {
    // 检查是否存在 favicon 文件
    let favicon_paths = vec![
        "img/favicon.ico",
        "templates/favicon.ico",
        "favicon.ico",
    ];
    
    for path in favicon_paths {
        if Path::new(path).exists() {
            return Ok(HttpResponse::Ok()
                .content_type("image/x-icon")
                .body(std::fs::read(path)?));
        }
    }
    
    // 如果不存在，返回 204 No Content（避免浏览器重复请求）
    Ok(HttpResponse::NoContent().finish())
}