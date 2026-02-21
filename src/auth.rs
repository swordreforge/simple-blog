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
use crate::models::{LoginRequest, User, UserWithPasswordHash};

const SESSION_TTL: i64 = 24 * 60 * 60 * 1000; // 24 hours in milliseconds

#[derive(Clone, Serialize, Deserialize)]
pub struct Session {
    pub user_id: i64,
    pub username: String,
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
            username: username.to_string(),
            expires_at: chrono::Utc::now().timestamp_millis() + SESSION_TTL,
        };

        self.sessions.write().await.insert(token.clone(), session);
        token
    }

    pub async fn verify_session_token(&self, token: &str) -> Option<User> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(token)?;

        if chrono::Utc::now().timestamp_millis() > session.expires_at {
            drop(sessions);
            self.sessions.write().await.remove(token);
            return None;
        }

        Some(User {
            id: session.user_id,
            username: session.username.clone(),
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
                        username: user.username,
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
            username: username.to_string(),
        })
    }

    pub async fn needs_admin_init(&self) -> Result<bool> {
        Ok(!self.db.has_user().await?)
    }
}

#[derive(Clone)]
pub struct AuthState {
    pub auth_manager: Arc<AuthManager>,
}

// Extractor for authentication
pub struct AuthenticatedUser {
    pub user: User,
}

#[axum::async_trait]
impl<S> axum::extract::FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = axum::http::StatusCode;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok());

        if let Some(auth_header) = auth_header {
            if let Some(token) = auth_header.strip_prefix("Bearer ") {
                if let Some(state) = parts.extensions.get::<AuthState>() {
                    if let Some(user) = state.auth_manager.verify_session_token(token).await {
                        return Ok(AuthenticatedUser { user });
                    }
                }
            }
        }

        Err(axum::http::StatusCode::UNAUTHORIZED)
    }
}