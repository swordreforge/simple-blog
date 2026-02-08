use super::backend::{CacheBackend, CacheError};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use dashmap::DashMap;
use std::time::Duration as StdDuration;

/// 本地缓存条目
#[derive(Debug, Clone)]
struct CacheEntry {
    value: String,
    expires_at: i64,
}

impl CacheEntry {
    fn new(value: String, ttl: StdDuration) -> Self {
        let expires_at = Utc::now()
            .checked_add_signed(Duration::seconds(ttl.as_secs() as i64))
            .map(|dt| dt.timestamp())
            .unwrap_or(i64::MAX);

        Self { value, expires_at }
    }

    fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.expires_at
    }
}

/// 本地内存缓存后端（降级方案）
pub struct LocalCacheBackend {
    cache: DashMap<String, CacheEntry>,
    max_size: usize,
}

impl LocalCacheBackend {
    /// 创建新的本地缓存后端
    pub fn new(max_size: Option<usize>) -> Self {
        Self {
            cache: DashMap::new(),
            max_size: max_size.unwrap_or(10000),
        }
    }

    /// 检查是否需要清理空间
    fn ensure_capacity(&self) {
        let current = self.cache.len();
        
        if current >= self.max_size {
            // 使用 LRU 策略：移除最旧的条目
            let mut oldest_key = None;
            let mut oldest_time = i64::MAX;

            for entry in self.cache.iter() {
                if entry.value().expires_at < oldest_time {
                    oldest_time = entry.value().expires_at;
                    oldest_key = Some(entry.key().clone());
                }
            }

            if let Some(key) = oldest_key {
                self.cache.remove(&key);
            }
        }
    }
}

#[async_trait]
impl CacheBackend for LocalCacheBackend {
    async fn get(&self, key: &str) -> Option<String> {
        if let Some(entry) = self.cache.get(key) {
            if entry.is_expired() {
                self.cache.remove(key);
                return None;
            }
            Some(entry.value.clone())
        } else {
            None
        }
    }

    async fn set(&self, key: &str, value: &str, ttl: StdDuration) -> Result<(), CacheError> {
        self.ensure_capacity();
        
        let entry = CacheEntry::new(value.to_string(), ttl);
        self.cache.insert(key.to_string(), entry);
        
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), CacheError> {
        self.cache.remove(key);
        Ok(())
    }
}