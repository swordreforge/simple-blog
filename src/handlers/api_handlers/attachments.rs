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
    
    // 检查是否有 passage_id 参数
    let passage_id = query.get("passage_id");
    
    match attachment_repo.get_all(1000, 0).await {
        Ok(attachments) => {
            let filtered: Vec<Attachment> = if let Some(pid) = passage_id {
                // 按 passage_id 过滤
                attachments.into_iter()
                    .filter(|a| a.passage_uuid.as_ref().map_or(false, |uuid| uuid == pid))
                    .collect()
            } else {
                // 不分页，返回所有附件
                attachments
            };
            
            let total = filtered.len() as i64;
            
            // 应用分页
            let paginated: Vec<Attachment> = filtered.into_iter()
                .skip(offset as usize)
                .take(limit as usize)
                .collect();
            
            let data: Vec<AttachmentResponse> = paginated.into_iter()
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

/// 获取单个附件
pub async fn get(
    repo: web::Data<Arc<dyn Repository>>,
    path: web::Path<i64>,
) -> HttpResponse {
    let id = path.into_inner();
    let attachment_repo = AttachmentRepository::new(repo.get_pool().clone());
    
    match attachment_repo.get_by_id(id).await {
        Ok(attachment) => {
            let data = AttachmentResponse {
                id: attachment.id.unwrap_or(0),
                file_name: attachment.file_name,
                stored_name: attachment.stored_name,
                file_path: attachment.file_path,
                file_type: attachment.file_type,
                file_size: attachment.file_size,
                passage_id: attachment.passage_uuid,
                visibility: attachment.visibility,
                show_in_passage: attachment.show_in_passage,
                uploaded_at: attachment.uploaded_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            };
            
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "data": data
            }))
        }
        Err(e) => {
            eprintln!("获取附件失败: {}", e);
            HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "message": "附件不存在"
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
    body: Option<web::Json<serde_json::Value>>,
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
            // 如果没有 action 参数，尝试从 JSON 请求体获取
            if let Some(json_body) = body {
                if let Some(visibility) = json_body.get("visibility").and_then(|v| v.as_str()) {
                    attachment.visibility = visibility.to_string();
                }
                if let Some(show_in_passage) = json_body.get("show_in_passage").and_then(|v| v.as_bool()) {
                    attachment.show_in_passage = show_in_passage;
                }
            } else {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "success": false,
                    "message": "无效的操作"
                }));
            }
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

/// 按日期获取附件列表
pub async fn list_by_date(
    repo: web::Data<Arc<dyn Repository>>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    let attachment_repo = AttachmentRepository::new(repo.get_pool().clone());
    
    let year = query.get("year");
    let month = query.get("month");
    let day = query.get("day");
    
    match attachment_repo.get_all(1000, 0).await {
        Ok(attachments) => {
            let filtered: Vec<Attachment> = attachments.into_iter()
                .filter(|a| {
                    let uploaded_date = a.uploaded_at.format("%Y-%m-%d").to_string();
                    let date_str = if let (Some(y), Some(m), Some(d)) = (year, month, day) {
                        format!("{}-{}-{}", y, m, d)
                    } else {
                        uploaded_date.clone()
                    };
                    uploaded_date == date_str
                })
                .collect();
            
            let data: Vec<AttachmentResponse> = filtered.into_iter()
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
                "data": data
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

/// 下载附件
pub async fn download(
    repo: web::Data<Arc<dyn Repository>>,
    path: web::Path<i64>,
) -> HttpResponse {
    let id = path.into_inner();
    let attachment_repo = AttachmentRepository::new(repo.get_pool().clone());
    
    match attachment_repo.get_by_id(id).await {
        Ok(attachment) => {
            // 读取文件内容
            match std::fs::read(&attachment.file_path) {
                Ok(content) => {
                    HttpResponse::Ok()
                        .insert_header(("Content-Type", attachment.content_type.clone()))
                        .insert_header(("Content-Disposition", format!("attachment; filename=\"{}\"", attachment.file_name)))
                        .body(content)
                }
                Err(e) => {
                    eprintln!("读取文件失败: {}", e);
                    HttpResponse::NotFound().json(serde_json::json!({
                        "success": false,
                        "message": "文件不存在"
                    }))
                }
            }
        }
        Err(e) => {
            eprintln!("获取附件失败: {}", e);
            HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "message": "附件不存在"
            }))
        }
    }
}