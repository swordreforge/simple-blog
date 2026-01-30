use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use crate::db::repositories::{TagRepository, PassageRepository, Repository};
use crate::db::models::Tag;
use std::sync::Arc;
use chrono::Utc;

/// 标签响应
#[derive(Debug, Serialize)]
pub struct TagResponse {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub color: String,
    pub category_id: i64,
    pub sort_order: i32,
    pub is_enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// 标签统计响应
#[derive(Debug, Serialize)]
pub struct TagCountResponse {
    pub id: i32,
    pub name: String,
    pub count: i32,
}

/// 创建标签请求
#[derive(Debug, Deserialize)]
pub struct CreateTagRequest {
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub category_id: Option<i64>,
    pub sort_order: Option<i32>,
    pub is_enabled: Option<bool>,
}

/// 更新标签请求
#[derive(Debug, Deserialize)]
pub struct UpdateTagRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub category_id: Option<i64>,
    pub sort_order: Option<i32>,
    pub is_enabled: Option<bool>,
}

/// 获取所有标签及使用次数（公开）
pub async fn list(repo: web::Data<Arc<dyn Repository>>) -> HttpResponse {
    let passage_repo = PassageRepository::new(repo.get_pool().clone());
    
    // 从数据库获取所有文章的标签
    let passages = match passage_repo.get_all(1000, 0).await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("获取标签失败: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "获取标签失败"
            }));
        }
    };
    
    // 统计标签使用次数
    let mut tag_count: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
    for passage in passages {
        let tags = parse_tags(&passage.tags);
        for tag in tags {
            *tag_count.entry(tag).or_insert(0) += 1;
        }
    }
    
    // 转换为API响应格式
    let data: Vec<TagCountResponse> = tag_count.into_iter()
        .enumerate()
        .map(|(i, (name, count))| TagCountResponse {
            id: (i + 1) as i32,
            name,
            count,
        })
        .collect();
    
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": data
    }))
}

/// 获取所有标签（管理员）
pub async fn admin_list(
    repo: web::Data<Arc<dyn Repository>>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    let tag_repo = TagRepository::new(repo.get_pool().clone());
    
    // 解析分页参数
    let limit: i64 = query.get("limit").and_then(|l| l.parse().ok()).unwrap_or(100);
    let offset: i64 = query.get("offset").and_then(|o| o.parse().ok()).unwrap_or(0);
    
    // 获取所有标签
    let tags = match tag_repo.get_all(limit, offset).await {
        Ok(t) => t,
        Err(e) => {
            eprintln!("获取标签失败: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "获取标签失败"
            }));
        }
    };
    
    // 获取总数
    let total = match tag_repo.count().await {
        Ok(c) => c,
        Err(_) => tags.len() as i64,
    };
    
    // 转换为响应格式
    let data: Vec<TagResponse> = tags.into_iter()
        .map(|tag| TagResponse {
            id: tag.id.unwrap_or(0),
            name: tag.name,
            description: tag.description,
            color: tag.color,
            category_id: tag.category_id,
            sort_order: tag.sort_order,
            is_enabled: tag.is_enabled,
            created_at: tag.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: tag.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        })
        .collect();
    
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": data,
        "total": total
    }))
}

/// 根据 ID 获取标签
pub async fn get(
    repo: web::Data<Arc<dyn Repository>>,
    path: web::Path<i64>,
) -> HttpResponse {
    let id = path.into_inner();
    let tag_repo = TagRepository::new(repo.get_pool().clone());
    
    match tag_repo.get_by_id(id).await {
        Ok(tag) => {
            let response = TagResponse {
                id: tag.id.unwrap_or(0),
                name: tag.name,
                description: tag.description,
                color: tag.color,
                category_id: tag.category_id,
                sort_order: tag.sort_order,
                is_enabled: tag.is_enabled,
                created_at: tag.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                updated_at: tag.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            };
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "data": response
            }))
        }
        Err(_) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "message": "标签不存在"
            }))
        }
    }
}

/// 创建标签
pub async fn create(
    repo: web::Data<Arc<dyn Repository>>,
    req: web::Json<CreateTagRequest>,
) -> HttpResponse {
    let tag_repo = TagRepository::new(repo.get_pool().clone());
    
    let now = Utc::now();
    let tag = Tag {
        id: None,
        name: req.name.clone(),
        description: req.description.clone().unwrap_or_default(),
        color: req.color.clone().unwrap_or_default(),
        category_id: req.category_id.unwrap_or(0),
        sort_order: req.sort_order.unwrap_or(0),
        is_enabled: req.is_enabled.unwrap_or(true),
        created_at: now,
        updated_at: now,
    };
    
    match tag_repo.create(&tag).await {
        Ok(id) => {
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "标签创建成功",
                "data": serde_json::json!({"id": id})
            }))
        }
        Err(e) => {
            eprintln!("创建标签失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "创建标签失败"
            }))
        }
    }
}

/// 更新标签
pub async fn update(
    repo: web::Data<Arc<dyn Repository>>,
    path: web::Path<i64>,
    req: web::Json<UpdateTagRequest>,
) -> HttpResponse {
    let id = path.into_inner();
    let tag_repo = TagRepository::new(repo.get_pool().clone());
    
    // 先获取现有标签
    let mut tag = match tag_repo.get_by_id(id).await {
        Ok(t) => t,
        Err(_) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "message": "标签不存在"
            }));
        }
    };
    
    // 更新字段
    if let Some(ref name) = req.name {
        tag.name = name.clone();
    }
    if let Some(ref description) = req.description {
        tag.description = description.clone();
    }
    if let Some(ref color) = req.color {
        tag.color = color.clone();
    }
    if let Some(category_id) = req.category_id {
        tag.category_id = category_id;
    }
    if let Some(sort_order) = req.sort_order {
        tag.sort_order = sort_order;
    }
    if let Some(is_enabled) = req.is_enabled {
        tag.is_enabled = is_enabled;
    }
    tag.updated_at = Utc::now();
    
    match tag_repo.update(&tag).await {
        Ok(_) => {
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "标签更新成功"
            }))
        }
        Err(e) => {
            eprintln!("更新标签失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "更新标签失败"
            }))
        }
    }
}

/// 删除标签
pub async fn delete(
    repo: web::Data<Arc<dyn Repository>>,
    path: web::Path<i64>,
) -> HttpResponse {
    let id = path.into_inner();
    let tag_repo = TagRepository::new(repo.get_pool().clone());
    
    match tag_repo.delete(id).await {
        Ok(_) => {
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "标签删除成功"
            }))
        }
        Err(e) => {
            eprintln!("删除标签失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "删除标签失败"
            }))
        }
    }
}

/// 批量删除标签请求
#[derive(Debug, Deserialize)]
pub struct BatchDeleteRequest {
    pub ids: Vec<i64>,
}

/// 批量删除标签
pub async fn delete_batch(
    repo: web::Data<Arc<dyn Repository>>,
    req: web::Json<BatchDeleteRequest>,
) -> HttpResponse {
    if req.ids.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "message": "标签ID列表不能为空"
        }));
    }
    
    let tag_repo = TagRepository::new(repo.get_pool().clone());
    
    match tag_repo.delete_batch(req.ids.clone()).await {
        Ok(count) => {
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": format!("成功删除 {} 个标签", count),
                "deleted_count": count
            }))
        }
        Err(e) => {
            eprintln!("批量删除标签失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "批量删除标签失败"
            }))
        }
    }
}

/// 解析标签 JSON 字符串
fn parse_tags(tags_str: &str) -> Vec<String> {
    if tags_str.is_empty() || tags_str == "[]" {
        return Vec::new();
    }
    
    // 尝试解析 JSON
    if let Ok(tags) = serde_json::from_str::<Vec<String>>(tags_str) {
        return tags;
    }
    
    // 如果解析失败，按逗号分割
    tags_str.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}