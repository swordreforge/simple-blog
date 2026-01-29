use actix_web::{web, HttpResponse, HttpRequest};
use serde::{Deserialize, Serialize};
use crate::db::repositories::{PassageRepository, Repository};
use std::sync::Arc;
use std::fs;
use std::path::Path;
use chrono::Utc;

/// 保存文章请求
#[derive(Debug, Deserialize)]
pub struct SaveArticleRequest {
    pub title: String,
    pub content: String,
    pub category: Option<String>,
    pub tags: Option<String>,
    pub summary: Option<String>,
}

/// 保存文章响应
#[derive(Debug, Serialize)]
pub struct SaveArticleResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<ArticleData>,
}

#[derive(Debug, Serialize)]
pub struct ArticleData {
    pub id: i64,
    pub title: String,
    pub file_path: String,
    pub created_at: String,
}

/// 保存 Markdown 文章到数据库
pub async fn save(
    req: web::Json<SaveArticleRequest>,
    repo: web::Data<Arc<dyn Repository>>,
    http_req: HttpRequest,
) -> HttpResponse {
    use actix_web::HttpMessage;
    
    let req_data = req.into_inner();
    
    // 检查用户权限（需要管理员权限）
    let role: String = http_req.extensions().get::<crate::middleware::auth::RoleKey>()
        .map(|r| r.0.clone())
        .unwrap_or_else(|| String::new());
    
    if role != "admin" {
        return HttpResponse::Ok().json(SaveArticleResponse {
            success: false,
            message: "需要管理员权限才能保存文章".to_string(),
            data: None,
        });
    }
    
    // 验证必填字段
    if req_data.title.is_empty() {
        return HttpResponse::Ok().json(SaveArticleResponse {
            success: false,
            message: "文章标题不能为空".to_string(),
            data: None,
        });
    }
    
    if req_data.content.is_empty() {
        return HttpResponse::Ok().json(SaveArticleResponse {
            success: false,
            message: "文章内容不能为空".to_string(),
            data: None,
        });
    }
    
    // 转换 Markdown 为 HTML
    let html_content = convert_markdown_to_html(&req_data.content);
    
    // 构建文件路径（按日期组织）
    let now = Utc::now();
    let date_dir = now.format("%Y/%m/%d").to_string();
    let file_path = format!("markdown/{}/{}.md", date_dir, req_data.title);
    
    // 保存 Markdown 文件到磁盘
    if let Err(e) = save_markdown_file(&file_path, &req_data.title, &req_data.content) {
        return HttpResponse::Ok().json(SaveArticleResponse {
            success: false,
            message: format!("保存Markdown文件失败: {}", e),
            data: None,
        });
    }
    
    // 处理标签（转换为JSON格式）
    
        let tags_json = if let Some(tags) = &req_data.tags {
    
            if tags.is_empty() {
    
                "[]".to_string()
    
            } else {
    
                let tag_list: Vec<String> = tags.split(',')
    
                    .map(|t| t.trim().to_string())
    
                    .filter(|t| !t.is_empty())
    
                    .collect();
    
    
    
                // 确保标签存在于 tags 表中
    
                if let Err(e) = ensure_tags_exist(&tag_list).await {
    
                    return HttpResponse::Ok().json(SaveArticleResponse {
    
                        success: false,
    
                        message: format!("处理标签失败: {}", e),
    
                        data: None,
    
                    });
    
                }
    
    
    
                serde_json::to_string(&tag_list).unwrap_or_else(|_| "[]".to_string())
    
            }
    
        } else {
    
            "[]".to_string()
    
        };
    
        
    
        // 设置默认分类
    
        let category = req_data.category.as_deref().unwrap_or("未分类").to_string();
    
        
    
        // 设置默认摘要
    
        let summary = req_data.summary.as_deref().unwrap_or("暂无摘要").to_string();
    
        
    
        // 创建文章记录
    let passage = crate::db::models::Passage {
        id: None,
        title: req_data.title.clone(),
        content: html_content,
        original_content: Some(req_data.content),
        summary: Some(summary),
        author: "admin".to_string(),
        tags: tags_json,
        category,
        status: "published".to_string(),
        file_path: Some(file_path.clone()),
        visibility: "public".to_string(),
        is_scheduled: false,
        published_at: None,
        created_at: now,
        updated_at: now,
    };
    
    let passage_repo = PassageRepository::new(repo.get_pool().clone());
    
    match passage_repo.create(&passage).await {
        Ok(_) => HttpResponse::Ok().json(SaveArticleResponse {
            success: true,
            message: "文章保存成功".to_string(),
            data: Some(ArticleData {
                id: passage.id.unwrap_or(0),
                title: passage.title,
                file_path,
                created_at: passage.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            }),
        }),
        Err(e) => HttpResponse::Ok().json(SaveArticleResponse {
            success: false,
            message: format!("保存到数据库失败: {}", e),
            data: None,
        }),
    }
}

/// 保存 Markdown 文件
fn save_markdown_file(file_path: &str, _title: &str, content: &str) -> Result<(), String> {
    // 创建目录
    if let Some(parent) = Path::new(file_path).parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("创建目录失败: {}", e))?;
    }
    
    // 写入文件
    fs::write(file_path, content)
        .map_err(|e| format!("写入文件失败: {}", e))?;
    
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

/// 确保标签存在于 tags 表中
async fn ensure_tags_exist(tag_names: &[String]) -> Result<(), String> {
    use crate::db::get_db_pool_sync;
    use crate::db::repositories::TagRepository;
    use std::sync::Arc;
    
    let pool = get_db_pool_sync().map_err(|e| format!("获取数据库连接失败: {}", e))?;
    let tag_repo = TagRepository::new(Arc::new(pool.clone()));
    
    for tag_name in tag_names {
        // 查找标签，如果不存在则创建
        if tag_repo.get_by_name(tag_name).await.is_err() {
            let now = chrono::Utc::now();
            let new_tag = crate::db::models::Tag {
                id: None,
                name: tag_name.clone(),
                description: format!("用户创建的标签: {}", tag_name),
                color: "#007bff".to_string(),
                category_id: 0,
                sort_order: 0,
                is_enabled: true,
                created_at: now,
                updated_at: now,
            };
            
            tag_repo.create(&new_tag).await
                .map_err(|e| format!("创建标签失败: {}", e))?;
        }
    }
    
    Ok(())
}