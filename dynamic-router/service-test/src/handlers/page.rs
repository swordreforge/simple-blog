use actix_web::Responder;

/// 首页响应
pub async fn index() -> impl Responder {
    match tokio::fs::read("./static/index.html").await {
        Ok(content) => actix_web::HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(content),
        Err(_) => actix_web::HttpResponse::NotFound().body("index.html not found"),
    }
}

/// 文件路由首页响应
pub async fn index_file() -> impl Responder {
    match tokio::fs::read("./static/index-file.html").await {
        Ok(content) => actix_web::HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(content),
        Err(_) => actix_web::HttpResponse::NotFound().body("index-file.html not found"),
    }
}