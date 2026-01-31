use actix_web::{HttpRequest, HttpResponse};
use crate::jwt::validate_token;

/// 鉴权检查函数：检查用户是否已登录且为管理员
pub fn require_admin_auth(req: &HttpRequest) -> Result<Option<(i64, String, String)>, HttpResponse> {
    // 从 cookie 中获取 token
    let token = req.cookie("auth_token");

    if token.is_none() {
        return Err(HttpResponse::Unauthorized().json(serde_json::json!({
            "success": false,
            "message": "Missing authorization token"
        })));
    }

    let cookie_ref = token.unwrap();
    let token_str = cookie_ref.value();

    // 验证 token
    match validate_token(token_str) {
        Ok(claims) => {
            // 检查是否为管理员
            if claims.role != "admin" {
                return Err(HttpResponse::Forbidden().json(serde_json::json!({
                    "success": false,
                    "message": "Permission denied: admin role required"
                })));
            }

            Ok(Some((claims.user_id, claims.username, claims.role)))
        }
        Err(_) => {
            // token 无效
            Err(HttpResponse::Unauthorized().json(serde_json::json!({
                "success": false,
                "message": "Invalid or expired token"
            })))
        }
    }
}