use super::backend::{CacheBackend, CacheError};

use async_trait::async_trait;

#[cfg(feature = "valkey")]
use std::time::Duration;

#[cfg(feature = "valkey")]
use redis::{aio::ConnectionManager, AsyncCommands, Client};

/// Valkey 缓存后端
#[cfg(feature = "valkey")]
pub struct ValkeyCacheBackend {
    manager: ConnectionManager,
    key_prefix: String,
}

#[cfg(feature = "valkey")]
impl ValkeyCacheBackend {
    /// 创建新的 Valkey 缓存后端
    pub async fn new(url: &str, key_prefix: Option<String>) -> Result<Self, CacheError> {
        let client = Client::open(url).map_err(|e| {
            CacheError::ConnectionError(format!("Failed to create Redis client: {}", e))
        })?;

        let manager = ConnectionManager::new(client)
            .await
            .map_err(|e| {
                CacheError::ConnectionError(format!("Failed to create connection manager: {}", e))
            })?;

        Ok(Self {
            manager,
            key_prefix: key_prefix.unwrap_or_else(|| "rustblog:".to_string()),
        })
    }

    /// 添加前缀到键
    fn prefixed_key(&self, key: &str) -> String {
        format!("{}{}", self.key_prefix, key)
    }
}

#[cfg(feature = "valkey")]
#[async_trait]
impl CacheBackend for ValkeyCacheBackend {
    async fn get(&self, key: &str) -> Option<String> {
        let prefixed_key = self.prefixed_key(key);
        let mut conn = self.manager.clone();
        conn.get(prefixed_key).await.ok()
    }

    async fn set(&self, key: &str, value: &str, ttl: Duration) -> Result<(), CacheError> {
        let prefixed_key = self.prefixed_key(key);
        let mut conn = self.manager.clone();
        
        conn.set_ex(prefixed_key, value, ttl.as_secs())
            .await
            .map_err(|e| CacheError::ConnectionError(format!("SET failed: {}", e)))
    }

    async fn delete(&self, key: &str) -> Result<(), CacheError> {
        let prefixed_key = self.prefixed_key(key);
        let mut conn = self.manager.clone();
        
        conn.del(prefixed_key)
            .await
            .map_err(|e| CacheError::ConnectionError(format!("DEL failed: {}", e)))
    }

    async fn delete_many(&self, keys: &[String]) -> Result<(), CacheError> {
        if keys.is_empty() {
            return Ok(());
        }

        let prefixed_keys: Vec<String> = keys.iter()
            .map(|k| self.prefixed_key(k))
            .collect();
        
        let mut conn = self.manager.clone();
        
        conn.del(prefixed_keys)
            .await
            .map_err(|e| CacheError::ConnectionError(format!("DEL many failed: {}", e)))
    }

    async fn delete_pattern(&self, pattern: &str) -> Result<(), CacheError> {
        let prefixed_pattern = self.prefixed_key(pattern);
        let mut conn = self.manager.clone();
        
        // 使用 SCAN 命令找到所有匹配的键，然后删除
        let mut keys_to_delete = Vec::new();
        let mut cursor: u64 = 0;
        
        loop {
            let (next_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(&prefixed_pattern)
                .arg("COUNT")
                .arg(100)
                .query_async(&mut conn)
                .await
                .map_err(|e| CacheError::ConnectionError(format!("SCAN failed: {}", e)))?;
            
            keys_to_delete.extend(keys);
            
            cursor = next_cursor;
            if cursor == 0 {
                break;
            }
        }
        
        if !keys_to_delete.is_empty() {
            conn.del::<_, ()>(keys_to_delete)
                .await
                .map_err(|e| CacheError::ConnectionError(format!("DEL pattern failed: {}", e)))?;
        }
        
        Ok(())
    }
}

/// 禁用 Valkey 特性时的存根实现
#[cfg(not(feature = "valkey"))]
pub struct ValkeyCacheBackend;

#[cfg(not(feature = "valkey"))]
impl ValkeyCacheBackend {
    pub async fn new(_url: &str, _key_prefix: Option<String>) -> Result<Self, CacheError> {
        Err(CacheError::ConnectionError(
            "Valkey feature is not enabled. Please rebuild with --features valkey".to_string(),
        ))
    }
}

#[cfg(not(feature = "valkey"))]
#[async_trait]
impl CacheBackend for ValkeyCacheBackend {
    async fn get(&self, _key: &str) -> Option<String> {
        None
    }

    async fn set(&self, _key: &str, _value: &str, _ttl: std::time::Duration) -> Result<(), CacheError> {
        Err(CacheError::ConnectionError(
            "Valkey feature is not enabled".to_string(),
        ))
    }

    async fn delete(&self, _key: &str) -> Result<(), CacheError> {
        Err(CacheError::ConnectionError(
            "Valkey feature is not enabled".to_string(),
        ))
    }

    async fn delete_many(&self, _keys: &[String]) -> Result<(), CacheError> {
        Err(CacheError::ConnectionError(
            "Valkey feature is not enabled".to_string(),
        ))
    }

    async fn delete_pattern(&self, _pattern: &str) -> Result<(), CacheError> {
        Err(CacheError::ConnectionError(
            "Valkey feature is not enabled".to_string(),
        ))
    }
}