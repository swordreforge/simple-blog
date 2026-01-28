use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use crate::db::repositories::{CategoryRepository, PassageRepository, Repository};
use crate::db::models::Category;
use std::sync::Arc;
use chrono::Utc;

/// 分类响应
#[derive(Debug, Serialize)]
pub struct CategoryResponse {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub sort_order: i32,
    pub is_enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// 创建分类请求
#[derive(Debug, Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub sort_order: Option<i32>,
    pub is_enabled: Option<bool>,
}

/// 更新分类请求
#[derive(Debug, Deserialize)]
pub struct UpdateCategoryRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub sort_order: Option<i32>,
    pub is_enabled: Option<bool>,
}

/// 获取所有分类（管理员）
pub async fn admin_list(
    repo: web::Data<Arc<dyn Repository>>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    let category_repo = CategoryRepository::new(repo.get_pool().clone());
    
    // 解析分页参数
    let limit: i64 = query.get("limit").and_then(|l| l.parse().ok()).unwrap_or(100);
    let offset: i64 = query.get("offset").and_then(|o| o.parse().ok()).unwrap_or(0);
    
    // 获取所有分类
    let categories = match category_repo.get_all(limit, offset).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("获取分类失败: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "获取分类失败"
            }));
        }
    };
    
    // 获取总数
    let total = match category_repo.count().await {
        Ok(c) => c,
        Err(_) => categories.len() as i64,
    };
    
    // 转换为响应格式
    let data: Vec<CategoryResponse> = categories.into_iter()
        .map(|cat| CategoryResponse {
            id: cat.id.unwrap_or(0),
            name: cat.name,
            description: cat.description,
            icon: cat.icon,
            sort_order: cat.sort_order,
            is_enabled: cat.is_enabled,
            created_at: cat.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: cat.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        })
        .collect();
    
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": data,
        "total": total
    }))
}

/// 获取所有分类（公开）
pub async fn list(repo: web::Data<Arc<dyn Repository>>) -> HttpResponse {
    let passage_repo = PassageRepository::new(repo.get_pool().clone());
    
    // 从数据库获取所有分类
    match passage_repo.get_all_categories().await {
        Ok(categories) => {
            // 转换为API响应格式
            let data: Vec<CategoryResponse> = categories.into_iter()
                .enumerate()
                .map(|(i, name)| CategoryResponse {
                    id: (i + 1) as i64,
                    name,
                    description: String::new(),
                    icon: String::new(),
                    sort_order: (i + 1) as i32,
                    is_enabled: true,
                    created_at: Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    updated_at: Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                })
                .collect();
            
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "data": data
            }))
        }
        Err(e) => {
            eprintln!("获取分类失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "获取分类失败"
            }))
        }
    }
}

/// 根据 ID 获取分类
pub async fn get(
    repo: web::Data<Arc<dyn Repository>>,
    path: web::Path<i64>,
) -> HttpResponse {
    let id = path.into_inner();
    let category_repo = CategoryRepository::new(repo.get_pool().clone());
    
    match category_repo.get_by_id(id).await {
        Ok(category) => {
            let response = CategoryResponse {
                id: category.id.unwrap_or(0),
                name: category.name,
                description: category.description,
                icon: category.icon,
                sort_order: category.sort_order,
                is_enabled: category.is_enabled,
                created_at: category.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                updated_at: category.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            };
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "data": response
            }))
        }
        Err(_) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "message": "分类不存在"
            }))
        }
    }
}

/// 创建分类
pub async fn create(
    repo: web::Data<Arc<dyn Repository>>,
    req: web::Json<CreateCategoryRequest>,
) -> HttpResponse {
    let category_repo = CategoryRepository::new(repo.get_pool().clone());
    
    let now = Utc::now();
    let category = Category {
        id: None,
        name: req.name.clone(),
        description: req.description.clone().unwrap_or_default(),
        icon: req.icon.clone().unwrap_or_default(),
        sort_order: req.sort_order.unwrap_or(0),
        is_enabled: req.is_enabled.unwrap_or(true),
        created_at: now,
        updated_at: now,
    };
    
    match category_repo.create(&category).await {
        Ok(id) => {
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "分类创建成功",
                "data": serde_json::json!({"id": id})
            }))
        }
        Err(e) => {
            eprintln!("创建分类失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "创建分类失败"
            }))
        }
    }
}

/// 更新分类
pub async fn update(
    repo: web::Data<Arc<dyn Repository>>,
    path: web::Path<i64>,
    req: web::Json<UpdateCategoryRequest>,
) -> HttpResponse {
    let id = path.into_inner();
    let category_repo = CategoryRepository::new(repo.get_pool().clone());
    
    // 先获取现有分类
    let mut category = match category_repo.get_by_id(id).await {
        Ok(c) => c,
        Err(_) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "message": "分类不存在"
            }));
        }
    };
    
    // 更新字段
    if let Some(ref name) = req.name {
        category.name = name.clone();
    }
    if let Some(ref description) = req.description {
        category.description = description.clone();
    }
    if let Some(ref icon) = req.icon {
        category.icon = icon.clone();
    }
    if let Some(sort_order) = req.sort_order {
        category.sort_order = sort_order;
    }
    if let Some(is_enabled) = req.is_enabled {
        category.is_enabled = is_enabled;
    }
    category.updated_at = Utc::now();
    
    match category_repo.update(&category).await {
        Ok(_) => {
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "分类更新成功"
            }))
        }
        Err(e) => {
            eprintln!("更新分类失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "更新分类失败"
            }))
        }
    }
}

/// 删除分类
pub async fn delete(
    repo: web::Data<Arc<dyn Repository>>,
    path: web::Path<i64>,
) -> HttpResponse {
    let id = path.into_inner();
    let category_repo = CategoryRepository::new(repo.get_pool().clone());
    
    match category_repo.delete(id).await {
        Ok(_) => {
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "分类删除成功"
            }))
        }
        Err(e) => {
            eprintln!("删除分类失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "删除分类失败"
            }))
        }
    }
}