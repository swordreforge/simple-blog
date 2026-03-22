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

        let glob_pattern = Pattern::new(pattern)
            .map_err(|e| CacheError::Unknown(format!("Invalid glob pattern: {}", e)))?;

        // 使用 Moka 的 invalidate_entries_if 实现模式匹配删除
        // 这是两阶段模型：立即逻辑删除 + 异步物理清理
        // 性能比手动遍历 + invalidate 更高，且不会阻塞工作线程
        let _predicate_id = self
            .cache
            .invalidate_entries_if(move |key: &String, _value: &String| glob_pattern.matches(key))
            .map_err(|e| CacheError::Unknown(format!("Failed to invalidate entries: {}", e)))?;

        tracing::debug!(
            "模式匹配删除已触发: {} (使用 Moka 的 invalidate_entries_if)",
            pattern
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_local_cache_backend_new() {
        let backend = LocalCacheBackend::new(None);
        assert_eq!(backend.cache.entry_count(), 0);
    }

    #[test]
    fn test_local_cache_backend_new_with_max_size() {
        let backend = LocalCacheBackend::new(Some(100));
        assert_eq!(backend.cache.entry_count(), 0);
    }

    #[test]
    fn test_local_cache_backend_clone() {
        let backend = LocalCacheBackend::new(None);
        let cloned = backend.clone();
        assert_eq!(backend.cache.entry_count(), cloned.cache.entry_count());
    }

    #[tokio::test]
    async fn test_local_cache_get_set() {
        let backend = LocalCacheBackend::new(None);
        let key = "test_key";
        let value = "test_value";

        // 设置值
        backend.set(key, value, Duration::from_secs(3600)).await.unwrap();

        // 获取值
        let retrieved = backend.get(key).await;
        assert_eq!(retrieved, Some(value.to_string()));
    }

    #[tokio::test]
    async fn test_local_cache_get_nonexistent() {
        let backend = LocalCacheBackend::new(None);
        let retrieved = backend.get("nonexistent_key").await;
        assert_eq!(retrieved, None);
    }

    #[tokio::test]
    async fn test_local_cache_delete() {
        let backend = LocalCacheBackend::new(None);
        let key = "test_key";
        let value = "test_value";

        // 设置值
        backend.set(key, value, Duration::from_secs(3600)).await.unwrap();

        // 删除值
        backend.delete(key).await.unwrap();

        // 验证删除
        let retrieved = backend.get(key).await;
        assert_eq!(retrieved, None);
    }

    #[tokio::test]
    async fn test_local_cache_delete_many() {
        let backend = LocalCacheBackend::new(None);

        // 设置多个值
        for i in 0..5 {
            backend.set(&format!("key_{}", i), &format!("value_{}", i), Duration::from_secs(3600)).await.unwrap();
        }

        // 删除多个值
        let keys = vec!["key_0".to_string(), "key_2".to_string(), "key_4".to_string()];
        backend.delete_many(&keys).await.unwrap();

        // 验证删除
        assert_eq!(backend.get("key_0").await, None);
        assert_eq!(backend.get("key_1").await, Some("value_1".to_string()));
        assert_eq!(backend.get("key_2").await, None);
        assert_eq!(backend.get("key_3").await, Some("value_3".to_string()));
        assert_eq!(backend.get("key_4").await, None);
    }

    #[tokio::test]
    async fn test_local_cache_delete_pattern() {
        let backend = LocalCacheBackend::new(None);

        // 设置多个值
        backend.set("user:1", "data1", Duration::from_secs(3600)).await.unwrap();
        backend.set("user:2", "data2", Duration::from_secs(3600)).await.unwrap();
        backend.set("post:1", "data3", Duration::from_secs(3600)).await.unwrap();
        backend.set("user:3", "data4", Duration::from_secs(3600)).await.unwrap();

        // 删除匹配模式的所有值
        backend.delete_pattern("user:*").await.unwrap();

        // 验证删除
        assert_eq!(backend.get("user:1").await, None);
        assert_eq!(backend.get("user:2").await, None);
        assert_eq!(backend.get("post:1").await, Some("data3".to_string()));
        assert_eq!(backend.get("user:3").await, None);
    }

    #[tokio::test]
    async fn test_local_cache_delete_pattern_invalid() {
        let backend = LocalCacheBackend::new(None);

        // 尝试删除使用无效模式
        let result = backend.delete_pattern("user:[").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_local_cache_overwrite() {
        let backend = LocalCacheBackend::new(None);
        let key = "test_key";

        // 设置初始值
        backend.set(key, "value1", Duration::from_secs(3600)).await.unwrap();
        assert_eq!(backend.get(key).await, Some("value1".to_string()));

        // 覆盖值
        backend.set(key, "value2", Duration::from_secs(3600)).await.unwrap();
        assert_eq!(backend.get(key).await, Some("value2".to_string()));
    }

    #[tokio::test]
    async fn test_local_cache_with_empty_value() {
        let backend = LocalCacheBackend::new(None);
        let key = "test_key";
        let value = "";

        // 设置空值
        backend.set(key, value, Duration::from_secs(3600)).await.unwrap();

        // 获取空值
        let retrieved = backend.get(key).await;
        assert_eq!(retrieved, Some("".to_string()));
    }

    #[tokio::test]
    async fn test_local_cache_with_special_characters() {
        let backend = LocalCacheBackend::new(None);
        let key = "test:key:with:colons";
        let value = "value with spaces and 特殊字符";

        backend.set(key, value, Duration::from_secs(3600)).await.unwrap();
        let retrieved = backend.get(key).await;
        assert_eq!(retrieved, Some(value.to_string()));
    }

    #[tokio::test]
    async fn test_local_cache_large_value() {
        let backend = LocalCacheBackend::new(None);
        let key = "large_key";
        let large_value = "x".repeat(10000);

        backend.set(key, &large_value, Duration::from_secs(3600)).await.unwrap();
        let retrieved = backend.get(key).await;
        assert_eq!(retrieved, Some(large_value));
    }

    #[tokio::test]
    async fn test_local_cache_send_sync() {
        // 确保缓存可以在异步任务中使用
        let backend = LocalCacheBackend::new(None);

        let backend1 = backend.clone();
        let backend2 = backend.clone();

        let handle1 = tokio::spawn(async move {
            backend1.set("key1", "value1", Duration::from_secs(3600)).await.unwrap();
        });

        let handle2 = tokio::spawn(async move {
            backend2.set("key2", "value2", Duration::from_secs(3600)).await.unwrap();
        });

        handle1.await.unwrap();
        handle2.await.unwrap();

        assert_eq!(backend.get("key1").await, Some("value1".to_string()));
        assert_eq!(backend.get("key2").await, Some("value2".to_string()));
    }
}
