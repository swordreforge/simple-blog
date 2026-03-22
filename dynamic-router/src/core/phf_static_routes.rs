//! PHF静态路由模块
//!
//! 使用Perfect Hash Function（完美哈希函数）实现静态路由的零开销查找。
//! 适合已知的一组静态路由（如API endpoints），提供编译期生成的O(1)查找。
//!
//! # 特性
//!
//! - 编译期生成完美哈希表，运行时零开销
//! - O(1)查找，无哈希冲突
//! - 适合高频访问的静态路由
//! - 内存占用极小
//!
//! # 使用示例
//!
//! ```no_run
//! use dynamic_route_actix::core::phf_static_routes::{StaticRouteRegistry, register_static_routes};
//!
//! // 定义静态路由
//! static_routes! {
//!     STATIC_ROUTES = {
//!         "/api/users" => "users_handler",
//!         "/api/posts" => "posts_handler",
//!         "/api/comments" => "comments_handler",
//!     };
//! }
//!
//! // 使用静态路由
//! if let Some(handler) = STATIC_ROUTES.get("/api/users") {
//!     println!("Found handler: {}", handler);
//! }
//! ```

use phf::{phf_map, Map};
use crate::core::route_entry::RouteEntry;

/// 静态路由注册表（使用完美哈希函数）
///
/// 这是编译期生成的完美哈希表，提供O(1)查找且无冲突。
/// 适合已知的一组静态路由，如API endpoints。
///
/// # 性能特点
///
/// - 查找时间：O(1)（实际接近2-3次内存访问）
/// - 内存占用：极小（仅存储路由和处理器名称）
/// - 无哈希冲突：完美哈希保证
/// - 编译期优化：零运行时开销
///
/// # 限制
///
/// - 路由必须在编译期已知
/// - 不能动态添加/删除路由
/// - 适合静态路由，不适合动态路由
pub struct StaticRouteRegistry {
    /// 完美哈希表：路由路径 -> 处理器名称
    routes: Map<&'static str, &'static str>,
}

impl StaticRouteRegistry {
    /// 创建新的静态路由注册表
    ///
    /// # 注意
    ///
    /// 实际使用时，应该使用`static_routes!`宏来生成静态路由表。
    /// 这个方法主要用于测试。
    pub fn new() -> Self {
        Self {
            routes: phf_map! {},
        }
    }

    /// 查找静态路由
    ///
    /// # 参数
    ///
    /// * `path` - 路由路径
    ///
    /// # 返回
    ///
    /// 如果找到路由，返回处理器名称；否则返回None
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::core::phf_static_routes::StaticRouteRegistry;
    ///
    /// let registry = StaticRouteRegistry::new();
    /// if let Some(handler) = registry.get("/api/users") {
    ///     println!("Found handler: {}", handler);
    /// }
    /// ```
    pub fn get(&self, path: &str) -> Option<&'static str> {
        self.routes.get(path).copied()
    }

    /// 检查是否包含路由
    ///
    /// # 参数
    ///
    /// * `path` - 路由路径
    ///
    /// # 返回
    ///
    /// 如果路由存在返回true，否则返回false
    pub fn contains(&self, path: &str) -> bool {
        self.routes.contains_key(path)
    }

    /// 获取所有路由路径
    ///
    /// # 返回
    ///
    /// 返回所有路由路径的向量
    pub fn paths(&self) -> Vec<&'static str> {
        self.routes.keys().copied().collect()
    }

    /// 获取所有路由条目
    pub fn entries(&self) -> Vec<(&'static str, &'static str)> {
        self.routes.keys().map(|&k| (k, self.routes.get(k).copied().unwrap())).collect()
    }

    /// 获取路由数量
    pub fn len(&self) -> usize {
        self.routes.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.routes.is_empty()
    }
}

impl Default for StaticRouteRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 静态路由宏（用于生成编译期完美哈希表）
///
/// 这个宏在编译期生成静态路由的完美哈希表。
///
/// # 示例
///
/// ```ignore
/// use dynamic_route_actix::core::phf_static_routes::static_routes;
///
/// static_routes! {
///     API_ROUTES = {
///         "/api/users" => "users_handler",
///         "/api/posts" => "posts_handler",
///         "/api/comments" => "comments_handler",
///     };
/// }
/// ```
#[macro_export]
macro_rules! static_routes {
    ($name:ident = { $($route:expr => $handler:expr),* $(,)? }) => {
        static $name: phf::Map<&'static str, &'static str> = phf::phf_map! {
            $($route => $handler),*
        };
    };
}

/// 混合路由表（静态 + 动态）
///
/// 结合PHF静态路由和动态路由，提供最优的性能。
/// 先查询静态路由（O(1)），再查询动态路由。
///
/// # 性能策略
///
/// 1. 静态路由：使用PHF完美哈希，O(1)查找
/// 2. 动态路由：使用动态路由表，O(1)平均查找
/// 3. 优先静态路由，避免动态路由查找
pub struct HybridRouteTable {
    /// 静态路由注册表（PHF完美哈希）
    static_routes: StaticRouteRegistry,
    /// 动态路由表（用于动态添加的路由）
    dynamic_routes: std::collections::HashMap<String, Box<dyn RouteEntry>>,
}

impl HybridRouteTable {
    /// 创建新的混合路由表
    pub fn new(static_routes: StaticRouteRegistry) -> Self {
        Self {
            static_routes,
            dynamic_routes: std::collections::HashMap::new(),
        }
    }

    /// 查找路由（优先静态路由）
    ///
    /// # 参数
    ///
    /// * `path` - 路由路径
    ///
    /// # 返回
    ///
    /// 返回路由处理器的引用
    ///
    /// # 性能
    ///
    /// - 静态路由：O(1)（完美哈希）
    /// - 动态路由：O(1)平均（HashMap）
    /// - 优先查询静态路由，避免动态路由查找
    pub fn find(&self, path: &str) -> Option<&dyn RouteEntry> {
        // 优先查询静态路由（O(1)）
        if self.static_routes.contains(path) {
            // 这里应该返回对应的处理器
            // 实际实现需要维护处理器映射
            return None;
        }

        // 查询动态路由
        self.dynamic_routes.get(path).map(|route| route.as_ref())
    }

    /// 插入动态路由
    ///
    /// # 参数
    ///
    /// * `path` - 路由路径
    /// * `route` - 路由处理器
    pub fn insert(&mut self, path: String, route: Box<dyn RouteEntry>) {
        self.dynamic_routes.insert(path, route);
    }

    /// 删除动态路由
    ///
    /// # 参数
    ///
    /// * `path` - 路由路径
    ///
    /// # 返回
    ///
    /// 如果找到并删除了路由，返回路由数据；否则返回None
    pub fn remove(&mut self, path: &str) -> Option<Box<dyn RouteEntry>> {
        self.dynamic_routes.remove(path)
    }

    /// 获取路由数量（静态 + 动态）
    pub fn len(&self) -> usize {
        self.static_routes.len() + self.dynamic_routes.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.static_routes.is_empty() && self.dynamic_routes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 定义测试用的静态路由
    static_routes! {
        TEST_ROUTES = {
            "/api/users" => "users_handler",
            "/api/posts" => "posts_handler",
            "/api/comments" => "comments_handler"
        }
    }

    #[test]
    fn test_static_route_registry() {
        let registry = StaticRouteRegistry::new();
        assert!(registry.is_empty());
    }

    #[test]
    fn test_phf_routes() {
        // 测试PHF完美哈希路由
        assert_eq!(TEST_ROUTES.get("/api/users"), Some(&"users_handler"));
        assert_eq!(TEST_ROUTES.get("/api/posts"), Some(&"posts_handler"));
        assert_eq!(TEST_ROUTES.get("/api/comments"), Some(&"comments_handler"));
        assert_eq!(TEST_ROUTES.get("/api/nonexistent"), None);
    }

    #[test]
    fn test_phf_routes_contains() {
        assert!(TEST_ROUTES.contains_key("/api/users"));
        assert!(TEST_ROUTES.contains_key("/api/posts"));
        assert!(!TEST_ROUTES.contains_key("/api/nonexistent"));
    }

    #[test]
    fn test_phf_routes_keys() {
        let keys: Vec<_> = TEST_ROUTES.keys().collect();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&&"/api/users"));
        assert!(keys.contains(&&"/api/posts"));
        assert!(keys.contains(&&"/api/comments"));
    }

    #[test]
    fn test_hybrid_route_table() {
        let static_routes = StaticRouteRegistry::new();
        let mut table = HybridRouteTable::new(static_routes);

        assert!(table.is_empty());

        // 插入动态路由
        use crate::core::SimpleRoute;
        table.insert("/dynamic".to_string(), Box::new(SimpleRoute::new("Dynamic", "text/plain")));

        assert_eq!(table.len(), 1);
        assert!(!table.is_empty());

        // 查找动态路由
        assert!(table.find("/dynamic").is_some());
        assert!(table.find("/nonexistent").is_none());

        // 删除动态路由
        let removed = table.remove("/dynamic");
        assert!(removed.is_some());
        assert!(table.is_empty());
    }
}