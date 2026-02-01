use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use crate::db::repositories::{FriendLinkRepository, Repository};
use std::sync::Arc;

/// 友链列表请求参数
#[derive(Debug, Deserialize)]
pub struct FriendLinkListQuery {
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

/// 获取友链列表
pub async fn list(
    query: web::Query<FriendLinkListQuery>,
    repo: web::Data<Arc<dyn Repository>>,
) -> HttpResponse {
    let friend_link_repo = FriendLinkRepository::new(repo.get_pool().clone());
    
    let links = if query.include_disabled.unwrap_or(false) {
        friend_link_repo.get_all_including_disabled().await
    } else {
        friend_link_repo.get_all().await
    };
    
    match links {
        Ok(links) => {
            let data: Vec<FriendLinkResponse> = links.into_iter().map(|l| FriendLinkResponse {
                id: l.id.unwrap_or(0),
                nickname: l.nickname,
                link_url: l.link_url,
                avatar_url: l.avatar_url,
                motto: l.motto,
                sort_order: l.sort_order,
                is_enabled: l.is_enabled,
                created_at: l.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                updated_at: l.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            }).collect();
            
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "data": data
            }))
        }
        Err(_) => HttpResponse::InternalServerError().json(CommonResponse {
            success: false,
            message: "获取友链列表失败".to_string(),
        })
    }
}

/// 获取单个友链
pub async fn get(
    path: web::Path<i64>,
    repo: web::Data<Arc<dyn Repository>>,
) -> HttpResponse {
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

/// 创建友链
pub async fn create(
    req: web::Json<CreateFriendLinkRequest>,
    repo: web::Data<Arc<dyn Repository>>,
) -> HttpResponse {
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
        Err(_) => HttpResponse::InternalServerError().json(CommonResponse {
            success: false,
            message: "创建友链失败".to_string(),
        })
    }
}

/// 更新友链
pub async fn update(
    path: web::Path<i64>,
    req: web::Json<UpdateFriendLinkRequest>,
    repo: web::Data<Arc<dyn Repository>>,
) -> HttpResponse {
    let id = path.into_inner();
    
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
        Err(_) => return HttpResponse::InternalServerError().json(CommonResponse {
            success: false,
            message: "获取友链失败".to_string(),
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
        Err(_) => HttpResponse::InternalServerError().json(CommonResponse {
            success: false,
            message: "更新友链失败".to_string(),
        })
    }
}

/// 删除友链
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
        Err(_) => HttpResponse::InternalServerError().json(CommonResponse {
            success: false,
            message: "删除友链失败".to_string(),
        })
    }
}

/// 批量删除友链请求
#[derive(Debug, Deserialize)]
pub struct BatchDeleteRequest {
    pub ids: Vec<i64>,
}

/// 批量删除友链
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