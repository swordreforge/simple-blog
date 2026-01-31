use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use chrono::{Duration, Utc};

/// JWT Claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: i64,
    pub username: String,
    pub role: String,
    pub exp: i64,
    pub iat: i64,
    pub nbf: i64,
    pub iss: String,
}

/// JWT 错误类型
#[derive(Debug)]
pub enum JwtError {
    InvalidToken(String),
    ExpiredToken,
    MissingToken,
    EncodingError(String),
    DecodingError(String),
}

impl std::fmt::Display for JwtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JwtError::InvalidToken(msg) => write!(f, "Invalid token: {}", msg),
            JwtError::ExpiredToken => write!(f, "Token has expired"),
            JwtError::MissingToken => write!(f, "Missing authorization token"),
            JwtError::EncodingError(msg) => write!(f, "Encoding error: {}", msg),
            JwtError::DecodingError(msg) => write!(f, "Decoding error: {}", msg),
        }
    }
}

impl std::error::Error for JwtError {}

/// JWT 服务
#[derive(Debug)]
pub struct JwtService {
    secret: String,
    token_expiration: Duration,
}

impl JwtService {
    /// 创建新的 JWT 服务
    pub fn new(secret: &str) -> Self {
        Self {
            secret: secret.to_string(),
            token_expiration: Duration::hours(24),
        }
    }

    /// 设置 token 过期时间
    pub fn with_expiration(mut self, duration: Duration) -> Self {
        self.token_expiration = duration;
        self
    }

    /// 生成 JWT token
    pub fn generate_token(&self, user_id: i64, username: &str, role: &str) -> Result<String, JwtError> {
        let now = Utc::now();
        let exp = now + self.token_expiration;

        let claims = Claims {
            user_id,
            username: username.to_string(),
            role: role.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            nbf: now.timestamp(),
            iss: "rustblog".to_string(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_ref()),
        )
        .map_err(|e| JwtError::EncodingError(e.to_string()))
    }

    /// 验证 JWT token
    pub fn validate_token(&self, token: &str) -> Result<Claims, JwtError> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_ref()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|e| JwtError::DecodingError(e.to_string()))?;

        let claims = token_data.claims;

        // 检查 token 是否过期
        if Utc::now().timestamp() > claims.exp {
            return Err(JwtError::ExpiredToken);
        }

        Ok(claims)
    }

    /// 刷新 token
    pub fn refresh_token(&self, token: &str) -> Result<String, JwtError> {
        let claims = self.validate_token(token)?;
        self.generate_token(claims.user_id, &claims.username, &claims.role)
    }
}

/// 全局 JWT 服务实例（使用 once_cell 延迟初始化）
use once_cell::sync::OnceCell;

static JWT_SERVICE: OnceCell<JwtService> = OnceCell::new();

/// 初始化全局 JWT 服务
pub fn init_jwt_service(secret: &str) {
    let service = JwtService::new(secret);
    JWT_SERVICE.set(service).expect("JWT service already initialized");
}

/// 获取全局 JWT 服务
pub fn get_jwt_service() -> &'static JwtService {
    JWT_SERVICE.get().expect("JWT service not initialized")
}

/// 生成 token（使用全局服务）
pub fn generate_token(user_id: i64, username: &str, role: &str) -> Result<String, JwtError> {
    get_jwt_service().generate_token(user_id, username, role)
}

/// 验证 token（使用全局服务）
pub fn validate_token(token: &str) -> Result<Claims, JwtError> {
    get_jwt_service().validate_token(token)
}

/// 刷新 token（使用全局服务）
pub fn refresh_token(token: &str) -> Result<String, JwtError> {
    get_jwt_service().refresh_token(token)
}