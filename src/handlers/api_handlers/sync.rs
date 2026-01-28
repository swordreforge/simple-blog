use actix_web::{web, HttpResponse};
use serde::Serialize;
use crate::db::repositories::{PassageRepository, Repository};
use std::sync::Arc;
use std::path::{Path, PathBuf};
use std::fs;
use chrono::Utc;

/// 同步响应
#[derive(Debug, Serialize)]
pub struct SyncResponse {
    pub success: bool,
    pub message: String,
}

/// 同步处理器 - 从 markdown 目录同步文章到数据库
pub async fn sync(repo: web::Data<Arc<dyn Repository>>) -> HttpResponse {
    let passage_repo = PassageRepository::new(repo.get_pool().clone());
    
    // 遍历 markdown 目录
    let markdown_dir = Path::new("markdown");
    
    if !markdown_dir.exists() {
        return HttpResponse::Ok().json(SyncResponse {
            success: false,
            message: "markdown 目录不存在".to_string(),
        });
    }
    
    let mut synced_count = 0;
    let mut error_count = 0;
    
    // 递归遍历目录并同步文件
    match sync_directory_async(markdown_dir, &passage_repo, &mut synced_count, &mut error_count).await {
        Ok(_) => {
            HttpResponse::Ok().json(SyncResponse {
                success: true,
                message: format!("同步成功: {} 篇文章已同步, {} 篇文章出错", synced_count, error_count),
            })
        }
        Err(e) => {
            HttpResponse::Ok().json(SyncResponse {
                success: false,
                message: format!("同步失败: {}", e),
            })
        }
    }
}

/// 异步同步目录（使用迭代而非递归）
async fn sync_directory_async(
    dir: &Path,
    passage_repo: &PassageRepository,
    synced_count: &mut i32,
    error_count: &mut i32,
) -> Result<(), String> {
    // 使用显式栈来模拟递归
    let mut dir_stack: Vec<PathBuf> = vec![dir.to_path_buf()];
    
    while let Some(current_dir) = dir_stack.pop() {
        let entries = fs::read_dir(&current_dir).map_err(|e| format!("读取目录失败: {}", e))?;
        
        for entry in entries {
            let entry = entry.map_err(|e| format!("读取条目失败: {}", e))?;
            let path = entry.path();
            
            if path.is_dir() {
                dir_stack.push(path);
            } else if path.extension().map_or(false, |ext| ext == "md") {
                match sync_markdown_file_async(&path, passage_repo).await {
                    Ok(_) => *synced_count += 1,
                    Err(e) => {
                        eprintln!("同步文件失败 {}: {}", path.display(), e);
                        *error_count += 1;
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// 异步同步单个 markdown 文件
async fn sync_markdown_file_async(
    path: &Path,
    passage_repo: &PassageRepository,
) -> Result<(), String> {
    // 读取文件内容
    let content = fs::read_to_string(path)
        .map_err(|e| format!("读取文件失败: {}", e))?;
    
    // 提取标题（从文件名或第一行）
    let title = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("未命名文章")
        .to_string();
    
    // 获取相对路径
    let file_path = path.to_string_lossy().to_string();
    
    // 转换 markdown 为 HTML
    let html_content = convert_markdown_to_html(&content);
    
    // 创建或更新文章
    let now = Utc::now();
    let passage = crate::db::models::Passage {
        id: None,
        title: title.clone(),
        content: html_content,
        original_content: Some(content.clone()),
        summary: None,
        author: "系统同步".to_string(),
        tags: "[]".to_string(),
        category: "未分类".to_string(),
        status: "published".to_string(),
        file_path: Some(file_path.clone()),
        visibility: "public".to_string(),
        is_scheduled: false,
        published_at: None,
        created_at: now,
        updated_at: now,
    };
    
    // 检查是否已存在
    if let Ok(_) = passage_repo.get_by_file_path(&file_path).await {
        // 更新现有文章
        // 注意：这里需要实现 update 方法
        eprintln!("文章已存在，跳过: {}", file_path);
    } else {
        // 创建新文章
        passage_repo.create(&passage).await
            .map_err(|e| format!("创建文章失败: {}", e))?;
    }
    
    Ok(())
}

/// 将 Markdown 转换为 HTML
fn convert_markdown_to_html(markdown: &str) -> String {
    use pulldown_cmark::{Parser, html};
    
    let parser = Parser::new(markdown);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    
    html_output
}