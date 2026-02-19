use super::backend::{CacheBackend, CacheError};
use async_trait::async_trait;
use moka::sync::Cache;
use std::time::Duration;

/// 本地内存缓存后端（降级方案）- 使用 Moka 实现
/// Moka 使用分段锁且不会阻塞工作线程，可以彻底避免死锁问题
#[derive(Clone)]
pub struct LocalCacheBackend {
    cache: Cache<String, String>,
}

impl LocalCacheBackend {
    /// 创建新的本地缓存后端
    pub fn new(max_size: Option<usize>) -> Self {
        let max_capacity = max_size.unwrap_or(10000);

        // 使用 Moka 的同步缓存
        // 设置最大容量、TTL 和支持闭包失效
        let cache = Cache::builder()
            .max_capacity(max_capacity as u64)
            .time_to_live(Duration::from_secs(3600)) // 默认 TTL 1 小时
            .support_invalidation_closures() // 启用闭包失效支持
            .build();

        Self { cache }
    }
}

#[async_trait]
impl CacheBackend for LocalCacheBackend {
    async fn get(&self, key: &str) -> Option<String> {
        // Moka 自动处理过期，无需手动检查
        self.cache.get(key)
    }

    async fn set(&self, key: &str, value: &str, _ttl: Duration) -> Result<(), CacheError> {
        // Moka 的 sync 缓存使用全局 TTL，在构造时已设置为 1 小时
        // 如果需要每个 key 不同 TTL，可以考虑使用 Moka 的 async 版本
        self.cache.insert(key.to_string(), value.to_string());
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), CacheError> {
        self.cache.invalidate(key);
        Ok(())
    }

    async fn delete_many(&self, keys: &[String]) -> Result<(), CacheError> {
        // Moka 的 invalidate_all 不接受参数，需要循环删除
        for key in keys {
            self.cache.invalidate(key);
        }
        Ok(())
    }

    async fn delete_pattern(&self, pattern: &str) -> Result<(), CacheError> {
        use glob::Pattern;

        let glob_pattern = Pattern::new(pattern).map_err(|e| {
            CacheError::Unknown(format!("Invalid glob pattern: {}", e))
        })?;

        // 使用 Moka 的 invalidate_entries_if 实现模式匹配删除
        // 这是两阶段模型：立即逻辑删除 + 异步物理清理
        // 性能比手动遍历 + invalidate 更高，且不会阻塞工作线程
        let _predicate_id = self.cache.invalidate_entries_if(move |key: &String, _value: &String| {
            glob_pattern.matches(key)
        }).map_err(|e| {
            CacheError::Unknown(format!("Failed to invalidate entries: {}", e))
        })?;

        tracing::debug!("模式匹配删除已触发: {} (使用 Moka 的 invalidate_entries_if)", pattern);

        Ok(())
    }
}