use super::RouteEntry;
use super::route_radix_tree::RouteRadixTree;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

/// 分片数
const NUM_SHARDS: usize = 16;

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

        Self {
            shards,
            count: Arc::new(AtomicUsize::new(0)),
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
        F: FnOnce(&Box<dyn RouteEntry>) -> R,
    {
        let shard_idx = Self::shard_index(path);
        let guard = self.shards[shard_idx].read().unwrap();
        guard.inner.find(path).map(|(route, _params)| f(route))
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
        let shard_idx = Self::shard_index(path);
        let guard = self.shards[shard_idx].read().unwrap();
        // 使用 clone_box 但由于 SimpleRoute 现在使用 Arc，开销已显著降低
        guard.inner.find(path).map(|(route, _params)| route.clone_box())
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
}
