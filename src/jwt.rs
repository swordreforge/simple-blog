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
    pub fn generate_token(&self, user_id: i64, username: &str, role: &str) -> crate::error::Result<String> {
        use crate::error::AppError;

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
        .map_err(|e| AppError::Jwt(format!("Token encoding failed: {}", e)))
    }

    /// 验证 JWT token
    pub fn validate_token(&self, token: &str) -> crate::error::Result<Claims> {
        use crate::error::AppError;

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_ref()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|e| AppError::Jwt(format!("Token decoding failed: {}", e)))?;

        let claims = token_data.claims;

        // 检查 token 是否过期
        if Utc::now().timestamp() > claims.exp {
            return Err(AppError::Jwt("Token has expired".to_string()));
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
                    tracing::info!("JWT密钥已从文件加载: {}", jwt_secret_file.display());
                    return secret.to_string();
                }
                tracing::warn!("JWT密钥文件为空，将生成新密钥");
            }
            Err(e) => {
                tracing::warn!("读取JWT密钥文件失败: {}, 将生成新密钥", e);
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
            tracing::info!("JWT密钥已生成并保存到: {}", jwt_secret_file.display());
        }
        Err(e) => {
            tracing::warn!("保存JWT密钥文件失败: {}", e);
        }
    }

    new_secret
}

/// 全局 JWT 服务实例（使用 once_cell 延迟初始化）
use once_cell::sync::OnceCell;

static JWT_SERVICE: OnceCell<JwtService> = OnceCell::new();

/// 初始化全局 JWT 服务
///
/// # 错误处理
/// 如果 JWT 服务已经初始化，将返回错误
pub fn init_jwt_service(secret: &str) -> crate::error::Result<()> {
    use crate::error::AppError;

    let service = JwtService::new(secret);
    JWT_SERVICE.set(service)
        .map_err(|_| AppError::Internal("JWT service already initialized".to_string()))
}

/// 获取全局 JWT 服务
///
/// # 返回值
/// 如果服务未初始化，返回错误
pub fn get_jwt_service() -> crate::error::Result<&'static JwtService> {
    use crate::error::AppError;

    JWT_SERVICE.get()
        .ok_or_else(|| AppError::Internal("JWT service not initialized".to_string()))
}

/// 生成 token（使用全局服务）
pub fn generate_token(user_id: i64, username: &str, role: &str) -> crate::error::Result<String> {
    use crate::error::AppError;

    let service = get_jwt_service()
        .map_err(|e| AppError::Jwt(format!("Failed to get JWT service: {}", e)))?;
    service.generate_token(user_id, username, role)
}

/// 验证 token（使用全局服务）
pub fn validate_token(token: &str) -> crate::error::Result<Claims> {
    use crate::error::AppError;

    let service = get_jwt_service()
        .map_err(|e| AppError::Jwt(format!("Failed to get JWT service: {}", e)))?;
    service.validate_token(token)
}