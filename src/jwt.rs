use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use chrono::{Duration, Utc};
use rand::Rng;
use std::fs;
use std::path::Path;

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
    EncodingError(String),
    DecodingError(String),
    ExpiredToken,
}

impl std::fmt::Display for JwtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JwtError::EncodingError(msg) => write!(f, "Encoding error: {}", msg),
            JwtError::DecodingError(msg) => write!(f, "Decoding error: {}", msg),
            JwtError::ExpiredToken => write!(f, "Token has expired"),
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
}

/// 生成32位随机密钥
fn generate_random_secret() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.r#gen();
    hex::encode(bytes)
}

/// 初始化JWT密钥，从文件读取或生成新密钥
/// 如果文件不存在或命令行提供了空字符串，生成32位随机密钥并保存
/// 如果文件已存在且命令行未提供密钥，读取文件中的密钥（不覆盖）
pub fn init_jwt_secret(base_dir: &Path, jwt_secret: Option<&str>) -> String {
    // 检查是否需要生成新密钥
    let need_generate = match jwt_secret {
        Some(secret) => secret.is_empty(), // 空字符串则生成
        None => false, // None表示未提供，不生成
    };

    // 尝试从文件读取（只有在不生成新密钥时）
    let jwt_secret_file = base_dir.join("data").join("jwt-secret");

    if !need_generate && jwt_secret_file.exists() {
        // 文件存在，读取密钥
        match fs::read_to_string(&jwt_secret_file) {
            Ok(secret) => {
                let secret = secret.trim();
                if !secret.is_empty() {
                    eprintln!("✅ JWT密钥已从文件加载: {}", jwt_secret_file.display());
                    return secret.to_string();
                }
                eprintln!("⚠️  JWT密钥文件为空，将生成新密钥");
            }
            Err(e) => {
                eprintln!("⚠️  读取JWT密钥文件失败: {}, 将生成新密钥", e);
            }
        }
    }

    // 生成新的随机密钥并保存
    let new_secret = generate_random_secret();
    
    // 确保data目录存在
    if let Some(parent) = jwt_secret_file.parent() {
        fs::create_dir_all(parent).ok();
    }

    // 保存密钥文件
    match fs::write(&jwt_secret_file, &new_secret) {
        Ok(()) => {
            eprintln!("✅ JWT密钥已生成并保存到: {}", jwt_secret_file.display());
        }
        Err(e) => {
            eprintln!("⚠️  保存JWT密钥文件失败: {}", e);
        }
    }

    new_secret
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