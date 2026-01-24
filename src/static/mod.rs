use actix_web::{HttpRequest, HttpResponse};
use actix_web::http::header;
use std::path::Path;

/// 静态文件服务
/// 单职责：提供静态文件的缓存控制和ETag支持
pub struct StaticFileService;

impl StaticFileService {
    /// 获取文件的 ETag
    pub fn get_etag(file_path: &Path) -> Option<String> {
        use std::fs;
        
        let metadata = fs::metadata(file_path).ok()?;
        let modified = metadata.modified().ok()?;
        let size = metadata.len();
        
        // 使用修改时间和文件大小生成 ETag
        let timestamp = modified
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?
            .as_secs();
        
        Some(format!("\"{}-{}\"", timestamp, size))
    }

    /// 检查客户端缓存是否有效
    pub fn check_cache(req: &HttpRequest, file_path: &Path) -> Option<HttpResponse> {
        if let Some(etag) = Self::get_etag(file_path) {
            // 检查 If-None-Match 头
            if let Some(if_none_match) = req.headers().get("if-none-match") {
                if let Ok(if_none_match_str) = if_none_match.to_str() {
                    if if_none_match_str == etag {
                        return Some(HttpResponse::NotModified()
                            .append_header(("ETag", etag))
                            .finish());
                    }
                }
            }
        }
        None
    }

    /// 添加缓存控制头
    pub fn add_cache_headers(mut response: HttpResponse, max_age: u32) -> HttpResponse {
        response.headers_mut().insert(
            header::CACHE_CONTROL,
            header::HeaderValue::from_str(&format!("public, max-age={}", max_age)).unwrap()
        );
        response.headers_mut().insert(
            header::VARY,
            header::HeaderValue::from_static("Accept-Encoding")
        );
        response
    }
}

/// 根据文件类型设置缓存时间
pub fn get_cache_max_age(file_path: &Path) -> u32 {
    let extension = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    
    match extension {
        // CSS/JS 文件：长期缓存（1年）
        "css" | "js" => 31536000,
        // 图片文件：长期缓存（1年）
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "ico" => 31536000,
        // 字体文件：长期缓存（1年）
        "woff" | "woff2" | "ttf" | "otf" => 31536000,
        // HTML 文件：短期缓存（5分钟）
        "html" | "htm" => 300,
        // 其他文件：中等缓存（1天）
        _ => 86400,
    }
}