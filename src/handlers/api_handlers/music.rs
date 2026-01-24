use actix_web::{web, HttpResponse, Responder};

/// 获取音乐列表
pub async fn list() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "music": []
    }))
}

/// 播放音乐
pub async fn play(path: web::Path<String>) -> impl Responder {
    let _id = path.into_inner();
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Music playing"
    }))
}