use actix_web::{web, HttpResponse, Result, middleware};
use actix_files::Files;
use std::path::Path;

/// 配置静态文件路由
/// 单职责：仅负责静态文件服务的路由配置
pub fn configure_static_routes(cfg: &mut web::ServiceConfig) {
    // Favicon 处理（优先级最高，避免返回 404）
    cfg.route("/favicon.ico", web::get().to(handle_favicon));

    // CSS 文件 - 从内嵌文件系统提供，添加长期缓存
    cfg.route("/css/{file:.*}", web::get().to(serve_embedded_css));

    // JavaScript 文件 - 从内嵌文件系统提供，添加长期缓存
    cfg.route("/js/{file:.*}", web::get().to(serve_embedded_js));

    // 图片文件 - 添加长期缓存
    cfg.service(
        web::scope("/img")
            .wrap(middleware::DefaultHeaders::new().add(("Cache-Control", "public, max-age=31536000, immutable")))
            .service(Files::new("", "img")
                .show_files_listing()
                .use_etag(true)
                .use_last_modified(true)
                .prefer_utf8(true)
            )
    );

    // 音乐文件 - 添加长期缓存
    cfg.service(
        web::scope("/music")
            .wrap(middleware::DefaultHeaders::new().add(("Cache-Control", "public, max-age=31536000")))
            .service(Files::new("", "music")
                .show_files_listing()
                .use_etag(true)
                .use_last_modified(true)
                .prefer_utf8(true)
            )
    );

    // 附件文件 - 添加长期缓存
    cfg.service(
        web::scope("/attachments")
            .wrap(middleware::DefaultHeaders::new().add(("Cache-Control", "public, max-age=31536000")))
            .service(Files::new("", "attachments")
                .show_files_listing()
                .use_etag(true)
                .use_last_modified(true)
                .prefer_utf8(true)
            )
    );

    // Markdown 文件 - 添加中等缓存
    cfg.service(
        web::scope("/markdown")
            .wrap(middleware::DefaultHeaders::new().add(("Cache-Control", "public, max-age=86400")))
            .service(Files::new("", "markdown")
                .show_files_listing()
                .use_etag(true)
                .use_last_modified(true)
                .prefer_utf8(true)
            )
    );
}

/// 从内嵌文件系统提供 CSS 文件
async fn serve_embedded_css(path: web::Path<String>) -> Result<HttpResponse> {
    let filename = path.into_inner();
    let embed_path = format!("templates/css/{}", filename);
    
    // 优先尝试从内嵌文件系统获取
    if let Some(content) = crate::embedded::get_embedded_file(&embed_path) {
        return Ok(HttpResponse::Ok()
            .content_type("text/css; charset=utf-8")
            .insert_header(("Cache-Control", "public, max-age=31536000, immutable"))
            .body(content));
    }
    
    // 如果内嵌文件不存在，尝试从文件系统读取（向后兼容）
    let file_path = Path::new("templates/css").join(&filename);
    if file_path.exists() {
        return Ok(HttpResponse::Ok()
            .content_type("text/css; charset=utf-8")
            .insert_header(("Cache-Control", "public, max-age=31536000, immutable"))
            .body(std::fs::read(file_path)?));
    }
    
    Ok(HttpResponse::NotFound().finish())
}

/// 从内嵌文件系统提供 JavaScript 文件
async fn serve_embedded_js(path: web::Path<String>) -> Result<HttpResponse> {
    let filename = path.into_inner();
    let embed_path = format!("templates/js/{}", filename);
    
    // 优先尝试从内嵌文件系统获取
    if let Some(content) = crate::embedded::get_embedded_file(&embed_path) {
        return Ok(HttpResponse::Ok()
            .content_type("text/javascript; charset=utf-8")
            .insert_header(("Cache-Control", "public, max-age=31536000, immutable"))
            .body(content));
    }
    
    // 尝试从 templates/js 目录读取
    let file_path = Path::new("templates/js").join(&filename);
    if file_path.exists() {
        return Ok(HttpResponse::Ok()
            .content_type("text/javascript; charset=utf-8")
            .insert_header(("Cache-Control", "public, max-age=31536000, immutable"))
            .body(std::fs::read(file_path)?));
    }
    
    // 尝试从 static/js 目录读取
    let file_path = Path::new("static/js").join(&filename);
    if file_path.exists() {
        return Ok(HttpResponse::Ok()
            .content_type("text/javascript; charset=utf-8")
            .insert_header(("Cache-Control", "public, max-age=31536000, immutable"))
            .body(std::fs::read(file_path)?));
    }
    
    Ok(HttpResponse::NotFound().finish())
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