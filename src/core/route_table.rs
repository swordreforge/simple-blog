use super::RouteEntry;
use super::route_radix_tree::RouteRadixTree;
use super::cache::RouteCache;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// 分片数
const NUM_SHARDS: usize = 16;

/// 缓存容量
const CACHE_CAPACITY: usize = 1000;

/// 缓存TTL（秒）
const CACHE_TTL_SECS: u64 = 60;

/// 路由表分片
struct RouteTableShard {
    inner: RouteRadixTree,
    /// 用于跟踪此分片中的路由数量
    count: usize,
}

impl RouteTableShard {
    fn new() -> Self {
        Self {
            inner: RouteRadixTree::new(),
            count: 0,
        }
    }
}

/// 线程安全的路由表
///
/// 使用分片锁（Sharded Locking）实现，将路由表分成多个分片，每个分片有自己的读写锁。
/// 这样可以显著减少高并发场景下的锁竞争。
///
/// # 线程安全
///
/// `RouteTable` 可以安全地在多个线程之间共享和克隆。
///
/// # 示例
///
/// ```
/// use dynamic_route_actix::RouteTable;
///
/// let table = RouteTable::new();
/// assert_eq!(table.count(), 0);
/// ```
#[derive(Clone)]
pub struct RouteTable {
    /// 分片数组，每个分片有自己的读写锁
    shards: [Arc<RwLock<RouteTableShard>>; NUM_SHARDS],
    /// 用于跟踪路由数量的原子计数器
    count: Arc<AtomicUsize>,
    /// 路由查找缓存（路径 -> Arc<RouteEntry>）
    cache: Arc<RouteCache<Arc<dyn RouteEntry>>>,
}

impl RouteTable {
    /// 创建一个新的空路由表
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::RouteTable;
    ///
    /// let table = RouteTable::new();
    /// assert_eq!(table.count(), 0);
    /// ```
    pub fn new() -> Self {
        // 创建分片数组
        let shard_vec: Vec<Arc<RwLock<RouteTableShard>>> =
            (0..NUM_SHARDS).map(|_| Arc::new(RwLock::new(RouteTableShard::new()))).collect();

        // 将Vec转换为数组（在编译时已知NUM_SHARDS的值）
        let shards: [Arc<RwLock<RouteTableShard>>; NUM_SHARDS] =
            shard_vec.try_into().unwrap_or_else(|_| panic!("Failed to convert Vec to array"));

        // 创建路由缓存
        let cache = Arc::new(RouteCache::new(
            CACHE_CAPACITY,
            Duration::from_secs(CACHE_TTL_SECS),
        ));

        Self {
            shards,
            count: Arc::new(AtomicUsize::new(0)),
            cache,
        }
    }

    /// 根据路径计算分片索引
    fn shard_index(path: &str) -> usize {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        path.hash(&mut hasher);
        (hasher.finish() as usize) % NUM_SHARDS
    }

    /// 向路由表中插入一个路由
    ///
    /// 如果路径已存在，将被新的路由覆盖。
    ///
    /// # 参数
    ///
    /// * `path` - 路由路径，例如 "/hello" 或 "/user/{id}"
    /// * `route` - 实现了 `RouteEntry` trait 的路由处理器
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::{RouteTable, SimpleRoute};
    ///
    /// let table = RouteTable::new();
    /// let route = SimpleRoute::new("Hello", "text/plain");
    /// table.insert("/hello".into(), Box::new(route));
    /// assert!(table.contains("/hello"));
    /// ```
    pub fn insert(&self, path: String, route: Box<dyn RouteEntry>) {
        let shard_idx = Self::shard_index(&path);
        let mut guard = self.shards[shard_idx].write().unwrap();
        let existed = guard.inner.contains(&path);
        guard.inner.insert(&path, route);
        if !existed {
            guard.count += 1;
            self.count.fetch_add(1, Ordering::SeqCst);
        }
        // 缓存失效：移除该路径的旧缓存（如果存在）
        self.cache.remove(&path);
    }

    /// 从路由表中移除指定路径的路由
    ///
    /// # 参数
    ///
    /// * `path` - 要移除的路由路径
    ///
    /// # 返回
    ///
    /// 如果路由存在并成功移除，返回 `true`；否则返回 `false`
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::{RouteTable, SimpleRoute};
    ///
    /// let table = RouteTable::new();
    /// let route = SimpleRoute::new("Hello", "text/plain");
    /// table.insert("/hello".into(), Box::new(route));
    /// assert!(table.remove("/hello"));
    /// assert!(!table.remove("/nonexistent"));
    /// ```
    pub fn remove(&self, path: &str) -> bool {
        let shard_idx = Self::shard_index(path);
        let mut guard = self.shards[shard_idx].write().unwrap();
        let removed = guard.inner.remove(path).is_some();
        if removed {
            guard.count -= 1;
            self.count.fetch_sub(1, Ordering::SeqCst);
        }
        // 缓存失效：从缓存中移除该路径
        self.cache.remove(path);
        removed
    }

    /// 获取指定路径的路由处理器
    ///
    /// # 参数
    ///
    /// * `path` - 要查询的路由路径
    /// * `f` - 一个闭包，接收路由处理器的引用并返回结果
    ///
    /// # 返回
    ///
    /// 如果路由存在，返回 `Some(T)`；否则返回 `None`
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::{RouteTable, SimpleRoute};
    ///
    /// let table = RouteTable::new();
    /// let route = SimpleRoute::new("Hello", "text/plain");
    /// table.insert("/hello".into(), Box::new(route));
    ///
    /// let result = table.get_with("/hello", |_route| {
    ///     "found"
    /// });
    /// assert_eq!(result, Some("found"));
    /// ```
    pub fn get_with<F, R>(&self, path: &str, f: F) -> Option<R>
    where
        F: FnOnce(&std::sync::Arc<dyn RouteEntry>) -> R,
    {
        // 首先尝试从缓存获取
        if let Some(cached_route) = self.cache.get(path) {
            return Some(f(&cached_route));
        }

        // 缓存未命中，从 Radix Tree 查找
        let shard_idx = Self::shard_index(path);
        let guard = self.shards[shard_idx].read().unwrap();
        guard.inner.find(path).map(|(route, _params)| {
            // 将结果写入缓存
            self.cache.insert(path, std::sync::Arc::clone(route));
            f(route)
        })
    }

    /// 获取指定路径的路由处理器的克隆
    ///
    /// # 参数
    ///
    /// * `path` - 要查询的路由路径
    ///
    /// # 返回
    ///
    /// 如果路由存在，返回 `Some(Box<dyn RouteEntry>)`；否则返回 `None`
    ///
    /// # 性能优化
    ///
    /// 由于RouteRadixTree现在使用Arc存储路由，此方法利用Arc的零成本克隆特性，
    /// 仅增加引用计数，不复制实际数据。
    ///
    /// **缓存优化**: 此方法使用 LRU 缓存来避免重复的 Radix Tree 遍历。
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::{RouteTable, SimpleRoute};
    ///
    /// let table = RouteTable::new();
    /// let route = SimpleRoute::new("Hello", "text/plain");
    /// table.insert("/hello".into(), Box::new(route));
    ///
    /// let route_clone = table.get_clone("/hello");
    /// assert!(route_clone.is_some());
    /// ```
    pub fn get_clone(&self, path: &str) -> Option<Box<dyn RouteEntry>> {
        // 首先尝试从缓存获取
        if let Some(cached_route) = self.cache.get(path) {
            return Some(cached_route.as_ref().clone_box());
        }

        // 缓存未命中，从 Radix Tree 查找
        let shard_idx = Self::shard_index(path);
        let guard = self.shards[shard_idx].read().unwrap();
        guard.inner.find(path).map(|(route, _params)| {
            // 将结果写入缓存
            self.cache.insert(path, std::sync::Arc::clone(route));
            route.clone_box()
        })
    }

    /// 获取指定路径的路由处理器的Arc引用（零拷贝）
    ///
    /// # 参数
    ///
    /// * `path` - 要查询的路由路径
    ///
    /// # 返回
    ///
    /// 如果路由存在，返回 `Some(Arc<dyn RouteEntry>)`；否则返回 `None`
    ///
    /// # 性能优化
    ///
    /// 这是性能最优的访问方式，直接返回Arc引用，零拷贝且零成本克隆。
    /// 适用于需要多次访问同一个路由或需要在不同线程间共享路由的场景。
    ///
    /// **缓存优化**: 此方法使用 LRU 缓存来避免重复的 Radix Tree 遍历。
    /// 对于频繁访问的路由，缓存命中率可以达到 90% 以上。
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::{RouteTable, SimpleRoute};
    /// use std::sync::Arc;
    ///
    /// let table = RouteTable::new();
    /// let route = SimpleRoute::new("Hello", "text/plain");
    /// table.insert("/hello".into(), Box::new(route));
    ///
    /// let route_arc = table.get_arc("/hello");
    /// assert!(route_arc.is_some());
    /// ```
    pub fn get_arc(&self, path: &str) -> Option<std::sync::Arc<dyn RouteEntry>> {
        // 首先尝试从缓存获取
        if let Some(cached_route) = self.cache.get(path) {
            return Some(cached_route);
        }

        // 缓存未命中，从 Radix Tree 查找
        let shard_idx = Self::shard_index(path);
        let guard = self.shards[shard_idx].read().unwrap();
        let result = guard.inner.find(path).map(|(route, _params)| {
            let route_arc = std::sync::Arc::clone(route);
            // 将结果写入缓存
            self.cache.insert(path, route_arc.clone());
            route_arc
        });

        result
    }

    /// 获取路由的数量
    ///
    /// # 返回
    ///
    /// 返回当前路由表中路由的数量
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::{RouteTable, SimpleRoute};
    ///
    /// let table = RouteTable::new();
    /// assert_eq!(table.count(), 0);
    ///
    /// table.insert("/route1".into(), Box::new(SimpleRoute::new("body", "text/plain")));
    /// assert_eq!(table.count(), 1);
    /// ```
    pub fn count(&self) -> usize {
        self.count.load(Ordering::SeqCst)
    }

    /// 检查路由表中是否包含指定路径
    ///
    /// # 参数
    ///
    /// * `path` - 要检查的路由路径
    ///
    /// # 返回
    ///
    /// 如果路由存在，返回 `true`；否则返回 `false`
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::{RouteTable, SimpleRoute};
    ///
    /// let table = RouteTable::new();
    /// table.insert("/hello".into(), Box::new(SimpleRoute::new("body", "text/plain")));
    /// assert!(table.contains("/hello"));
    /// assert!(!table.contains("/nonexistent"));
    /// ```
    pub fn contains(&self, path: &str) -> bool {
        let shard_idx = Self::shard_index(path);
        let guard = self.shards[shard_idx].read().unwrap();
        guard.inner.contains(path)
    }

    /// 获取所有路由的路径列表
    ///
    /// # 返回
    ///
    /// 返回一个包含所有路由路径的 `Vec<String>`
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::{RouteTable, SimpleRoute};
    ///
    /// let table = RouteTable::new();
    /// table.insert("/route1".into(), Box::new(SimpleRoute::new("body", "text/plain")));
    /// table.insert("/route2".into(), Box::new(SimpleRoute::new("body", "text/plain")));
    ///
    /// let paths = table.list_paths();
    /// assert_eq!(paths.len(), 2);
    /// ```
    pub fn list_paths(&self) -> Vec<String> {
        let mut paths = Vec::new();
        for shard in &self.shards {
            let guard = shard.read().unwrap();
            paths.extend(guard.inner.list_paths());
        }
        paths
    }

    /// 清空路由表
    ///
    /// 移除所有路由，使路由表变为空。
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::{RouteTable, SimpleRoute};
    ///
    /// let table = RouteTable::new();
    /// table.insert("/route1".into(), Box::new(SimpleRoute::new("body", "text/plain")));
    /// assert_eq!(table.count(), 1);
    ///
    /// table.clear();
    /// assert_eq!(table.count(), 0);
    /// ```
    pub fn clear(&self) {
        for shard in &self.shards {
            let mut guard = shard.write().unwrap();
            guard.inner.clear();
            guard.count = 0;
        }
        self.count.store(0, Ordering::SeqCst);
        // 清空缓存
        self.cache.clear();
    }

    /// 缓存预热
    ///
    /// 批量预加载指定路径到缓存中，减少首次访问延迟。
    /// 适用于启动时预加载高频路由或批量查询前的缓存预热。
    ///
    /// # 参数
    ///
    /// * `paths` - 要预热的路径列表
    ///
    /// # 性能优化
    ///
    /// 预热可以显著提升后续查询性能，特别是对于高频访问的路由。
    /// 建议在应用启动时调用此方法预热核心路由。
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::{RouteTable, SimpleRoute};
    ///
    /// let table = RouteTable::new();
    /// table.insert("/api/users".into(), Box::new(SimpleRoute::new("users", "application/json")));
    /// table.insert("/api/posts".into(), Box::new(SimpleRoute::new("posts", "application/json")));
    ///
    /// // 预热缓存
    /// table.warmup_cache(&["/api/users", "/api/posts"]);
    ///
    /// // 后续查询将直接从缓存获取
    /// assert!(table.get_arc("/api/users").is_some());
    /// ```
    pub fn warmup_cache(&self, paths: &[&str]) {
        for path in paths {
            // 尝试从路由表查找并缓存
            let shard_idx = Self::shard_index(path);
            let guard = self.shards[shard_idx].read().unwrap();
            if let Some((route, _params)) = guard.inner.find(path) {
                self.cache.insert(path, std::sync::Arc::clone(route));
            }
        }
    }

    /// 获取缓存统计信息
    ///
    /// # 返回
    ///
    /// 返回缓存的统计信息，包括命中率、未命中率等。
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::{RouteTable, SimpleRoute};
    ///
    /// let table = RouteTable::new();
    /// table.insert("/hello".into(), Box::new(SimpleRoute::new("Hello", "text/plain")));
    ///
    /// // 首次查询（缓存未命中）
    /// table.get_arc("/hello");
    /// // 再次查询（缓存命中）
    /// table.get_arc("/hello");
    ///
    /// let stats = table.cache_stats();
    /// assert_eq!(stats.hits, 1);
    /// assert_eq!(stats.misses, 1);
    /// ```
    pub fn cache_stats(&self) -> crate::core::cache::CacheStats {
        self.cache.stats()
    }

    /// 获取缓存命中率
    ///
    /// # 返回
    ///
    /// 返回缓存的命中率（0.0 到 1.0）。
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::{RouteTable, SimpleRoute};
    ///
    /// let table = RouteTable::new();
    /// table.insert("/hello".into(), Box::new(SimpleRoute::new("Hello", "text/plain")));
    ///
    /// table.get_arc("/hello");
    /// table.get_arc("/hello");
    ///
    /// let hit_rate = table.cache_hit_rate();
    /// assert_eq!(hit_rate, 0.5); // 1 hit / 2 total
    /// ```
    pub fn cache_hit_rate(&self) -> f64 {
        self.cache.hit_rate()
    }

    /// 重置缓存统计信息
    ///
    /// 清除缓存的命中、未命中等统计信息，重新开始计数。
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::{RouteTable, SimpleRoute};
    ///
    /// let table = RouteTable::new();
    /// table.insert("/hello".into(), Box::new(SimpleRoute::new("Hello", "text/plain")));
    ///
    /// table.get_arc("/hello");
    /// table.reset_cache_stats();
    ///
    /// let stats = table.cache_stats();
    /// assert_eq!(stats.hits, 0);
    /// assert_eq!(stats.misses, 0);
    /// ```
    pub fn reset_cache_stats(&self) {
        self.cache.reset_stats();
    }

    /// 清理过期的缓存条目
    ///
    /// 定期调用此方法可以清理过期的缓存条目，释放内存。
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::{RouteTable, SimpleRoute};
    ///
    /// let table = RouteTable::new();
    /// table.insert("/hello".into(), Box::new(SimpleRoute::new("Hello", "text/plain")));
    ///
    /// table.get_arc("/hello");
    ///
    /// // 清理过期缓存
    /// table.cleanup_cache();
    /// ```
    pub fn cleanup_cache(&self) {
        self.cache.cleanup_expired();
    }

    /// 批量插入路由
    ///
    /// 一次性插入多个路由，减少锁竞争。
    ///
    /// # 参数
    ///
    /// * `routes` - 要插入的路由集合
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::{RouteTable, SimpleRoute, RouteEntry};
    /// use std::collections::HashMap;
    ///
    /// let table = RouteTable::new();
    /// let mut routes: HashMap<String, Box<dyn RouteEntry>> = HashMap::new();
    /// routes.insert("/route1".to_string(), Box::new(SimpleRoute::new("body1", "text/plain")));
    /// routes.insert("/route2".to_string(), Box::new(SimpleRoute::new("body2", "text/plain")));
    ///
    /// table.batch_insert(routes);
    /// assert_eq!(table.count(), 2);
    /// ```
    pub fn batch_insert(&self, routes: std::collections::HashMap<String, Box<dyn RouteEntry>>) {
        // 将路由按分片分组
        let mut shard_routes: Vec<Vec<(String, Box<dyn RouteEntry>)>> =
            (0..NUM_SHARDS).map(|_| Vec::new()).collect();

        for (path, route) in routes {
            let shard_idx = Self::shard_index(&path);
            shard_routes[shard_idx].push((path, route));
        }

        // 批量插入到各个分片
        let mut total_new_count = 0;
        for (shard_idx, routes) in shard_routes.into_iter().enumerate() {
            let mut guard = self.shards[shard_idx].write().unwrap();
            let mut new_count = 0;

            for (path, route) in routes {
                let existed = guard.inner.contains(&path);
                guard.inner.insert(&path, route);
                if !existed {
                    new_count += 1;
                }
            }

            guard.count += new_count;
            total_new_count += new_count;
        }

        self.count.fetch_add(total_new_count, Ordering::SeqCst);
    }
}

impl Default for RouteTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::SimpleRoute;
    use super::*;

    #[test]
    fn test_route_table_creation() {
        let table = RouteTable::new();
        assert_eq!(table.count(), 0);
        assert!(table.list_paths().is_empty());
    }

    #[test]
    fn test_route_table_insert_and_get() {
        let table = RouteTable::new();
        let route = SimpleRoute::new("hello", "text/plain");
        table.insert("/test".into(), Box::new(route));
        assert!(table.contains("/test"));

        // 测试 get_with 方法
        let result = table.get_with("/test", |_route| "found");
        assert_eq!(result, Some("found"));

        // 测试不存在的路由
        let result = table.get_with("/nonexistent", |_route| "found");
        assert_eq!(result, None);
    }

    #[test]
    fn test_route_table_remove() {
        let table = RouteTable::new();
        let route = SimpleRoute::new("hello", "text/plain");
        table.insert("/test".into(), Box::new(route));
        assert!(table.remove("/test"));
        assert!(!table.remove("/nonexistent"));
        assert_eq!(table.count(), 0);
    }

    #[test]
    fn test_route_table_count() {
        let table = RouteTable::new();
        assert_eq!(table.count(), 0);

        table.insert(
            "/route1".into(),
            Box::new(SimpleRoute::new("body1", "text/plain")),
        );
        assert_eq!(table.count(), 1);

        table.insert(
            "/route2".into(),
            Box::new(SimpleRoute::new("body2", "text/plain")),
        );
        assert_eq!(table.count(), 2);

        table.remove("/route1");
        assert_eq!(table.count(), 1);
    }

    #[test]
    fn test_route_table_contains() {
        let table = RouteTable::new();
        assert!(!table.contains("/any"));

        table.insert(
            "/hello".into(),
            Box::new(SimpleRoute::new("body", "text/plain")),
        );
        assert!(table.contains("/hello"));
        assert!(!table.contains("/world"));
    }

    #[test]
    fn test_route_table_list_paths() {
        let table = RouteTable::new();
        table.insert(
            "/route1".into(),
            Box::new(SimpleRoute::new("body", "text/plain")),
        );
        table.insert(
            "/route2".into(),
            Box::new(SimpleRoute::new("body", "text/plain")),
        );
        table.insert(
            "/route3".into(),
            Box::new(SimpleRoute::new("body", "text/plain")),
        );

        let paths = table.list_paths();
        assert_eq!(paths.len(), 3);
        assert!(paths.contains(&"/route1".to_string()));
        assert!(paths.contains(&"/route2".to_string()));
        assert!(paths.contains(&"/route3".to_string()));
    }

    #[test]
    fn test_route_table_clear() {
        let table = RouteTable::new();
        table.insert(
            "/route1".into(),
            Box::new(SimpleRoute::new("body", "text/plain")),
        );
        table.insert(
            "/route2".into(),
            Box::new(SimpleRoute::new("body", "text/plain")),
        );

        assert_eq!(table.count(), 2);
        table.clear();
        assert_eq!(table.count(), 0);
        assert!(table.list_paths().is_empty());
    }

    #[test]
    fn test_route_table_overwrite() {
        let table = RouteTable::new();
        let route1 = SimpleRoute::new("body1", "text/plain");
        let route2 = SimpleRoute::new("body2", "text/plain");

        table.insert("/test".into(), Box::new(route1));
        table.insert("/test".into(), Box::new(route2));

        // 覆盖后数量应该还是 1
        assert_eq!(table.count(), 1);
        assert!(table.contains("/test"));
    }

    #[test]
    fn test_concurrent_access() {
        use std::sync::Arc;
        let table = Arc::new(RouteTable::new());
        let mut handles = vec![];

        for i in 0..10 {
            let table_clone = Arc::clone(&table);
            let handle = std::thread::spawn(move || {
                let route = SimpleRoute::new(format!("body-{}", i), "text/plain");
                table_clone.insert(format!("/path-{}", i), Box::new(route));
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert!(table.contains("/path-0"));
        assert!(table.contains("/path-9"));
        assert_eq!(table.count(), 10);
    }

    #[test]
    fn test_special_characters_in_path() {
        let table = RouteTable::new();
        let special_paths = vec![
            "/path/with spaces",
            "/path/with-unicode/测试",
            "/path/with/slashes//multiple",
        ];

        for path in special_paths {
            table.insert(
                path.to_string(),
                Box::new(SimpleRoute::new("body", "text/plain")),
            );
            assert!(table.contains(path));
        }
    }

    #[test]
    fn test_empty_route_table() {
        let table = RouteTable::new();
        assert!(!table.contains("/any"));
        assert!(!table.remove("/any"));
        assert_eq!(table.count(), 0);
        assert!(table.list_paths().is_empty());
    }

    #[test]
    fn test_route_table_default() {
        let table = RouteTable::default();
        assert_eq!(table.count(), 0);
    }

    #[test]
    fn test_route_table_clone() {
        let table = RouteTable::new();
        table.insert(
            "/test".into(),
            Box::new(SimpleRoute::new("body", "text/plain")),
        );

        let cloned = table.clone();
        assert_eq!(cloned.count(), 1);
        assert!(cloned.contains("/test"));

        // 验证克隆是共享的
        cloned.insert(
            "/test2".into(),
            Box::new(SimpleRoute::new("body2", "text/plain")),
        );
        assert_eq!(table.count(), 2);
        assert!(table.contains("/test2"));
    }

    #[test]
    fn test_cache_hit_and_miss() {
        let table = RouteTable::new();
        table.insert(
            "/cached".into(),
            Box::new(SimpleRoute::new("cached", "text/plain")),
        );

        // 首次查询（缓存未命中）
        table.get_arc("/cached");
        let stats = table.cache_stats();
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hits, 0);

        // 再次查询（缓存命中）
        table.get_arc("/cached");
        let stats = table.cache_stats();
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hits, 1);
    }

    #[test]
    fn test_cache_hit_rate() {
        let table = RouteTable::new();
        table.insert(
            "/route".into(),
            Box::new(SimpleRoute::new("route", "text/plain")),
        );

        // 首次查询（缓存未命中）
        table.get_arc("/route");
        assert_eq!(table.cache_hit_rate(), 0.0);

        // 再次查询（缓存命中）
        table.get_arc("/route");
        assert_eq!(table.cache_hit_rate(), 0.5); // 1 hit / 2 total

        // 第三次查询（缓存命中）
        table.get_arc("/route");
        assert_eq!(table.cache_hit_rate(), 0.6666666666666666); // 2 hits / 3 total
    }

    #[test]
    fn test_cache_warmup() {
        let table = RouteTable::new();
        table.insert(
            "/api/users".into(),
            Box::new(SimpleRoute::new("users", "application/json")),
        );
        table.insert(
            "/api/posts".into(),
            Box::new(SimpleRoute::new("posts", "application/json")),
        );

        // 预热缓存
        table.warmup_cache(&["/api/users", "/api/posts"]);

        // 后续查询应该从缓存获取
        table.get_arc("/api/users");
        table.get_arc("/api/posts");

        let stats = table.cache_stats();
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 0);
    }

    #[test]
    fn test_cache_invalidation_on_insert() {
        let table = RouteTable::new();
        table.insert(
            "/route".into(),
            Box::new(SimpleRoute::new("old", "text/plain")),
        );

        // 首次查询并缓存
        table.get_arc("/route");
        let stats = table.cache_stats();
        assert_eq!(stats.misses, 1);

        // 更新路由
        table.insert(
            "/route".into(),
            Box::new(SimpleRoute::new("new", "text/plain")),
        );

        // 缓存应该失效，重新查询
        let route = table.get_arc("/route");
        assert!(route.is_some());
        let stats = table.cache_stats();
        // 应该有一次缓存未命中
        assert_eq!(stats.misses, 2);
    }

    #[test]
    fn test_cache_invalidation_on_remove() {
        let table = RouteTable::new();
        table.insert(
            "/route".into(),
            Box::new(SimpleRoute::new("route", "text/plain")),
        );

        // 首次查询并缓存
        table.get_arc("/route");
        assert!(table.get_arc("/route").is_some());

        // 移除路由
        table.remove("/route");

        // 缓存应该失效，查询返回 None
        assert!(table.get_arc("/route").is_none());
    }

    #[test]
    fn test_cache_clear() {
        let table = RouteTable::new();
        table.insert(
            "/route1".into(),
            Box::new(SimpleRoute::new("route1", "text/plain")),
        );
        table.insert(
            "/route2".into(),
            Box::new(SimpleRoute::new("route2", "text/plain")),
        );

        // 查询以填充缓存
        table.get_arc("/route1");
        table.get_arc("/route2");

        assert_eq!(table.cache_stats().hits, 0);
        assert_eq!(table.cache_stats().misses, 2);

        // 清空路由表
        table.clear();

        // 缓存也应该被清空
        assert_eq!(table.cache_stats().hits, 0);
        assert_eq!(table.cache_stats().misses, 2);
        assert!(table.get_arc("/route1").is_none());
        assert!(table.get_arc("/route2").is_none());
    }

    #[test]
    fn test_cache_performance() {
        let table = RouteTable::new();
        let num_routes = 100;

        // 插入路由
        for i in 0..num_routes {
            table.insert(
                format!("/route-{}", i),
                Box::new(SimpleRoute::new(format!("body-{}", i), "text/plain")),
            );
        }

        // 预热缓存
        let routes: Vec<String> = (0..num_routes).map(|i| format!("/route-{}", i)).collect();
        table.warmup_cache(&routes.iter().map(|s| s.as_str()).collect::<Vec<_>>());

        // 多次查询（应该全部命中缓存）
        for _ in 0..10 {
            for i in 0..num_routes {
                table.get_arc(&format!("/route-{}", i));
            }
        }

        let stats = table.cache_stats();
        // 所有查询应该命中缓存
        assert_eq!(stats.hits, num_routes * 10);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.hit_rate(), 1.0);
    }
}
