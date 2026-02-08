use super::backend::{CacheBackend, CacheError};
use async_trait::async_trait;
use std::time::Duration;

#[cfg(feature = "valkey")]
use redis::{aio::ConnectionManager, AsyncCommands, Client};

/// Valkey 缓存后端
#[cfg(feature = "valkey")]
pub struct ValkeyCacheBackend {
    client: Client,
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

        let manager = ConnectionManager::new(client.clone())
            .await
            .map_err(|e| {
                CacheError::ConnectionError(format!("Failed to create connection manager: {}", e))
            })?;

        Ok(Self {
            client,
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

    async fn exists(&self, key: &str) -> bool {
        let prefixed_key = self.prefixed_key(key);
        let mut conn = self.manager.clone();
        
        conn.exists(prefixed_key).await.unwrap_or(0) > 0
    }

    async fn clear(&self) -> Result<(), CacheError> {
        let mut conn = self.manager.clone();
        
        // 使用 SCAN 而不是 KEYS 以避免阻塞
        let pattern = format!("{}*", self.key_prefix);
        let mut iter: redis::AsyncIter<'_, String> = conn.scan_match(pattern).await.map_err(|e| {
            CacheError::ConnectionError(format!("SCAN failed: {}", e))
        })?;

        let mut keys = Vec::new();
        while let Some(key) = iter.next_item().await {
            keys.push(key);
        }

        // drop iter 以释放对 conn 的借用
        drop(iter);

        if !keys.is_empty() {
            let mut conn = self.manager.clone();
            conn.del::<_, ()>(keys)
                .await
                .map_err(|e| CacheError::ConnectionError(format!("DEL batch failed: {}", e)))?;
        }

        Ok(())
    }

    fn backend_name(&self) -> &'static str {
        "valkey"
    }

    async fn health_check(&self) -> bool {
        let mut conn = self.manager.clone();
        conn.ping::<()>().await.is_ok()
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
#[async_trait::async_trait]
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

    async fn exists(&self, _key: &str) -> bool {
        false
    }

    async fn clear(&self) -> Result<(), CacheError> {
        Err(CacheError::ConnectionError(
            "Valkey feature is not enabled".to_string(),
        ))
    }

    fn backend_name(&self) -> &'static str {
        "valkey-disabled"
    }

    async fn health_check(&self) -> bool {
        false
    }
}