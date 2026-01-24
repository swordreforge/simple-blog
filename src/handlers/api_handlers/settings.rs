use actix_web::{web, HttpResponse, Responder};

/// 获取设置
pub async fn get() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "settings": {
            "title": "RustBlog",
            "name": "Dango",
            "greeting": "Welcome to RustBlog",
            "background_image": "",
            "global_opacity": 0.9,
            "blur_amount": 20,
            "saturate_amount": 180
        }
    }))
}

/// 更新设置
pub async fn update() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Settings updated"
    }))
}