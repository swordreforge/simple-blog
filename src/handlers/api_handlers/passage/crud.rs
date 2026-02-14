use actix_web::{web, HttpResponse, HttpRequest, HttpMessage};
use serde::{Deserialize, Serialize};
use crate::db::repositories::{PassageRepository, AttachmentRepository, Repository};
use crate::db::models::Passage;
use crate::view_batch::{ViewBatchProcessor, ViewRecord, is_local_ip};
use std::sync::Arc;
use chrono::Utc;

use super::markdown::{convert_markdown_to_html, update_markdown_file, update_markdown_file_name, extract_summary};
use super::validation::{ensure_tags_exist, ensure_category_exist};

/// 文章响应
#[derive(Debug, Serialize)]
pub struct PassageResponse {
    pub id: i64,
    pub uuid: String,
    pub title: String,
    pub content: String,
    pub html_content: Option<String>,
    pub summary: Option<String>,
    pub author: String,
    pub tags: String,
    pub category: String,
    pub status: String,
    pub file_path: Option<String>,
    pub visibility: String,
    pub is_scheduled: bool,
    pub published_at: Option<String>,
    pub cover_image: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// 创建文章请求
#[derive(Debug, Deserialize)]
pub struct CreatePassageRequest {
    pub title: String,
    pub content: String,
    pub summary: Option<String>,
    pub author: Option<String>,
    pub tags: Option<String>,
    pub category: Option<String>,
    pub status: Option<String>,
    pub file_path: Option<String>,
    pub visibility: Option<String>,
    pub is_scheduled: Option<bool>,
    pub published_at: Option<String>,
    pub cover_image: Option<String>,
    pub created_at: Option<String>,
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
    pub cover_image: Option<String>,
}

/// 获取文章列表（公开）
pub async fn list(
    repo: web::Data<Arc<dyn Repository>>,
    query: web::Query<std::collections::HashMap<String, String>>,
    app_cache: web::Data<Arc<crate::cache::AppCache>>,
) -> HttpResponse {
    let passage_repo = PassageRepository::new(repo.get_pool().clone());

    // 解析并验证分页参数
    let limit: i64 = query.get("limit")
        .and_then(|l| l.parse::<i64>().ok())
        .filter(|&l| l > 0 && l <= 1000)
        .unwrap_or(10);

    let page: i64 = query.get("page")
        .and_then(|p| p.parse::<i64>().ok())
        .filter(|&p| p > 0)
        .unwrap_or(1);

    let offset = (page - 1) * limit;

    // 解析日期筛选参数
    let year: Option<i32> = query.get("year")
        .and_then(|y| y.parse::<i32>().ok());

    let month: Option<i32> = query.get("month")
        .and_then(|m| m.parse::<i32>().ok())
        .filter(|&m| m >= 1 && m <= 12);

    let day: Option<i32> = query.get("day")
        .and_then(|d| d.parse::<i32>().ok())
        .filter(|&d| d >= 1 && d <= 31);

    // 生成缓存键（包含日期参数）
    let date_part = match (year, month, day) {
        (Some(y), Some(m), Some(d)) => format!("{}-{:02}-{:02}", y, m, d),
        (Some(y), Some(m), None) => format!("{}-{:02}", y, m),
        (Some(y), None, None) => format!("{}", y),
        _ => "all".to_string(),
    };

    // 检查是否使用游标分页
    let cursor_param = query.get("cursor");
    let use_cursor = cursor_param.is_some();

    // 生成缓存键（包含日期和游标参数）
    let cache_key = if use_cursor {
        if let Some(cursor) = cursor_param {
            format!("passage:list:{}:cursor:{}:limit:{}", date_part, cursor, limit)
        } else {
            format!("passage:list:{}:cursor:first:limit:{}", date_part, limit)
        }
    } else {
        format!("passage:list:{}:page:{}:limit:{}", date_part, page, limit)
    };

    // 尝试从缓存获取
    if let Some(manager) = app_cache.manager() {
        if let Some(cached_data) = manager.get(&cache_key).await {
            if let Ok(response) = serde_json::from_str::<serde_json::Value>(&cached_data) {
                return HttpResponse::Ok()
                    .insert_header(("Cache-Control", "public, max-age=60"))
                    .insert_header(("X-Cache", "HIT"))
                    .json(response);
            }
        }
    }

    // 缓存未命中，从数据库获取
    if use_cursor {
        // 游标分页
        let cursor = query.get("cursor").map(|s| s.to_string());
        match passage_repo.get_published_cursor(cursor, limit).await {
            Ok((passages, next_cursor)) => {
                let data: Vec<PassageResponse> = passages.into_iter()
                    .map(|p| PassageResponse {
                        id: p.id.unwrap_or(0),
                        uuid: p.uuid.unwrap_or_default(),
                        title: p.title,
                        content: p.original_content.unwrap_or_default(),
                        html_content: None,
                        summary: p.summary,
                        author: p.author,
                        tags: p.tags,
                        category: p.category,
                        status: p.status,
                        file_path: p.file_path,
                        visibility: p.visibility,
                        is_scheduled: p.is_scheduled,
                        published_at: p.published_at.map(|d: chrono::DateTime<chrono::Utc>| d.format("%Y-%m-%d %H:%M:%S").to_string()),
                        cover_image: p.cover_image,
                        created_at: p.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                        updated_at: p.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                    })
                    .collect();

                let response = serde_json::json!({
                    "success": true,
                    "data": data,
                    "pagination": {
                        "has_more": next_cursor.is_some() && data.len() >= limit as usize,
                        "next_cursor": next_cursor,
                        "limit": limit
                    }
                });

                tracing::debug!("游标分页响应: 数据数量={}, next_cursor={:?}, has_more={}",
                    data.len(), next_cursor, next_cursor.is_some() && data.len() >= limit as usize);

                // 存储到缓存（TTL 5 分钟）
                if let Some(manager) = app_cache.manager() {
                    if let Ok(json_str) = serde_json::to_string(&response) {
                        let _ = manager.set(&cache_key, &json_str).await;
                    }
                }

                HttpResponse::Ok()
                    .insert_header(("Cache-Control", "public, max-age=60"))
                    .insert_header(("X-Cache", "MISS"))
                    .json(response)
            }
            Err(e) => {
                tracing::error!("获取文章列表失败: {}", e);
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "success": false,
                    "message": "获取文章列表失败"
                }))
            }
        }
    } else {
        // 传统分页
        let result = if year.is_some() || month.is_some() || day.is_some() {
            passage_repo.get_published_by_date(year, month, day, limit, offset).await
        } else {
            passage_repo.get_published(limit, offset).await
        };

        match result {
            Ok(passages) => {
                // 传统分页响应
                // 获取总数（总是实时查询，不使用缓存）
                let total: i64 = if year.is_some() || month.is_some() || day.is_some() {
                    match passage_repo.count_published_by_date(year, month, day).await {
                        Ok(c) => c,
                        Err(_) => passages.len() as i64,
                    }
                } else {
                    match passage_repo.count_published().await {
                        Ok(c) => c,
                        Err(_) => passages.len() as i64,
                    }
                };

                // 如果当前页超出了实际页数，返回空数据
                let total_pages = (total + limit - 1) / limit;
                if page > total_pages && total_pages > 0 {
                    return HttpResponse::Ok().json(serde_json::json!({
                        "success": true,
                        "data": [],
                        "pagination": {
                            "page": page,
                            "limit": limit,
                            "total": total,
                            "total_pages": total_pages,
                            "has_more": false
                        }
                    }));
                }

                // 计算下一页游标（使用最后一条记录）
                let next_cursor = passages.last().map(|p| {
                    format!("{}|{}", p.created_at.format("%Y-%m-%d %H:%M:%S%:z"), p.id.unwrap_or(0))
                });

                let data: Vec<PassageResponse> = passages.into_iter()
                    .map(|p| PassageResponse {
                        id: p.id.unwrap_or(0),
                        uuid: p.uuid.unwrap_or_default(),
                        title: p.title,
                        content: p.original_content.unwrap_or_default(),
                        html_content: None,
                        summary: p.summary,
                        author: p.author,
                        tags: p.tags,
                        category: p.category,
                        status: p.status,
                        file_path: p.file_path,
                        visibility: p.visibility,
                        is_scheduled: p.is_scheduled,
                        published_at: p.published_at.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()),
                        cover_image: p.cover_image,
                        created_at: p.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                        updated_at: p.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                    })
                    .collect();

                let response = serde_json::json!({
                    "success": true,
                    "data": data,
                    "pagination": {
                        "page": page,
                        "limit": limit,
                        "total": total,
                        "total_pages": total_pages,
                        "has_more": page < total_pages,
                        "next_cursor": next_cursor
                    }
                });

                // 存储到缓存（TTL 5 分钟）
                if let Some(manager) = app_cache.manager() {
                    if let Ok(json_str) = serde_json::to_string(&response) {
                        let _ = manager.set(&cache_key, &json_str).await;
                    }
                }

                HttpResponse::Ok()
                    .insert_header(("Cache-Control", "public, max-age=60"))
                    .insert_header(("X-Cache", "MISS"))
                    .json(response)
            }
            Err(e) => {
                tracing::error!("获取文章列表失败: {}", e);
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "success": false,
                    "message": "获取文章列表失败"
                }))
            }
        }
    }
}

/// 获取单篇文章
pub async fn get(
    repo: web::Data<Arc<dyn Repository>>,
    path: web::Path<String>,
    req: HttpRequest,
    view_batch_processor: web::Data<Arc<ViewBatchProcessor>>,
    app_cache: web::Data<Arc<crate::cache::AppCache>>,
) -> HttpResponse {
    let param = path.into_inner();
    let passage_repo = PassageRepository::new(repo.get_pool().clone());

    // 获取用户角色
    let role: String = req.extensions().get::<crate::middleware::auth::RoleKey>()
        .map(|r| r.0.clone())
        .unwrap_or_else(|| String::new());

    // 智能识别：如果是纯数字且较小（< 1000000），则按 ID 查询；否则按 UUID 查询
    let passage = if let Ok(id) = param.parse::<i64>() {
        // 只对较小的数字 ID 进行 ID 查询（避免将 Snowflake UUID 误识别为 ID）
        if id < 1_000_000 {
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
            // 数字太大，视为 UUID
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
        if role != "admin" || role.is_empty() {
            return HttpResponse::Ok().json(serde_json::json!({
                "success": false,
                "message": "文章未发布",
                "status": passage.status
            }));
        }
    }

    if passage.visibility != "public" {
        if role != "admin" || role.is_empty() {
            return HttpResponse::Ok().json(serde_json::json!({
                "success": false,
                "message": "文章不可见",
                "visibility": passage.visibility
            }));
        }
    }

    if passage.is_scheduled {
        if let Some(published_at) = passage.published_at {
            if published_at > Utc::now() && (role != "admin" || role.is_empty()) {
                return HttpResponse::Ok().json(serde_json::json!({
                    "success": false,
                    "message": "文章尚未发布",
                    "is_scheduled": true,
                    "published_at": published_at.format("%Y-%m-%d %H:%M:%S").to_string()
                }));
            }
        }
    }

    // 生成缓存键
    let cache_key = format!("passage:get:{}", param);

    // 尝试从缓存获取（仅对公开文章缓存）
    if passage.status == "published" && passage.visibility == "public" {
        if let Some(manager) = app_cache.manager() {
            if let Some(cached_data) = manager.get(&cache_key).await {
                if let Ok(response) = serde_json::from_str::<serde_json::Value>(&cached_data) {
                    return HttpResponse::Ok()
                        .insert_header(("Cache-Control", "public, max-age=300"))
                        .insert_header(("X-Cache", "HIT"))
                        .json(response);
                }
            }
        }
    }

    // 缓存未命中，继续处理

    // 使用批量处理器记录文章阅读（不阻塞响应）
    let passage_uuid = passage.uuid.clone().unwrap_or_default();
    let user_agent = req.headers().get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    // 获取客户端IP（简化版）
    let ip = "127.0.0.1".to_string(); // TODO: 从请求中获取真实IP

    // 过滤本地IP，不记录
    if !is_local_ip(&ip) {
        // 使用 GeoIP 获取地理位置信息
        let geo_location = crate::geoip::lookup_ip(&ip);
        let country = geo_location.country;
        let city = geo_location.city;
        let region = geo_location.region;

        // 使用批量处理器发送阅读记录
        let view_record = ViewRecord {
            passage_uuid: passage_uuid.clone(),
            ip: ip.clone(),
            user_agent: Some(user_agent.clone()),
            country,
            city,
            region,
            view_time: Utc::now(),
        };

        if let Err(e) = view_batch_processor.record_view(view_record) {
            eprintln!("发送阅读记录到批量处理器失败: {}", e);
        }
    }

    let response = PassageResponse {
        id: passage.id.unwrap_or(0),
        uuid: passage.uuid.unwrap_or_default(),
        title: passage.title,
        content: passage.original_content.unwrap_or_default(),
        html_content: Some(passage.content),
        summary: passage.summary,
        author: passage.author,
        tags: passage.tags,
        category: passage.category,
        status: passage.status.clone(),
        file_path: passage.file_path,
        visibility: passage.visibility.clone(),
        is_scheduled: passage.is_scheduled,
        published_at: passage.published_at.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()),
        cover_image: passage.cover_image,
        created_at: passage.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        updated_at: passage.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
    };

    // 生成 ETag
    use md5::{Md5, Digest};
    let etag_data = format!("{}:{}", response.id, response.updated_at);
    let etag = format!("\"{:x}\"", Md5::digest(etag_data.as_bytes()));

    // 检查 If-None-Match
    if let Some(if_none_match) = req.headers().get("if-none-match") {
        if let Ok(if_none_match_str) = if_none_match.to_str() {
            if if_none_match_str == etag {
                return HttpResponse::NotModified()
                    .insert_header(("ETag", etag))
                    .finish();
            }
        }
    }

    let response_json = serde_json::json!({
        "success": true,
        "data": response
    });

    // 存储到缓存（仅对公开文章，TTL 10 分钟）
    if passage.status == "published" && passage.visibility == "public" {
        if let Some(manager) = app_cache.manager() {
            if let Ok(json_str) = serde_json::to_string(&response_json) {
                let _ = manager.set(&cache_key, &json_str).await;
            }
        }
    }

    HttpResponse::Ok()
        .insert_header(("ETag", etag))
        .insert_header(("Cache-Control", "public, max-age=300"))
        .insert_header(("X-Cache", "MISS"))
        .json(response_json)
}

/// 创建文章
pub async fn create(
    repo: web::Data<Arc<dyn Repository>>,
    req_json: web::Json<CreatePassageRequest>,
    app_cache: web::Data<Arc<crate::cache::AppCache>>,
    req: HttpRequest,
) -> HttpResponse {
    let passage_repo = PassageRepository::new(repo.get_pool().clone());

    // 获取用户信息用于审计日志
    let user_id = req.extensions().get::<crate::middleware::auth::UserIDKey>()
        .map(|u| u.0)
        .unwrap_or(0);
    let username = req.extensions().get::<crate::middleware::auth::UsernameKey>()
        .map(|u| u.0.clone())
        .unwrap_or_else(|| "unknown".to_string());
    let client_ip = req.connection_info().realip_remote_addr()
        .map(|addr| addr.to_string());

    // 转换 Markdown 为 HTML
    let html_content = convert_markdown_to_html(&req_json.content);

    // 处理分类，确保分类存在
    let category_name = req_json.category.as_deref().unwrap_or("未分类");
    let _ = ensure_category_exist(category_name).await;

    // 处理标签
    let tags_json = if let Some(ref tags) = req_json.tags {
        // 解析标签：支持 JSON 数组和逗号分隔的字符串
        let tag_list: Vec<String> = if tags.trim().starts_with('[') {
            // JSON 格式
            serde_json::from_str(tags).unwrap_or_default()
        } else {
            // 逗号分隔格式
            tags.split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect()
        };

        // 确保标签存在
        if !tag_list.is_empty() {
            let _ = ensure_tags_exist(&tag_list).await;
        }

        // 返回 JSON 格式的标签列表
        serde_json::to_string(&tag_list).unwrap_or_else(|_| "[]".to_string())
    } else {
        "[]".to_string()
    };

    let now = Utc::now();

    // 如果没有提供 file_path，则自动生成
    let file_path = if let Some(ref path) = req_json.file_path {
        path.clone()
    } else {
        // 自动生成文件路径：markdown/YYYY/MM/DD/title.md
        let date = now.format("%Y/%m/%d").to_string();
        let safe_title = req_json.title.chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' { c } else { '_' })
            .collect::<String>()
            .replace(' ', "-");
        format!("markdown/{}/{}.md", date, safe_title)
    };

    // 创建 Markdown 文件
    if let Err(e) = update_markdown_file(&file_path, &req_json.content) {
        eprintln!("创建 Markdown 文件失败: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "message": format!("创建 Markdown 文件失败: {}", e)
        }));
    }

    // 如果没有提供摘要，则自动生成
    let summary = req_json.summary.clone().or_else(|| Some(extract_summary(&html_content)));

    // 如果提供了创建时间，使用指定的；否则使用当前时间
    let created_at = req_json.created_at.as_ref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or(now);

    let passage = Passage {
        id: None,
        uuid: None,
        title: req_json.title.clone(),
        content: html_content,
        original_content: Some(req_json.content.clone()),
        summary: summary,
        author: req_json.author.clone().unwrap_or_else(|| "Anonymous".to_string()),
        tags: tags_json,
        category: req_json.category.clone().unwrap_or_else(|| "未分类".to_string()),
        status: req_json.status.clone().unwrap_or_else(|| "draft".to_string()),
        file_path: Some(file_path),
        visibility: req_json.visibility.clone().unwrap_or_else(|| "public".to_string()),
        is_scheduled: req_json.is_scheduled.unwrap_or(false),
        published_at: req_json.published_at.as_ref().and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok()).map(|dt| dt.with_timezone(&Utc)),
        cover_image: req_json.cover_image.clone().or_else(|| Some("/img/passage-cover.webp".to_string())),
        created_at,
        updated_at: now,
    };

    match passage_repo.create(&passage).await {
        Ok(id) => {
            // 获取刚创建的文章信息
            match passage_repo.get_by_id(id).await {
                Ok(created_passage) => {
                    let uuid = created_passage.uuid.unwrap_or_else(|| String::new());
                    let status = created_passage.status.clone();
                    let title = created_passage.title.clone();

                    // 清除缓存
                    if status == "published" {
                        crate::cache::invalidate_all_passage_cache(app_cache.manager()).await;
                    }

                    // 记录审计日志
                    crate::audit::AUDIT_LOGGER.log_passage_create(
                        user_id,
                        &username,
                        id,
                        &uuid,
                        &title,
                        client_ip,
                    );

                    HttpResponse::Ok().json(serde_json::json!({
                        "success": true,
                        "message": "文章创建成功",
                        "data": {
                            "id": id,
                            "uuid": uuid
                        }
                    }))
                }
                Err(e) => {
                    eprintln!("获取创建的文章失败: {}", e);
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "success": false,
                        "message": "文章创建成功但无法获取详情"
                    }))
                }
            }
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
    req_json: web::Json<UpdatePassageRequest>,
    app_cache: web::Data<Arc<crate::cache::AppCache>>,
    req: HttpRequest,
) -> HttpResponse {
    let id = path.into_inner();
    let passage_repo = PassageRepository::new(repo.get_pool().clone());

    // 获取用户信息用于审计日志
    let user_id = req.extensions().get::<crate::middleware::auth::UserIDKey>()
        .map(|u| u.0)
        .unwrap_or(0);
    let username = req.extensions().get::<crate::middleware::auth::UsernameKey>()
        .map(|u| u.0.clone())
        .unwrap_or_else(|| "unknown".to_string());
    let client_ip = req.connection_info().realip_remote_addr()
        .map(|addr| addr.to_string());

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

    // 保存原始 UUID 用于缓存失效
    let passage_uuid = passage.uuid.clone();

    // 更新字段
    let mut file_updated = false;
    if let Some(ref title) = req_json.title {
        passage.title = title.clone();
        file_updated = true;
    }
    if let Some(ref content) = req_json.content {
        // 转换 Markdown 为 HTML
        let html_content = convert_markdown_to_html(content);
        passage.content = html_content;
        passage.original_content = Some(content.clone());
        file_updated = true;
    }
    if let Some(ref original_content) = req_json.original_content {
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
            if let Some(ref title) = req_json.title {
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
    if let Some(ref summary) = req_json.summary {
        passage.summary = Some(summary.clone());
    }
    if let Some(ref author) = req_json.author {
        passage.author = author.clone();
    }
    if let Some(ref category) = req_json.category {
        // 确保分类存在
        let _ = ensure_category_exist(category).await;
        passage.category = category.clone();
    }
    if let Some(ref tags) = req_json.tags {
        // 解析标签：支持 JSON 数组和逗号分隔的字符串
        let tag_list: Vec<String> = if tags.trim().starts_with('[') {
            // JSON 格式
            serde_json::from_str(tags).unwrap_or_default()
        } else {
            // 逗号分隔格式
            tags.split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect()
        };

        // 确保标签存在
        if !tag_list.is_empty() {
            let _ = ensure_tags_exist(&tag_list).await;
        }

        // 保存为 JSON 格式
        passage.tags = serde_json::to_string(&tag_list).unwrap_or_else(|_| "[]".to_string());
    }
    if let Some(ref status) = req_json.status {
        passage.status = status.clone();
    }
    if let Some(ref file_path) = req_json.file_path {
        passage.file_path = Some(file_path.clone());
    }
    if let Some(ref visibility) = req_json.visibility {
        passage.visibility = visibility.clone();
    }
    if let Some(is_scheduled) = req_json.is_scheduled {
        passage.is_scheduled = is_scheduled;
    }
    if let Some(ref published_at) = req_json.published_at {
        passage.published_at = chrono::DateTime::parse_from_rfc3339(published_at)
            .ok()
            .map(|dt| dt.with_timezone(&chrono::Utc));
    }
    if let Some(ref cover_image) = req_json.cover_image {
        passage.cover_image = Some(cover_image.clone());
    }
    passage.updated_at = chrono::Utc::now();

    match passage_repo.update(&passage).await {
        Ok(_) => {
            // 失效相关缓存
            if let Some(uuid) = passage_uuid.clone() {
                let cache_keys = vec![
                    format!("passage:get:{}", uuid),
                    format!("passage:get:{}", id),
                    // 清除列表缓存（因为文章状态可能改变）
                    "passage:list:page:1:limit:10".to_string(),
                    "passage:list:page:1:limit:20".to_string(),
                ];

                if let Some(manager) = app_cache.manager() {
                    for key in cache_keys {
                        let _ = manager.delete(&key).await;
                    }
                }
            }

            // 记录审计日志
            if let Some(uuid) = passage_uuid {
                crate::audit::AUDIT_LOGGER.log_passage_update(
                    user_id,
                    &username,
                    id,
                    &uuid,
                    client_ip,
                );
            }

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
    app_cache: web::Data<Arc<crate::cache::AppCache>>,
    req: HttpRequest,
) -> HttpResponse {
    let uuid = path.into_inner();
    let passage_repo = PassageRepository::new(repo.get_pool().clone());
    let attachment_repo = AttachmentRepository::new(repo.get_pool().clone());

    // 获取用户信息用于审计日志
    let user_id = req.extensions().get::<crate::middleware::auth::UserIDKey>()
        .map(|u| u.0)
        .unwrap_or(0);
    let username = req.extensions().get::<crate::middleware::auth::UsernameKey>()
        .map(|u| u.0.clone())
        .unwrap_or_else(|| "unknown".to_string());
    let client_ip = req.connection_info().realip_remote_addr()
        .map(|addr| addr.to_string());

    // 1. 获取文章信息以获取文件路径
    let passage = match passage_repo.get_by_uuid(&uuid).await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("获取文章信息失败: {}", e);
            return HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "message": "文章不存在"
            }));
        }
    };

    // 2. 删除 Markdown 文件
    let mut deleted_markdown = false;
    if let Some(file_path) = &passage.file_path {
        if let Err(e) = std::fs::remove_file(file_path) {
            eprintln!("删除 Markdown 文件失败 {}: {}", file_path, e);
        } else {
            deleted_markdown = true;
        }
    }

    // 3. 查询关联的附件
    let attachments = match attachment_repo.get_by_passage_uuids(vec![uuid.clone()]).await {
        Ok(attachments) => attachments,
        Err(e) => {
            eprintln!("查询附件失败: {}", e);
            Vec::new()
        }
    };

    // 4. 删除附件物理文件
    let mut deleted_files = 0;
    for attachment in &attachments {
        if let Err(e) = std::fs::remove_file(&attachment.file_path) {
            eprintln!("删除附件文件失败 {}: {}", attachment.file_path, e);
        } else {
            deleted_files += 1;
        }
    }

    // 5. 删除文章记录
    match passage_repo.delete_by_uuid(&uuid).await {
        Ok(_) => {
            // 清除缓存
            let cache_keys = vec![
                format!("passage:get:{}", uuid),
                if let Some(id) = passage.id { format!("passage:get:{}", id) } else { String::new() },
                "passage:list:page:1:limit:10".to_string(),
                "passage:list:page:1:limit:20".to_string(),
            ];

            if let Some(manager) = app_cache.manager() {
                for key in cache_keys {
                    if !key.is_empty() {
                        let _ = manager.delete(&key).await;
                    }
                }
            }

            // 记录审计日志
            crate::audit::AUDIT_LOGGER.log_passage_delete(
                user_id,
                &username,
                &uuid,
                &passage.title,
                client_ip,
            );

            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": format!("文章删除成功，删除了 {} 个 Markdown 文件，{} 个附件文件",
                    if deleted_markdown { 1 } else { 0 }, deleted_files)
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
    app_cache: web::Data<Arc<crate::cache::AppCache>>,
) -> HttpResponse {
    if req.ids.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "message": "文章ID列表不能为空"
        }));
    }

    let passage_repo = PassageRepository::new(repo.get_pool().clone());
    let attachment_repo = AttachmentRepository::new(repo.get_pool().clone());

    // 1. 批量获取文章信息（修复 N+1 查询问题）
    let passages = match passage_repo.get_by_ids(&req.ids).await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("批量获取文章信息失败: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "获取文章信息失败"
            }));
        }
    };

    // 一次性收集所有 uuid 和 file_path
    let uuids: Vec<String> = passages.iter()
        .filter_map(|p| p.uuid.clone())
        .collect();
    let file_paths: Vec<String> = passages.iter()
        .filter_map(|p| p.file_path.clone())
        .collect();

    // 2. 删除 Markdown 文件
    let mut deleted_markdown_files = 0;
    for file_path in &file_paths {
        if let Err(e) = std::fs::remove_file(file_path) {
            eprintln!("删除 Markdown 文件失败 {}: {}", file_path, e);
        } else {
            deleted_markdown_files += 1;
        }
    }

    // 3. 查询关联的附件
    let attachments = match attachment_repo.get_by_passage_uuids(uuids.clone()).await {
        Ok(attachments) => attachments,
        Err(e) => {
            eprintln!("查询附件失败: {}", e);
            // 即使查询附件失败，也继续删除文章
            Vec::new()
        }
    };

    // 4. 删除附件物理文件
    let mut deleted_files = 0;
    for attachment in &attachments {
        if let Err(e) = std::fs::remove_file(&attachment.file_path) {
            eprintln!("删除附件文件失败 {}: {}", attachment.file_path, e);
        } else {
            deleted_files += 1;
        }
    }

    // 5. 删除文章记录（会自动删除关联的数据库记录，通过 CASCADE）
    match passage_repo.delete_batch(req.ids.clone()).await {
        Ok(count) => {
            // 清除缓存（优化：使用批量删除）
            if let Some(manager) = app_cache.manager() {
                // 构造批量删除的键列表
                let mut cache_keys = Vec::new();
                for uuid in &uuids {
                    cache_keys.push(format!("passage:get:{}", uuid));
                }
                for id in &req.ids {
                    cache_keys.push(format!("passage:get:{}", id));
                }

                // 使用批量删除方法（性能优化：N 次删除 → 1 次批量删除）
                if let Err(e) = manager.delete_many(&cache_keys).await {
                    eprintln!("批量删除缓存失败，尝试逐个删除: {}", e);
                    // 回退到逐个删除
                    for key in &cache_keys {
                        let _ = manager.delete(key).await;
                    }
                }

                // 清除所有文章列表缓存（批量删除会影响所有分页）
                crate::cache::invalidate_all_passage_cache(app_cache.manager()).await;
            }

            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": format!("成功删除 {} 篇文章，{} 个 Markdown 文件，{} 个附件文件", count, deleted_markdown_files, deleted_files),
                "deleted_count": count,
                "deleted_markdown_files": deleted_markdown_files,
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