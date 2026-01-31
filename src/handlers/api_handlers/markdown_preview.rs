use actix_web::{web, HttpResponse};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::fs;

/// Markdown 预览响应
#[derive(Debug, Serialize)]
pub struct MarkdownPreviewResponse {
    pub success: bool,
    pub data: Option<MarkdownPreviewData>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MarkdownPreviewData {
    pub content: String,
    pub title: String,
    pub file_name: String,
    pub file_path: String,
}

/// 验证并规范化 markdown 文件路径
fn validate_markdown_path(path: &str) -> Result<PathBuf, String> {
    let cwd = std::env::current_dir()
        .map_err(|_| "无法获取当前目录".to_string())?;
    
    // URL 解码路径
    let decoded_path = match urlencoding::decode(path) {
        Ok(decoded) => decoded.into_owned(),
        Err(e) => {
            return Err(format!("URL 解码失败: {}", e));
        }
    };
    
    // 规范化用户路径
    let normalized_path = Path::new(&decoded_path);
    let normalized_path: PathBuf = normalized_path
        .components()
        .filter(|comp| !matches!(comp, std::path::Component::ParentDir | std::path::Component::CurDir))
        .collect();
    
    // 构建完整路径
    let full_path = cwd.join("markdown").join(&normalized_path);
    
    // 确保文件以 .md 结尾
    if !full_path.extension().map_or(false, |ext| ext == "md") {
        return Err(format!("文件必须是 .md 格式，当前扩展名: {:?}", full_path.extension()));
    }
    
    // 检查文件是否存在
    if !full_path.exists() {
        return Err(format!("文件不存在: {}, 完整路径: {}", path, full_path.display()));
    }
    
    // 检查是否是文件
    if !full_path.is_file() {
        return Err(format!("路径不是文件: {}", path));
    }
    
    Ok(full_path)
}

/// 提取 markdown 文件的标题
fn extract_markdown_title(content: &str) -> String {
    // 查找第一个 H1 标题
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") {
            return trimmed[2..].trim().to_string();
        }
    }
    
    // 如果没有找到标题，使用文件名
    "无标题".to_string()
}

/// Markdown 预览 API
pub async fn preview(query: web::Query<std::collections::HashMap<String, String>>) -> HttpResponse {
    let path_str = query.get("path")
        .cloned()
        .unwrap_or_default();
    
    // 验证路径
    let full_path = match validate_markdown_path(&path_str) {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::BadRequest().json(MarkdownPreviewResponse {
                success: false,
                data: None,
                message: Some(e),
            });
        }
    };
    
    // 读取文件内容
    let content = match fs::read_to_string(&full_path) {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::InternalServerError().json(MarkdownPreviewResponse {
                success: false,
                data: None,
                message: Some(format!("读取文件失败: {}", e)),
            });
        }
    };
    
    // 提取标题
    let title = extract_markdown_title(&content);
    
    // 获取文件名
    let file_name = full_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown.md")
        .to_string();
    
    // 获取相对路径
    let cwd = std::env::current_dir().unwrap();
    let markdown_base = cwd.join("markdown");
    let file_path = full_path
        .strip_prefix(&markdown_base)
        .ok()
        .and_then(|p| p.to_str())
        .unwrap_or(&path_str)
        .to_string();
    
    HttpResponse::Ok().json(MarkdownPreviewResponse {
        success: true,
        data: Some(MarkdownPreviewData {
            content,
            title,
            file_name,
            file_path,
        }),
        message: None,
    })
}