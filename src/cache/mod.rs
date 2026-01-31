/// 应用层缓存模块
/// 
/// 使用 LRU (Least Recently Used) 缓存策略减少数据库访问
/// 预期效果：API 响应时间减少 60-80%，数据库负载降低 60-80%

use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};

/// 缓存键类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CacheKey {
    /// 设置缓存键
    Setting(String),
    /// 设置分类缓存键
    SettingCategory(String),
    /// 文章缓存键
    Passage(String),
    /// 文章列表缓存键
    PassageList { limit: i64, page: i64 },
    /// 统计数据缓存键
    Stats(String),
}

/// 缓存值类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheValue {
    /// JSON 字符串值
    Json(String),
    /// 设置值
    Setting(String),
}

/// 缓存配置
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// 缓存最大条目数
    pub max_size: usize,
    /// 设置缓存 TTL (秒)
    pub setting_ttl: u64,
    /// 文章缓存 TTL (秒)
    pub passage_ttl: u64,
    /// 统计数据缓存 TTL (秒)
    pub stats_ttl: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size: 1000,
            setting_ttl: 300,  // 5分钟
            passage_ttl: 60,   // 1分钟
            stats_ttl: 120,    // 2分钟
        }
    }
}

/// 缓存条目
#[derive(Debug, Clone)]
struct CacheEntry {
    value: String,
    expires_at: chrono::DateTime<chrono::Utc>,
}

impl CacheEntry {
    fn new(value: String, ttl_seconds: u64) -> Self {
        let expires_at = chrono::Utc::now() + chrono::Duration::seconds(ttl_seconds as i64);
        Self { value, expires_at }
    }

    fn is_expired(&self) -> bool {
        chrono::Utc::now() > self.expires_at
    }
}

/// 应用缓存
pub struct AppCache {
    config: CacheConfig,
    cache: Arc<RwLock<LruCache<CacheKey, CacheEntry>>>,
}

impl AppCache {
    /// 创建新的应用缓存
    pub fn new(config: CacheConfig) -> Self {
        let size = NonZeroUsize::new(config.max_size).unwrap_or(NonZeroUsize::new(1000).unwrap());
        Self {
            config,
            cache: Arc::new(RwLock::new(LruCache::new(size))),
        }
    }

    /// 获取缓存值
    pub async fn get(&self, key: &CacheKey) -> Option<String> {
        let mut cache = self.cache.write().await;
        if let Some(entry) = cache.get_mut(key) {
            // 检查是否过期
            if entry.is_expired() {
                cache.pop(key);
                return None;
            }
            // LRU：更新访问时间
            return Some(entry.value.clone());
        }
        None
    }

    /// 设置缓存值
    pub async fn set(&self, key: CacheKey, value: String) {
        let ttl = self.get_ttl_for_key(&key);
        let mut cache = self.cache.write().await;
        cache.put(key, CacheEntry::new(value, ttl));
    }

    /// 使指定键失效
    pub async fn invalidate(&self, key: &CacheKey) {
        let mut cache = self.cache.write().await;
        cache.pop(key);
    }

    /// 使指定分类的所有缓存失效
    pub async fn invalidate_category(&self, category: &str) {
        let mut cache = self.cache.write().await;
        // 收集需要删除的键
        let keys_to_remove: Vec<CacheKey> = cache
            .iter()
            .filter(|(k, _)| {
                matches!(k, CacheKey::SettingCategory(cat) if cat == category)
                    || matches!(k, CacheKey::Setting(key) if key.starts_with(&format!("{}_", category)))
            })
            .map(|(k, _)| k.clone())
            .collect();
        
        // 删除这些键
        for key in keys_to_remove {
            cache.pop(&key);
        }
    }

    /// 使所有设置缓存失效
    pub async fn invalidate_settings(&self) {
        let mut cache = self.cache.write().await;
        let keys_to_remove: Vec<CacheKey> = cache
            .iter()
            .filter(|(k, _)| matches!(k, CacheKey::Setting(_) | CacheKey::SettingCategory(_)))
            .map(|(k, _)| k.clone())
            .collect();
        
        for key in keys_to_remove {
            cache.pop(&key);
        }
    }

    /// 使所有文章缓存失效
    pub async fn invalidate_passages(&self) {
        let mut cache = self.cache.write().await;
        let keys_to_remove: Vec<CacheKey> = cache
            .iter()
            .filter(|(k, _)| matches!(k, CacheKey::Passage(_) | CacheKey::PassageList { .. }))
            .map(|(k, _)| k.clone())
            .collect();
        
        for key in keys_to_remove {
            cache.pop(&key);
        }
    }

    /// 使所有统计缓存失效
    pub async fn invalidate_stats(&self) {
        let mut cache = self.cache.write().await;
        let keys_to_remove: Vec<CacheKey> = cache
            .iter()
            .filter(|(k, _)| matches!(k, CacheKey::Stats(_)))
            .map(|(k, _)| k.clone())
            .collect();
        
        for key in keys_to_remove {
            cache.pop(&key);
        }
    }

    /// 清空所有缓存
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// 获取缓存统计信息
    pub async fn stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        CacheStats {
            len: cache.len(),
            capacity: cache.cap(),
        }
    }

    /// 根据键类型获取 TTL
    fn get_ttl_for_key(&self, key: &CacheKey) -> u64 {
        match key {
            CacheKey::Setting(_) | CacheKey::SettingCategory(_) => self.config.setting_ttl,
            CacheKey::Passage(_) | CacheKey::PassageList { .. } => self.config.passage_ttl,
            CacheKey::Stats(_) => self.config.stats_ttl,
        }
    }
}

/// 缓存统计信息
#[derive(Debug, Clone, Serialize)]
pub struct CacheStats {
    /// 当前缓存条目数
    pub len: usize,
    /// 缓存容量
    pub capacity: NonZeroUsize,
}

/// 设置缓存助手
pub struct SettingCache {
    cache: Arc<AppCache>,
}

impl SettingCache {
    pub fn new(cache: Arc<AppCache>) -> Self {
        Self { cache }
    }

    /// 获取设置值
    pub async fn get(&self, key: &str) -> Option<String> {
        self.cache.get(&CacheKey::Setting(key.to_string())).await
    }

    /// 设置缓存值
    pub async fn set(&self, key: String, value: String) {
        self.cache.set(CacheKey::Setting(key), value).await;
    }

    /// 使设置失效
    pub async fn invalidate(&self, key: &str) {
        self.cache.invalidate(&CacheKey::Setting(key.to_string())).await;
    }

    /// 使所有设置失效
    pub async fn invalidate_all(&self) {
        self.cache.invalidate_settings().await;
    }
}

/// 统计数据缓存助手
pub struct StatsCache {
    cache: Arc<AppCache>,
}

impl StatsCache {
    pub fn new(cache: Arc<AppCache>) -> Self {
        Self { cache }
    }

    /// 获取统计数据
    pub async fn get(&self, key: &str) -> Option<String> {
        self.cache.get(&CacheKey::Stats(key.to_string())).await
    }

    /// 设置统计数据
    pub async fn set(&self, key: String, value: String) {
        self.cache.set(CacheKey::Stats(key), value).await;
    }

    /// 使统计失效
    pub async fn invalidate(&self, key: &str) {
        self.cache.invalidate(&CacheKey::Stats(key.to_string())).await;
    }

    /// 使所有统计失效
    pub async fn invalidate_all(&self) {
        self.cache.invalidate_stats().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_basic() {
        let cache = AppCache::new(CacheConfig::default());
        
        // 测试设置和获取
        cache.set(CacheKey::Setting("test_key".to_string()), "test_value".to_string()).await;
        let value = cache.get(&CacheKey::Setting("test_key".to_string())).await;
        assert_eq!(value, Some("test_value".to_string()));
        
        // 测试失效
        cache.invalidate(&CacheKey::Setting("test_key".to_string())).await;
        let value = cache.get(&CacheKey::Setting("test_key".to_string())).await;
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let config = CacheConfig {
            max_size: 100,
            setting_ttl: 1, // 1秒
            ..Default::default()
        };
        let cache = AppCache::new(config);
        
        cache.set(CacheKey::Setting("test_key".to_string()), "test_value".to_string()).await;
        
        // 立即获取应该成功
        let value = cache.get(&CacheKey::Setting("test_key".to_string())).await;
        assert_eq!(value, Some("test_value".to_string()));
        
        // 等待过期
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // 过期后应该返回 None
        let value = cache.get(&CacheKey::Setting("test_key".to_string())).await;
        assert_eq!(value, None);
    }
}