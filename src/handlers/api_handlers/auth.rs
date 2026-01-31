use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::jwt::generate_token;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub encrypted_password: String,
    #[serde(default)]
    pub session_id: String,
    #[serde(default)]
    pub client_public_key: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub encrypted_password: String,
    #[serde(default)]
    pub session_id: String,
    #[serde(default)]
    pub client_public_key: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub success: bool,
    pub message: String,
    pub token: Option<String>,
    pub user: Option<UserDTO>,
}

#[derive(Debug, Serialize)]
pub struct UserDTO {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub role: String,
    pub status: String,
}

/// 用户登录
pub async fn login(
    _rate_limit: crate::middleware::ratelimit::RateLimitCheck,
    req: web::Json<LoginRequest>,
    repo: web::Data<Arc<dyn crate::db::repositories::Repository>>,
) -> impl Responder {
    use crate::db::repositories::UserRepository;
    
    let username = &req.username;

    // 获取密码（支持明文和加密两种方式）
    let password = if !req.encrypted_password.is_empty() && !req.session_id.is_empty() && !req.client_public_key.is_empty() {
        // 使用ECC加密方式，需要解密
        match decrypt_password(&req.encrypted_password, &req.session_id, &req.client_public_key) {
            Ok(p) => p,
            Err(e) => {
                return HttpResponse::BadRequest().json(AuthResponse {
                    success: false,
                    message: format!("密码解密失败: {}", e),
                    token: None,
                    user: None,
                });
            }
        }
    } else if !req.password.is_empty() {
        // 使用明文密码（不推荐，仅作为降级方案）
        req.password.clone()
    } else {
        return HttpResponse::BadRequest().json(AuthResponse {
            success: false,
            message: "密码不能为空".to_string(),
            token: None,
            user: None,
        });
    };

    // 从数据库获取用户
    let user_repo = UserRepository::new(repo.get_pool().clone());
    let user = match user_repo.get_by_username(username).await {
        Ok(u) => u,
        Err(_e) => {
            return HttpResponse::Unauthorized().json(AuthResponse {
                success: false,
                message: "用户名或密码错误".to_string(),
                token: None,
                user: None,
            });
        }
    };

    // 验证密码
    match verify_password(&password, &user.password) {
        Ok(true) => {
            // 密码验证成功，生成 JWT token
            let user_id = user.id.unwrap_or(0);
            let token = match generate_token(user_id, &user.username, &user.role) {
                Ok(t) => t,
                Err(e) => {
                    return HttpResponse::InternalServerError().json(AuthResponse {
                        success: false,
                        message: format!("生成 token 失败: {}", e),
                        token: None,
                        user: None,
                    });
                }
            };

            // 设置 cookie
            let mut response = HttpResponse::Ok();
            response.cookie(
                actix_web::cookie::Cookie::build("auth_token", token.clone())
                    .path("/")
                    .http_only(true)
                    .max_age(actix_web::cookie::time::Duration::hours(24))
                    .finish()
            );

            response.json(AuthResponse {
                success: true,
                message: "登录成功".to_string(),
                token: Some(token),
                user: Some(UserDTO {
                    id: user_id,
                    username: user.username,
                    email: user.email,
                    role: user.role,
                    status: user.status,
                }),
            })
        }
        Ok(false) => {
            HttpResponse::Unauthorized().json(AuthResponse {
                success: false,
                message: "用户名或密码错误".to_string(),
                token: None,
                user: None,
            })
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(AuthResponse {
                success: false,
                message: format!("密码验证失败: {}", e),
                token: None,
                user: None,
            })
        }
    }
}

/// 用户注册
pub async fn register(
    _rate_limit: crate::middleware::ratelimit::RateLimitCheck,
    req: web::Json<RegisterRequest>,
    repo: web::Data<Arc<dyn crate::db::repositories::Repository>>,
) -> impl Responder {
    use crate::db::repositories::UserRepository;
    use argon2::{Argon2, PasswordHasher, password_hash::{SaltString, rand_core::OsRng}};

    let username = &req.username;
    let email = &req.email;
    let role = if let Some(r) = req.role.clone() { r } else { "user".to_string() };

    // 获取密码（支持明文和加密两种方式）
    let password = if !req.encrypted_password.is_empty() && !req.session_id.is_empty() && !req.client_public_key.is_empty() {
        // 使用ECC加密方式，需要解密
        match decrypt_password(&req.encrypted_password, &req.session_id, &req.client_public_key) {
            Ok(p) => p,
            Err(e) => {
                return HttpResponse::BadRequest().json(AuthResponse {
                    success: false,
                    message: format!("密码解密失败: {}", e),
                    token: None,
                    user: None,
                });
            }
        }
    } else if !req.password.is_empty() {
        // 使用明文密码（不推荐，仅作为降级方案）
        req.password.clone()
    } else {
        return HttpResponse::BadRequest().json(AuthResponse {
            success: false,
            message: "密码不能为空".to_string(),
            token: None,
            user: None,
        });
    };

    // 检查用户名是否已存在
    let user_repo = UserRepository::new(repo.get_pool().clone());
    match user_repo.get_by_username(username).await {
        Ok(_) => {
            return HttpResponse::BadRequest().json(AuthResponse {
                success: false,
                message: "用户名已存在".to_string(),
                token: None,
                user: None,
            });
        }
        Err(_) => {
            // 用户不存在，继续注册
        }
    }

    // 使用 Argon2id 哈希密码
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt);

    let hashed_password = match password_hash {
        Ok(hash) => hash.to_string(),
        Err(e) => {
            return HttpResponse::InternalServerError().json(AuthResponse {
                success: false,
                message: format!("密码哈希失败: {}", e),
                token: None,
                user: None,
            });
        }
    };

    // 创建用户
    let new_user = crate::db::models::User {
        id: None,
        username: username.clone(),
        email: email.clone(),
        password: hashed_password,
        role: role.clone(),
        status: "active".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    match user_repo.create(&new_user).await {
        Ok(_) => {
            // 获取创建的用户
            let user = user_repo.get_by_username(username).await;
            match user {
                Ok(u) => {
                    // 生成 JWT token
                    let user_id = u.id.unwrap_or(0);
                    let token = match generate_token(user_id, &u.username, &u.role) {
                        Ok(t) => t,
                        Err(e) => {
                            return HttpResponse::InternalServerError().json(AuthResponse {
                                success: false,
                                message: format!("生成 token 失败: {}", e),
                                token: None,
                                user: None,
                            });
                        }
                    };

                    // 设置 cookie
                    let mut response = HttpResponse::Ok();
                    response.cookie(
                        actix_web::cookie::Cookie::build("auth_token", token.clone())
                            .path("/")
                            .http_only(true)
                            .max_age(actix_web::cookie::time::Duration::hours(24))
                            .finish()
                    );

                    response.json(AuthResponse {
                        success: true,
                        message: "注册成功".to_string(),
                        token: Some(token),
                        user: Some(UserDTO {
                            id: user_id,
                            username: u.username,
                            email: u.email,
                            role: u.role,
                            status: u.status,
                        }),
                    })
                }
                Err(_e) => {
                    HttpResponse::Ok().json(AuthResponse {
                        success: true,
                        message: "注册成功，但无法获取用户信息".to_string(),
                        token: Some(format!("token_{}", username)),
                        user: None,
                    })
                }
            }
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(AuthResponse {
                success: false,
                message: format!("注册失败: {}", e),
                token: None,
                user: None,
            })
        }
    }
}

/// 用户登出
pub async fn logout() -> impl Responder {
    let mut response = HttpResponse::Ok();
    response.cookie(
        actix_web::cookie::Cookie::build("auth_token", "")
            .path("/")
            .http_only(true)
            .max_age(actix_web::cookie::time::Duration::ZERO)
            .finish()
    );
    response.json(serde_json::json!({
        "success": true,
        "message": "登出成功"
    }))
}

/// 检查登录状态
pub async fn check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "logged_in": false,
        "user": serde_json::Value::Null
    }))
}

/// 验证密码（使用 Argon2id）
pub fn verify_password(password: &str, hashed_password: &str) -> Result<bool, String> {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    
    // 解析哈希值
    let parsed_hash = PasswordHash::new(hashed_password)
        .map_err(|e| format!("Failed to parse password hash: {}", e))?;
    
    // 验证密码
    let argon2 = Argon2::default();
    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(_e) => Ok(false),
    }
}

/// 解密密码（使用ECC）
pub fn decrypt_password(encrypted_password: &str, session_id: &str, client_public_key: &str) -> Result<String, String> {
    use super::crypto::GLOBAL_SESSION_MANAGER;

    // 获取会话
    let session = GLOBAL_SESSION_MANAGER.get_session(session_id)
        .ok_or_else(|| format!("Session not found: {}", session_id))?;

    // 检查会话是否过期
    if session.is_expired() {
        return Err(format!("Session expired: {}", session_id));
    }

    // 解密密码
    session.hybrid_decrypt(encrypted_password, client_public_key)
}