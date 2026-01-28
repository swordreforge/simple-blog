use actix_web::{dev::Payload, Error, FromRequest, HttpRequest};
use actix_web::HttpMessage;
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