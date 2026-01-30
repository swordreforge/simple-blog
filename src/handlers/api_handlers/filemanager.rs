use actix_web::{web, HttpResponse, HttpRequest, Responder};
use actix_files::NamedFile;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;

/// 文件信息
#[derive(Debug, Serialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: Option<i64>,
    pub modified: Option<String>,
}

/// 文件列表响应
#[derive(Debug, Serialize)]
pub struct FileListResponse {
    pub success: bool,
    pub data: Vec<FileInfo>,
    pub current_path: String,
}

/// 创建目录请求
#[derive(Debug, Deserialize)]
pub struct CreateDirRequest {
    pub path: String,
    pub name: String,
}

/// 通用响应
#[derive(Debug, Serialize)]
pub struct CommonResponse {
    pub success: bool,
    pub message: String,
}

/// 获取文件列表
pub async fn list(query: web::Query<std::collections::HashMap<String, String>>) -> HttpResponse {
    let path_str = query.get("path")
        .cloned()
        .unwrap_or_else(|| ".".to_string());
    
    // 验证路径安全性
    let safe_path = match validate_path(&path_str) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("路径验证失败: {} - 原始路径: {}", e, path_str);
            return HttpResponse::BadRequest().json(FileListResponse {
                success: false,
                data: vec![],
                current_path: path_str,
            });
        }
    };
    
    let path = Path::new(&safe_path);
    
    if !path.exists() || !path.is_dir() {
        eprintln!("路径不存在或不是目录: {}", safe_path);
        return HttpResponse::BadRequest().json(FileListResponse {
            success: false,
            data: vec![],
            current_path: safe_path,
        });
    }
    
    // 读取目录内容
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("读取目录失败: {}", e);
            return HttpResponse::InternalServerError().json(FileListResponse {
                success: false,
                data: vec![],
                current_path: safe_path,
            });
        }
    };
    
    let mut files: Vec<FileInfo> = Vec::new();
    
    for entry in entries {
        if let Ok(entry) = entry {
            let entry_path = entry.path();
            let metadata = entry_path.metadata().ok();
            
            let file_info = FileInfo {
                name: entry.file_name().to_string_lossy().to_string(),
                path: entry_path.to_string_lossy().to_string(),
                is_dir: metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false),
                size: metadata.as_ref().map(|m| m.len() as i64),
                modified: metadata.as_ref()
                    .and_then(|m| m.modified().ok())
                    .map(|t| {
                        let datetime: chrono::DateTime<chrono::Utc> = t.into();
                        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
                    }),
            };
            
            files.push(file_info);
        }
    }
    
    // 排序：目录在前，文件在后
    files.sort_by(|a, b| {
        if a.is_dir && !b.is_dir {
            return std::cmp::Ordering::Less;
        }
        if !a.is_dir && b.is_dir {
            return std::cmp::Ordering::Greater;
        }
        a.name.cmp(&b.name)
    });
    
    // 获取父目录路径
    let parent_path = get_parent_path(&safe_path);
    
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": {
            "files": files,
            "current_path": safe_path,
            "parent_path": parent_path,
        }
    }))
}

/// 验证路径安全，防止路径穿越
fn validate_path(user_path: &str) -> Result<String, String> {
    // 获取工作目录的绝对路径
    let cwd = std::env::current_dir()
        .map_err(|_| "无法获取当前目录".to_string())?;
    
    // 规范化用户路径（去除 . 和 ..）
    let normalized_path = Path::new(user_path);
    let normalized_path: PathBuf = normalized_path
        .components()
        .filter(|comp| !matches!(comp, std::path::Component::ParentDir | std::path::Component::CurDir))
        .collect();
    
    // 构建完整路径
    let full_path = cwd.join(&normalized_path);
    
    // 检查路径是否在工作目录或允许的子目录中
    // 允许的根目录: ./img, ./markdown, ./attachments 和 ./music
    let allowed_dirs = vec![
        cwd.join("img"),
        cwd.join("markdown"),
        cwd.join("attachments"),
        cwd.join("music"),
    ];
    
    let is_allowed = allowed_dirs.iter().any(|allowed_dir| {
        full_path.starts_with(allowed_dir) || full_path == *allowed_dir
    });
    
    if !is_allowed {
        return Err(format!("访问被拒绝：路径超出允许范围 ({})", user_path));
    }
    
    // 始终返回相对路径（使用正则化后的路径）
    // 对于 markdown 目录，确保返回正确的相对路径
    let relative_path = normalized_path.to_string_lossy().to_string();
    
    // 处理空路径的情况
    if relative_path.is_empty() || relative_path == "." {
        return Ok(".".to_string());
    }
    
    Ok(relative_path)
}

/// 获取父目录路径
fn get_parent_path(path: &str) -> String {
    if path == "." || path == "/" {
        return String::new();
    }
    
    let parent = Path::new(path).parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    
    if parent.is_empty() {
        String::new()
    } else {
        parent
    }
}

/// 下载文件
pub async fn download(query: web::Query<std::collections::HashMap<String, String>>, req: HttpRequest) -> impl Responder {
    let path_str = query.get("path")
        .cloned()
        .unwrap_or_default();
    
    // 验证路径安全性
    let safe_path = match validate_path(&path_str) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("下载文件路径验证失败: {} - 原始路径: {}", e, path_str);
            return HttpResponse::BadRequest().json(CommonResponse {
                success: false,
                message: format!("无效的文件路径: {}", e),
            });
        }
    };
    
    let path = Path::new(&safe_path);
    
    // 检查文件是否存在
    if !path.exists() {
        eprintln!("文件不存在: {}", safe_path);
        return HttpResponse::NotFound().json(CommonResponse {
            success: false,
            message: "文件不存在".to_string(),
        });
    }
    
    // 检查是否是目录
    if path.is_dir() {
        eprintln!("路径是目录，不是文件: {}", safe_path);
        return HttpResponse::BadRequest().json(CommonResponse {
            success: false,
            message: "不能下载目录".to_string(),
        });
    }
    
    match NamedFile::open(path) {
        Ok(file) => file.into_response(&req),
        Err(e) => {
            eprintln!("打开文件失败: {}", e);
            HttpResponse::InternalServerError().json(CommonResponse {
                success: false,
                message: "打开文件失败".to_string(),
            })
        }
    }
}

/// 创建目录
pub async fn create_dir(
    req: web::Json<CreateDirRequest>,
) -> HttpResponse {
    let full_path = Path::new(&req.path).join(&req.name);
    
    // 安全检查
    let path_str = full_path.to_string_lossy().to_string();
    if path_str.contains("..") {
        return HttpResponse::BadRequest().json(CommonResponse {
            success: false,
            message: "无效的路径".to_string(),
        });
    }
    
    // 创建目录
    match fs::create_dir_all(&full_path) {
        Ok(_) => HttpResponse::Ok().json(CommonResponse {
            success: true,
            message: "目录创建成功".to_string(),
        }),
        Err(_e) => HttpResponse::InternalServerError().json(CommonResponse {
            success: false,
            message: format!("创建目录失败: {}", _e),
        }),
    }
}
/// 预览文件内容
pub async fn preview(query: web::Query<std::collections::HashMap<String, String>>) -> HttpResponse {
    let path_str = query.get("path")
        .cloned()
        .unwrap_or_default();
    
    // 验证路径安全性
    let safe_path = match validate_path(&path_str) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("预览文件路径验证失败: {} - 原始路径: {}", e, path_str);
            return HttpResponse::BadRequest().json(CommonResponse {
                success: false,
                message: format!("无效的文件路径: {}", e),
            });
        }
    };
    
    let path = Path::new(&safe_path);
    
    // 检查文件是否存在
    if !path.exists() {
        eprintln!("文件不存在: {}", safe_path);
        return HttpResponse::NotFound().json(CommonResponse {
            success: false,
            message: "文件不存在".to_string(),
        });
    }
    
    // 检查是否是目录
    if path.is_dir() {
        eprintln!("路径是目录，不是文件: {}", safe_path);
        return HttpResponse::BadRequest().json(CommonResponse {
            success: false,
            message: "不能预览目录".to_string(),
        });
    }
    
    // 读取文件内容
    match fs::read_to_string(path) {
        Ok(content) => {
            // 获取文件扩展名
            let extension = path.extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("");
            
            // 判断文件类型
            let content_type = match extension.to_lowercase().as_str() {
                "md" | "markdown" => "markdown",
                "txt" | "log" => "text",
                "html" | "htm" => "html",
                "json" => "json",
                "xml" => "xml",
                _ => "text",
            };
            
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "data": {
                    "content": content,
                    "content_type": content_type,
                    "file_name": path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown"),
                }
            }))
        }
        Err(e) => {
            eprintln!("读取文件失败: {}", e);
            HttpResponse::InternalServerError().json(CommonResponse {
                success: false,
                message: format!("读取文件失败: {}", e),
            })
        }
    }
}
