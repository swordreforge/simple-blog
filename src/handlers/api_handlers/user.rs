use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use crate::db::repositories::{UserRepository, Repository};
use crate::db::models::User;
use std::sync::Arc;
use chrono::Utc;

/// 用户信息响应
#[derive(Debug, Serialize)]
pub struct UserInfoResponse {
    pub logged_in: bool,
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub role: Option<String>,
}

/// 用户响应
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub role: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

/// 创建用户请求
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub email: String,
    pub role: Option<String>,
    pub status: Option<String>,
}

/// 更新用户请求
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub password: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
    pub status: Option<String>,
}

/// 获取当前用户信息
pub async fn info(req: HttpRequest) -> HttpResponse {
    use actix_web::HttpMessage;
    
    // 尝试从扩展中获取用户信息
    let user_id = req.extensions().get::<crate::middleware::auth::UserIDKey>().map(|k| k.0);
    let username = req.extensions().get::<crate::middleware::auth::UsernameKey>().map(|k| k.0.clone());
    let role = req.extensions().get::<crate::middleware::auth::RoleKey>().map(|k| k.0.clone());
    
    // 如果没有用户信息，返回未登录状态
    if user_id.is_none() || username.is_none() || role.is_none() {
        return HttpResponse::Ok().json(serde_json::json!({
            "success": false,
            "message": "未登录",
            "data": UserInfoResponse {
                logged_in: false,
                user_id: None,
                username: None,
                role: None,
            }
        }));
    }
    
    // 返回用户信息
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "获取用户信息成功",
        "data": UserInfoResponse {
            logged_in: true,
            user_id,
            username,
            role,
        }
    }))
}

/// 获取所有用户（管理员）
pub async fn admin_list(
    repo: web::Data<Arc<dyn Repository>>,
    query: web::Query<std::collections::HashMap<String, String>>,
    req: HttpRequest,
) -> HttpResponse {
    // 鉴权检查
    if req.cookie("auth_token").is_none() {
        return crate::middleware::auth::missing_token_response();
    }
    if crate::middleware::auth::check_admin_auth(&req).is_none() {
        return crate::middleware::auth::forbidden_response();
    }

    let user_repo = UserRepository::new(repo.get_pool().clone());
    
    // 解析分页参数
    let limit: i64 = query.get("limit").and_then(|l| l.parse().ok()).unwrap_or(20);
    let _offset: i64 = query.get("offset").and_then(|o| o.parse().ok()).unwrap_or(0);
    let page: i64 = query.get("page").and_then(|p| p.parse().ok()).unwrap_or(1);
    let calculated_offset = (page - 1) * limit;
    
    // 获取所有用户
    match user_repo.get_all(limit, calculated_offset).await {
        Ok(users) => {
            let total = match user_repo.count().await {
                Ok(c) => c,
                Err(_) => users.len() as i64,
            };
            
            let data: Vec<UserResponse> = users.into_iter()
                .map(|u| UserResponse {
                    id: u.id.unwrap_or(0),
                    username: u.username,
                    email: u.email,
                    role: u.role,
                    status: u.status,
                    created_at: u.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                    updated_at: u.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
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
            eprintln!("获取用户列表失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "获取用户列表失败"
            }))
        }
    }
}

/// 根据 ID 获取用户
pub async fn get(
    repo: web::Data<Arc<dyn Repository>>,
    path: web::Path<i64>,
    req: HttpRequest,
) -> HttpResponse {
    // 鉴权检查
    if req.cookie("auth_token").is_none() {
        return crate::middleware::auth::missing_token_response();
    }
    if crate::middleware::auth::check_admin_auth(&req).is_none() {
        return crate::middleware::auth::forbidden_response();
    }

    let id = path.into_inner();
    let user_repo = UserRepository::new(repo.get_pool().clone());
    
    match user_repo.get_by_id(id).await {
        Ok(user) => {
            let response = UserResponse {
                id: user.id.unwrap_or(0),
                username: user.username,
                email: user.email,
                role: user.role,
                status: user.status,
                created_at: user.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                updated_at: user.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            };
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "data": response
            }))
        }
        Err(e) => {
            eprintln!("获取用户失败: {}", e);
            HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "message": "用户不存在"
            }))
        }
    }
}

/// 创建用户
pub async fn create(
    repo: web::Data<Arc<dyn Repository>>,
    req: web::Json<CreateUserRequest>,
) -> HttpResponse {
    let user_repo = UserRepository::new(repo.get_pool().clone());

    // 使用 Argon2id 哈希密码
    let hashed_password = match hash_password(&req.password) {
        Ok(hash) => hash,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": format!("密码哈希失败: {}", e)
            }));
        }
    };

    let now = Utc::now();
    let user = User {
        id: None,
        username: req.username.clone(),
        password: hashed_password,
        email: req.email.clone(),
        role: req.role.clone().unwrap_or_else(|| "user".to_string()),
        status: req.status.clone().unwrap_or_else(|| "active".to_string()),
        created_at: now,
        updated_at: now,
    };

    match user_repo.create(&user).await {
        Ok(id) => {
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "用户创建成功",
                "data": serde_json::json!({"id": id})
            }))
        }
        Err(e) => {
            let error_msg = e.to_string();
            eprintln!("创建用户失败: {}", e);

            // 检查是否是 UNIQUE 约束错误
            if error_msg.contains("UNIQUE constraint failed") {
                if error_msg.contains("email") {
                    return HttpResponse::BadRequest().json(serde_json::json!({
                        "success": false,
                        "message": "该邮箱已被使用，请更换邮箱"
                    }));
                } else if error_msg.contains("username") {
                    return HttpResponse::BadRequest().json(serde_json::json!({
                        "success": false,
                        "message": "该用户名已被使用，请更换用户名"
                    }));
                }
            }

            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "创建用户失败"
            }))
        }
    }
}

/// 更新用户
pub async fn update(
    repo: web::Data<Arc<dyn Repository>>,
    path: web::Path<i64>,
    req: web::Json<UpdateUserRequest>,
) -> HttpResponse {
    let id = path.into_inner();
    let user_repo = UserRepository::new(repo.get_pool().clone());
    
    // 先获取现有用户
    let mut user = match user_repo.get_by_id(id).await {
        Ok(u) => u,
        Err(e) => {
            eprintln!("获取用户失败: {}", e);
            return HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "message": "用户不存在"
            }));
        }
    };
    
    // 更新字段
    if let Some(ref username) = req.username {
        user.username = username.clone();
    }
    if let Some(ref password) = req.password {
        // 使用 Argon2id 哈希密码
        let hashed_password = match hash_password(password) {
            Ok(hash) => hash,
            Err(e) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "success": false,
                    "message": format!("密码哈希失败: {}", e)
                }));
            }
        };
        user.password = hashed_password;
    }
    if let Some(ref email) = req.email {
        user.email = email.clone();
    }
    if let Some(ref role) = req.role {
        user.role = role.clone();
    }
    if let Some(ref status) = req.status {
        user.status = status.clone();
    }
    user.updated_at = Utc::now();
    
    match user_repo.update(&user).await {
        Ok(_) => {
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "用户更新成功"
            }))
        }
        Err(e) => {
            let error_msg = e.to_string();
            eprintln!("更新用户失败: {}", e);
            
            // 检查是否是 UNIQUE 约束错误
            if error_msg.contains("UNIQUE constraint failed") {
                if error_msg.contains("email") {
                    return HttpResponse::BadRequest().json(serde_json::json!({
                        "success": false,
                        "message": "该邮箱已被其他用户使用，请更换邮箱"
                    }));
                } else if error_msg.contains("username") {
                    return HttpResponse::BadRequest().json(serde_json::json!({
                        "success": false,
                        "message": "该用户名已被其他用户使用，请更换用户名"
                    }));
                }
            }
            
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "更新用户失败"
            }))
        }
    }
}

/// 删除用户
pub async fn delete(
    repo: web::Data<Arc<dyn Repository>>,
    path: web::Path<i64>,
) -> HttpResponse {
    let id = path.into_inner();
    let user_repo = UserRepository::new(repo.get_pool().clone());
    
    match user_repo.delete(id).await {
        Ok(_) => {
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "用户删除成功"
            }))
        }
        Err(e) => {
            eprintln!("删除用户失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "删除用户失败"
            }))
        }
    }
}

/// 批量删除用户请求
#[derive(Debug, Deserialize)]
pub struct BatchDeleteRequest {
    pub ids: Vec<i64>,
}

/// 批量删除用户
pub async fn delete_batch(
    repo: web::Data<Arc<dyn Repository>>,
    req: web::Json<BatchDeleteRequest>,
) -> HttpResponse {
    if req.ids.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "message": "用户ID列表不能为空"
        }));
    }

    let user_repo = UserRepository::new(repo.get_pool().clone());

    match user_repo.delete_batch(req.ids.clone()).await {
        Ok(count) => {
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": format!("成功删除 {} 个用户", count),
                "deleted_count": count
            }))
        }
        Err(e) => {
            eprintln!("批量删除用户失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "批量删除用户失败"
            }))
        }
    }
}

/// 哈希密码（使用 Argon2id）
pub fn hash_password(password: &str) -> Result<String, String> {
    use argon2::{Argon2, PasswordHasher, password_hash::{SaltString, rand_core::OsRng}};

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt);

    password_hash
        .map(|hash| hash.to_string())
        .map_err(|e| format!("密码哈希失败: {}", e))
}