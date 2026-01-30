use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use base64::{Engine as _, engine::general_purpose};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use sha2::{Sha256, Digest};
use rand::RngCore;
use p256::{ecdsa::{SigningKey, VerifyingKey}, PublicKey};
use p256::elliptic_curve::sec1::ToEncodedPoint;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};

/// 解析PEM格式的公钥
fn parse_pem_public_key(pem_data: &str) -> Result<Vec<u8>, String> {
    // 移除PEM头尾和换行符
    let clean_pem = pem_data
        .replace("-----BEGIN PUBLIC KEY-----", "")
        .replace("-----END PUBLIC KEY-----", "")
        .replace("\n", "")
        .replace("\r", "")
        .trim()
        .to_string();
    
    // 解码base64
    general_purpose::STANDARD.decode(clean_pem)
        .map_err(|e| format!("Failed to decode PEM: {}", e))
}

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
    
    /// 派生共享密钥（ECDH）
    pub fn derive_shared_secret(&self, client_public_key_bytes: &[u8]) -> Result<[u8; 32], String> {
        use p256::PublicKey;
        use p256::elliptic_curve::group::GroupEncoding;
        use p256::NonZeroScalar;
        use spki::DecodePublicKey;

        // 解析客户端公钥（支持PKIX/DER格式和SEC1格式）
        let client_public_key = match PublicKey::from_public_key_der(client_public_key_bytes) {
            Ok(key) => key,
            Err(_) => {
                // 如果DER格式失败，尝试SEC1格式
                PublicKey::from_sec1_bytes(client_public_key_bytes)
                    .map_err(|e| format!("Failed to parse client public key: {}", e))?
            }
        };

        // 使用服务器的私钥和客户端的公钥进行 ECDH 计算
        // 从 SigningKey 获取私钥的 scalar 值
        let server_private_bytes = self.signing_key.to_bytes();
        let server_scalar = NonZeroScalar::from_repr(server_private_bytes);
        let server_scalar: Option<NonZeroScalar> = server_scalar.into();
        let server_scalar = server_scalar.ok_or_else(|| "Invalid private key".to_string())?;

        // 获取客户端公钥的坐标
        let client_point = client_public_key.as_affine();

        // 计算共享密钥：server_private * client_public
        let shared_point = *client_point * *server_scalar.as_ref();

        // 获取共享密钥的 X 坐标（32字节）
        // 使用 to_encoded_point 获取未压缩格式，然后提取X坐标
        let encoded_point = shared_point.to_encoded_point(false);
        let point_bytes = encoded_point.as_bytes();

        // 未压缩格式: 0x04 + X (32字节) + Y (32字节)
        // 提取X坐标（跳过第一个字节0x04）
        let x_coordinate = &point_bytes[1..33];

        // 直接使用X坐标作为AES密钥（与Go版本和Web Crypto API保持一致）
        let mut key = [0u8; 32];
        key.copy_from_slice(x_coordinate);

        Ok(key)
    }
    
    /// 混合解密（ECDH + AES-GCM）
    pub fn hybrid_decrypt(&self, encrypted_data_b64: &str, client_public_key_input: &str) -> Result<String, String> {
        use aes_gcm::aead::Aead;

        // 解析客户端公钥（支持PEM格式）
        let client_public_key_bytes = if client_public_key_input.contains("-----BEGIN PUBLIC KEY-----") {
            parse_pem_public_key(client_public_key_input)?
        } else if client_public_key_input.contains('-') {
            match base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(client_public_key_input) {
                Ok(data) => data,
                Err(_) => {
                    general_purpose::STANDARD.decode(client_public_key_input)
                        .map_err(|e| format!("Failed to decode client public key: {}", e))?
                }
            }
        } else {
            general_purpose::STANDARD.decode(client_public_key_input)
                .map_err(|e| format!("Failed to decode client public key: {}", e))?
        };

        // 派生共享密钥
        let shared_key = self.derive_shared_secret(&client_public_key_bytes)?;

        // 解码加密数据（尝试base64和base64url）
        let encrypted_data = match general_purpose::STANDARD.decode(encrypted_data_b64) {
            Ok(data) => data,
            Err(_) => {
                base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(encrypted_data_b64)
                    .map_err(|e| format!("Failed to decode encrypted data: {}", e))?
            }
        };

        // 创建AES-GCM解密器
        let cipher = Aes256Gcm::new(&shared_key.into());

        // 提取nonce（前12字节）
        if encrypted_data.len() < 12 {
            return Err("Encrypted data too short".to_string());
        }

        let nonce = Nonce::from_slice(&encrypted_data[..12]);
        let ciphertext = &encrypted_data[12..];

        // 解密
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        String::from_utf8(plaintext)
            .map_err(|e| format!("Invalid UTF-8: {}", e))
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