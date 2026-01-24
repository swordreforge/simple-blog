use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub success: bool,
    pub message: String,
    pub token: Option<String>,
}

/// 用户登录
pub async fn login(req: web::Json<LoginRequest>) -> impl Responder {
    HttpResponse::Ok().json(AuthResponse {
        success: true,
        message: "Login successful".to_string(),
        token: Some("placeholder_token".to_string()),
    })
}

/// 用户注册
pub async fn register(req: web::Json<RegisterRequest>) -> impl Responder {
    HttpResponse::Ok().json(AuthResponse {
        success: true,
        message: "Registration successful".to_string(),
        token: Some("placeholder_token".to_string()),
    })
}

/// 用户登出
pub async fn logout() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Logout successful"
    }))
}

/// 检查登录状态
pub async fn check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "logged_in": false,
        "user": serde_json::Value::Null
    }))
}