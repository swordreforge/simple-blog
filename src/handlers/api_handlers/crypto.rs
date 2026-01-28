use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use base64::{Engine as _, engine::general_purpose};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use sha2::{Sha256, Digest};
use rand::RngCore;
use p256::{ecdsa::{SigningKey, VerifyingKey, signature::Signer}, PublicKey, SecretKey};

/// ECC 会话管理器
#[derive(Clone)]
pub struct ECCSession {
    pub session_id: String,
    pub signing_key: SigningKey,
    pub verifying_key: VerifyingKey,
    pub created_at: chrono::DateTime<Utc>,
}

impl ECCSession {
    pub fn new(session_id: String) -> Self {
        // 生成新的 ECC 密钥对
        let signing_key = SigningKey::random(&mut rand::thread_rng());
        let verifying_key = signing_key.verifying_key().clone();
        
        ECCSession {
            session_id,
            signing_key,
            verifying_key,
            created_at: Utc::now(),
        }
    }
    
    pub fn is_expired(&self) -> bool {
        Utc::now() - self.created_at > Duration::hours(1)
    }
    
    pub fn get_public_key_jwk(&self) -> serde_json::Value {
        // 获取公钥的点坐标
        let encoded_point = self.verifying_key.to_encoded_point(false);
        let point = encoded_point.as_bytes();
        
        // 未压缩格式: 0x04 + X (32 bytes) + Y (32 bytes)
        let x_bytes = &point[1..33];
        let y_bytes = &point[33..65];
        
        // 使用 base64url 编码（不带填充）
        let x = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(x_bytes);
        let y = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(y_bytes);
        
        serde_json::json!({
            "kty": "EC",
            "crv": "P-256",
            "x": x,
            "y": y,
            "use": "enc",
            "alg": "ECDH-ES+A256KW"
        })
    }
    
    pub fn get_expiry(&self) -> chrono::DateTime<Utc> {
        self.created_at + Duration::hours(1)
    }
}

/// 全局会话管理器
pub struct SessionManager {
    pub sessions: Arc<Mutex<HashMap<String, ECCSession>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        SessionManager {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub fn create_session(&self, session_id: String) -> ECCSession {
        let session = ECCSession::new(session_id.clone());
        self.sessions.lock().unwrap().insert(session_id.clone(), session.clone());
        session
    }
    
    pub fn get_session(&self, session_id: &str) -> Option<ECCSession> {
        let sessions = self.sessions.lock().unwrap();
        sessions.get(session_id).cloned()
    }
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_SESSION_MANAGER: SessionManager = SessionManager::new();
}

/// 获取公钥请求
#[derive(Debug, Deserialize)]
pub struct GetPublicKeyRequest {
    pub session_id: Option<String>,
}

/// 获取公钥响应
#[derive(Debug, Serialize)]
pub struct GetPublicKeyResponse {
    pub success: bool,
    pub session_id: String,
    pub public_key: serde_json::Value,
    pub key_format: String,
    pub algorithm: String,
    pub curve: String,
    pub expires_at: i64,
    pub expires_in: i64,
}

/// 解密数据请求
#[derive(Debug, Deserialize)]
pub struct DecryptDataRequest {
    pub session_id: String,
    pub client_public_key: String,
    pub encrypted_data: String,
}

/// 解密数据响应
#[derive(Debug, Serialize)]
pub struct DecryptDataResponse {
    pub success: bool,
    pub decrypted: Option<String>,
    pub error: Option<String>,
}

/// 生成会话ID
fn generate_session_id() -> String {
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    format!("session_{}", hex::encode(bytes))
}

/// 获取 ECC 公钥
pub async fn get_public_key(
    query: web::Query<GetPublicKeyRequest>,
) -> HttpResponse {
    let session_id = if let Some(sid) = &query.session_id {
        sid.clone()
    } else {
        generate_session_id()
    };
    
    let session = GLOBAL_SESSION_MANAGER.create_session(session_id.clone());
    let public_key_jwk = session.get_public_key_jwk();
    let expires_at = session.get_expiry();
    let expires_in = (expires_at - Utc::now()).num_seconds();
    
    HttpResponse::Ok().json(GetPublicKeyResponse {
        success: true,
        session_id,
        public_key: public_key_jwk,
        key_format: "jwk".to_string(),
        algorithm: "ECDH-ES".to_string(),
        curve: "P-256".to_string(),
        expires_at: expires_at.timestamp(),
        expires_in,
    })
}

/// 解密数据（简化版）
pub async fn decrypt_data(
    req: web::Json<DecryptDataRequest>,
) -> HttpResponse {
    // 验证请求参数
    if req.session_id.is_empty() || req.client_public_key.is_empty() || req.encrypted_data.is_empty() {
        return HttpResponse::BadRequest().json(DecryptDataResponse {
            success: false,
            decrypted: None,
            error: Some("missing required fields".to_string()),
        });
    }
    
    // 获取会话
    let session = match GLOBAL_SESSION_MANAGER.get_session(&req.session_id) {
        Some(s) => s,
        None => {
            return HttpResponse::NotFound().json(DecryptDataResponse {
                success: false,
                decrypted: None,
                error: Some("session not found".to_string()),
            });
        }
    };
    
    // 检查会话是否过期
    if session.is_expired() {
        return HttpResponse::Gone().json(DecryptDataResponse {
            success: false,
            decrypted: None,
            error: Some("session expired".to_string()),
        });
    }
    
    // 解析加密数据
    let encrypted_data = match general_purpose::STANDARD.decode(&req.encrypted_data) {
        Ok(data) => data,
        Err(_) => {
            return HttpResponse::BadRequest().json(DecryptDataResponse {
                success: false,
                decrypted: None,
                error: Some("invalid encrypted data".to_string()),
            });
        }
    };
    
    // 简化解密（仅用于演示）
    let key_bytes = session.signing_key.to_bytes();
    let key_b64 = general_purpose::STANDARD.encode(&key_bytes);
    let decrypted = simple_decrypt(&encrypted_data, &key_b64);
    
    HttpResponse::Ok().json(DecryptDataResponse {
        success: true,
        decrypted: Some(String::from_utf8_lossy(&decrypted).to_string()),
        error: None,
    })
}

/// 简单解密（仅用于演示，实际应使用 AES-GCM）
fn simple_decrypt(data: &[u8], key: &str) -> Vec<u8> {
    let key_hash = Sha256::digest(key.as_bytes());
    data.iter()
        .enumerate()
        .map(|(i, &b)| b ^ key_hash[i % key_hash.len()])
        .collect()
}