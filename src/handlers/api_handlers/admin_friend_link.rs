use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use crate::db::repositories::{FriendLinkRepository, Repository};
use std::sync::Arc;

/// 友链列表请求参数
#[derive(Debug, Deserialize)]
pub struct AdminFriendLinkListQuery {
    pub page: Option<usize>,
    pub page_size: Option<usize>,
    pub include_disabled: Option<bool>,
}

/// 创建友链请求
#[derive(Debug, Deserialize)]
pub struct CreateFriendLinkRequest {
    pub nickname: String,
    pub link_url: String,
    pub avatar_url: String,
    pub motto: String,
    pub sort_order: Option<i32>,
    pub is_enabled: Option<bool>,
}

/// 更新友链请求
#[derive(Debug, Deserialize)]
pub struct UpdateFriendLinkRequest {
    pub nickname: String,
    pub link_url: String,
    pub avatar_url: String,
    pub motto: String,
    pub sort_order: Option<i32>,
    pub is_enabled: Option<bool>,
}

/// 友链响应
#[derive(Debug, Serialize)]
pub struct FriendLinkResponse {
    pub id: i64,
    pub nickname: String,
    pub link_url: String,
    pub avatar_url: String,
    pub motto: String,
    pub sort_order: i32,
    pub is_enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// 通用响应
#[derive(Debug, Serialize)]
pub struct CommonResponse {
    pub success: bool,
    pub message: String,
}

/// 分页响应
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub success: bool,
    pub data: Vec<T>,
    pub total: i64,
    pub page: usize,
    pub page_size: usize,
}

/// 获取友链列表（Admin）
pub async fn list(
    query: web::Query<AdminFriendLinkListQuery>,
    repo: web::Data<Arc<dyn Repository>>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    // 鉴权检查
    if req.cookie("auth_token").is_none() {
        return crate::middleware::auth::missing_token_response();
    }
    if crate::middleware::auth::check_admin_auth(&req).is_none() {
        return crate::middleware::auth::forbidden_response();
    }

    let friend_link_repo = FriendLinkRepository::new(repo.get_pool().clone());
    
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    
    let links = if query.include_disabled.unwrap_or(true) {
        friend_link_repo.get_all_including_disabled().await
    } else {
        friend_link_repo.get_all().await
    };
    
    match links {
        Ok(all_links) => {
            let total = all_links.len() as i64;
            
            // 简单分页
            let start = (page - 1) * page_size;
            let end = std::cmp::min(start + page_size, all_links.len());
            let page_data: Vec<FriendLinkResponse> = if start < all_links.len() {
                all_links[start..end].iter().map(|l| FriendLinkResponse {
                    id: l.id.unwrap_or(0),
                    nickname: l.nickname.clone(),
                    link_url: l.link_url.clone(),
                    avatar_url: l.avatar_url.clone(),
                    motto: l.motto.clone(),
                    sort_order: l.sort_order,
                    is_enabled: l.is_enabled,
                    created_at: l.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                    updated_at: l.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                }).collect()
            } else {
                vec![]
            };
            
            HttpResponse::Ok().json(PaginatedResponse {
                success: true,
                data: page_data,
                total,
                page,
                page_size,
            })
        }
        Err(_) => HttpResponse::InternalServerError().json(CommonResponse {
            success: false,
            message: "获取友链列表失败".to_string(),
        })
    }
}

/// 获取单个友链（Admin）
pub async fn get(
    path: web::Path<i64>,
    repo: web::Data<Arc<dyn Repository>>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    // 鉴权检查
    if req.cookie("auth_token").is_none() {
        return crate::middleware::auth::missing_token_response();
    }
    if crate::middleware::auth::check_admin_auth(&req).is_none() {
        return crate::middleware::auth::forbidden_response();
    }

    let id = path.into_inner();
    let friend_link_repo = FriendLinkRepository::new(repo.get_pool().clone());
    
    match friend_link_repo.get_by_id(id).await {
        Ok(Some(link)) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "data": FriendLinkResponse {
                id: link.id.unwrap_or(0),
                nickname: link.nickname,
                link_url: link.link_url,
                avatar_url: link.avatar_url,
                motto: link.motto,
                sort_order: link.sort_order,
                is_enabled: link.is_enabled,
                created_at: link.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                updated_at: link.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            }
        })),
        Ok(None) => HttpResponse::NotFound().json(CommonResponse {
            success: false,
            message: "友链不存在".to_string(),
        }),
        Err(_) => HttpResponse::InternalServerError().json(CommonResponse {
            success: false,
            message: "获取友链失败".to_string(),
        })
    }
}

/// 创建友链（Admin）
pub async fn create(
    req_json: web::Json<CreateFriendLinkRequest>,
    repo: web::Data<Arc<dyn Repository>>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    // 鉴权检查
    if req.cookie("auth_token").is_none() {
        return crate::middleware::auth::missing_token_response();
    }
    if crate::middleware::auth::check_admin_auth(&req).is_none() {
        return crate::middleware::auth::forbidden_response();
    }

    let req = req_json.into_inner();
    
    // 验证必填字段
    if req.nickname.is_empty() || req.link_url.is_empty() {
        return HttpResponse::BadRequest().json(CommonResponse {
            success: false,
            message: "昵称和链接不能为空".to_string(),
        });
    }
    
    let friend_link_repo = FriendLinkRepository::new(repo.get_pool().clone());
    
    let link = crate::db::models::FriendLink {
        id: None,
        nickname: req.nickname.clone(),
        link_url: req.link_url.clone(),
        avatar_url: req.avatar_url.clone(),
        motto: req.motto.clone(),
        sort_order: req.sort_order.unwrap_or(0),
        is_enabled: req.is_enabled.unwrap_or(true),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    match friend_link_repo.create(&link).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "友链创建成功"
        })),
        Err(e) => HttpResponse::InternalServerError().json(CommonResponse {
            success: false,
            message: format!("创建友链失败: {}", e),
        })
    }
}

/// 更新友链（Admin）
pub async fn update(
    path: web::Path<i64>,
    req_json: web::Json<UpdateFriendLinkRequest>,
    repo: web::Data<Arc<dyn Repository>>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    // 鉴权检查
    if req.cookie("auth_token").is_none() {
        return crate::middleware::auth::missing_token_response();
    }
    if crate::middleware::auth::check_admin_auth(&req).is_none() {
        return crate::middleware::auth::forbidden_response();
    }

    let id = path.into_inner();
    let req = req_json.into_inner();
    
    // 验证必填字段
    if req.nickname.is_empty() || req.link_url.is_empty() {
        return HttpResponse::BadRequest().json(CommonResponse {
            success: false,
            message: "昵称和链接不能为空".to_string(),
        });
    }
    
    let friend_link_repo = FriendLinkRepository::new(repo.get_pool().clone());
    
    // 获取现有友链
    let existing_link = match friend_link_repo.get_by_id(id).await {
        Ok(Some(link)) => link,
        Ok(None) => return HttpResponse::NotFound().json(CommonResponse {
            success: false,
            message: "友链不存在".to_string(),
        }),
        Err(e) => return HttpResponse::InternalServerError().json(CommonResponse {
            success: false,
            message: format!("获取友链失败: {}", e),
        })
    };
    
    let link = crate::db::models::FriendLink {
        id: Some(id),
        nickname: req.nickname.clone(),
        link_url: req.link_url.clone(),
        avatar_url: req.avatar_url.clone(),
        motto: req.motto.clone(),
        sort_order: req.sort_order.unwrap_or(existing_link.sort_order),
        is_enabled: req.is_enabled.unwrap_or(existing_link.is_enabled),
        created_at: existing_link.created_at,
        updated_at: chrono::Utc::now(),
    };
    
    match friend_link_repo.update(&link).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "友链更新成功"
        })),
        Err(e) => HttpResponse::InternalServerError().json(CommonResponse {
            success: false,
            message: format!("更新友链失败: {}", e),
        })
    }
}

/// 删除友链（Admin）
pub async fn delete(
    path: web::Path<i64>,
    repo: web::Data<Arc<dyn Repository>>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    // 鉴权检查
    if req.cookie("auth_token").is_none() {
        return crate::middleware::auth::missing_token_response();
    }
    if crate::middleware::auth::check_admin_auth(&req).is_none() {
        return crate::middleware::auth::forbidden_response();
    }

    let id = path.into_inner();
    
    if id <= 0 {
        return HttpResponse::BadRequest().json(CommonResponse {
            success: false,
            message: "无效的友链ID".to_string(),
        });
    }
    
    let friend_link_repo = FriendLinkRepository::new(repo.get_pool().clone());
    
    match friend_link_repo.delete(id).await {
        Ok(_) => HttpResponse::Ok().json(CommonResponse {
            success: true,
            message: "友链删除成功".to_string(),
        }),
        Err(e) => HttpResponse::InternalServerError().json(CommonResponse {
            success: false,
            message: format!("删除友链失败: {}", e),
        })
    }
}

/// 批量删除友链请求
#[derive(Debug, Deserialize)]
pub struct BatchDeleteRequest {
    pub ids: Vec<i64>,
}

/// 批量删除友链（Admin）
pub async fn delete_batch(
    req_json: web::Json<BatchDeleteRequest>,
    repo: web::Data<Arc<dyn Repository>>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    // 鉴权检查
    if req.cookie("auth_token").is_none() {
        return crate::middleware::auth::missing_token_response();
    }
    if crate::middleware::auth::check_admin_auth(&req).is_none() {
        return crate::middleware::auth::forbidden_response();
    }

    if req_json.ids.is_empty() {
        return HttpResponse::BadRequest().json(CommonResponse {
            success: false,
            message: "友链ID列表不能为空".to_string(),
        });
    }
    
    let friend_link_repo = FriendLinkRepository::new(repo.get_pool().clone());
    
    let mut deleted_count = 0;
    for id in &req_json.ids {
        if friend_link_repo.delete(*id).await.is_ok() {
            deleted_count += 1;
        }
    }

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": format!("成功删除 {} 条友链", deleted_count),
        "deleted_count": deleted_count
    }))
}

/// 批量更新友链状态请求
#[derive(Debug, Deserialize)]
pub struct BatchUpdateStatusRequest {
    pub ids: Vec<i64>,
    pub is_enabled: bool,
}

/// 批量更新友链状态（Admin）
pub async fn batch_update_status(
    req_json: web::Json<BatchUpdateStatusRequest>,
    repo: web::Data<Arc<dyn Repository>>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    // 鉴权检查
    if req.cookie("auth_token").is_none() {
        return crate::middleware::auth::missing_token_response();
    }
    if crate::middleware::auth::check_admin_auth(&req).is_none() {
        return crate::middleware::auth::forbidden_response();
    }

    if req_json.ids.is_empty() {
        return HttpResponse::BadRequest().json(CommonResponse {
            success: false,
            message: "友链ID列表不能为空".to_string(),
        });
    }
    
    let friend_link_repo = FriendLinkRepository::new(repo.get_pool().clone());
    
    let mut updated_count = 0;
    for id in &req_json.ids {
        if let Ok(Some(mut link)) = friend_link_repo.get_by_id(*id).await {
            link.is_enabled = req_json.is_enabled;
            link.updated_at = chrono::Utc::now();
            if friend_link_repo.update(&link).await.is_ok() {
                updated_count += 1;
            }
        }
    }

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": format!("成功更新 {} 条友链状态", updated_count),
        "updated_count": updated_count
    }))
}