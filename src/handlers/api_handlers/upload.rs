use actix_web::{web, HttpResponse};
use actix_multipart::Multipart;
use serde::Serialize;
use std::path::Path;
use chrono::Utc;
use mime_guess::MimeGuess;

/// 上传响应
#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<UploadData>,
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct UploadData {
    pub file_name: String,
    pub file_path: String,
    pub file_size: i64,
    pub content_type: String,
}

/// 文件上传处理器（流式写入）
pub async fn upload(mut payload: Multipart, query: web::Query<std::collections::HashMap<String, String>>) -> HttpResponse {
    use futures_util::stream::StreamExt;
    
    // 获取日期参数（可选）
    let year = query.get("year").cloned().unwrap_or_else(|| Utc::now().format("%Y").to_string());
    let month = query.get("month").cloned().unwrap_or_else(|| Utc::now().format("%m").to_string());
    let day = query.get("day").cloned().unwrap_or_else(|| Utc::now().format("%d").to_string());
    
    // 支持的文件类型
    let supported_extensions: std::collections::HashSet<&str> = [
        ".md", ".jpg", ".jpeg", ".png", ".gif", ".webp", ".bmp", ".svg"
    ].iter().cloned().collect();
    
    // 遍历 multipart 表单
    while let Some(field_result) = payload.next().await {
        let mut field = match field_result {
            Ok(f) => f,
            Err(_) => continue,
        };
        
        // 只处理文件字段
        let content_disposition = field.content_disposition();
        let filename = match content_disposition {
            Some(cd) => match cd.get_filename() {
                Some(name) => name.to_string(),
                None => continue,
            },
            None => continue,
        };
        
        // 检查文件大小限制（10MB）
        let file_size = match get_field_size(&mut field).await {
            Ok(size) => size,
            Err(_) => {
                return HttpResponse::BadRequest().json(UploadResponse {
                    success: false,
                    message: "无法获取文件大小".to_string(),
                    data: None,
                    code: "FILE_SIZE_ERROR".to_string(),
                });
            }
        };
        
        if file_size > 10 * 1024 * 1024 {
            return HttpResponse::BadRequest().json(UploadResponse {
                success: false,
                message: format!("文件大小 {:.2}MB 超过 10MB 限制", file_size as f64 / (1024.0 * 1024.0)),
                data: None,
                code: "FILE_TOO_LARGE".to_string(),
            });
        }
        
        // 检查文件类型
        let ext = Path::new(&filename)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{}", e.to_lowercase()))
            .unwrap_or_default();
        
        if !supported_extensions.contains(&ext.as_str()) {
            return HttpResponse::BadRequest().json(UploadResponse {
                success: false,
                message: format!("不支持的文件类型: {}", ext),
                data: None,
                code: "UNSUPPORTED_FILE_TYPE".to_string(),
            });
        }
        
        // 获取内容类型
        let content_type = MimeGuess::from_path(&filename)
            .first_or_octet_stream()
            .to_string();
        
        // 构建文件路径（按日期组织）
        let date_dir = format!("{}/{}/{}", year, month, day);
        let base_dir = if ext == ".md" {
            "markdown"
        } else {
            "attachments"
        };
        
        let file_dir = format!("{}/{}", base_dir, date_dir);
        let file_path = format!("{}/{}", file_dir, filename);
        
        // 创建目录（异步）
        if let Err(e) = tokio::fs::create_dir_all(&file_dir).await {
            return HttpResponse::InternalServerError().json(UploadResponse {
                success: false,
                message: format!("创建目录失败: {}", e),
                data: None,
                code: "CREATE_DIR_FAILED".to_string(),
            });
        }
        
        // 使用 tokio 异步文件写入（流式写入，降低内存使用）
        let mut file = match tokio::fs::File::create(&file_path).await {
            Ok(f) => f,
            Err(e) => {
                return HttpResponse::InternalServerError().json(UploadResponse {
                    success: false,
                    message: format!("创建文件失败: {}", e),
                    data: None,
                    code: "CREATE_FILE_FAILED".to_string(),
                });
            }
        };
        
        // 使用 tokio io 异步写入
        use tokio::io::AsyncWriteExt;
        let mut bytes_written = 0u64;
        use futures_util::stream::StreamExt;
        while let Some(chunk) = field.next().await {
            let chunk = match chunk {
                Ok(c) => c,
                Err(_) => break,
            };
            if let Err(e) = file.write_all(&chunk).await {
                return HttpResponse::InternalServerError().json(UploadResponse {
                    success: false,
                    message: format!("写入文件失败: {}", e),
                    data: None,
                    code: "WRITE_FILE_FAILED".to_string(),
                });
            }
            bytes_written += chunk.len() as u64;
        }
        
        // 确保所有数据刷新到磁盘
        if let Err(e) = file.flush().await {
            return HttpResponse::InternalServerError().json(UploadResponse {
                success: false,
                message: format!("刷新文件失败: {}", e),
                data: None,
                code: "FLUSH_FILE_FAILED".to_string(),
            });
        }
        
        return HttpResponse::Ok().json(UploadResponse {
            success: true,
            message: "上传成功".to_string(),
            data: Some(UploadData {
                file_name: filename,
                file_path,
                file_size: bytes_written as i64,
                content_type,
            }),
            code: "UPLOAD_SUCCESS".to_string(),
        });
    }
    
    HttpResponse::BadRequest().json(UploadResponse {
        success: false,
        message: "未找到文件".to_string(),
        data: None,
        code: "NO_FILE_PROVIDED".to_string(),
    })
}

/// 获取字段大小
async fn get_field_size(field: &mut actix_multipart::Field) -> Result<i64, ()> {
    use futures_util::stream::StreamExt;
    let mut size = 0i64;
    while let Some(chunk) = field.next().await {
        let chunk = match chunk {
            Ok(c) => c,
            Err(_) => break,
        };
        size += chunk.len() as i64;
    }
    Ok(size)
}