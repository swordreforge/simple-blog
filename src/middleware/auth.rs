use actix_web::{dev::Payload, Error, FromRequest, HttpRequest, HttpResponse, HttpMessage};
use std::future::{ready, Ready};

/// 用户 ID 键
#[derive(Debug, Clone)]
pub struct UserIDKey(pub i64);

/// 用户名键
#[derive(Debug, Clone)]
pub struct UsernameKey(pub String);

/// 角色键
#[derive(Debug, Clone)]
pub struct RoleKey(pub String);

// 实现 FromRequest trait 以便从请求中提取这些键
impl FromRequest for UserIDKey {
    type Error = Error;
    type Future = Ready<Result<UserIDKey, Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        if let Some(user_id) = req.extensions().get::<UserIDKey>() {
            ready(Ok(user_id.clone()))
        } else {
            ready(Ok(UserIDKey(0)))
        }
    }
}

impl FromRequest for UsernameKey {
    type Error = Error;
    type Future = Ready<Result<UsernameKey, Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        if let Some(username) = req.extensions().get::<UsernameKey>() {
            ready(Ok(username.clone()))
        } else {
            ready(Ok(UsernameKey(String::new())))
        }
    }
}

impl FromRequest for RoleKey {
    type Error = Error;
    type Future = Ready<Result<RoleKey, Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        if let Some(role) = req.extensions().get::<RoleKey>() {
            ready(Ok(role.clone()))
        } else {
            ready(Ok(RoleKey(String::new())))
        }
    }
}

/// 检查请求是否有有效的admin权限
/// 返回Some(())表示有权限，None表示无权限或无效token
pub fn check_admin_auth(req: &actix_web::HttpRequest) -> Option<(i64, String, String)> {
    let token = req.cookie("auth_token").map(|c| c.value().to_string());
    match token {
        Some(token_str) => {
            match crate::jwt::validate_token(&token_str) {
                Ok(claims) => {
                    if claims.role == "admin" {
                        Some((claims.user_id, claims.username, claims.role))
                    } else {
                        None
                    }
                }
                Err(_) => None,
            }
        }
        None => None,
    }
}

/// 返回权限被拒绝的响应
pub fn forbidden_response() -> HttpResponse {
    HttpResponse::Forbidden().json(serde_json::json!({
        "success": false,
        "message": "Permission denied: admin role required"
    }))
}

/// 返回缺少token的响应
pub fn missing_token_response() -> HttpResponse {
    HttpResponse::Unauthorized().json(serde_json::json!({
        "success": false,
        "message": "Missing authorization token"
    }))
}