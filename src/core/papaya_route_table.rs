//! 基于 Papaya 的无锁路由表
//!
//! 使用 papaya 并发哈希表实现高性能无锁路由表。
//! Papaya 针对读密集型工作负载进行了优化，提供极低的读延迟。
//!
//! # 特性
//!
//! - 无锁读取：多个线程可以同时读取，无需加锁
//! - 高性能：专门优化读操作，适合路由查找场景
//! - 线程安全：所有操作都是线程安全的
//! - 内存安全：使用 Rust 的类型系统保证内存安全
//!
//! # 性能特点
//!
//! - 读操作：极低延迟，不会被写操作阻塞
//! - 写操作：中等性能，但不会阻塞读操作
//! - 内存效率：每个条目单独分配，适合大量路由
//!
//! # 使用示例
//!
//! ```
//! use dynamic_route_actix::core::{PapayaRouteTable, SimpleRoute};
//!
//! #[tokio::main]
//! async fn main() {
//!     let table = PapayaRouteTable::new();
//!
//!     // 插入路由
//!     let route = SimpleRoute::new("Hello, World!", "text/plain");
//!     table.insert("/hello".to_string(), Box::new(route)).await;
//!
//!     // 查找路由
//!     if let Some(found) = table.get("/hello").await {
//!         println!("Found route: {:?}", found);
//!     }
//!
//!     // 删除路由
//!     table.remove("/hello").await;
//! }
//! ```

use crate::core::route_entry::RouteEntry;
use papaya::HashMap;
use seize::Collector;

/// 基于 Papaya 的无锁路由表
///
/// 使用 papaya 并发哈希表实现，提供高性能的无锁路由查找功能。
/// 专门针对读密集型场景优化，适合动态路由系统。
///
/// # 线程安全
///
/// 所有方法都是线程安全的，可以安全地在多线程环境中使用。
///
/// # 性能
///
/// - 读操作：O(1) 平均时间复杂度，极低延迟
/// - 写操作：O(1) 平均时间复杂度，中等延迟
/// - 内存：每个路由条目单独分配
#[derive(Debug)]
pub struct PapayaRouteTable {
    /// 路由存储：使用 papaya HashMap
    routes: HashMap<String, Box<dyn RouteEntry>>,
}

impl Default for PapayaRouteTable {
    fn default() -> Self {
        Self::new()
    }
}

impl PapayaRouteTable {
    /// 创建新的 Papaya 路由表
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::core::PapayaRouteTable;
    ///
    /// let table = PapayaRouteTable::new();
    /// ```
    pub fn new() -> Self {
        let collector = Collector::new();
        let routes = HashMap::builder()
            .collector(collector)
            .build();

        Self { routes }
    }

    /// 插入路由
    ///
    /// # 参数
    ///
    /// * `path` - 路由路径
    /// * `route` - 路由处理器
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::core::{PapayaRouteTable, SimpleRoute};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let table = PapayaRouteTable::new();
    ///     let route = SimpleRoute::new("Hello", "text/plain");
    ///     table.insert("/hello".to_string(), Box::new(route)).await;
    /// }
    /// ```
    pub async fn insert(&self, path: String, route: Box<dyn RouteEntry>) {
        self.routes.pin().insert(path, route);
    }

    /// 查找路由
    ///
    /// # 参数
    ///
    /// * `path` - 路由路径
    ///
    /// # 返回
    ///
    /// 如果找到路由，返回路由数据的克隆，否则返回 `None`
    ///
    /// # 注意
    ///
    /// 由于 papaya 的生命周期限制，这里返回克隆的数据而不是引用
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::core::{PapayaRouteTable, SimpleRoute};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let table = PapayaRouteTable::new();
    ///     let route = SimpleRoute::new("Hello", "text/plain");
    ///     table.insert("/hello".to_string(), Box::new(route)).await;
    ///
    ///     if let Some(found) = table.get("/hello").await {
    ///         println!("Found route");
    ///     }
    /// }
    /// ```
    pub async fn get(&self, path: &str) -> Option<Box<dyn RouteEntry>> {
        self.routes.pin().get(path).map(|route| route.clone_box())
    }

    /// 删除路由
    ///
    /// # 参数
    ///
    /// * `path` - 路由路径
    ///
    /// # 返回
    ///
    /// 如果找到并删除了路由，返回路由数据，否则返回 `None`
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::core::{PapayaRouteTable, SimpleRoute};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let table = PapayaRouteTable::new();
    ///     let route = SimpleRoute::new("Hello", "text/plain");
    ///     table.insert("/hello".to_string(), Box::new(route)).await;
    ///
    ///     let removed = table.remove("/hello").await;
    ///     assert!(removed.is_some());
    /// }
    /// ```
    pub async fn remove(&self, path: &str) -> Option<Box<dyn RouteEntry>> {
        self.routes.pin().remove(path).map(|route| route.clone_box())
    }

    /// 检查路由是否存在
    ///
    /// # 参数
    ///
    /// * `path` - 路由路径
    ///
    /// # 返回
    ///
    /// 如果路由存在返回 `true`，否则返回 `false`
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::core::{PapayaRouteTable, SimpleRoute};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let table = PapayaRouteTable::new();
    ///     let route = SimpleRoute::new("Hello", "text/plain");
    ///     table.insert("/hello".to_string(), Box::new(route)).await;
    ///
    ///     assert!(table.contains("/hello").await);
    /// }
    /// ```
    pub async fn contains(&self, path: &str) -> bool {
        self.routes.pin().get(path).is_some()
    }

    /// 获取路由数量
    ///
    /// # 返回
    ///
    /// 返回当前路由表中的路由数量
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::core::{PapayaRouteTable, SimpleRoute};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let table = PapayaRouteTable::new();
    ///     assert_eq!(table.len().await, 0);
    ///
    ///     let route = SimpleRoute::new("Hello", "text/plain");
    ///     table.insert("/hello".to_string(), Box::new(route)).await;
    ///     assert_eq!(table.len().await, 1);
    /// }
    /// ```
    pub async fn len(&self) -> usize {
        self.routes.pin().len()
    }

    /// 检查路由表是否为空
    ///
    /// # 返回
    ///
    /// 如果路由表为空返回 `true`，否则返回 `false`
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::core::PapayaRouteTable;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let table = PapayaRouteTable::new();
    ///     assert!(table.is_empty().await);
    /// }
    /// ```
    pub async fn is_empty(&self) -> bool {
        self.routes.pin().len() == 0
    }

    /// 清空所有路由
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::core::{PapayaRouteTable, SimpleRoute};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let table = PapayaRouteTable::new();
    ///     let route = SimpleRoute::new("Hello", "text/plain");
    ///     table.insert("/hello".to_string(), Box::new(route)).await;
    ///
    ///     table.clear().await;
    ///     assert!(table.is_empty().await);
    /// }
    /// ```
    pub async fn clear(&self) {
        self.routes.pin().clear();
    }

    /// 批量插入路由
    ///
    /// # 参数
    ///
    /// * `routes` - 要插入的路由集合
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::core::{PapayaRouteTable, SimpleRoute};
    /// use std::collections::HashMap;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let table = PapayaRouteTable::new();
    ///     let mut routes = HashMap::new();
    ///
    ///     routes.insert("/hello".to_string(), Box::new(SimpleRoute::new("Hello", "text/plain")));
    ///     routes.insert("/goodbye".to_string(), Box::new(SimpleRoute::new("Goodbye", "text/plain")));
    ///
    ///     table.insert_batch(routes).await;
    ///     assert_eq!(table.len().await, 2);
    /// }
    /// ```
    pub async fn insert_batch(&self, routes: std::collections::HashMap<String, Box<dyn RouteEntry>>) {
        let guard = self.routes.pin();
        for (path, route) in routes {
            guard.insert(path, route);
        }
    }

    /// 获取所有路由路径
    ///
    /// # 返回
    ///
    /// 返回所有路由路径的向量
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::core::{PapayaRouteTable, SimpleRoute};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let table = PapayaRouteTable::new();
    ///     let route = SimpleRoute::new("Hello", "text/plain");
    ///     table.insert("/hello".to_string(), Box::new(route)).await;
    ///
    ///     let paths = table.get_all_paths().await;
    ///     assert_eq!(paths, vec!["/hello"]);
    /// }
    /// ```
    pub async fn get_all_paths(&self) -> Vec<String> {
        self.routes.pin().keys().map(|k| k.clone()).collect()
    }

    /// 原子更新或插入路由
    ///
    /// # 参数
    ///
    /// * `path` - 路由路径
    /// * `updater` - 更新函数，接收当前值（如果有），返回操作结果
    ///
    /// # 注意
    ///
    /// 由于 papaya 的生命周期限制，此方法不返回 Compute 结果。
    /// 如果需要获取操作结果，请直接使用 papaya 的 API。
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::core::{PapayaRouteTable, SimpleRoute};
    /// use papaya::Operation;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let table = PapayaRouteTable::new();
    ///
    ///     let updater = |current: Option<(&String, &Box<dyn RouteEntry>)>| {
    ///         if current.is_some() {
    ///             Operation::Abort(())
    ///         } else {
    ///             let route = SimpleRoute::new("Hello", "text/plain");
    ///             Operation::Insert(Box::new(route))
    ///         }
    ///     };
    ///
    ///     table.update_or_insert("/hello".to_string(), updater).await;
    ///     assert!(table.contains("/hello").await);
    /// }
    /// ```
    pub async fn update_or_insert<F, T>(&self, path: String, updater: F)
    where
        F: FnMut(Option<(&String, &Box<dyn RouteEntry>)>) -> papaya::Operation<Box<dyn RouteEntry>, T>,
        T: Send + 'static,
    {
        self.routes.pin().compute(path, updater);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::SimpleRoute;

    #[tokio::test]
    async fn test_papaya_route_table_insert_and_get() {
        let table = PapayaRouteTable::new();

        // 插入路由
        let route = SimpleRoute::new("Hello, World!", "text/plain");
        table.insert("/hello".to_string(), Box::new(route)).await;

        // 查找路由
        let found = table.get("/hello").await;
        assert!(found.is_some());
    }

    #[tokio::test]
    async fn test_papaya_route_table_remove() {
        let table = PapayaRouteTable::new();

        // 插入路由
        let route = SimpleRoute::new("Hello", "text/plain");
        table.insert("/hello".to_string(), Box::new(route)).await;

        // 删除路由
        let removed = table.remove("/hello").await;
        assert!(removed.is_some());

        // 验证已删除
        assert!(!table.contains("/hello").await);
    }

    #[tokio::test]
    async fn test_papaya_route_table_contains() {
        let table = PapayaRouteTable::new();

        assert!(!table.contains("/hello").await);

        let route = SimpleRoute::new("Hello", "text/plain");
        table.insert("/hello".to_string(), Box::new(route)).await;

        assert!(table.contains("/hello").await);
    }

    #[tokio::test]
    async fn test_papaya_route_table_len() {
        let table = PapayaRouteTable::new();

        assert_eq!(table.len().await, 0);

        let route1 = SimpleRoute::new("Hello", "text/plain");
        table.insert("/hello".to_string(), Box::new(route1)).await;

        assert_eq!(table.len().await, 1);

        let route2 = SimpleRoute::new("Hello", "text/plain");
        table.insert("/goodbye".to_string(), Box::new(route2)).await;

        assert_eq!(table.len().await, 2);
    }

    #[tokio::test]
    async fn test_papaya_route_table_is_empty() {
        let table = PapayaRouteTable::new();

        assert!(table.is_empty().await);

        let route = SimpleRoute::new("Hello", "text/plain");
        table.insert("/hello".to_string(), Box::new(route)).await;

        assert!(!table.is_empty().await);
    }

    #[tokio::test]
    async fn test_papaya_route_table_clear() {
        let table = PapayaRouteTable::new();

        let route1 = SimpleRoute::new("Hello", "text/plain");
        let route2 = SimpleRoute::new("Goodbye", "text/plain");
        table.insert("/hello".to_string(), Box::new(route1)).await;
        table.insert("/goodbye".to_string(), Box::new(route2)).await;

        assert_eq!(table.len().await, 2);

        table.clear().await;

        assert!(table.is_empty().await);
    }

    #[tokio::test]
    async fn test_papaya_route_table_insert_batch() {
        let table = PapayaRouteTable::new();
        let mut routes = std::collections::HashMap::new();

        routes.insert("/hello".to_string(), Box::new(SimpleRoute::new("Hello", "text/plain")) as Box<dyn RouteEntry>);
        routes.insert("/goodbye".to_string(), Box::new(SimpleRoute::new("Goodbye", "text/plain")) as Box<dyn RouteEntry>);

        table.insert_batch(routes).await;

        assert_eq!(table.len().await, 2);
        assert!(table.contains("/hello").await);
        assert!(table.contains("/goodbye").await);
    }

    #[tokio::test]
    async fn test_papaya_route_table_get_all_paths() {
        let table = PapayaRouteTable::new();

        let route1 = SimpleRoute::new("Hello", "text/plain");
        let route2 = SimpleRoute::new("Goodbye", "text/plain");
        table.insert("/hello".to_string(), Box::new(route1)).await;
        table.insert("/goodbye".to_string(), Box::new(route2)).await;

        let paths = table.get_all_paths().await;
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&"/hello".to_string()));
        assert!(paths.contains(&"/goodbye".to_string()));
    }

    #[tokio::test]
    async fn test_papaya_route_table_update_or_insert() {
        let table = PapayaRouteTable::new();

        // 插入新路由
        let updater = |current: Option<(&String, &Box<dyn RouteEntry>)>| {
            assert!(current.is_none());
            let route = SimpleRoute::new("Hello", "text/plain");
            papaya::Operation::<Box<dyn RouteEntry>, ()>::Insert(Box::new(route) as Box<dyn RouteEntry>)
        };

        table.update_or_insert("/hello".to_string(), updater).await;
        assert!(table.contains("/hello").await);

        // 尝试更新已存在的路由（应该中止）
        let updater = |current: Option<(&String, &Box<dyn RouteEntry>)>| {
            assert!(current.is_some());
            papaya::Operation::<Box<dyn RouteEntry>, ()>::Abort(())
        };

        table.update_or_insert("/hello".to_string(), updater).await;
    }

    #[tokio::test]
    async fn test_papaya_route_table_concurrent_access() {
        use std::sync::Arc;
        let table = Arc::new(PapayaRouteTable::new());
        let mut handles = vec![];

        // 并发插入
        for i in 0..10 {
            let table_clone = Arc::clone(&table);
            let handle = tokio::spawn(async move {
                let route = SimpleRoute::new(format!("Route {}", i), "text/plain");
                table_clone.insert(format!("/route{}", i), Box::new(route)).await;
            });
            handles.push(handle);
        }

        // 等待所有插入完成
        for handle in handles {
            handle.await.unwrap();
        }

        assert_eq!(table.len().await, 10);

        // 并发读取
        let mut handles = vec![];
        for i in 0..10 {
            let table_clone = Arc::clone(&table);
            let handle = tokio::spawn(async move {
                table_clone.get(&format!("/route{}", i)).await
            });
            handles.push(handle);
        }

        // 等待所有读取完成
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_some());
        }
    }
}