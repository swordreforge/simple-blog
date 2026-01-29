use actix_web::{web, HttpResponse};
use serde::Serialize;
use crate::db::repositories::{PassageRepository, Repository};
use std::sync::Arc;
use std::path::{Path, PathBuf};
use std::fs;
use chrono::{Utc, NaiveDate, DateTime};

/// 同步响应
#[derive(Debug, Serialize)]
pub struct SyncResponse {
    pub success: bool,
    pub message: String,
}

/// 同步结果
#[derive(Debug)]
pub struct SyncResult {
    pub synced_count: usize,
    pub updated_count: usize,
    pub deleted_count: usize,
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
    let mut updated_count = 0;
    let mut deleted_count = 0;
    
    // 递归遍历目录并同步文件
    match sync_directory_async(markdown_dir, &passage_repo, &mut synced_count, &mut updated_count, &mut deleted_count).await {
        Ok(_) => {
            HttpResponse::Ok().json(SyncResponse {
                success: true,
                message: format!("同步成功: {} 篇文章已同步, {} 篇文章已更新, {} 篇文章已删除", synced_count, updated_count, deleted_count),
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

/// 内部同步函数 - 用于启动时的自动同步
pub async fn sync_directory_internal(passage_repo: &PassageRepository) -> Result<SyncResult, String> {
    let markdown_dir = Path::new("markdown");
    
    if !markdown_dir.exists() {
        return Ok(SyncResult {
            synced_count: 0,
            updated_count: 0,
            deleted_count: 0,
            message: "markdown 目录不存在，跳过同步".to_string(),
        });
    }
    
    let mut synced_count = 0;
    let mut updated_count = 0;
    let mut deleted_count = 0;
    
    sync_directory_async(markdown_dir, passage_repo, &mut synced_count, &mut updated_count, &mut deleted_count).await?;
    
    Ok(SyncResult {
        synced_count,
        updated_count,
        deleted_count,
        message: format!(
            "文章同步完成: {} 篇已同步, {} 篇已更新, {} 篇已删除",
            synced_count, updated_count, deleted_count
        ),
    })
}

/// 异步同步目录（使用迭代而非递归）
async fn sync_directory_async(
    dir: &Path,
    passage_repo: &PassageRepository,
    synced_count: &mut usize,
    updated_count: &mut usize,
    deleted_count: &mut usize,
) -> Result<(), String> {
    // 使用显式栈来模拟递归
    let mut dir_stack: Vec<PathBuf> = vec![dir.to_path_buf()];
    let mut md_files: Vec<PathBuf> = Vec::new();
    
    while let Some(current_dir) = dir_stack.pop() {
        let entries = fs::read_dir(&current_dir).map_err(|e| format!("读取目录失败: {}", e))?;
        
        for entry in entries {
            let entry = entry.map_err(|e| format!("读取条目失败: {}", e))?;
            let path = entry.path();
            
            if path.is_dir() {
                dir_stack.push(path);
            } else if path.extension().map_or(false, |ext| ext == "md") {
                md_files.push(path);
            }
        }
    }
    
    // 同步所有 markdown 文件
    for path in md_files {
        match sync_markdown_file_async(&path, passage_repo, synced_count, updated_count).await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("同步文件失败 {}: {}", path.display(), e);
            }
        }
    }
    
    // 清理数据库中不存在的文件记录
    cleanup_orphaned_passages(passage_repo, dir, deleted_count).await?;
    
    Ok(())
}

/// 异步同步单个 markdown 文件
async fn sync_markdown_file_async(
    path: &Path,
    passage_repo: &PassageRepository,
    synced_count: &mut usize,
    updated_count: &mut usize,
) -> Result<(), String> {
    // 读取文件内容
    let content = fs::read_to_string(path)
        .map_err(|e| format!("读取文件失败: {}", e))?;
    
    // 提取标题（从文件名）
    let title = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("未命名文章")
        .to_string();
    
    // 获取相对路径
    let file_path = path.to_string_lossy().to_string();
    
    // 从路径提取日期（格式：markdown/YYYY/MM/DD/filename.md）
    let created_at = extract_date_from_path(&file_path).unwrap_or_else(Utc::now);
    
    // 转换 markdown 为 HTML
    let html_content = convert_markdown_to_html(&content);
    
    // 生成摘要
    let summary = extract_summary(&html_content);
    
    // 生成标签名称列表
    let tag_names = extract_tag_names(&file_path);
    let tags_json = serde_json::to_string(&tag_names).unwrap_or_else(|_| "[]".to_string());
    
    let now = Utc::now();
    
    // 检查是否已存在
    if let Ok(existing) = passage_repo.get_by_file_path(&file_path).await {
        // 更新现有文章
        let updated_passage = crate::db::models::Passage {
            id: existing.id,
            title,
            content: html_content,
            original_content: Some(content.clone()),
            summary,
            author: existing.author,
            tags: tags_json.clone(),
            category: existing.category,
            status: existing.status,
            file_path: Some(file_path.clone()),
            visibility: existing.visibility,
            is_scheduled: existing.is_scheduled,
            published_at: existing.published_at,
            created_at: existing.created_at,
            updated_at: now,
        };
        
        // 更新文章（使用 SQL 直接更新）
        update_passage(passage_repo, &updated_passage).await
            .map_err(|e| format!("更新文章失败: {}", e))?;
        
        // 更新标签关联
        if let Some(passage_id) = existing.id {
            update_passage_tags(passage_id, &tag_names).await?;
        }
        
        *updated_count += 1;
        println!("✏️  已更新文章: {}", file_path);
    } else {
        // 创建新文章
        let passage = crate::db::models::Passage {
            id: None,
            title: title.clone(),
            content: html_content,
            original_content: Some(content.clone()),
            summary,
            author: "Admin".to_string(),
            tags: tags_json.clone(),
            category: "未分类".to_string(),
            status: "published".to_string(),
            file_path: Some(file_path.clone()),
            visibility: "public".to_string(),
            is_scheduled: false,
            published_at: None,
            created_at,
            updated_at: now,
        };
        
        let passage_id = passage_repo.create(&passage).await
            .map_err(|e| format!("创建文章失败: {}", e))?;
        
        // 创建标签关联
        create_passage_tags(passage_id, &tag_names).await?;
        
        *synced_count += 1;
        println!("✅ 已同步文章: {}", file_path);
    }
    
    Ok(())
}

/// 从文件路径提取日期
fn extract_date_from_path(file_path: &str) -> Option<DateTime<Utc>> {
    // 移除 markdown/ 前缀
    let path = file_path.strip_prefix("markdown/")?;
    
    // 分割路径
    let parts: Vec<&str> = path.split('/').collect();
    
    // 检查是否有 年/月/日 格式
    if parts.len() >= 3 {
        let year = parts[0].parse::<i32>().ok()?;
        let month = parts[1].parse::<u32>().ok()?;
        let day = parts[2].parse::<u32>().ok()?;
        
        if let Some(naive_date) = NaiveDate::from_ymd_opt(year, month, day) {
            return Some(DateTime::<Utc>::from_naive_utc_and_offset(
                naive_date.and_hms_opt(0, 0, 0).unwrap(),
                Utc,
            ));
        }
    }
    
    None
}

/// 提取摘要
fn extract_summary(html_content: &str) -> Option<String> {
    use regex::Regex;
    
    // 移除 HTML 标签
    let re = Regex::new(r"<[^>]*>").unwrap();
    let text = re.replace_all(html_content, "");
    
    // 移除多余的空白
    let text: String = text.split_whitespace().collect::<Vec<&str>>().join(" ");
    
    // 截取前 200 个字符
    if text.chars().count() > 200 {
        Some(text.chars().take(200).collect::<String>() + "...")
    } else {
        Some(text)
    }
}

/// 提取标签名称列表
fn extract_tag_names(path: &str) -> Vec<String> {
    // 移除 markdown/ 前缀和 .md 后缀
    let path = path.strip_prefix("markdown/").unwrap_or(path);
    let path = path.strip_suffix(".md").unwrap_or(path);
    
    // 分割路径
    let parts: Vec<&str> = path.split('/').collect();
    
    // 使用年份和月份作为标签
    let mut tags = Vec::new();
    if parts.len() >= 2 {
        tags.push(parts[0].to_string());  // 年份
        tags.push(parts[1].to_string());  // 月份
    }
    
    tags
}

/// 创建文章标签关联
async fn create_passage_tags(passage_id: i64, tag_names: &[String]) -> Result<(), String> {
    use crate::db::get_db_pool_sync;
    use crate::db::repositories::TagRepository;
    use rusqlite::params;
    use std::sync::Arc;
    
    let pool = get_db_pool_sync().map_err(|e| format!("获取数据库连接失败: {}", e))?;
    let conn = pool.get().map_err(|e| format!("获取连接失败: {}", e))?;
    
    let tag_repo = TagRepository::new(Arc::new(pool.clone()));
    
    for tag_name in tag_names {
        // 查找或创建标签
        let tag_id = match tag_repo.get_by_name(tag_name).await {
            Ok(tag) => {
                tag.id.unwrap_or(0)
            }
            Err(_) => {
                // 标签不存在，创建新标签
                let now = chrono::Utc::now();
                let new_tag = crate::db::models::Tag {
                    id: None,
                    name: tag_name.clone(),
                    description: format!("自动生成的标签: {}", tag_name),
                    color: "#007bff".to_string(),
                    category_id: 0,
                    sort_order: 0,
                    is_enabled: true,
                    created_at: now,
                    updated_at: now,
                };
                
                tag_repo.create(&new_tag).await
                    .map_err(|e| format!("创建标签失败: {}", e))?
            }
        };
        
        // 创建文章-标签关联
        conn.execute(
            "INSERT OR IGNORE INTO passage_tags (passage_id, tag_id, created_at) VALUES (?, ?, ?)",
            params![passage_id, tag_id, chrono::Utc::now()],
        ).map_err(|e| format!("创建标签关联失败: {}", e))?;
    }
    
    Ok(())
}

/// 更新文章标签关联
async fn update_passage_tags(passage_id: i64, tag_names: &[String]) -> Result<(), String> {
    use crate::db::get_db_pool_sync;
    use crate::db::repositories::TagRepository;
    use rusqlite::params;
    use std::sync::Arc;
    
    let pool = get_db_pool_sync().map_err(|e| format!("获取数据库连接失败: {}", e))?;
    let conn = pool.get().map_err(|e| format!("获取连接失败: {}", e))?;
    
    let tag_repo = TagRepository::new(Arc::new(pool.clone()));
    
    // 删除旧的标签关联
    conn.execute(
        "DELETE FROM passage_tags WHERE passage_id = ?",
        params![passage_id],
    ).map_err(|e| format!("删除旧标签关联失败: {}", e))?;
    
    // 创建新的标签关联
    create_passage_tags(passage_id, tag_names).await?;
    
    Ok(())
}

/// 更新文章
async fn update_passage(
    passage_repo: &PassageRepository,
    passage: &crate::db::models::Passage,
) -> Result<(), String> {
    use crate::db::get_db_pool_sync;
    use rusqlite::params;
    
    let pool = get_db_pool_sync().map_err(|e| format!("获取数据库连接失败: {}", e))?;
    let conn = pool.get().map_err(|e| format!("获取连接失败: {}", e))?;
    
    if let Some(id) = passage.id {
        conn.execute(
            "UPDATE passages SET title = ?, content = ?, original_content = ?, summary = ?, tags = ?, updated_at = ? WHERE id = ?",
            params![
                &passage.title,
                &passage.content,
                &passage.original_content,
                &passage.summary,
                &passage.tags,
                &passage.updated_at,
                id,
            ],
        ).map_err(|e| format!("更新失败: {}", e))?;
    }
    
    Ok(())
}

/// 清理数据库中不存在的文章记录
async fn cleanup_orphaned_passages(
    passage_repo: &PassageRepository,
    markdown_dir: &Path,
    deleted_count: &mut usize,
) -> Result<(), String> {
    use crate::db::get_db_pool_sync;
    use rusqlite::params;
    
    let pool = get_db_pool_sync().map_err(|e| format!("获取数据库连接失败: {}", e))?;
    let conn = pool.get().map_err(|e| format!("获取连接失败: {}", e))?;
    
    // 获取所有有 file_path 的文章
    let mut stmt = conn.prepare("SELECT id, file_path FROM passages WHERE file_path IS NOT NULL")
        .map_err(|e| format!("查询失败: {}", e))?;
    
    let passage_rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, Option<String>>(1)?,
        ))
    }).map_err(|e| format!("查询失败: {}", e))?;
    
    for row in passage_rows {
        if let Ok((id, file_path)) = row {
            if let Some(fp) = file_path {
                let full_path = Path::new(&fp);
                if !full_path.exists() {
                    conn.execute("DELETE FROM passages WHERE id = ?", params![id])
                        .map_err(|e| format!("删除失败: {}", e))?;
                    *deleted_count += 1;
                    println!("🗑️  已删除不存在的文章记录: {}", fp);
                }
            }
        }
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