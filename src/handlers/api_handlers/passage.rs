use actix_web::{web, HttpResponse, HttpRequest, HttpMessage};
use serde::{Deserialize, Serialize};
use crate::db::repositories::{PassageRepository, Repository};
use crate::db::models::Passage;
use std::sync::Arc;
use chrono::Utc;

/// 文章响应
#[derive(Debug, Serialize)]
pub struct PassageResponse {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub summary: Option<String>,
    pub author: String,
    pub tags: String,
    pub category: String,
    pub status: String,
    pub file_path: Option<String>,
    pub visibility: String,
    pub is_scheduled: bool,
    pub published_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// 创建文章请求
#[derive(Debug, Deserialize)]
pub struct CreatePassageRequest {
    pub title: String,
    pub content: String,
    pub original_content: Option<String>,
    pub summary: Option<String>,
    pub author: Option<String>,
    pub tags: Option<String>,
    pub category: Option<String>,
    pub status: Option<String>,
    pub file_path: Option<String>,
    pub visibility: Option<String>,
    pub is_scheduled: Option<bool>,
    pub published_at: Option<String>,
}

/// 更新文章请求
#[derive(Debug, Deserialize)]
pub struct UpdatePassageRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub original_content: Option<String>,
    pub summary: Option<String>,
    pub author: Option<String>,
    pub tags: Option<String>,
    pub category: Option<String>,
    pub status: Option<String>,
    pub file_path: Option<String>,
    pub visibility: Option<String>,
    pub is_scheduled: Option<bool>,
    pub published_at: Option<String>,
}

/// 获取文章列表（公开）
pub async fn list(repo: web::Data<Arc<dyn Repository>>) -> HttpResponse {
    let passage_repo = PassageRepository::new(repo.get_pool().clone());
    
    match passage_repo.get_published(100, 0).await {
        Ok(passages) => {
            let data: Vec<PassageResponse> = passages.into_iter()
                .map(|p| PassageResponse {
                    id: p.id.unwrap_or(0),
                    title: p.title,
                    content: p.content,
                    summary: p.summary,
                    author: p.author,
                    tags: p.tags,
                    category: p.category,
                    status: p.status,
                    file_path: p.file_path,
                    visibility: p.visibility,
                    is_scheduled: p.is_scheduled,
                    published_at: p.published_at.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()),
                    created_at: p.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                    updated_at: p.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                })
                .collect();
            
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "data": data
            }))
        }
        Err(e) => {
            eprintln!("获取文章列表失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "获取文章列表失败"
            }))
        }
    }
}

/// 获取文章列表（管理员）
pub async fn admin_list(
    repo: web::Data<Arc<dyn Repository>>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    let passage_repo = PassageRepository::new(repo.get_pool().clone());
    
    // 解析分页参数
    let limit: i64 = query.get("limit").and_then(|l| l.parse().ok()).unwrap_or(20);
    let _offset: i64 = query.get("offset").and_then(|o| o.parse().ok()).unwrap_or(0);
    let page: i64 = query.get("page").and_then(|p| p.parse().ok()).unwrap_or(1);
    let calculated_offset = (page - 1) * limit;
    
    // 获取所有文章
    match passage_repo.get_all(limit, calculated_offset).await {
        Ok(passages) => {
            let total = match passage_repo.count().await {
                Ok(c) => c,
                Err(_) => passages.len() as i64,
            };
            
            let data: Vec<PassageResponse> = passages.into_iter()
                .map(|p| PassageResponse {
                    id: p.id.unwrap_or(0),
                    title: p.title,
                    content: p.content,
                    summary: p.summary,
                    author: p.author,
                    tags: p.tags,
                    category: p.category,
                    status: p.status,
                    file_path: p.file_path,
                    visibility: p.visibility,
                    is_scheduled: p.is_scheduled,
                    published_at: p.published_at.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()),
                    created_at: p.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                    updated_at: p.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                })
                .collect();
            
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "data": data,
                "total": total,
                "page": page,
                "limit": limit
            }))
        }
        Err(e) => {
            eprintln!("获取文章列表失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "获取文章列表失败"
            }))
        }
    }
}

/// 获取单篇文章
pub async fn get(
    repo: web::Data<Arc<dyn Repository>>,
    path: web::Path<i64>,
    req: HttpRequest,
) -> HttpResponse {
    let id = path.into_inner();
    let passage_repo = PassageRepository::new(repo.get_pool().clone());
    
    // 获取用户角色
    let role: String = req.extensions().get::<crate::middleware::auth::RoleKey>()
        .map(|r| r.0.clone())
        .unwrap_or_else(|| String::new());
    
    // 检查访问权限
    match passage_repo.get_by_id(id).await {
        Ok(passage) => {
            // 检查文章状态和可见性
            if passage.status != "published" {
                if role != "admin" {
                    return HttpResponse::Ok().json(serde_json::json!({
                        "success": false,
                        "message": "文章未发布",
                        "status": passage.status
                    }));
                }
            }
            
            if passage.visibility != "public" {
                if role != "admin" {
                    return HttpResponse::Ok().json(serde_json::json!({
                        "success": false,
                        "message": "文章不可见",
                        "visibility": passage.visibility
                    }));
                }
            }
            
            if passage.is_scheduled {
                if let Some(published_at) = passage.published_at {
                    if published_at > Utc::now() && role != "admin" {
                        return HttpResponse::Ok().json(serde_json::json!({
                            "success": false,
                            "message": "文章尚未发布",
                            "is_scheduled": true,
                            "published_at": published_at.format("%Y-%m-%d %H:%M:%S").to_string()
                        }));
                    }
                }
            }
            
            // 异步记录文章阅读（不阻塞响应）
            let repo_clone = repo.get_pool().clone();
            let user_agent = req.headers().get("user-agent")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("unknown")
                .to_string();
            
            tokio::spawn(async move {
                // 获取客户端IP（简化版）
                let ip = "127.0.0.1".to_string(); // TODO: 从请求中获取真实IP
                
                // 获取地理位置信息（简化版）
                let country = "unknown".to_string();
                let city = "unknown".to_string();
                let region = "unknown".to_string();
                
                // 记录阅读
                let view_repo = crate::db::repositories::ArticleViewRepository::new(repo_clone);
                if let Err(e) = view_repo.record_view(id, &ip, Some(&user_agent), &country, &city, &region).await {
                    eprintln!("记录阅读失败: {}", e);
                }
            });
            
            let response = PassageResponse {
                id: passage.id.unwrap_or(0),
                title: passage.title,
                content: passage.content,
                summary: passage.summary,
                author: passage.author,
                tags: passage.tags,
                category: passage.category,
                status: passage.status,
                file_path: passage.file_path,
                visibility: passage.visibility,
                is_scheduled: passage.is_scheduled,
                published_at: passage.published_at.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()),
                created_at: passage.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                updated_at: passage.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            };
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "data": response
            }))
        }
        Err(e) => {
            eprintln!("获取文章失败: {}", e);
            HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "message": "文章不存在"
            }))
        }
    }
}

/// 创建文章
pub async fn create(
    repo: web::Data<Arc<dyn Repository>>,
    req: web::Json<CreatePassageRequest>,
) -> HttpResponse {
    let passage_repo = PassageRepository::new(repo.get_pool().clone());
    
    let now = Utc::now();
    let passage = Passage {
        id: None,
        title: req.title.clone(),
        content: req.content.clone(),
        original_content: req.original_content.clone(),
        summary: req.summary.clone(),
        author: req.author.clone().unwrap_or_else(|| "Anonymous".to_string()),
        tags: req.tags.clone().unwrap_or_else(|| "[]".to_string()),
        category: req.category.clone().unwrap_or_else(|| "未分类".to_string()),
        status: req.status.clone().unwrap_or_else(|| "draft".to_string()),
        file_path: req.file_path.clone(),
        visibility: req.visibility.clone().unwrap_or_else(|| "public".to_string()),
        is_scheduled: req.is_scheduled.unwrap_or(false),
        published_at: req.published_at.as_ref().and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok()).map(|dt| dt.with_timezone(&Utc)),
        created_at: now,
        updated_at: now,
    };
    
    match passage_repo.create(&passage).await {
        Ok(_) => {
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "文章创建成功"
            }))
        }
        Err(e) => {
            eprintln!("创建文章失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "创建文章失败"
            }))
        }
    }
}

/// 更新文章
pub async fn update(
    repo: web::Data<Arc<dyn Repository>>,
    path: web::Path<i64>,
    req: web::Json<UpdatePassageRequest>,
) -> HttpResponse {
    let id = path.into_inner();
    let passage_repo = PassageRepository::new(repo.get_pool().clone());
    
    // 先获取现有文章
    let mut passage = match passage_repo.get_by_id(id).await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("获取文章失败: {}", e);
            return HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "message": "文章不存在"
            }));
        }
    };
    
    // 更新字段
    if let Some(ref title) = req.title {
        passage.title = title.clone();
    }
    if let Some(ref content) = req.content {
        passage.content = content.clone();
    }
    if let Some(ref original_content) = req.original_content {
        passage.original_content = Some(original_content.clone());
    }
    if let Some(ref summary) = req.summary {
        passage.summary = Some(summary.clone());
    }
    if let Some(ref author) = req.author {
        passage.author = author.clone();
    }
    if let Some(ref tags) = req.tags {
        passage.tags = tags.clone();
    }
    if let Some(ref category) = req.category {
        passage.category = category.clone();
    }
    if let Some(ref status) = req.status {
        passage.status = status.clone();
    }
    if let Some(ref file_path) = req.file_path {
        passage.file_path = Some(file_path.clone());
    }
    if let Some(ref visibility) = req.visibility {
        passage.visibility = visibility.clone();
    }
    if let Some(is_scheduled) = req.is_scheduled {
        passage.is_scheduled = is_scheduled;
    }
    if let Some(ref published_at) = req.published_at {
        passage.published_at = chrono::DateTime::parse_from_rfc3339(published_at)
            .ok()
            .map(|dt| dt.with_timezone(&chrono::Utc));
    }
    passage.updated_at = chrono::Utc::now();
    
    match passage_repo.update(&passage).await {
        Ok(_) => {
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "文章更新成功"
            }))
        }
        Err(e) => {
            eprintln!("更新文章失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "更新文章失败"
            }))
        }
    }
}

/// 删除文章
pub async fn delete(
    repo: web::Data<Arc<dyn Repository>>,
    path: web::Path<i64>,
) -> HttpResponse {
    let id = path.into_inner();
    let passage_repo = PassageRepository::new(repo.get_pool().clone());
    
    match passage_repo.delete(id).await {
        Ok(_) => {
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "文章删除成功"
            }))
        }
        Err(e) => {
            eprintln!("删除文章失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "删除文章失败"
            }))
        }
    }
}