//! 无锁缓存实现
//!
//! 使用原子操作和无锁数据结构实现高性能缓存，减少锁竞争。

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// 无锁缓存统计信息
///
/// 使用原子操作实现无锁统计
#[derive(Debug)]
pub struct LockfreeCacheStats {
    /// 缓存命中次数
    pub hits: AtomicUsize,
    /// 缓存未命中次数
    pub misses: AtomicUsize,
    /// 缓存驱逐次数
    pub evictions: AtomicUsize,
    /// 当前缓存大小
    pub size: AtomicUsize,
    /// 总查询次数
    pub total_queries: AtomicUsize,
}

impl Default for LockfreeCacheStats {
    fn default() -> Self {
        Self {
            hits: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
            evictions: AtomicUsize::new(0),
            size: AtomicUsize::new(0),
            total_queries: AtomicUsize::new(0),
        }
    }
}

impl LockfreeCacheStats {
    /// 记录缓存命中
    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
        self.total_queries.fetch_add(1, Ordering::Relaxed);
    }

    /// 记录缓存未命中
    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
        self.total_queries.fetch_add(1, Ordering::Relaxed);
    }

    /// 记录缓存驱逐
    pub fn record_eviction(&self) {
        self.evictions.fetch_add(1, Ordering::Relaxed);
    }

    /// 更新缓存大小
    pub fn update_size(&self, delta: isize) {
        if delta > 0 {
            self.size.fetch_add(delta as usize, Ordering::Relaxed);
        } else {
            self.size.fetch_sub(delta.abs() as usize, Ordering::Relaxed);
        }
    }

    /// 获取缓存命中率
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let total = self.total_queries.load(Ordering::Relaxed);
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    /// 获取统计快照
    pub fn snapshot(&self) -> CacheStatsSnapshot {
        CacheStatsSnapshot {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            evictions: self.evictions.load(Ordering::Relaxed),
            size: self.size.load(Ordering::Relaxed),
            total_queries: self.total_queries.load(Ordering::Relaxed),
        }
    }
}

/// 缓存统计快照
#[derive(Debug, Clone)]
pub struct CacheStatsSnapshot {
    pub hits: usize,
    pub misses: usize,
    pub evictions: usize,
    pub size: usize,
    pub total_queries: usize,
}

/// 缓存条目
#[derive(Debug)]
struct CacheEntry<T> {
    value: T,
    access_count: AtomicUsize,
    last_access: AtomicU64,
    created_at: Instant,
}

impl<T> CacheEntry<T> {
    fn new(value: T) -> Self {
        Self {
            value,
            access_count: AtomicUsize::new(1),
            last_access: AtomicU64::new(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            ),
            created_at: Instant::now(),
        }
    }

    fn touch(&self) {
        self.access_count.fetch_add(1, Ordering::Relaxed);
        self.last_access.store(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            Ordering::Relaxed,
        );
    }

    fn access_count(&self) -> usize {
        self.access_count.load(Ordering::Relaxed)
    }
}

// 简化的SystemTime模拟
struct SystemTime;
const UNIX_EPOCH: SystemTime = SystemTime;

impl SystemTime {
    fn now() -> Self {
        Self
    }

    fn duration_since(&self, _epoch: SystemTime) -> Result<Duration, std::time::SystemTimeError> {
        // 简化实现，返回当前时间戳
        Ok(Duration::from_secs(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        ))
    }
}

/// 无锁LRU缓存
///
/// 使用RwLock保护HashMap，但统计信息使用原子操作实现无锁更新
pub struct LockfreeLruCache<K, V> {
    data: std::sync::RwLock<HashMap<K, Box<CacheEntry<V>>>>,
    max_size: usize,
    stats: LockfreeCacheStats,
}

impl<K, V> LockfreeLruCache<K, V>
where
    K: std::hash::Hash + Eq + Clone,
{
    pub fn new(max_size: usize) -> Self {
        Self {
            data: std::sync::RwLock::new(HashMap::with_capacity(max_size)),
            max_size,
            stats: LockfreeCacheStats::default(),
        }
    }

    /// 获取缓存值
    pub fn get(&self, key: &K) -> Option<V>
    where
        V: Clone,
    {
        let guard = self.data.read().unwrap();
        if let Some(entry) = guard.get(key) {
            self.stats.record_hit();
            entry.touch();
            Some(entry.value.clone())
        } else {
            self.stats.record_miss();
            None
        }
    }

    /// 插入缓存值
    pub fn insert(&self, key: K, value: V) {
        let mut guard = self.data.write().unwrap();

        // 检查是否需要驱逐
        if guard.len() >= self.max_size && !guard.contains_key(&key) {
            self.evict_lru(&mut guard);
        }

        let is_new = !guard.contains_key(&key);
        guard.insert(key, Box::new(CacheEntry::new(value)));

        if is_new {
            self.stats.update_size(1);
        }
    }

    /// 移除缓存值
    pub fn remove(&self, key: &K) -> Option<V> {
        let mut guard = self.data.write().unwrap();
        if guard.remove(key).is_some() {
            self.stats.update_size(-1);
            // 需要返回值，但这里简化实现
            None
        } else {
            None
        }
    }

    /// 清空缓存
    pub fn clear(&self) {
        let mut guard = self.data.write().unwrap();
        let old_size = guard.len();
        guard.clear();
        self.stats.update_size(-(old_size as isize));
    }

    /// 获取缓存大小
    pub fn len(&self) -> usize {
        self.stats.size.load(Ordering::Relaxed)
    }

    /// 检查缓存是否为空
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 获取统计信息
    pub fn stats(&self) -> &LockfreeCacheStats {
        &self.stats
    }

    /// 驱逐最少使用的条目
    fn evict_lru(&self, guard: &mut HashMap<K, Box<CacheEntry<V>>>) {
        if guard.is_empty() {
            return;
        }

        // 找到访问次数最少的条目
        let mut lru_key = None;
        let mut min_access = usize::MAX;

        for (key, entry) in guard.iter() {
            let access_count = entry.access_count();
            if access_count < min_access {
                min_access = access_count;
                lru_key = Some(key.clone());
            }
        }

        if let Some(key) = lru_key {
            guard.remove(&key);
            self.stats.record_eviction();
            self.stats.update_size(-1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lockfree_cache_stats() {
        let stats = LockfreeCacheStats::default();

        // 测试命中记录
        stats.record_hit();
        assert_eq!(stats.hits.load(Ordering::Relaxed), 1);
        assert_eq!(stats.total_queries.load(Ordering::Relaxed), 1);

        // 测试未命中记录
        stats.record_miss();
        assert_eq!(stats.misses.load(Ordering::Relaxed), 1);
        assert_eq!(stats.total_queries.load(Ordering::Relaxed), 2);

        // 测试命中率
        assert_eq!(stats.hit_rate(), 0.5);

        // 测试大小更新
        stats.update_size(1);
        assert_eq!(stats.size.load(Ordering::Relaxed), 1);

        stats.update_size(-1);
        assert_eq!(stats.size.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_lockfree_lru_cache_basic() {
        let cache = LockfreeLruCache::new(3);

        // 测试插入
        cache.insert("key1", "value1");
        cache.insert("key2", "value2");

        assert_eq!(cache.len(), 2);

        // 测试获取
        let value = cache.get(&"key1");
        assert_eq!(value, Some("value1"));

        // 测试未命中
        let value = cache.get(&"key3");
        assert_eq!(value, None);

        // 验证统计信息
        assert_eq!(cache.stats().hits.load(Ordering::Relaxed), 1);
        assert_eq!(cache.stats().misses.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_lockfree_lru_cache_eviction() {
        let cache = LockfreeLruCache::new(2);

        cache.insert("key1", "value1");
        cache.insert("key2", "value2");

        // 访问key1增加其访问计数
        cache.get(&"key1");

        // 插入key3，应该驱逐key2（访问次数最少）
        cache.insert("key3", "value3");

        assert_eq!(cache.len(), 2);
        assert_eq!(cache.get(&"key1"), Some("value1"));
        assert_eq!(cache.get(&"key2"), None);
        assert_eq!(cache.get(&"key3"), Some("value3"));

        // 验证驱逐统计
        assert_eq!(cache.stats().evictions.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_lockfree_cache_concurrent_access() {
        let cache = Arc::new(LockfreeLruCache::new(100));
        let mut handles = vec![];

        // 多线程并发写入
        for thread_id in 0..10 {
            let cache_clone = Arc::clone(&cache);
            let handle = std::thread::spawn(move || {
                for i in 0..100 {
                    let key = format!("thread-{}-key-{}", thread_id, i);
                    cache_clone.insert(key, format!("value-{}", i));
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // 验证缓存大小
        assert_eq!(cache.len(), 100);

        // 验证统计信息
        let stats = cache.stats();
        assert!(stats.size.load(Ordering::Relaxed) > 0);
    }
}