use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use crate::db::repositories::{AboutMainCardRepository, AboutSubCardRepository, Repository};
use std::sync::Arc;

/// 主卡片响应
#[derive(Debug, Serialize)]
pub struct MainCardResponse {
    pub id: i64,
    pub title: String,
    pub icon: String,
    pub layout_type: String,
    pub custom_css: String,
    pub sort_order: i32,
    pub is_enabled: bool,
}

/// 次卡片响应
#[derive(Debug, Serialize)]
pub struct SubCardResponse {
    pub id: i64,
    pub main_card_id: i64,
    pub title: String,
    pub description: String,
    pub icon: String,
    pub link_url: String,
    pub layout_type: String,
    pub custom_css: String,
    pub sort_order: i32,
    pub is_enabled: bool,
}

/// 创建/更新主卡片请求
#[derive(Debug, Deserialize)]
pub struct MainCardRequest {
    pub title: String,
    pub icon: String,
    pub layout_type: String,
    pub custom_css: String,
    pub sort_order: i32,
    pub is_enabled: bool,
}

/// 创建/更新次卡片请求
#[derive(Debug, Deserialize)]
pub struct SubCardRequest {
    pub main_card_id: i64,
    pub title: String,
    pub description: String,
    pub icon: String,
    pub link_url: String,
    pub layout_type: String,
    pub custom_css: String,
    pub sort_order: i32,
    pub is_enabled: bool,
}

/// 获取关于页面内容
pub async fn get() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "content": "About RustBlog"
    }))
}

/// 更新关于页面内容
pub async fn update() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "About page updated"
    }))
}

/// 获取所有主卡片（公开）
pub async fn get_main_cards(repo: web::Data<Arc<dyn Repository>>) -> HttpResponse {
    let main_card_repo = AboutMainCardRepository::new(repo.get_pool().clone());
    
    match main_card_repo.get_all().await {
        Ok(cards) => {
            // 只返回启用的卡片
            let data: Vec<MainCardResponse> = cards.into_iter()
                .filter(|card| card.is_enabled)
                .map(|card| MainCardResponse {
                    id: card.id.unwrap_or(0),
                    title: card.title,
                    icon: card.icon,
                    layout_type: card.layout_type,
                    custom_css: card.custom_css,
                    sort_order: card.sort_order,
                    is_enabled: card.is_enabled,
                })
                .collect();
            
            HttpResponse::Ok().json(data)
        }
        Err(e) => {
            eprintln!("获取主卡片失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "获取主卡片失败"
            }))
        }
    }
}

/// 获取所有主卡片（管理员）
pub async fn get_main_cards_admin(
    repo: web::Data<Arc<dyn Repository>>,
    http_req: actix_web::HttpRequest,
) -> HttpResponse {
    // 鉴权检查
    if http_req.cookie("auth_token").is_none() {
        return crate::middleware::auth::missing_token_response();
    }
    if crate::middleware::auth::check_admin_auth(&http_req).is_none() {
        return crate::middleware::auth::forbidden_response();
    }
    let main_card_repo = AboutMainCardRepository::new(repo.get_pool().clone());
    
    match main_card_repo.get_all().await {
        Ok(cards) => {
            let data: Vec<MainCardResponse> = cards.into_iter()
                .map(|card| MainCardResponse {
                    id: card.id.unwrap_or(0),
                    title: card.title,
                    icon: card.icon,
                    layout_type: card.layout_type,
                    custom_css: card.custom_css,
                    sort_order: card.sort_order,
                    is_enabled: card.is_enabled,
                })
                .collect();
            
            HttpResponse::Ok().json(data)
        }
        Err(e) => {
            eprintln!("获取主卡片失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "获取主卡片失败"
            }))
        }
    }
}

/// 获取所有次卡片（公开）
pub async fn get_sub_cards(
    repo: web::Data<Arc<dyn Repository>>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    let sub_card_repo = AboutSubCardRepository::new(repo.get_pool().clone());
    
    match sub_card_repo.get_all().await {
        Ok(cards) => {
            // 获取查询参数中的 main_card_id
            let filter_main_card_id = query.get("main_card_id").and_then(|id| id.parse::<i64>().ok());
            
            // 只返回启用的卡片，并按 main_card_id 过滤
            let data: Vec<SubCardResponse> = cards.into_iter()
                .filter(|card| {
                    card.is_enabled && (
                        filter_main_card_id.is_none() || 
                        filter_main_card_id == Some(card.main_card_id)
                    )
                })
                .map(|card| SubCardResponse {
                    id: card.id.unwrap_or(0),
                    main_card_id: card.main_card_id,
                    title: card.title,
                    description: card.description,
                    icon: card.icon,
                    link_url: card.link_url,
                    layout_type: card.layout_type,
                    custom_css: card.custom_css,
                    sort_order: card.sort_order,
                    is_enabled: card.is_enabled,
                })
                .collect();
            
            HttpResponse::Ok().json(data)
        }
        Err(e) => {
            eprintln!("获取次卡片失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "获取次卡片失败"
            }))
        }
    }
}

/// 获取所有次卡片（管理员）
pub async fn get_sub_cards_admin(
    repo: web::Data<Arc<dyn Repository>>,
    http_req: actix_web::HttpRequest,
) -> HttpResponse {
    // 鉴权检查
    if http_req.cookie("auth_token").is_none() {
        return crate::middleware::auth::missing_token_response();
    }
    if crate::middleware::auth::check_admin_auth(&http_req).is_none() {
        return crate::middleware::auth::forbidden_response();
    }
    let sub_card_repo = AboutSubCardRepository::new(repo.get_pool().clone());
    
    match sub_card_repo.get_all().await {
        Ok(cards) => {
            let data: Vec<SubCardResponse> = cards.into_iter()
                .map(|card| SubCardResponse {
                    id: card.id.unwrap_or(0),
                    main_card_id: card.main_card_id,
                    title: card.title,
                    description: card.description,
                    icon: card.icon,
                    link_url: card.link_url,
                    layout_type: card.layout_type,
                    custom_css: card.custom_css,
                    sort_order: card.sort_order,
                    is_enabled: card.is_enabled,
                })
                .collect();
            
            HttpResponse::Ok().json(data)
        }
        Err(e) => {
            eprintln!("获取次卡片失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "获取次卡片失败"
            }))
        }
    }
}

/// 创建主卡片
pub async fn create_main_card(
    req: web::Json<MainCardRequest>,
    repo: web::Data<Arc<dyn Repository>>,
) -> HttpResponse {
    let main_card_repo = AboutMainCardRepository::new(repo.get_pool().clone());
    let card = crate::db::models::AboutMainCard {
        id: None,
        title: req.title.clone(),
        icon: req.icon.clone(),
        layout_type: req.layout_type.clone(),
        custom_css: req.custom_css.clone(),
        sort_order: req.sort_order,
        is_enabled: req.is_enabled,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    match main_card_repo.create(&card).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "主卡片创建成功"
        })),
        Err(e) => {
            eprintln!("创建主卡片失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "创建主卡片失败"
            }))
        }
    }
}

/// 更新主卡片
pub async fn update_main_card(
    query: web::Query<std::collections::HashMap<String, String>>,
    req: web::Json<MainCardRequest>,
    repo: web::Data<Arc<dyn Repository>>,
) -> HttpResponse {
    let id_str = query.get("id").cloned().unwrap_or_default();
    let id: i64 = match id_str.parse() {
        Ok(i) => i,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "message": "无效的主卡片ID"
            }));
        }
    };
    
    let main_card_repo = AboutMainCardRepository::new(repo.get_pool().clone());
    let mut card = match main_card_repo.get_by_id(id).await {
        Ok(c) => c,
        Err(_) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "message": "主卡片不存在"
            }));
        }
    };
    
    card.title = req.title.clone();
    card.icon = req.icon.clone();
    card.layout_type = req.layout_type.clone();
    card.custom_css = req.custom_css.clone();
    card.sort_order = req.sort_order;
    card.is_enabled = req.is_enabled;
    card.updated_at = chrono::Utc::now();
    
    match main_card_repo.update(&card).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "主卡片更新成功"
        })),
        Err(e) => {
            eprintln!("更新主卡片失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "更新主卡片失败"
            }))
        }
    }
}

/// 删除主卡片
pub async fn delete_main_card(
    query: web::Query<std::collections::HashMap<String, String>>,
    repo: web::Data<Arc<dyn Repository>>,
) -> HttpResponse {
    let id_str = query.get("id").cloned().unwrap_or_default();
    let id: i64 = match id_str.parse() {
        Ok(i) => i,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "message": "无效的主卡片ID"
            }));
        }
    };
    
    let main_card_repo = AboutMainCardRepository::new(repo.get_pool().clone());
    
    match main_card_repo.delete(id).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "主卡片删除成功"
        })),
        Err(e) => {
            eprintln!("删除主卡片失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "删除主卡片失败"
            }))
        }
    }
}

/// 创建次卡片
pub async fn create_sub_card(
    req: web::Json<SubCardRequest>,
    repo: web::Data<Arc<dyn Repository>>,
) -> HttpResponse {
    let sub_card_repo = AboutSubCardRepository::new(repo.get_pool().clone());
    let card = crate::db::models::AboutSubCard {
        id: None,
        main_card_id: req.main_card_id,
        title: req.title.clone(),
        description: req.description.clone(),
        icon: req.icon.clone(),
        link_url: req.link_url.clone(),
        layout_type: req.layout_type.clone(),
        custom_css: req.custom_css.clone(),
        sort_order: req.sort_order,
        is_enabled: req.is_enabled,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    match sub_card_repo.create(&card).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "次卡片创建成功"
        })),
        Err(e) => {
            eprintln!("创建次卡片失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "创建次卡片失败"
            }))
        }
    }
}

/// 更新次卡片
pub async fn update_sub_card(
    query: web::Query<std::collections::HashMap<String, String>>,
    req: web::Json<SubCardRequest>,
    repo: web::Data<Arc<dyn Repository>>,
) -> HttpResponse {
    let id_str = query.get("id").cloned().unwrap_or_default();
    let id: i64 = match id_str.parse() {
        Ok(i) => i,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "message": "无效的次卡片ID"
            }));
        }
    };
    
    let sub_card_repo = AboutSubCardRepository::new(repo.get_pool().clone());
    let mut card = match sub_card_repo.get_by_id(id).await {
        Ok(c) => c,
        Err(_) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "message": "次卡片不存在"
            }));
        }
    };
    
    card.main_card_id = req.main_card_id;
    card.title = req.title.clone();
    card.description = req.description.clone();
    card.icon = req.icon.clone();
    card.link_url = req.link_url.clone();
    card.layout_type = req.layout_type.clone();
    card.custom_css = req.custom_css.clone();
    card.sort_order = req.sort_order;
    card.is_enabled = req.is_enabled;
    card.updated_at = chrono::Utc::now();
    
    match sub_card_repo.update(&card).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "次卡片更新成功"
        })),
        Err(e) => {
            eprintln!("更新次卡片失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "更新次卡片失败"
            }))
        }
    }
}

/// 删除次卡片
pub async fn delete_sub_card(
    query: web::Query<std::collections::HashMap<String, String>>,
    repo: web::Data<Arc<dyn Repository>>,
) -> HttpResponse {
    let id_str = query.get("id").cloned().unwrap_or_default();
    let id: i64 = match id_str.parse() {
        Ok(i) => i,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "message": "无效的次卡片ID"
            }));
        }
    };
    
    let sub_card_repo = AboutSubCardRepository::new(repo.get_pool().clone());
    
    match sub_card_repo.delete(id).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "次卡片删除成功"
        })),
        Err(e) => {
            eprintln!("删除次卡片失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "删除次卡片失败"
            }))
        }
    }
}

/// 切换主卡片启用/禁用状态
pub async fn toggle_main_card_enabled(
    query: web::Query<std::collections::HashMap<String, String>>,
    repo: web::Data<Arc<dyn Repository>>,
) -> HttpResponse {
    let id_str = query.get("id").cloned().unwrap_or_default();
    let id: i64 = match id_str.parse() {
        Ok(i) => i,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "message": "无效的主卡片ID"
            }));
        }
    };
    
    let main_card_repo = AboutMainCardRepository::new(repo.get_pool().clone());
    let mut card = match main_card_repo.get_by_id(id).await {
        Ok(c) => c,
        Err(_) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "message": "主卡片不存在"
            }));
        }
    };
    
    // 切换启用状态
    card.is_enabled = !card.is_enabled;
    card.updated_at = chrono::Utc::now();
    
    match main_card_repo.update(&card).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": if card.is_enabled {
                "主卡片已启用"
            } else {
                "主卡片已禁用"
            },
            "is_enabled": card.is_enabled
        })),
        Err(e) => {
            eprintln!("更新主卡片状态失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "更新主卡片状态失败"
            }))
        }
    }
}

/// 切换次卡片启用/禁用状态
pub async fn toggle_sub_card_enabled(
    query: web::Query<std::collections::HashMap<String, String>>,
    repo: web::Data<Arc<dyn Repository>>,
) -> HttpResponse {
    let id_str = query.get("id").cloned().unwrap_or_default();
    let id: i64 = match id_str.parse() {
        Ok(i) => i,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "message": "无效的次卡片ID"
            }));
        }
    };
    
    let sub_card_repo = AboutSubCardRepository::new(repo.get_pool().clone());
    let mut card = match sub_card_repo.get_by_id(id).await {
        Ok(c) => c,
        Err(_) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "message": "次卡片不存在"
            }));
        }
    };
    
    // 切换启用状态
    card.is_enabled = !card.is_enabled;
    card.updated_at = chrono::Utc::now();
    
    match sub_card_repo.update(&card).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": if card.is_enabled {
                "次卡片已启用"
            } else {
                "次卡片已禁用"
            },
            "is_enabled": card.is_enabled
        })),
        Err(e) => {
            eprintln!("更新次卡片状态失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "更新次卡片状态失败"
            }))
        }
    }
}