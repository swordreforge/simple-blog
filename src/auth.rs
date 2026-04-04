use anyhow::Result;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::db::Database;
use crate::models::{LoginRequest, User};

const SESSION_TTL: i64 = 24 * 60 * 60 * 1000; // 24 hours in milliseconds

#[derive(Clone, Serialize, Deserialize)]
pub struct Session {
    pub user_id: i64,
    pub username: Arc<str>,
    pub expires_at: i64,
}

pub struct AuthManager {
    db: Arc<Database>,
    sessions: Arc<RwLock<HashMap<String, Session>>>,
}

impl AuthManager {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn hash_password(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?
            .to_string();
        Ok(password_hash)
    }

    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| anyhow::anyhow!("Failed to parse password hash: {}", e))?;
        let argon2 = Argon2::default();
        Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }

    pub async fn generate_session_token(&self, user_id: i64, username: &str) -> String {
        let token: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();

        let session = Session {
            user_id,
            username: Arc::from(username),  // 使用 Arc::from 直接创建 Arc<str>
            expires_at: chrono::Utc::now().timestamp_millis() + SESSION_TTL,
        };

        self.sessions.write().await.insert(token.clone(), session);
        token
    }

    pub async fn verify_session_token(&self, token: &str) -> Option<User> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(token)?;

        // 检查会话是否过期
        if chrono::Utc::now().timestamp_millis() > session.expires_at {
            drop(sessions);
            // 会话过期，移除并返回 None
            self.sessions.write().await.remove(token);
            return None;
        }

        // 返回用户信息，使用 Arc::clone 避免深拷贝
        Some(User {
            id: session.user_id,
            username: session.username.clone(),  // Arc::clone 只是增加引用计数
        })
    }

    pub async fn revoke_session_token(&self, token: &str) {
        self.sessions.write().await.remove(token);
    }

    pub async fn cleanup_expired_sessions(&self) {
        let now = chrono::Utc::now().timestamp_millis();
        let mut sessions = self.sessions.write().await;
        sessions.retain(|_, session| session.expires_at > now);
    }

    /// 获取当前会话数量（包括过期的）
    pub async fn get_session_count(&self) -> usize {
        self.sessions.read().await.len()
    }

    pub async fn login(&self, request: &LoginRequest) -> Result<Option<(User, String)>> {
        let user = self
            .db
            .get_user_by_username(&request.username)
            .await?;

        if let Some(user) = user {
            if self.verify_password(&request.password, &user.password_hash)? {
                let token = self
                    .generate_session_token(user.id, &user.username)
                    .await;
                return Ok(Some((
                    User {
                        id: user.id,
                        username: user.username.into(),  // 将 String 转换为 Arc<str>
                    },
                    token,
                )));
            }
        }

        Ok(None)
    }

    pub async fn create_admin_account(&self, username: &str, password: &str) -> Result<User> {
        if self.db.has_user().await? {
            anyhow::bail!("Admin already initialized");
        }

        let password_hash = self.hash_password(password)?;
        let user_id = self.db.create_user(username, &password_hash).await?;

        Ok(User {
            id: user_id,
            username: Arc::from(username),  // 使用 Arc::from 直接创建 Arc<str>
        })
    }

    pub async fn needs_admin_init(&self) -> Result<bool> {
        Ok(!self.db.has_user().await?)
    }
}