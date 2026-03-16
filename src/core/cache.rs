//! 缓存模块
//!
//! 提供路由缓存和批量操作支持，优化性能。
//! 实现了LRU缓存替换算法、缓存预热机制和智能缓存失效策略。

use crate::core::RouteEntry;
use lru::LruCache;
use std::collections::{HashMap, HashSet};
use std::num::NonZeroUsize;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// 路由缓存条目
#[derive(Clone)]
struct CacheEntry<T> {
    value: T,
    timestamp: Instant,
    ttl: Duration,
    access_count: u64,
    last_access_time: Instant,
    access_frequency: f64, // 访问频率（次/秒）
}

impl<T> CacheEntry<T> {
    fn new(value: T, ttl: Duration) -> Self {
        let now = Instant::now();
        Self {
            value,
            timestamp: now,
            ttl,
            access_count: 1,
            last_access_time: now,
            access_frequency: 1.0,
        }
    }

    fn is_expired(&self) -> bool {
        self.timestamp.elapsed() >= self.ttl
    }

    fn record_access(&mut self) {
        self.access_count += 1;
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_access_time).as_secs_f64();
        self.last_access_time = now;

        // 计算访问频率（指数移动平均）
        let alpha = 0.3; // 平滑因子
        let current_frequency = if elapsed > 0.0 { 1.0 / elapsed } else { 1.0 };
        self.access_frequency = alpha * current_frequency + (1.0 - alpha) * self.access_frequency;
    }

    /// 计算缓存条目的优先级分数
    /// 分数越高，越应该保留在缓存中
    fn priority_score(&self) -> f64 {
        // 考虑因素：
        // 1. 访问频率（权重：0.4）
        // 2. 访问次数（权重：0.3）
        // 3. 剩余TTL（权重：0.3）

        let freq_score = self.access_frequency.min(10.0) / 10.0;
        let count_score = (self.access_count as f64).min(100.0) / 100.0;
        let ttl_score = (1.0 - (self.timestamp.elapsed().as_secs_f64() / self.ttl.as_secs_f64())).max(0.0);

        0.4 * freq_score + 0.3 * count_score + 0.3 * ttl_score
    }
}

/// 缓存统计信息
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// 缓存命中次数
    pub hits: u64,
    /// 缓存未命中次数
    pub misses: u64,
    /// 缓存驱逐次数
    pub evictions: u64,
    /// 缓存过期清理次数
    pub expirations: u64,
    /// 总访问次数
    pub total_accesses: u64,
}

impl CacheStats {
    fn new() -> Self {
        Self {
            hits: 0,
            misses: 0,
            evictions: 0,
            expirations: 0,
            total_accesses: 0,
        }
    }

    /// 计算缓存命中率
    pub fn hit_rate(&self) -> f64 {
        if self.total_accesses == 0 {
            0.0
        } else {
            self.hits as f64 / self.total_accesses as f64
        }
    }
}

/// 路由缓存
///
/// 提供带 LRU 替换策略和 TTL 的缓存功能，减少重复查找的开销。
///
/// # 示例
///
/// ```
/// use dynamic_route_actix::core::cache::RouteCache;
/// use std::time::Duration;
///
/// let cache = RouteCache::new(1000, Duration::from_secs(60));
/// cache.insert("/hello", "world".to_string());
///
/// if let Some(value) = cache.get("/hello") {
///     assert_eq!(value, "world");
/// }
/// ```
pub struct RouteCache<T> {
    // LRU缓存
    lru_cache: Arc<RwLock<LruCache<String, CacheEntry<T>>>>,
    ttl: Duration,
    stats: Arc<RwLock<CacheStats>>,
}

impl<T> RouteCache<T>
where
    T: Clone,
{
    /// 创建新的缓存
    ///
    /// # 参数
    ///
    /// * `capacity` - 缓存容量
    /// * `ttl` - 缓存条目的生存时间
    pub fn new(capacity: usize, ttl: Duration) -> Self {
        Self {
            lru_cache: Arc::new(RwLock::new(LruCache::new(
                NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(1000).unwrap()),
            ))),
            ttl,
            stats: Arc::new(RwLock::new(CacheStats::new())),
        }
    }

    /// 插入缓存条目
    ///
    /// # 参数
    ///
    /// * `key` - 缓存键
    /// * `value` - 缓存值
    pub fn insert(&self, key: &str, value: T) {
        let mut guard = self.lru_cache.write().unwrap();
        // 检查是否会导致驱逐
        let will_evict = guard.len() == guard.cap().get();
        guard.put(key.to_string(), CacheEntry::new(value, self.ttl));
        // LRU会自动驱逐最久未使用的条目
        if will_evict {
            let mut stats = self.stats.write().unwrap();
            stats.evictions += 1;
        }
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
        let mut stats_guard = self.stats.write().unwrap();
        stats_guard.total_accesses += 1;

        let mut cache_guard = self.lru_cache.write().unwrap();

        if let Some(entry) = cache_guard.get_mut(key) {
            if entry.is_expired() {
                // 缓存已过期
                cache_guard.pop(key);
                stats_guard.misses += 1;
                stats_guard.expirations += 1;
                return None;
            }

            // 缓存命中，记录访问
            entry.record_access();
            let value = entry.value.clone();
            stats_guard.hits += 1;
            drop(cache_guard);
            drop(stats_guard);
            return Some(value);
        }

        // 缓存未命中
        stats_guard.misses += 1;
        drop(stats_guard);
        None
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
        let mut guard = self.lru_cache.write().unwrap();
        guard.pop(key).is_some()
    }

    /// 清空缓存
    pub fn clear(&self) {
        let mut guard = self.lru_cache.write().unwrap();
        guard.clear();
    }

    /// 清理过期的缓存条目
    pub fn cleanup_expired(&self) {
        let mut guard = self.lru_cache.write().unwrap();
        let mut keys_to_remove = Vec::new();

        for (key, entry) in guard.iter() {
            if entry.is_expired() {
                keys_to_remove.push(key.clone());
            }
        }

        let count = keys_to_remove.len();
        for key in keys_to_remove {
            guard.pop(&key);
        }

        if count > 0 {
            let mut stats = self.stats.write().unwrap();
            stats.expirations += count as u64;
        }
    }

    /// 智能缓存失效
    ///
    /// 基于访问频率和优先级分数，智能地移除低优先级的缓存条目
    /// 以释放空间给新的、更重要的条目
    pub fn smart_evict(&self, target_size: usize) {
        let mut guard = self.lru_cache.write().unwrap();

        while guard.len() > target_size {
            // 找到优先级最低的条目
            let key_to_remove = guard.iter().min_by(|a, b| {
                let score_a = a.1.priority_score();
                let score_b = b.1.priority_score();
                score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
            }).map(|(k, _)| k.clone());

            if let Some(key) = key_to_remove {
                guard.pop(&key);
                let mut stats = self.stats.write().unwrap();
                stats.evictions += 1;
            } else {
                break;
            }
        }
    }

    /// 缓存预热
    ///
    /// 批量插入预定义的路由，提前填充缓存
    ///
    /// # 参数
    ///
    /// * `entries` - 要预热的缓存条目集合
    pub fn warmup(&self, entries: HashMap<String, T>) {
        let mut guard = self.lru_cache.write().unwrap();

        for (key, value) in entries {
            guard.put(key, CacheEntry::new(value, self.ttl));
        }
    }

    /// 获取缓存大小
    pub fn size(&self) -> usize {
        let guard = self.lru_cache.read().unwrap();
        guard.len()
    }

    /// 获取缓存容量
    pub fn capacity(&self) -> usize {
        let guard = self.lru_cache.read().unwrap();
        guard.cap().get()
    }

    /// 获取缓存命中率
    pub fn hit_rate(&self) -> f64 {
        let stats = self.stats.read().unwrap();
        stats.hit_rate()
    }

    /// 获取缓存统计信息
    pub fn stats(&self) -> CacheStats {
        let stats = self.stats.read().unwrap();
        stats.clone()
    }

    /// 重置缓存统计信息
    pub fn reset_stats(&self) {
        let mut stats = self.stats.write().unwrap();
        *stats = CacheStats::new();
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
    /// 缓存容量
    pub cache_capacity: usize,
    /// 缓存 TTL
    pub cache_ttl: Duration,
    /// 是否启用批量操作
    pub enable_batch_operations: bool,
    /// 批量操作的最大大小
    pub max_batch_size: usize,
    /// 是否启用智能缓存失效
    pub enable_smart_eviction: bool,
    /// 自动清理过期缓存的间隔
    pub cleanup_interval: Duration,
}

impl Default for PerformanceOptions {
    fn default() -> Self {
        Self {
            enable_cache: true,
            cache_capacity: 1000,
            cache_ttl: Duration::from_secs(60),
            enable_batch_operations: true,
            max_batch_size: 1000,
            enable_smart_eviction: true,
            cleanup_interval: Duration::from_secs(300),
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
        let cache = RouteCache::new(1000, Duration::from_secs(60));

        cache.insert("/hello", "world".to_string());
        assert_eq!(cache.get("/hello"), Some("world".to_string()));
        assert_eq!(cache.get("/nonexistent"), None);
    }

    #[test]
    fn test_cache_expiry() {
        let cache = RouteCache::new(1000, Duration::from_millis(100));

        cache.insert("/hello", "world".to_string());
        assert_eq!(cache.get("/hello"), Some("world".to_string()));

        // 等待缓存过期
        thread::sleep(Duration::from_millis(150));
        assert_eq!(cache.get("/hello"), None);
    }

    #[test]
    fn test_lru_eviction() {
        let cache = RouteCache::new(3, Duration::from_secs(60));

        cache.insert("/route1", "value1".to_string());
        cache.insert("/route2", "value2".to_string());
        cache.insert("/route3", "value3".to_string());
        assert_eq!(cache.size(), 3);

        // 插入第4个条目，应该驱逐LRU条目
        cache.insert("/route4", "value4".to_string());
        assert_eq!(cache.size(), 3);

        // route1应该被驱逐
        assert_eq!(cache.get("/route1"), None);
        assert_eq!(cache.get("/route2"), Some("value2".to_string()));
        assert_eq!(cache.get("/route3"), Some("value3".to_string()));
        assert_eq!(cache.get("/route4"), Some("value4".to_string()));
    }

    #[test]
    fn test_cache_remove() {
        let cache = RouteCache::new(1000, Duration::from_secs(60));

        cache.insert("/hello", "world".to_string());
        assert!(cache.remove("/hello"));
        assert!(!cache.remove("/hello"));
        assert_eq!(cache.get("/hello"), None);
    }

    #[test]
    fn test_cache_clear() {
        let cache = RouteCache::new(1000, Duration::from_secs(60));

        cache.insert("/route1", "value1".to_string());
        cache.insert("/route2", "value2".to_string());
        assert_eq!(cache.size(), 2);

        cache.clear();
        assert_eq!(cache.size(), 0);
    }

    #[test]
    fn test_cache_cleanup_expired() {
        let cache = RouteCache::new(1000, Duration::from_millis(100));

        cache.insert("/route1", "value1".to_string());
        cache.insert("/route2", "value2".to_string());
        assert_eq!(cache.size(), 2);

        // 等待缓存过期
        thread::sleep(Duration::from_millis(150));

        cache.cleanup_expired();
        assert_eq!(cache.size(), 0);
    }

    #[test]
    fn test_cache_stats() {
        let cache = RouteCache::new(1000, Duration::from_secs(60));

        cache.insert("/route1", "value1".to_string());
        cache.insert("/route2", "value2".to_string());

        // 命中
        cache.get("/route1");
        cache.get("/route1");
        cache.get("/route2");

        // 未命中
        cache.get("/route3");
        cache.get("/route4");

        let stats = cache.stats();
        assert_eq!(stats.hits, 3);
        assert_eq!(stats.misses, 2);
        assert_eq!(stats.total_accesses, 5);
        assert_eq!(stats.hit_rate(), 0.6);
    }

    #[test]
    fn test_cache_warmup() {
        let cache = RouteCache::new(1000, Duration::from_secs(60));

        let mut entries = HashMap::new();
        entries.insert("/route1".to_string(), "value1".to_string());
        entries.insert("/route2".to_string(), "value2".to_string());
        entries.insert("/route3".to_string(), "value3".to_string());

        cache.warmup(entries);
        assert_eq!(cache.size(), 3);
        assert_eq!(cache.get("/route1"), Some("value1".to_string()));
        assert_eq!(cache.get("/route2"), Some("value2".to_string()));
        assert_eq!(cache.get("/route3"), Some("value3".to_string()));
    }

    #[test]
    fn test_smart_eviction() {
        let cache = RouteCache::new(1000, Duration::from_secs(60));

        // 插入多个条目
        for i in 0..10 {
            cache.insert(&format!("/route-{}", i), format!("value-{}", i));
        }

        // 频繁访问某些条目
        for _ in 0..100 {
            cache.get("/route-0");
            cache.get("/route-1");
        }

        // 智能驱逐到目标大小
        cache.smart_evict(5);
        assert_eq!(cache.size(), 5);

        // 高频访问的条目应该保留
        assert_eq!(cache.get("/route-0"), Some("value-0".to_string()));
        assert_eq!(cache.get("/route-1"), Some("value-1".to_string()));
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
        assert_eq!(options.cache_capacity, 1000);
        assert_eq!(options.cache_ttl, Duration::from_secs(60));
        assert!(options.enable_batch_operations);
        assert_eq!(options.max_batch_size, 1000);
        assert!(options.enable_smart_eviction);
        assert_eq!(options.cleanup_interval, Duration::from_secs(300));
    }

    #[test]
    fn test_cache_concurrent_access() {
        use std::sync::Arc;
        let cache = Arc::new(RouteCache::new(1000, Duration::from_secs(60)));
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

    #[test]
    fn test_cache_entry_priority_score() {
        let entry = CacheEntry::new("value".to_string(), Duration::from_secs(60));
        let score = entry.priority_score();
        assert!(score >= 0.0 && score <= 1.0);
    }
}