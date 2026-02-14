use actix_web::{web, HttpResponse, HttpRequest};

use super::crud::{PassageResponse, UpdatePassageRequest};
use super::markdown::{convert_markdown_to_html, update_markdown_file, update_markdown_file_name};
use super::validation::{ensure_tags_exist, ensure_category_exist};

/// 通过查询参数更新文章（用于管理后台）
pub async fn update_by_query(
    state: web::Data<crate::app_state::AppState>,
    query: web::Query<std::collections::HashMap<String, String>>,
    req_json: web::Json<UpdatePassageRequest>,
    http_req: HttpRequest,
) -> HttpResponse {
    // 鉴权检查
    if http_req.cookie("auth_token").is_none() {
        return crate::middleware::auth::missing_token_response();
    }
    if crate::middleware::auth::check_admin_auth(&http_req).is_none() {
        return crate::middleware::auth::forbidden_response();
    }

    let passage_repo = state.passage_repository();

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

    // 记录原始值用于检测变化
    let original_status = passage.status;
    let original_visibility = passage.visibility;
    let original_tags = passage.tags.clone();
    let original_category = passage.category.clone();

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
            let content_to_save = passage.original_content.as_ref().unwrap_or({
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
    if let Some(ref category) = req_json.category {
        // 确保分类存在
        let _ = ensure_category_exist(category).await;
        passage.category = category.clone();
    }
    if let Some(ref status) = req_json.status {
        passage.status = crate::db::models::PassageStatus::from_str(status)
            .unwrap_or(passage.status);
    }
    if let Some(ref file_path) = req_json.file_path {
        passage.file_path = Some(file_path.clone());
    }
    if let Some(ref visibility) = req_json.visibility {
        passage.visibility = crate::db::models::PassageVisibility::from_str(visibility)
            .unwrap_or(passage.visibility);
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
            let uuid = passage.uuid.clone().unwrap_or_default();

            // 清除文章详情缓存（任何更新都需要）
            if let Some(manager) = state.cache.manager() {
                let detail_key = format!("passage:get:{}", uuid);
                let _ = manager.delete(&detail_key).await;
            }

            // 检测是否需要清除列表缓存
            let mut should_clear_list_cache = false;

            // 状态变更（draft ↔ publish）
            if req_json.status.is_some() && passage.status != original_status {
                should_clear_list_cache = true;
            }

            // 可见性变更
            if req_json.visibility.is_some() && passage.visibility != original_visibility {
                should_clear_list_cache = true;
            }

            // 定时发布状态变更
            if req_json.is_scheduled.is_some() {
                should_clear_list_cache = true;
            }

            // 标签变更（可能影响过滤）
            if req_json.tags.is_some() && passage.tags != original_tags {
                should_clear_list_cache = true;
            }

            // 分类变更（可能影响过滤）
            if req_json.category.is_some() && passage.category != original_category {
                should_clear_list_cache = true;
            }

            // 清除列表缓存（如果需要）
            if should_clear_list_cache {
                crate::cache::invalidate_all_passage_cache(state.cache.manager()).await;
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

/// 通过查询参数删除文章（用于管理后台）
pub async fn delete_by_query(
    state: web::Data<crate::app_state::AppState>,
    query: web::Query<std::collections::HashMap<String, String>>,
    http_req: HttpRequest,
) -> HttpResponse {
    // 鉴权检查
    if http_req.cookie("auth_token").is_none() {
        return crate::middleware::auth::missing_token_response();
    }
    if crate::middleware::auth::check_admin_auth(&http_req).is_none() {
        return crate::middleware::auth::forbidden_response();
    }

    let passage_repo = state.passage_repository();
    let attachment_repo = state.attachment_repository();

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

    // 获取文章信息（包含文件路径和 UUID）
    let passage = match passage_repo.get_by_id(id).await {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "message": format!("获取文章失败: {}", e)
            }));
        }
    };

    let uuid = match &passage.uuid {
        Some(u) => u.clone(),
        None => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "message": "文章 UUID 不存在"
            }));
        }
    };

    // 删除 Markdown 文件
    if let Some(file_path) = &passage.file_path {
        if let Err(e) = std::fs::remove_file(file_path) {
            eprintln!("删除 Markdown 文件失败 {}: {}", file_path, e);
        }
    }

    // 查询关联的附件
    let attachments = match attachment_repo.get_by_passage_uuids(vec![uuid.clone()]).await {
        Ok(attachments) => attachments,
        Err(e) => {
            eprintln!("查询附件失败: {}", e);
            Vec::new()
        }
    };

    // 删除附件物理文件
    let mut deleted_files = 0;
    for attachment in &attachments {
        if let Err(e) = std::fs::remove_file(&attachment.file_path) {
            eprintln!("删除附件文件失败 {}: {}", attachment.file_path, e);
        } else {
            deleted_files += 1;
        }
    }

    // 删除文章记录
    match passage_repo.delete_by_uuid(&uuid).await {
        Ok(_) => {
            // 清除文章详情缓存
            if let Some(manager) = state.cache.manager() {
                let detail_key = format!("passage:get:{}", uuid);
                let _ = manager.delete(&detail_key).await;
            }
            // 清除所有文章列表缓存（删除会影响分页）
            crate::cache::invalidate_all_passage_cache(state.cache.manager()).await;

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

/// 通过查询参数获取单篇文章或文章列表（用于管理后台）
pub async fn get_by_query(
    state: web::Data<crate::app_state::AppState>,
    query: web::Query<std::collections::HashMap<String, String>>,
    http_req: HttpRequest,
) -> HttpResponse {
    // 鉴权检查
    if http_req.cookie("auth_token").is_none() {
        return crate::middleware::auth::missing_token_response();
    }
    if crate::middleware::auth::check_admin_auth(&http_req).is_none() {
        return crate::middleware::auth::forbidden_response();
    }

    let passage_repo = state.passage_repository();

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
                    content: passage.original_content.unwrap_or_default(),
                    html_content: Some(passage.content),
                    summary: passage.summary,
                    author: passage.author,
                    tags: passage.tags,
                    category: passage.category,
                    status: passage.status,
                    file_path: passage.file_path,
                    visibility: passage.visibility,
                    is_scheduled: passage.is_scheduled,
                    published_at: passage.published_at.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()),
                    cover_image: passage.cover_image,
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
        // 如果没有 id 参数，返回文章列表
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
                let total_pages = (total + limit - 1) / limit;
                let has_more = page < total_pages;

                let data: Vec<PassageResponse> = passages.into_iter()
                    .map(|p| PassageResponse {
                        id: p.id.unwrap_or(0),
                        uuid: p.uuid.unwrap_or_default(),
                        title: p.title,
                        content: p.original_content.unwrap_or_default(),
                        html_content: Some(p.content),
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

                HttpResponse::Ok().json(serde_json::json!({
                    "success": true,
                    "data": data,
                    "pagination": {
                        "page": page,
                        "limit": limit,
                        "total": total,
                        "total_pages": total_pages,
                        "has_more": has_more
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
}