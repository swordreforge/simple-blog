use actix_web::{web, HttpResponse, Responder};

/// 获取关于页面内容
pub async fn get() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "content": "About RustBlog"
    }))
}

/// 更新关于页面内容
pub async fn update() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "About page updated"
    }))
}