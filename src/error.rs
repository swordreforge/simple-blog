//! 统一错误处理模块
//!
//! 提供应用程序中使用的所有错误类型
//! 避免使用 unwrap() 和 expect()，使用 ? 操作符进行错误传播

use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use std::fmt;

/// 应用程序统一错误类型
#[derive(Debug)]
pub enum AppError {
    /// 数据库错误
    Database(String),
    /// JWT 错误
    Jwt(String),
    /// 认证错误
    Auth(String),
    /// 验证错误
    Validation(String),
    /// 未找到资源
    NotFound(String),
    /// 权限不足
    Forbidden(String),
    /// IO 错误
    Io(String),
    /// JSON 序列化/反序列化错误
    Json(String),
    /// 内部服务器错误
    Internal(String),
    /// 缓存错误
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