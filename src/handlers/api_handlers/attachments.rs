use actix_web::{web, HttpResponse};
use actix_multipart::Multipart;
use serde::Serialize;
use crate::db::models::Attachment;
use chrono::Utc;
use tokio::fs;
use futures_util::future;

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
    pub visibility: crate::db::models::PassageVisibility,
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
    state: web::Data<crate::app_state::AppState>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    let attachment_repo = state.attachment_repository();
    
    // 解析并验证分页参数
    let limit: i64 = query.get("limit")
        .and_then(|l| l.parse::<i64>().ok())
        .filter(|&l| l > 0 && l <= 1000)
        .unwrap_or(20);
    
    let offset: i64 = query.get("offset")
        .and_then(|o| o.parse::<i64>().ok())
        .filter(|&o| o >= 0)
        .unwrap_or(0);
    
    // 检查是否有 passage_id 参数
    let passage_id = query.get("passage_id");
    
    match attachment_repo.get_all(1000, 0).await {
        Ok(attachments) => {
            let filtered: Vec<Attachment> = if let Some(pid) = passage_id {
                // 按 passage_id 过滤
                attachments.into_iter()
                    .filter(|a| a.passage_uuid.as_ref() == Some(pid))
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
    state: web::Data<crate::app_state::AppState>,
    path: web::Path<i64>,
) -> HttpResponse {
    let id = path.into_inner();
    let attachment_repo = state.attachment_repository();
    
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
    state: web::Data<crate::app_state::AppState>,
    mut payload: Multipart,
) -> HttpResponse {
    use futures_util::stream::StreamExt;

    let attachment_repo = state.attachment_repository();
    
    // 先收集所有字段，获取 passage_id
    let mut passage_uuid: Option<String> = None;
    let mut _file_field_name: Option<String> = None;
    let mut file_data: Option<(Vec<u8>, String, String)> = None;

    // 遍历所有字段
    while let Some(field_result) = payload.next().await {
        let mut field = match field_result {
            Ok(f) => f,
            Err(e) => {
                eprintln!("获取字段失败: {}", e);
                continue;
            }
        };

        let name = field.name();

        // 检查是否是 passage_id 字段（普通文本字段）
        if name == Some("passage_id") {
            let mut _passage_id_str = String::new();
            let mut field_content = Vec::new();
            while let Some(chunk) = field.next().await {
                if let Ok(data) = chunk {
                    field_content.extend_from_slice(&data);
                }
            }
            let passage_id_str = String::from_utf8_lossy(&field_content).to_string();

            // 根据 passage_id 查找 passage_uuid
            if !passage_id_str.is_empty() {
                let passage_repo = state.passage_repository();

                if let Ok(id) = passage_id_str.parse::<i64>() {
                    if let Ok(passage) = passage_repo.get_by_id(id).await {
                        passage_uuid = passage.uuid;
                    }
                }
            }
            continue;
        }

        // 处理文件字段
        if let Some(filename) = field.content_disposition().and_then(|cd| cd.get_filename().map(|s| s.to_string())) {
            _file_field_name = Some(filename.clone());
            
            // 读取文件内容
            let mut file_bytes = Vec::new();
            while let Some(chunk) = field.next().await {
                if let Ok(data) = chunk {
                    file_bytes.extend_from_slice(&data);
                }
            }
            
            // 获取 content type
            let content_type = field.content_type().map(|ct| ct.to_string())
                .unwrap_or_else(|| "application/octet-stream".to_string());
            
            file_data = Some((file_bytes, filename, content_type));
        }
    }
    
    // 如果没有文件数据，返回错误
    let (file_bytes, filename, content_type) = match file_data {
        Some(data) => data,
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "message": "没有上传文件"
            }));
        }
    };
    
    // 确定文件类型
    let file_type = determine_file_type(&filename, &content_type);
    
    // 生成存储文件名
    let timestamp = Utc::now().timestamp();
    let stored_name = format!("{}_{}", timestamp, filename);
    
    // 保存文件到磁盘
    let file_path = format!("attachments/{}", stored_name);
    if let Err(e) = fs::create_dir_all("attachments").await {
        eprintln!("创建附件目录失败: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "message": "创建附件目录失败"
        }));
    }

    if let Err(e) = fs::write(&file_path, &file_bytes).await {
        eprintln!("保存文件失败: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "message": "保存文件失败"
        }));
    }
    
    let file_size = file_bytes.len() as i64;
    
    // 创建附件记录
    let now = Utc::now();
    let attachment = Attachment {
        id: None,
        file_name: filename.clone(),
        stored_name,
        file_path,
        file_type,
        content_type,
        file_size,
        passage_uuid,
        visibility: crate::db::models::PassageVisibility::Public,
        show_in_passage: false,
        uploaded_at: now,
    };
    
    match attachment_repo.create(&attachment).await {
        Ok(_) => {
            HttpResponse::Ok().json(UploadResponse {
                success: true,
                message: "附件上传成功".to_string(),
                data: Some(AttachmentData {
                    id: 0,
                    file_name: attachment.file_name,
                    file_size: attachment.file_size,
                    file_type: attachment.file_type,
                    url: format!("/{}", attachment.file_path),
                }),
            })
        }
        Err(e) => {
            eprintln!("创建附件记录失败: {}", e);
            HttpResponse::InternalServerError().json(UploadResponse {
                success: false,
                message: "附件上传失败".to_string(),
                data: None,
            })
        }
    }
}

/// 删除附件
pub async fn delete(
    state: web::Data<crate::app_state::AppState>,
    path: web::Path<i64>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    // 鉴权检查
    if req.cookie("auth_token").is_none() {
        return crate::middleware::auth::missing_token_response();
    }
    if crate::middleware::auth::check_admin_auth(&req).is_none() {
        return crate::middleware::auth::forbidden_response();
    }

    let id = path.into_inner();
    let attachment_repo = state.attachment_repository();
    
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
    if let Err(e) = fs::remove_file(&attachment.file_path).await {
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

/// 批量删除附件
pub async fn delete_batch(
    state: web::Data<crate::app_state::AppState>,
    body: web::Json<serde_json::Value>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    // 鉴权检查
    if req.cookie("auth_token").is_none() {
        return crate::middleware::auth::missing_token_response();
    }
    if crate::middleware::auth::check_admin_auth(&req).is_none() {
        return crate::middleware::auth::forbidden_response();
    }

    // 解析 IDs
    let ids: Vec<i64> = match body.get("ids") {
        Some(ids) => match ids.as_array() {
            Some(arr) => arr.iter().filter_map(|v| v.as_i64()).collect(),
            None => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "success": false,
                    "message": "ids 必须是数组"
                }));
            }
        },
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "message": "缺少 ids 参数"
            }));
        }
    };

    if ids.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "message": "ids 不能为空"
        }));
    }

    let attachment_repo = state.attachment_repository();

    // 先获取附件信息（用于删除文件）
    let attachments = match attachment_repo.get_by_ids(ids.clone()).await {
        Ok(a) => a,
        Err(e) => {
            eprintln!("获取附件信息失败: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "获取附件信息失败"
            }));
        }
    };

    // 删除文件（并行删除以优化性能）
    let delete_tasks: Vec<_> = attachments.iter()
        .map(|attachment| {
            let path = attachment.file_path.clone();
            tokio::spawn(async move {
                let result = fs::remove_file(&path).await;
                (path, result)
            })
        })
        .collect();

    // 等待所有删除任务完成
    let results = future::join_all(delete_tasks).await;
    let mut files_deleted = 0;
    let mut files_failed = 0;
    for result in results {
        match result {
            Ok((_path, Ok(()))) => files_deleted += 1,
            Ok((path, Err(e))) => {
                eprintln!("删除文件失败 {}: {}", path, e);
                files_failed += 1;
            }
            Err(_) => files_failed += 1,
        }
    }

    // 删除数据库记录
    match attachment_repo.delete_batch(ids).await {
        Ok(rows_affected) => {
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": format!("成功删除 {} 个附件", rows_affected),
                "data": {
                    "deleted": rows_affected,
                    "files_deleted": files_deleted,
                    "files_failed": files_failed
                }
            }))
        }
        Err(e) => {
            eprintln!("批量删除附件记录失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "批量删除附件失败"
            }))
        }
    }
}

/// 更新附件
pub async fn update(
    state: web::Data<crate::app_state::AppState>,
    path: web::Path<i64>,
    query: web::Query<std::collections::HashMap<String, String>>,
    body: Option<web::Json<serde_json::Value>>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    // 鉴权检查
    if req.cookie("auth_token").is_none() {
        return crate::middleware::auth::missing_token_response();
    }
    if crate::middleware::auth::check_admin_auth(&req).is_none() {
        return crate::middleware::auth::forbidden_response();
    }

    let id = path.into_inner();
    let action = query.get("action").map(|s| s.as_str());

    let attachment_repo = state.attachment_repository();
    
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
                attachment.visibility = crate::db::models::PassageVisibility::from_str(visibility)
                    .unwrap_or(attachment.visibility);
            }
        }
        _ => {
            // 如果没有 action 参数，尝试从 JSON 请求体获取
            if let Some(json_body) = body {
                if let Some(visibility) = json_body.get("visibility").and_then(|v| v.as_str()) {
                    attachment.visibility = crate::db::models::PassageVisibility::from_str(visibility)
                        .unwrap_or(attachment.visibility);
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
    state: web::Data<crate::app_state::AppState>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    let attachment_repo = state.attachment_repository();
    
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
    state: web::Data<crate::app_state::AppState>,
    path: web::Path<i64>,
) -> HttpResponse {
    let id = path.into_inner();
    let attachment_repo = state.attachment_repository();
    
    match attachment_repo.get_by_id(id).await {
        Ok(attachment) => {
            // 读取文件内容
            match fs::read(&attachment.file_path).await {
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