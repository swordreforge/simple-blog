//! 缓存模块
//!
//! 提供路由缓存和批量操作支持，优化性能。

use crate::core::RouteEntry;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// 路由缓存条目
#[derive(Clone)]
struct CacheEntry<T> {
    value: T,
    timestamp: Instant,
    ttl: Duration,
}

impl<T> CacheEntry<T> {
    fn new(value: T, ttl: Duration) -> Self {
        Self {
            value,
            timestamp: Instant::now(),
            ttl,
        }
    }

    fn is_expired(&self) -> bool {
        self.timestamp.elapsed() >= self.ttl
    }
}

/// 路由缓存
///
/// 提供带 TTL 的缓存功能，减少重复查找的开销。
///
/// # 示例
///
/// ```
/// use dynamic_route_actix::core::cache::RouteCache;
/// use std::time::Duration;
///
/// let cache = RouteCache::new(Duration::from_secs(60));
/// cache.insert("/hello", "world".to_string());
///
/// if let Some(value) = cache.get("/hello") {
///     assert_eq!(value, "world");
/// }
/// ```
pub struct RouteCache<T> {
    entries: Arc<RwLock<HashMap<String, CacheEntry<T>>>>,
    ttl: Duration,
}

impl<T> RouteCache<T>
where
    T: Clone,
{
    /// 创建新的缓存
    ///
    /// # 参数
    ///
    /// * `ttl` - 缓存条目的生存时间
    pub fn new(ttl: Duration) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        }
    }

    /// 插入缓存条目
    ///
    /// # 参数
    ///
    /// * `key` - 缓存键
    /// * `value` - 缓存值
    pub fn insert(&self, key: &str, value: T) {
        let mut guard = self.entries.write().unwrap();
        guard.insert(key.to_string(), CacheEntry::new(value, self.ttl));
    }

    /// 获取缓存条目
    ///
    /// # 参数
    ///
    /// * `key` - 缓存键
    ///
    /// # 返回
    ///
    /// 如果条目存在且未过期，返回 `Some(T)`；否则返回 `None`
    pub fn get(&self, key: &str) -> Option<T> {
        // 先使用读锁尝试获取
        {
            let guard = self.entries.read().unwrap();
            if let Some(entry) = guard.get(key) {
                if !entry.is_expired() {
                    return Some(entry.value.clone());
                }
            }
        }

        // 如果不存在或已过期，使用写锁删除
        let mut guard = self.entries.write().unwrap();
        if let Some(entry) = guard.get(key) {
            if entry.is_expired() {
                guard.remove(key);
                return None;
            }
            Some(entry.value.clone())
        } else {
            None
        }
    }

    /// 移除缓存条目
    ///
    /// # 参数
    ///
    /// * `key` - 缓存键
    ///
    /// # 返回
    ///
    /// 如果条目存在，返回 `true`；否则返回 `false`
    pub fn remove(&self, key: &str) -> bool {
        let mut guard = self.entries.write().unwrap();
        guard.remove(key).is_some()
    }

    /// 清空缓存
    pub fn clear(&self) {
        let mut guard = self.entries.write().unwrap();
        guard.clear();
    }

    /// 清理过期的缓存条目
    pub fn cleanup_expired(&self) {
        let mut guard = self.entries.write().unwrap();
        guard.retain(|_, entry| !entry.is_expired());
    }

    /// 获取缓存大小
    pub fn size(&self) -> usize {
        let guard = self.entries.read().unwrap();
        guard.len()
    }

    /// 获取缓存命中率（需要手动维护统计信息）
    pub fn hit_rate(&self) -> f64 {
        // 在实际实现中，需要维护命中和未命中的计数
        0.0
    }
}

/// 批量操作
///
/// 提供批量插入、删除等操作，减少锁竞争。
pub struct BatchOperations;

impl BatchOperations {
    /// 批量插入路由
    ///
    /// # 参数
    ///
    /// * `table` - 路由表
    /// * `routes` - 要插入的路由集合
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::{RouteTable, SimpleRoute, RouteEntry, core::cache::BatchOperations};
    /// use std::collections::HashMap;
    ///
    /// let table = RouteTable::new();
    /// let mut routes: HashMap<String, Box<dyn RouteEntry>> = HashMap::new();
    /// routes.insert("/route1".to_string(), Box::new(SimpleRoute::new("body1", "text/plain")));
    /// routes.insert("/route2".to_string(), Box::new(SimpleRoute::new("body2", "text/plain")));
    ///
    /// BatchOperations::batch_insert(&table, routes);
    /// ```
    pub fn batch_insert(table: &crate::RouteTable, routes: HashMap<String, Box<dyn RouteEntry>>) {
        // 使用优化的批量插入方法
        table.batch_insert(routes);
    }

    /// 批量删除路由
    ///
    /// # 参数
    ///
    /// * `table` - 路由表
    /// * `paths` - 要删除的路径集合
    ///
    /// # 返回
    ///
    /// 返回成功删除的路由数量
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::{RouteTable, SimpleRoute, core::cache::BatchOperations};
    /// use std::collections::HashSet;
    ///
    /// let table = RouteTable::new();
    /// table.insert("/route1".to_string(), Box::new(SimpleRoute::new("body1", "text/plain")));
    /// table.insert("/route2".to_string(), Box::new(SimpleRoute::new("body2", "text/plain")));
    ///
    /// let paths = vec!["/route1".to_string()].into_iter().collect();
    /// let deleted = BatchOperations::batch_remove(&table, paths);
    /// assert_eq!(deleted, 1);
    /// ```
    pub fn batch_remove(table: &crate::RouteTable, paths: HashSet<String>) -> usize {
        let mut count = 0;
        for path in paths {
            if table.remove(&path) {
                count += 1;
            }
        }
        count
    }

    /// 批量检查路由是否存在
    ///
    /// # 参数
    ///
    /// * `table` - 路由表
    /// * `paths` - 要检查的路径集合
    ///
    /// # 返回
    ///
    /// 返回包含每个路径是否存在的结果映射
    pub fn batch_contains(
        table: &crate::RouteTable,
        paths: HashSet<String>,
    ) -> HashMap<String, bool> {
        paths
            .into_iter()
            .map(|path| (path.clone(), table.contains(&path)))
            .collect()
    }
}

/// 性能优化选项
#[derive(Debug, Clone)]
pub struct PerformanceOptions {
    /// 是否启用缓存
    pub enable_cache: bool,
    /// 缓存 TTL
    pub cache_ttl: Duration,
    /// 是否启用批量操作
    pub enable_batch_operations: bool,
    /// 批量操作的最大大小
    pub max_batch_size: usize,
}

impl Default for PerformanceOptions {
    fn default() -> Self {
        Self {
            enable_cache: true,
            cache_ttl: Duration::from_secs(60),
            enable_batch_operations: true,
            max_batch_size: 1000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RouteTable, SimpleRoute};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_cache_insert_and_get() {
        let cache = RouteCache::new(Duration::from_secs(60));

        cache.insert("/hello", "world".to_string());
        assert_eq!(cache.get("/hello"), Some("world".to_string()));
        assert_eq!(cache.get("/nonexistent"), None);
    }

    #[test]
    fn test_cache_expiry() {
        let cache = RouteCache::new(Duration::from_millis(100));

        cache.insert("/hello", "world".to_string());
        assert_eq!(cache.get("/hello"), Some("world".to_string()));

        // 等待缓存过期
        thread::sleep(Duration::from_millis(150));
        assert_eq!(cache.get("/hello"), None);
    }

    #[test]
    fn test_cache_remove() {
        let cache = RouteCache::new(Duration::from_secs(60));

        cache.insert("/hello", "world".to_string());
        assert!(cache.remove("/hello"));
        assert!(!cache.remove("/hello"));
        assert_eq!(cache.get("/hello"), None);
    }

    #[test]
    fn test_cache_clear() {
        let cache = RouteCache::new(Duration::from_secs(60));

        cache.insert("/route1", "value1".to_string());
        cache.insert("/route2", "value2".to_string());
        assert_eq!(cache.size(), 2);

        cache.clear();
        assert_eq!(cache.size(), 0);
    }

    #[test]
    fn test_cache_cleanup_expired() {
        let cache = RouteCache::new(Duration::from_millis(100));

        cache.insert("/route1", "value1".to_string());
        cache.insert("/route2", "value2".to_string());
        assert_eq!(cache.size(), 2);

        // 等待缓存过期
        thread::sleep(Duration::from_millis(150));

        cache.cleanup_expired();
        assert_eq!(cache.size(), 0);
    }

    #[test]
    fn test_batch_insert() {
        let table = RouteTable::new();
        let mut routes: HashMap<String, Box<dyn RouteEntry>> = HashMap::new();

        for i in 0..10 {
            routes.insert(
                format!("/route-{}", i),
                Box::new(SimpleRoute::new(format!("body-{}", i), "text/plain")),
            );
        }

        BatchOperations::batch_insert(&table, routes);
        assert_eq!(table.count(), 10);
    }

    #[test]
    fn test_batch_remove() {
        let table = RouteTable::new();

        for i in 0..10 {
            table.insert(
                format!("/route-{}", i),
                Box::new(SimpleRoute::new(format!("body-{}", i), "text/plain")),
            );
        }

        let paths: HashSet<String> = (0..5).map(|i| format!("/route-{}", i)).collect();
        let deleted = BatchOperations::batch_remove(&table, paths);

        assert_eq!(deleted, 5);
        assert_eq!(table.count(), 5);
    }

    #[test]
    fn test_batch_contains() {
        let table = RouteTable::new();

        table.insert(
            "/route1".to_string(),
            Box::new(SimpleRoute::new("body1", "text/plain")),
        );
        table.insert(
            "/route2".to_string(),
            Box::new(SimpleRoute::new("body2", "text/plain")),
        );

        let paths: HashSet<String> = vec![
            "/route1".to_string(),
            "/route2".to_string(),
            "/route3".to_string(),
        ]
        .into_iter()
        .collect();

        let results = BatchOperations::batch_contains(&table, paths);

        assert_eq!(results.get("/route1"), Some(&true));
        assert_eq!(results.get("/route2"), Some(&true));
        assert_eq!(results.get("/route3"), Some(&false));
    }

    #[test]
    fn test_performance_options_default() {
        let options = PerformanceOptions::default();
        assert!(options.enable_cache);
        assert_eq!(options.cache_ttl, Duration::from_secs(60));
        assert!(options.enable_batch_operations);
        assert_eq!(options.max_batch_size, 1000);
    }

    #[test]
    fn test_cache_concurrent_access() {
        use std::sync::Arc;
        let cache = Arc::new(RouteCache::new(Duration::from_secs(60)));
        let mut handles = vec![];

        // 并发写入
        for i in 0..10 {
            let cache_clone = Arc::clone(&cache);
            let handle = thread::spawn(move || {
                cache_clone.insert(&format!("/route-{}", i), format!("value-{}", i));
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // 验证所有数据都已写入
        for i in 0..10 {
            assert_eq!(
                cache.get(&format!("/route-{}", i)),
                Some(format!("value-{}", i))
            );
        }

        assert_eq!(cache.size(), 10);
    }
}
