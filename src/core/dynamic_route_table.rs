//! 动态路由表
//!
//! 集成动态分片和负载均衡功能的高性能路由表

use super::dynamic_sharding::{
    DynamicShardManager, DynamicShardingConfig, LoadBalanceStrategy,
};
use super::route_entry::RouteEntry;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 动态路由表配置
#[derive(Debug, Clone)]
pub struct DynamicRouteTableConfig {
    /// 动态分片配置
    pub sharding: DynamicShardingConfig,
    /// 是否使用哈希分配路由
    pub use_hash_distribution: bool,
}

impl Default for DynamicRouteTableConfig {
    fn default() -> Self {
        Self {
            sharding: DynamicShardingConfig::default(),
            use_hash_distribution: true,
        }
    }
}

/// 动态路由表
///
/// 使用动态分片和负载均衡来管理路由，提供更好的并发性能和负载分布。
///
/// # 特性
///
/// - 动态分片：根据负载自动调整分片数量
/// - 负载均衡：支持多种负载均衡策略
/// - 自动重平衡：定期检查并重新分配路由
/// - 性能监控：跟踪每个分片的访问指标
///
/// # 示例
///
/// ```
/// use dynamic_route_actix::{DynamicRouteTable, DynamicRouteTableConfig, SimpleRoute};
///
/// let config = DynamicRouteTableConfig::default();
/// let table = DynamicRouteTable::new(config);
///
/// let route = SimpleRoute::new("Hello", "text/plain");
/// table.insert("/hello".into(), Box::new(route));
/// ```
#[derive(Clone)]
pub struct DynamicRouteTable {
    manager: Arc<RwLock<DynamicShardManager>>,
    config: DynamicRouteTableConfig,
}

impl DynamicRouteTable {
    /// 创建新的动态路由表
    pub fn new(config: DynamicRouteTableConfig) -> Self {
        Self {
            manager: Arc::new(RwLock::new(DynamicShardManager::new(config.sharding.clone()))),
            config,
        }
    }

    /// 使用默认配置创建动态路由表
    pub fn default_config() -> Self {
        Self::new(DynamicRouteTableConfig::default())
    }

    /// 向路由表中插入一个路由
    ///
    /// # 参数
    ///
    /// * `path` - 路由路径
    /// * `route` - 路由处理器
    pub fn insert(&self, path: String, route: Box<dyn RouteEntry>) {
        let manager = self.manager.read().unwrap();
        let shard_idx = if self.config.use_hash_distribution {
            manager.hash_shard_index(&path)
        } else {
            manager.select_shard(&path)
        };

        if let Some(shard) = manager.get_shard(shard_idx) {
            let mut guard = shard.write().unwrap();
            let existed = guard.contains(&path);
            guard.insert(&path, route);
            if !existed {
                manager.increment_total_routes();
            }
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
    pub fn remove(&self, path: &str) -> bool {
        let manager = self.manager.read().unwrap();
        let shard_idx = manager.hash_shard_index(path);

        if let Some(shard) = manager.get_shard(shard_idx) {
            let mut guard = shard.write().unwrap();
            let removed = guard.remove(path).is_some();
            if removed {
                manager.decrement_total_routes();
            }
            removed
        } else {
            false
        }
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
    pub fn get_with<F, R>(&self, path: &str, f: F) -> Option<R>
    where
        F: FnOnce(&Box<dyn RouteEntry>) -> R,
    {
        let manager = self.manager.read().unwrap();
        let shard_idx = manager.hash_shard_index(path);

        if let Some(shard) = manager.get_shard(shard_idx) {
            let mut guard = shard.write().unwrap();
            guard.find(path).map(|(route, _params)| f(route))
        } else {
            None
        }
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
    pub fn get_clone(&self, path: &str) -> Option<Box<dyn RouteEntry>> {
        let manager = self.manager.read().unwrap();
        let shard_idx = manager.hash_shard_index(path);

        if let Some(shard) = manager.get_shard(shard_idx) {
            let mut guard = shard.write().unwrap();
            guard.find(path).map(|(route, _params)| route.clone_box())
        } else {
            None
        }
    }

    /// 获取路由的数量
    pub fn count(&self) -> usize {
        let manager = self.manager.read().unwrap();
        manager.total_routes()
    }

    /// 检查路由表中是否包含指定路径
    pub fn contains(&self, path: &str) -> bool {
        let manager = self.manager.read().unwrap();
        let shard_idx = manager.hash_shard_index(path);

        if let Some(shard) = manager.get_shard(shard_idx) {
            let guard = shard.read().unwrap();
            guard.contains(path)
        } else {
            false
        }
    }

    /// 获取所有路由的路径列表
    pub fn list_paths(&self) -> Vec<String> {
        let manager = self.manager.read().unwrap();
        let mut paths = Vec::new();

        for i in 0..manager.shard_count() {
            if let Some(shard) = manager.get_shard(i) {
                let guard = shard.read().unwrap();
                paths.extend(guard.list_paths());
            }
        }

        paths
    }

    /// 清空路由表
    pub fn clear(&self) {
        let manager = self.manager.read().unwrap();

        for i in 0..manager.shard_count() {
            if let Some(shard) = manager.get_shard(i) {
                let mut guard = shard.write().unwrap();
                guard.clear();
            }
        }
        manager.reset_total_routes();
    }

    /// 批量插入路由
    pub fn batch_insert(&self, routes: HashMap<String, Box<dyn RouteEntry>>) {
        let manager = self.manager.read().unwrap();

        for (path, route) in routes {
            let shard_idx = if self.config.use_hash_distribution {
                manager.hash_shard_index(&path)
            } else {
                manager.select_shard(&path)
            };

            if let Some(shard) = manager.get_shard(shard_idx) {
                let mut guard = shard.write().unwrap();
                let existed = guard.contains(&path);
                guard.insert(&path, route);
                if !existed {
                    manager.increment_total_routes();
                }
            }
        }
    }

    /// 执行负载重平衡
    ///
    /// # 返回
    ///
    /// 返回移动的路由数量
    pub fn rebalance(&self) -> Result<usize, String> {
        let mut manager = self.manager.write().unwrap();
        manager.rebalance()
    }

    /// 获取负载不均衡程度
    ///
    /// # 返回
    ///
    /// 返回 0.0 到 1.0 之间的值，值越大表示负载越不均衡
    pub fn get_imbalance(&self) -> f64 {
        let manager = self.manager.read().unwrap();
        manager.calculate_imbalance()
    }

    /// 获取所有分片的指标
    pub fn get_shard_metrics(&self) -> Vec<super::dynamic_sharding::ShardMetrics> {
        let manager = self.manager.read().unwrap();
        manager.get_all_metrics()
    }

    /// 获取分片数量
    pub fn shard_count(&self) -> usize {
        let manager = self.manager.read().unwrap();
        manager.shard_count()
    }

    /// 调整分片数量
    ///
    /// # 参数
    ///
    /// * `increase` - `true` 增加分片，`false` 减少分片
    pub fn adjust_shard_count(&self, increase: bool) -> Result<(), String> {
        let mut manager = self.manager.write().unwrap();
        manager.adjust_shard_count(increase)
    }

    /// 设置负载均衡策略
    pub fn set_load_balance_strategy(&self, strategy: LoadBalanceStrategy) {
        let mut manager = self.manager.write().unwrap();
        manager.set_strategy(strategy);
    }

    /// 获取当前负载均衡策略
    pub fn get_load_balance_strategy(&self) -> LoadBalanceStrategy {
        let manager = self.manager.read().unwrap();
        manager.config().strategy
    }
}

impl Default for DynamicRouteTable {
    fn default() -> Self {
        Self::default_config()
    }
}

#[cfg(test)]
mod tests {
    use super::super::SimpleRoute;
    use super::*;

    #[test]
    fn test_dynamic_route_table_creation() {
        let table = DynamicRouteTable::default_config();
        assert_eq!(table.count(), 0);
        assert!(table.list_paths().is_empty());
    }

    #[test]
    fn test_dynamic_route_table_insert_and_get() {
        let table = DynamicRouteTable::default_config();
        let route = SimpleRoute::new("hello", "text/plain");
        table.insert("/test".into(), Box::new(route));
        assert!(table.contains("/test"));

        let result = table.get_with("/test", |_route| "found");
        assert_eq!(result, Some("found"));

        let result = table.get_with("/nonexistent", |_route| "found");
        assert_eq!(result, None);
    }

    #[test]
    fn test_dynamic_route_table_remove() {
        let table = DynamicRouteTable::default_config();
        let route = SimpleRoute::new("hello", "text/plain");
        table.insert("/test".into(), Box::new(route));
        assert!(table.remove("/test"));
        assert!(!table.remove("/nonexistent"));
        assert_eq!(table.count(), 0);
    }

    #[test]
    fn test_dynamic_route_table_count() {
        let table = DynamicRouteTable::default_config();
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
    fn test_dynamic_route_table_contains() {
        let table = DynamicRouteTable::default_config();
        assert!(!table.contains("/any"));

        table.insert(
            "/hello".into(),
            Box::new(SimpleRoute::new("body", "text/plain")),
        );
        assert!(table.contains("/hello"));
        assert!(!table.contains("/world"));
    }

    #[test]
    fn test_dynamic_route_table_list_paths() {
        let table = DynamicRouteTable::default_config();
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
    }

    #[test]
    fn test_dynamic_route_table_clear() {
        let table = DynamicRouteTable::default_config();
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
    fn test_dynamic_route_table_batch_insert() {
        let table = DynamicRouteTable::default_config();
        let mut routes: HashMap<String, Box<dyn RouteEntry>> = HashMap::new();

        for i in 0..100 {
            routes.insert(
                format!("/route-{}", i),
                Box::new(SimpleRoute::new(format!("body-{}", i), "text/plain")),
            );
        }

        table.batch_insert(routes);
        assert_eq!(table.count(), 100);
    }

    #[test]
    fn test_rebalance() {
        let config = DynamicRouteTableConfig {
            sharding: DynamicShardingConfig {
                imbalance_threshold: 0.1,
                ..Default::default()
            },
            ..Default::default()
        };
        let table = DynamicRouteTable::new(config);

        // 插入大量路由到同一个分片（通过哈希）
        for i in 0..50 {
            table.insert(
                format!("/test{}", i),
                Box::new(SimpleRoute::new("body", "text/plain")),
            );
        }

        let imbalance_before = table.get_imbalance();
        let moved = table.rebalance().unwrap();

        // 要么移动了路由，要么已经均衡
        if moved > 0 {
            let imbalance_after = table.get_imbalance();
            assert!(imbalance_after <= imbalance_before || imbalance_after < 0.5);
        }
    }

    #[test]
    fn test_shard_count() {
        let table = DynamicRouteTable::default_config();
        assert_eq!(table.shard_count(), 8); // default initial_shards

        table.adjust_shard_count(true).unwrap();
        assert_eq!(table.shard_count(), 9);
    }

    #[test]
    fn test_get_shard_metrics() {
        let table = DynamicRouteTable::default_config();
        let route = SimpleRoute::new("test", "text/plain");

        table.insert("/test1".into(), Box::new(route.clone()));
        table.insert("/test2".into(), Box::new(route.clone()));

        let metrics = table.get_shard_metrics();
        assert_eq!(metrics.len(), table.shard_count());

        // 检查总路由数
        let total_routes: usize = metrics.iter().map(|m| m.route_count).sum();
        assert_eq!(total_routes, 2);
    }

    #[test]
    fn test_concurrent_access() {
        use std::sync::Arc;
        let table = Arc::new(DynamicRouteTable::default_config());
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
}