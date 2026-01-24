use actix_web::{web, HttpResponse, Responder};

/// 上传附件
pub async fn upload() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Attachment uploaded"
    }))
}

/// 获取附件列表
pub async fn list() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "attachments": []
    }))
}

/// 删除附件
pub async fn delete(path: web::Path<String>) -> impl Responder {
    let _id = path.into_inner();
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Attachment deleted"
    }))
}