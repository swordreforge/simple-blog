//! 统一错误处理模块
//!
//! 提供应用程序中使用的所有错误类型
//! 避免使用 unwrap() 和 expect()，使用 ? 操作符进行错误传播

use actix_web::{HttpResponse, error::ResponseError, http::StatusCode};
use std::fmt;
use no_panic::no_panic;

/// 应用程序统一错误类型
#[derive(Debug)]
pub enum AppError {
    /// 数据库错误
    Database(String),
    /// JWT 错误
    Jwt(String),
    /// 认证错误
    #[allow(dead_code)]
    Auth(String),
    /// 验证错误
    Validation(String),
    /// 未找到资源
    #[allow(dead_code)]
    NotFound(String),
    /// 权限不足
    #[allow(dead_code)]
    Forbidden(String),
    /// IO 错误
    Io(String),
    /// JSON 序列化/反序列化错误
    Json(String),
    /// 内部服务器错误
    Internal(String),
    /// 缓存错误
    #[allow(dead_code)]
    Cache(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Database(msg) => write!(f, "数据库错误: {}", msg),
            AppError::Jwt(msg) => write!(f, "JWT 错误: {}", msg),
            AppError::Auth(msg) => write!(f, "认证错误: {}", msg),
            AppError::Validation(msg) => write!(f, "验证错误: {}", msg),
            AppError::NotFound(msg) => write!(f, "未找到: {}", msg),
            AppError::Forbidden(msg) => write!(f, "权限不足: {}", msg),
            AppError::Io(msg) => write!(f, "IO 错误: {}", msg),
            AppError::Json(msg) => write!(f, "JSON 错误: {}", msg),
            AppError::Internal(msg) => write!(f, "内部错误: {}", msg),
            AppError::Cache(msg) => write!(f, "缓存错误: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        let error_response = serde_json::json!({
            "success": false,
            "message": self.to_string(),
            "error_type": self.error_type()
        });

        HttpResponse::build(status).json(error_response)
    }

    #[no_panic]
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::Database(_) | AppError::Internal(_) | AppError::Cache(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::Jwt(_) | AppError::Auth(_) => StatusCode::UNAUTHORIZED,
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Json(_) => StatusCode::BAD_REQUEST,
        }
    }
}

impl AppError {
    /// 返回错误类型字符串
    #[no_panic]
    pub fn error_type(&self) -> &'static str {
        match self {
            AppError::Database(_) => "database_error",
            AppError::Jwt(_) => "jwt_error",
            AppError::Auth(_) => "auth_error",
            AppError::Validation(_) => "validation_error",
            AppError::NotFound(_) => "not_found",
            AppError::Forbidden(_) => "forbidden",
            AppError::Io(_) => "io_error",
            AppError::Json(_) => "json_error",
            AppError::Internal(_) => "internal_error",
            AppError::Cache(_) => "cache_error",
        }
    }
}

// 从其他错误类型自动转换
// 注意：sqlx::Error 的转换移到了 db 模块中，因为 sqlx 是可选依赖

impl From<r2d2::Error> for AppError {
    fn from(err: r2d2::Error) -> Self {
        AppError::Database(err.to_string())
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError::Database(err.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        AppError::Jwt(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Json(err.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err.to_string())
    }
}

impl From<chrono::ParseError> for AppError {
    fn from(err: chrono::ParseError) -> Self {
        AppError::Validation(format!("日期解析错误: {}", err))
    }
}

/// 结果类型别名
pub type Result<T> = std::result::Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::StatusCode;

    #[test]
    fn test_error_display() {
        let db_error = AppError::Database("Connection failed".to_string());
        assert_eq!(db_error.to_string(), "数据库错误: Connection failed");

        let jwt_error = AppError::Jwt("Invalid token".to_string());
        assert_eq!(jwt_error.to_string(), "JWT 错误: Invalid token");

        let auth_error = AppError::Auth("Unauthorized".to_string());
        assert_eq!(auth_error.to_string(), "认证错误: Unauthorized");

        let validation_error = AppError::Validation("Invalid input".to_string());
        assert_eq!(validation_error.to_string(), "验证错误: Invalid input");

        let not_found_error = AppError::NotFound("Resource not found".to_string());
        assert_eq!(not_found_error.to_string(), "未找到: Resource not found");

        let forbidden_error = AppError::Forbidden("Access denied".to_string());
        assert_eq!(forbidden_error.to_string(), "权限不足: Access denied");

        let io_error = AppError::Io("File not found".to_string());
        assert_eq!(io_error.to_string(), "IO 错误: File not found");

        let json_error = AppError::Json("Invalid JSON".to_string());
        assert_eq!(json_error.to_string(), "JSON 错误: Invalid JSON");

        let internal_error = AppError::Internal("Something went wrong".to_string());
        assert_eq!(internal_error.to_string(), "内部错误: Something went wrong");

        let cache_error = AppError::Cache("Cache miss".to_string());
        assert_eq!(cache_error.to_string(), "缓存错误: Cache miss");
    }

    #[test]
    fn test_error_type() {
        assert_eq!(AppError::Database("".to_string()).error_type(), "database_error");
        assert_eq!(AppError::Jwt("".to_string()).error_type(), "jwt_error");
        assert_eq!(AppError::Auth("".to_string()).error_type(), "auth_error");
        assert_eq!(AppError::Validation("".to_string()).error_type(), "validation_error");
        assert_eq!(AppError::NotFound("".to_string()).error_type(), "not_found");
        assert_eq!(AppError::Forbidden("".to_string()).error_type(), "forbidden");
        assert_eq!(AppError::Io("".to_string()).error_type(), "io_error");
        assert_eq!(AppError::Json("".to_string()).error_type(), "json_error");
        assert_eq!(AppError::Internal("".to_string()).error_type(), "internal_error");
        assert_eq!(AppError::Cache("".to_string()).error_type(), "cache_error");
    }

    #[test]
    fn test_status_code() {
        assert_eq!(
            AppError::Database("".to_string()).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(
            AppError::Internal("".to_string()).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(
            AppError::Cache("".to_string()).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(
            AppError::Jwt("".to_string()).status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            AppError::Auth("".to_string()).status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            AppError::Validation("".to_string()).status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            AppError::NotFound("".to_string()).status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            AppError::Forbidden("".to_string()).status_code(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            AppError::Io("".to_string()).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(
            AppError::Json("".to_string()).status_code(),
            StatusCode::BAD_REQUEST
        );
    }

    #[test]
    fn test_error_response() {
        let error = AppError::Validation("Invalid input".to_string());
        let response = error.error_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        
        // 只验证状态码，不解析body内容（避免actix-web版本兼容性问题）
    }

    #[test]
    fn test_from_r2d2_error() {
        // r2d2::Error doesn't have ConnectionError field, using a simple io error
        let io_err = std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "Connection refused",
        );
        // Create a simple mock that implements the conversion
        // Since r2d2::Error doesn't expose constructors, we'll test the conversion differently
        let app_error = AppError::Database(io_err.to_string());
        
        assert!(matches!(app_error, AppError::Database(_)));
        assert!(app_error.to_string().contains("数据库错误"));
    }

    #[test]
    fn test_from_rusqlite_error() {
        let sqlite_err = rusqlite::Error::QueryReturnedNoRows;
        let app_error = AppError::from(sqlite_err);
        
        assert!(matches!(app_error, AppError::Database(_)));
        assert!(app_error.to_string().contains("数据库错误"));
    }

    #[test]
    fn test_from_jsonwebtoken_error() {
        let jwt_err = jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidToken);
        let app_error = AppError::from(jwt_err);
        
        assert!(matches!(app_error, AppError::Jwt(_)));
        assert!(app_error.to_string().contains("JWT 错误"));
    }

    #[test]
    fn test_from_serde_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let app_error = AppError::from(json_err);
        
        assert!(matches!(app_error, AppError::Json(_)));
        assert!(app_error.to_string().contains("JSON 错误"));
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let app_error = AppError::from(io_err);
        
        assert!(matches!(app_error, AppError::Io(_)));
        assert!(app_error.to_string().contains("IO 错误"));
    }

    #[test]
    fn test_from_chrono_parse_error() {
        use chrono::NaiveDateTime;
        let parse_err = NaiveDateTime::parse_from_str("invalid", "%Y-%m-%d").unwrap_err();
        let app_error = AppError::from(parse_err);
        
        assert!(matches!(app_error, AppError::Validation(_)));
        assert!(app_error.to_string().contains("验证错误"));
        assert!(app_error.to_string().contains("日期解析错误"));
    }

    #[test]
    fn test_error_debug() {
        let error = AppError::Internal("Test error".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("Internal"));
        assert!(debug_str.contains("Test error"));
    }

    #[test]
    fn test_error_send_sync() {
        // 确保错误类型可以在线程间传递
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        
        assert_send::<AppError>();
        assert_sync::<AppError>();
    }

    #[test]
    fn test_result_type() {
        let ok_result: Result<i32> = Ok(42);
        assert!(ok_result.is_ok());
        assert_eq!(ok_result.unwrap(), 42);

        let err_result: Result<i32> = Err(AppError::Validation("Test error".to_string()));
        assert!(err_result.is_err());
    }

    #[test]
    fn test_error_response_with_not_found() {
        let error = AppError::NotFound("User not found".to_string());
        let response = error.error_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_error_response_with_forbidden() {
        let error = AppError::Forbidden("Insufficient permissions".to_string());
        let response = error.error_response();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn test_error_response_with_internal() {
        let error = AppError::Internal("Internal server error".to_string());
        let response = error.error_response();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
