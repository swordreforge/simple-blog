use actix_web::{web, HttpResponse};
use actix_multipart::Multipart;
use serde::Serialize;
use crate::db::repositories::{AttachmentRepository, Repository};
use crate::db::models::Attachment;
use std::sync::Arc;
use chrono::Utc;

/// 附件响应
#[derive(Debug, Serialize)]
pub struct AttachmentResponse {
    pub id: i64,
    pub file_name: String,
    pub stored_name: String,
    pub file_path: String,
    pub file_type: String,
    pub file_size: i64,
    pub passage_id: Option<String>,
    pub visibility: String,
    pub show_in_passage: bool,
    pub uploaded_at: String,
}

/// 上传响应
#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<AttachmentData>,
}

#[derive(Debug, Serialize)]
pub struct AttachmentData {
    pub id: i64,
    pub file_name: String,
    pub file_size: i64,
    pub file_type: String,
    pub url: String,
}

/// 获取附件列表
pub async fn list(
    repo: web::Data<Arc<dyn Repository>>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    let attachment_repo = AttachmentRepository::new(repo.get_pool().clone());
    
    // 解析分页参数
    let limit: i64 = query.get("limit").and_then(|l| l.parse().ok()).unwrap_or(20);
    let offset: i64 = query.get("offset").and_then(|o| o.parse().ok()).unwrap_or(0);
    
    match attachment_repo.get_all(limit, offset).await {
        Ok(attachments) => {
            let total = match attachment_repo.count().await {
                Ok(c) => c,
                Err(_) => attachments.len() as i64,
            };
            
            let data: Vec<AttachmentResponse> = attachments.into_iter()
                .map(|a| AttachmentResponse {
                    id: a.id.unwrap_or(0),
                    file_name: a.file_name,
                    stored_name: a.stored_name,
                    file_path: a.file_path,
                    file_type: a.file_type,
                    file_size: a.file_size,
                    passage_id: a.passage_uuid,
                    visibility: a.visibility,
                    show_in_passage: a.show_in_passage,
                    uploaded_at: a.uploaded_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                })
                .collect();
            
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "data": data,
                "total": total
            }))
        }
        Err(e) => {
            eprintln!("获取附件列表失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "获取附件列表失败"
            }))
        }
    }
}

/// 上传附件
pub async fn upload(
    repo: web::Data<Arc<dyn Repository>>,
    mut payload: Multipart,
) -> HttpResponse {
    use futures_util::stream::StreamExt;
    
    let attachment_repo = AttachmentRepository::new(repo.get_pool().clone());
    
    // 处理文件上传
    while let Some(field) = payload.next().await {
        let mut field = match field {
            Ok(f) => f,
            Err(e) => {
                eprintln!("获取字段失败: {}", e);
                continue;
            }
        };
        
        let content_disposition = field.content_disposition();
        let filename = content_disposition
            .and_then(|cd| cd.get_filename().map(|s| s.to_string()))
            .unwrap_or_else(|| "unknown".to_string());
        
        // 读取文件内容
        let mut file_bytes = Vec::new();
        while let Some(chunk) = field.next().await {
            let data = match chunk {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("读取文件块失败: {}", e);
                    continue;
                }
            };
            file_bytes.extend_from_slice(&data);
        }
        
        // 获取 content type
        let content_type_str = field.content_type().map(|ct| ct.to_string()).unwrap_or_else(|| "application/octet-stream".to_string());
        let file_type = determine_file_type(&filename, &content_type_str);
        
        // 生成存储文件名
        let timestamp = Utc::now().timestamp();
        let stored_name = format!("{}_{}", timestamp, filename);
        
        // 保存文件到磁盘
        let file_path = format!("attachments/{}", stored_name);
        if let Err(e) = std::fs::create_dir_all("attachments") {
            eprintln!("创建附件目录失败: {}", e);
            continue;
        }
        
        if let Err(e) = std::fs::write(&file_path, &file_bytes) {
            eprintln!("保存文件失败: {}", e);
            continue;
        }
        
        let file_size = file_bytes.len() as i64;
        
        // 创建附件记录
        let now = Utc::now();
        let attachment = Attachment {
            id: None,
            file_name: filename.to_string(),
            stored_name,
            file_path,
            file_type,
            content_type: content_type_str,
            file_size,
            passage_uuid: None,
            visibility: "public".to_string(),
            show_in_passage: false,
            uploaded_at: now,
        };
        
        match attachment_repo.create(&attachment).await {
            Ok(_) => {
                return HttpResponse::Ok().json(UploadResponse {
                    success: true,
                    message: "附件上传成功".to_string(),
                    data: Some(AttachmentData {
                        id: 0,
                        file_name: attachment.file_name,
                        file_size: attachment.file_size,
                        file_type: attachment.file_type,
                        url: format!("/{}", attachment.file_path),
                    }),
                });
            }
            Err(e) => {
                eprintln!("创建附件记录失败: {}", e);
                return HttpResponse::InternalServerError().json(UploadResponse {
                    success: false,
                    message: "附件上传失败".to_string(),
                    data: None,
                });
            }
        }
    }
    
    HttpResponse::BadRequest().json(UploadResponse {
        success: false,
        message: "没有上传文件".to_string(),
        data: None,
    })
}

/// 删除附件
pub async fn delete(
    repo: web::Data<Arc<dyn Repository>>,
    path: web::Path<i64>,
) -> HttpResponse {
    let id = path.into_inner();
    let attachment_repo = AttachmentRepository::new(repo.get_pool().clone());
    
    // 先获取附件信息
    let attachment = match attachment_repo.get_by_id(id).await {
        Ok(a) => a,
        Err(_) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "message": "附件不存在"
            }));
        }
    };
    
    // 删除文件
    if let Err(e) = std::fs::remove_file(&attachment.file_path) {
        eprintln!("删除文件失败: {}", e);
    }
    
    // 删除数据库记录
    match attachment_repo.delete(id).await {
        Ok(_) => {
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "附件删除成功"
            }))
        }
        Err(e) => {
            eprintln!("删除附件记录失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "删除附件失败"
            }))
        }
    }
}

/// 更新附件
pub async fn update(
    repo: web::Data<Arc<dyn Repository>>,
    path: web::Path<i64>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    let id = path.into_inner();
    let action = query.get("action").map(|s| s.as_str());
    
    let attachment_repo = AttachmentRepository::new(repo.get_pool().clone());
    
    // 先获取现有附件
    let mut attachment = match attachment_repo.get_by_id(id).await {
        Ok(a) => a,
        Err(_) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "message": "附件不存在"
            }));
        }
    };
    
    // 根据操作类型更新
    match action {
        Some("title") => {
            // 更新文件名
            if let Some(new_title) = query.get("title") {
                attachment.file_name = new_title.clone();
                attachment.stored_name = new_title.clone();
            }
        }
        Some("visibility") => {
            // 更新可见性
            if let Some(visibility) = query.get("visibility") {
                attachment.visibility = visibility.clone();
            }
        }
        _ => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "message": "无效的操作"
            }));
        }
    }
    
    attachment.uploaded_at = Utc::now();
    
    match attachment_repo.update(&attachment).await {
        Ok(_) => {
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "附件更新成功"
            }))
        }
        Err(e) => {
            eprintln!("更新附件失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "更新附件失败"
            }))
        }
    }
}

/// 确定文件类型
fn determine_file_type(filename: &str, content_type: &str) -> String {
    let ext = std::path::Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    
    match ext.to_lowercase().as_str() {
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "svg" => "image".to_string(),
        "mp4" | "avi" | "mov" | "wmv" | "flv" | "mkv" => "video".to_string(),
        "mp3" | "wav" | "ogg" | "flac" | "aac" => "audio".to_string(),
        "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "txt" => "document".to_string(),
        "zip" | "rar" | "7z" | "tar" | "gz" => "archive".to_string(),
        _ => {
            if content_type.starts_with("image/") {
                "image".to_string()
            } else if content_type.starts_with("video/") {
                "video".to_string()
            } else if content_type.starts_with("audio/") {
                "audio".to_string()
            } else {
                "unknown".to_string()
            }
        }
    }
}