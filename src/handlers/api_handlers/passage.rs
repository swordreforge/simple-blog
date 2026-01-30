use actix_web::{web, HttpResponse, HttpRequest, HttpMessage};
use serde::{Deserialize, Serialize};
use crate::db::repositories::{PassageRepository, AttachmentRepository, Repository};
use crate::db::models::{Passage, Attachment};
use std::sync::Arc;
use chrono::Utc;

/// 检查是否为本地IP
fn is_local_ip(ip: &str) -> bool {
    // 常见的本地IP和私有网络IP
    ip == "127.0.0.1" || 
    ip == "::1" || 
    ip == "localhost" || 
    ip == "0.0.0.0" || 
    ip.is_empty() ||
    ip.starts_with("127.") ||
    ip.starts_with("192.168.") ||
    ip.starts_with("10.") ||
    ip.starts_with("172.16.") || ip.starts_with("172.17.") || ip.starts_with("172.18.") ||
    ip.starts_with("172.19.") || ip.starts_with("172.20.") || ip.starts_with("172.21.") ||
    ip.starts_with("172.22.") || ip.starts_with("172.23.") || ip.starts_with("172.24.") ||
    ip.starts_with("172.25.") || ip.starts_with("172.26.") || ip.starts_with("172.27.") ||
    ip.starts_with("172.28.") || ip.starts_with("172.29.") || ip.starts_with("172.30.") ||
    ip.starts_with("172.31.")
}

/// 文章响应
#[derive(Debug, Serialize)]
pub struct PassageResponse {
    pub id: i64,
    pub uuid: String,  // Flake UUID
    pub title: String,
    pub content: String,  // 原始 Markdown 内容
    pub html_content: Option<String>,  // 渲染后的 HTML 内容
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
pub async fn list(
    repo: web::Data<Arc<dyn Repository>>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    let passage_repo = PassageRepository::new(repo.get_pool().clone());
    
    // 解析分页参数
    let limit: i64 = query.get("limit").and_then(|l| l.parse().ok()).unwrap_or(10);
    let page: i64 = query.get("page").and_then(|p| p.parse().ok()).unwrap_or(1);
    let offset = (page - 1) * limit;
    
    // 获取已发布的文章（不包含完整内容，只返回摘要）
    match passage_repo.get_published(limit, offset).await {
        Ok(passages) => {
            // 获取总数
            let total = match passage_repo.count_published().await {
                Ok(c) => c,
                Err(_) => passages.len() as i64,
            };
            
            let data: Vec<PassageResponse> = passages.into_iter()
                .map(|p| PassageResponse {
                    id: p.id.unwrap_or(0),
                    uuid: p.uuid.unwrap_or_default(),
                    title: p.title,
                    content: p.original_content.unwrap_or_default(), // 返回原始 Markdown 内容
                    html_content: None, // 列表不返回 HTML 内容
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
            
            let total_pages = (total + limit - 1) / limit;

            HttpResponse::Ok()
                .insert_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
                .insert_header(("Pragma", "no-cache"))
                .insert_header(("Expires", "0"))
                .json(serde_json::json!({
                    "success": true,
                    "data": data,
                    "pagination": {
                        "page": page,
                        "limit": limit,
                        "total": total,
                        "total_pages": total_pages,
                        "has_more": page < total_pages
                    }
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
                    uuid: p.uuid.unwrap_or_default(),
                    title: p.title,
                    content: p.original_content.unwrap_or_default(), // 返回原始 Markdown 内容
                    html_content: Some(p.content), // 返回渲染后的 HTML
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
    path: web::Path<String>,
    req: HttpRequest,
) -> HttpResponse {
    let param = path.into_inner();
    let passage_repo = PassageRepository::new(repo.get_pool().clone());
    
    // 获取用户角色
    let role: String = req.extensions().get::<crate::middleware::auth::RoleKey>()
        .map(|r| r.0.clone())
        .unwrap_or_else(|| String::new());
    
    // 智能识别：如果是纯数字，则按 ID 查询；否则按 UUID 查询
    let passage = if let Ok(id) = param.parse::<i64>() {
        // 数字 ID 查询
        match passage_repo.get_by_id(id).await {
            Ok(p) => p,
            Err(e) => {
                eprintln!("获取文章失败: {}", e);
                return HttpResponse::NotFound().json(serde_json::json!({
                    "success": false,
                    "message": "文章不存在"
                }));
            }
        }
    } else {
        // UUID 查询
        match passage_repo.get_by_uuid(&param).await {
            Ok(p) => p,
            Err(e) => {
                eprintln!("获取文章失败: {}", e);
                return HttpResponse::NotFound().json(serde_json::json!({
                    "success": false,
                    "message": "文章不存在"
                }));
            }
        }
    };
    
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
    let passage_uuid = passage.uuid.clone().unwrap_or_default();
    let user_agent = req.headers().get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    
    tokio::spawn(async move {
        // 获取客户端IP（简化版）
        let ip = "127.0.0.1".to_string(); // TODO: 从请求中获取真实IP

        // 过滤本地IP，不记录
        if is_local_ip(&ip) {
            return;
        }

        // 使用 GeoIP 获取地理位置信息
        let geo_location = crate::geoip::lookup_ip(&ip);
        let country = geo_location.country;
        let city = geo_location.city;
        let region = geo_location.region;

        // 记录阅读
        let view_repo = crate::db::repositories::ArticleViewRepository::new(repo_clone);
        if let Err(e) = view_repo.record_view(&passage_uuid, &ip, Some(&user_agent), &country, &city, &region).await {
            eprintln!("记录阅读失败: {}", e);
        }
    });
    
    let response = PassageResponse {
        id: passage.id.unwrap_or(0),
        uuid: passage.uuid.unwrap_or_default(),
        title: passage.title,
        content: passage.original_content.unwrap_or_default(), // 返回原始 Markdown 内容
        html_content: Some(passage.content), // 返回渲染后的 HTML
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
    HttpResponse::Ok()
        .insert_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
        .insert_header(("Pragma", "no-cache"))
        .insert_header(("Expires", "0"))
        .json(serde_json::json!({
            "success": true,
            "data": response
        }))
}

/// 根据数字 ID 获取文章
pub async fn get_by_id(
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
            let passage_uuid = passage.uuid.clone().unwrap_or_default();
            let user_agent = req.headers().get("user-agent")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("unknown")
                .to_string();
            
            tokio::spawn(async move {
                // 获取客户端IP（简化版）
                let ip = "127.0.0.1".to_string(); // TODO: 从请求中获取真实IP

                // 过滤本地IP，不记录
                if is_local_ip(&ip) {
                    return;
                }

                // 使用 GeoIP 获取地理位置信息
                let geo_location = crate::geoip::lookup_ip(&ip);
                let country = geo_location.country;
                let city = geo_location.city;
                let region = geo_location.region;

                // 记录阅读
                let view_repo = crate::db::repositories::ArticleViewRepository::new(repo_clone);
                if let Err(e) = view_repo.record_view(&passage_uuid, &ip, Some(&user_agent), &country, &city, &region).await {
                    eprintln!("记录阅读失败: {}", e);
                }
            });
            
            let response = PassageResponse {
                id: passage.id.unwrap_or(0),
                uuid: passage.uuid.unwrap_or_default(),
                title: passage.title,
                content: passage.original_content.unwrap_or_default(), // 返回原始 Markdown 内容
                html_content: Some(passage.content), // 返回渲染后的 HTML
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
            HttpResponse::Ok()
                .insert_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
                .insert_header(("Pragma", "no-cache"))
                .insert_header(("Expires", "0"))
                .json(serde_json::json!({
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
    
    // 转换 Markdown 为 HTML
    let html_content = convert_markdown_to_html(&req.content);
    
    // 处理标签
    let tags_json = if let Some(ref tags) = req.tags {
        // 解析标签 JSON 并确保标签存在于 tags 表中
        if let Ok(tag_list) = serde_json::from_str::<Vec<String>>(tags) {
            ensure_tags_exist(&tag_list).await;
            tags.clone()
        } else {
            "[]".to_string()
        }
    } else {
        "[]".to_string()
    };
    
    let now = Utc::now();
    let passage = Passage {
        id: None,
        uuid: None,  // UUID 将在 Repository 中生成
        title: req.title.clone(),
        content: html_content,
        original_content: Some(req.content.clone()),
        summary: req.summary.clone(),
        author: req.author.clone().unwrap_or_else(|| "Anonymous".to_string()),
        tags: tags_json,
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
    let mut file_updated = false;
    if let Some(ref title) = req.title {
        passage.title = title.clone();
        file_updated = true;
    }
    if let Some(ref content) = req.content {
        // 转换 Markdown 为 HTML
        let html_content = convert_markdown_to_html(content);
        passage.content = html_content;
        passage.original_content = Some(content.clone());
        file_updated = true;
    }
    if let Some(ref original_content) = req.original_content {
        passage.original_content = Some(original_content.clone());
        file_updated = true;
    }
    
    // 如果内容或标题更新了，同时更新 Markdown 文件
    if file_updated {
        if let Some(ref file_path) = passage.file_path {
            let content_to_save = passage.original_content.as_ref().unwrap_or_else(|| {
                // 如果没有原始内容，从 HTML 逆向生成（不推荐，但作为后备方案）
                &passage.content
            });
            
            // 更新文件名（如果标题改变了）
            if let Some(ref title) = req.title {
                let new_file_path = update_markdown_file_name(file_path, title, content_to_save);
                if new_file_path != *file_path {
                    passage.file_path = Some(new_file_path);
                }
            } else {
                // 标题没变，只更新内容
                if let Err(e) = update_markdown_file(file_path, content_to_save) {
                    eprintln!("更新Markdown文件失败: {}", e);
                }
            }
        }
    }
    if let Some(ref summary) = req.summary {
        passage.summary = Some(summary.clone());
    }
    if let Some(ref author) = req.author {
        passage.author = author.clone();
    }
    if let Some(ref tags) = req.tags {
        // 解析标签 JSON 并确保标签存在于 tags 表中
        if let Ok(tag_list) = serde_json::from_str::<Vec<String>>(tags) {
            ensure_tags_exist(&tag_list).await;
        }
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
    path: web::Path<String>,
) -> HttpResponse {
    let uuid = path.into_inner();
    let passage_repo = PassageRepository::new(repo.get_pool().clone());
    let attachment_repo = AttachmentRepository::new(repo.get_pool().clone());

    // 1. 查询关联的附件
    let attachments = match attachment_repo.get_by_passage_uuids(vec![uuid.clone()]).await {
        Ok(attachments) => attachments,
        Err(e) => {
            eprintln!("查询附件失败: {}", e);
            Vec::new()
        }
    };

    // 2. 删除附件物理文件
    let mut deleted_files = 0;
    for attachment in &attachments {
        if let Err(e) = std::fs::remove_file(&attachment.file_path) {
            eprintln!("删除附件文件失败 {}: {}", attachment.file_path, e);
        } else {
            deleted_files += 1;
        }
    }

    // 3. 删除文章记录
    match passage_repo.delete_by_uuid(&uuid).await {
        Ok(_) => {
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": format!("文章删除成功，删除了 {} 个附件文件", deleted_files)
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

/// 批量删除文章请求
#[derive(Debug, Deserialize)]
pub struct BatchDeleteRequest {
    pub ids: Vec<i64>,
}

/// 批量删除文章
pub async fn delete_batch(
    repo: web::Data<Arc<dyn Repository>>,
    req: web::Json<BatchDeleteRequest>,
) -> HttpResponse {
    if req.ids.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "message": "文章ID列表不能为空"
        }));
    }

    let passage_repo = PassageRepository::new(repo.get_pool().clone());
    let attachment_repo = AttachmentRepository::new(repo.get_pool().clone());

    // 1. 获取文章 UUID 列表
    let uuids = match passage_repo.get_uuids_by_ids(req.ids.clone()).await {
        Ok(uuids) => uuids,
        Err(e) => {
            eprintln!("获取文章 UUID 失败: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "批量删除文章失败"
            }));
        }
    };

    // 2. 查询关联的附件
    let attachments = match attachment_repo.get_by_passage_uuids(uuids).await {
        Ok(attachments) => attachments,
        Err(e) => {
            eprintln!("查询附件失败: {}", e);
            // 即使查询附件失败，也继续删除文章
            Vec::new()
        }
    };

    // 3. 删除附件物理文件
    let mut deleted_files = 0;
    for attachment in &attachments {
        if let Err(e) = std::fs::remove_file(&attachment.file_path) {
            eprintln!("删除附件文件失败 {}: {}", attachment.file_path, e);
        } else {
            deleted_files += 1;
        }
    }

    // 4. 删除文章记录（会自动删除关联的数据库记录，通过 CASCADE）
    match passage_repo.delete_batch(req.ids.clone()).await {
        Ok(count) => {
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": format!("成功删除 {} 篇文章，{} 个附件文件", count, deleted_files),
                "deleted_count": count,
                "deleted_files": deleted_files
            }))
        }
        Err(e) => {
            eprintln!("批量删除文章失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "批量删除文章失败"
            }))
        }
    }
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

/// 更新 Markdown 文件
fn update_markdown_file(file_path: &str, content: &str) -> Result<(), String> {
    use std::fs;
    use std::path::Path;
    
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

/// 更新 Markdown 文件名（如果标题改变）
fn update_markdown_file_name(old_path: &str, new_title: &str, content: &str) -> String {
    use std::fs;
    use std::path::Path;
    
    // 构建新文件路径
    if let Some(parent) = Path::new(old_path).parent() {
        let new_path = parent.join(format!("{}.md", new_title));
        
        // 删除旧文件
        let _ = fs::remove_file(old_path);
        
        // 创建新文件
        if let Err(e) = update_markdown_file(new_path.to_str().unwrap(), content) {
            eprintln!("更新文件名失败: {}", e);
            return old_path.to_string();
        }
        
        new_path.to_str().map(|s| s.to_string()).unwrap_or_else(|| old_path.to_string())
    } else {
        old_path.to_string()
    }
}

/// 通过查询参数更新文章（用于管理后台）
pub async fn update_by_query(
    repo: web::Data<Arc<dyn Repository>>,
    query: web::Query<std::collections::HashMap<String, String>>,
    req: web::Json<UpdatePassageRequest>,
) -> HttpResponse {
    let passage_repo = PassageRepository::new(repo.get_pool().clone());
    
    // 从查询参数中获取文章 ID
    let id: i64 = match query.get("id").and_then(|s| s.parse().ok()) {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "message": "缺少文章 ID 参数"
            }));
        }
    };
    
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
    let mut file_updated = false;
    if let Some(ref title) = req.title {
        passage.title = title.clone();
        file_updated = true;
    }
    if let Some(ref content) = req.content {
        // 转换 Markdown 为 HTML
        let html_content = convert_markdown_to_html(content);
        passage.content = html_content;
        passage.original_content = Some(content.clone());
        file_updated = true;
    }
    if let Some(ref original_content) = req.original_content {
        passage.original_content = Some(original_content.clone());
        file_updated = true;
    }
    
    // 如果内容或标题更新了，同时更新 Markdown 文件
    if file_updated {
        if let Some(ref file_path) = passage.file_path {
            let content_to_save = passage.original_content.as_ref().unwrap_or_else(|| {
                // 如果没有原始内容，从 HTML 逆向生成（不推荐，但作为后备方案）
                &passage.content
            });
            
            // 更新文件名（如果标题改变了）
            if let Some(ref title) = req.title {
                let new_file_path = update_markdown_file_name(file_path, title, content_to_save);
                if new_file_path != *file_path {
                    passage.file_path = Some(new_file_path);
                }
            } else {
                // 标题没变，只更新内容
                if let Err(e) = update_markdown_file(file_path, content_to_save) {
                    eprintln!("更新Markdown文件失败: {}", e);
                }
            }
        }
    }
    if let Some(ref summary) = req.summary {
        passage.summary = Some(summary.clone());
    }
    if let Some(ref author) = req.author {
        passage.author = author.clone();
    }
    if let Some(ref tags) = req.tags {
        // 解析标签 JSON 并确保标签存在于 tags 表中
        if let Ok(tag_list) = serde_json::from_str::<Vec<String>>(tags) {
            ensure_tags_exist(&tag_list).await;
        }
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

/// 通过查询参数获取单篇文章或文章列表（用于管理后台）
pub async fn get_by_query(
    repo: web::Data<Arc<dyn Repository>>,
    query: web::Query<std::collections::HashMap<String, String>>,
    req: HttpRequest,
) -> HttpResponse {
    let passage_repo = PassageRepository::new(repo.get_pool().clone());
    
    // 检查是否有 id 查询参数
    if let Some(id_str) = query.get("id") {
        // 如果有 id 参数，返回单篇文章
        let id: i64 = match id_str.parse() {
            Ok(id) => id,
            Err(_) => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "success": false,
                    "message": "无效的文章 ID"
                }));
            }
        };
        
        match passage_repo.get_by_id(id).await {
            Ok(passage) => {
                let response = PassageResponse {
                    id: passage.id.unwrap_or(0),
                    uuid: passage.uuid.unwrap_or_default(),
                    title: passage.title,
                    content: passage.original_content.unwrap_or_default(), // 返回原始 Markdown 内容
                    html_content: Some(passage.content), // 返回渲染后的 HTML
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
    } else {
        // 如果没有 id 参数，返回文章列表（调用 admin_list 的逻辑）
        let limit: i64 = query.get("limit").and_then(|l| l.parse().ok()).unwrap_or(20);
        let _offset: i64 = query.get("offset").and_then(|o| o.parse().ok()).unwrap_or(0);
        let page: i64 = query.get("page").and_then(|p| p.parse().ok()).unwrap_or(1);
        let calculated_offset = (page - 1) * limit;
        
        match passage_repo.get_all(limit, calculated_offset).await {
            Ok(passages) => {
                let total = match passage_repo.count().await {
                    Ok(c) => c,
                    Err(_) => passages.len() as i64,
                };
                
                let data: Vec<PassageResponse> = passages.into_iter()
                    .map(|p| PassageResponse {
                        id: p.id.unwrap_or(0),
                        uuid: p.uuid.unwrap_or_default(),
                        title: p.title,
                        content: p.original_content.unwrap_or_default(), // 返回原始 Markdown 内容
                        html_content: Some(p.content), // 返回渲染后的 HTML
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
}