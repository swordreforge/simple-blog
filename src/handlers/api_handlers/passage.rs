use actix_web::{web, HttpResponse, Responder};

/// 获取文章列表
pub async fn list() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "passages": []
    }))
}

/// 获取单篇文章
pub async fn get(path: web::Path<String>) -> impl Responder {
    let _id = path.into_inner();
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "passage": null
    }))
}

/// 创建文章
pub async fn create() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Passage created"
    }))
}

/// 更新文章
pub async fn update(path: web::Path<String>) -> impl Responder {
    let _id = path.into_inner();
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Passage updated"
    }))
}

/// 删除文章
pub async fn delete(path: web::Path<String>) -> impl Responder {
    let _id = path.into_inner();
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Passage deleted"
    }))
}