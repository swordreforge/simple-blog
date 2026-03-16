//! 路由类型注册表
//!
//! 提供运行时路由类型注册和动态实例化功能，支持多种自定义路由类型的序列化和反序列化。

use super::RouteEntry;
use super::SerializableRoute;
use std::collections::HashMap;
use std::sync::RwLock;

/// 路由工厂函数类型
///
/// 用于从序列化数据创建路由实例。
pub type RouteFactory = fn(SerializableRoute) -> Box<dyn RouteEntry>;

/// 全局路由类型注册表
///
/// 使用 RwLock 实现线程安全的类型注册表，支持在运行时动态注册和查询路由类型。
///
/// # 线程安全
///
/// 注册表使用 `RwLock` 实现，允许多个读操作或单个写操作并发执行。
///
/// # 示例
///
/// ```
/// use dynamic_route_actix::core::route_registry::{RouteRegistry, RouteFactory, RouteRegistryError};
///
/// // 定义一个工厂函数
/// fn custom_factory(data: dynamic_route_actix::SerializableRoute) -> Box<dyn dynamic_route_actix::RouteEntry> {
///     Box::new(dynamic_route_actix::SimpleRoute::new(data.body, data.content_type))
/// }
///
/// // 注册自定义路由类型
/// RouteRegistry::register("CustomRoute", custom_factory);
///
/// // 获取工厂函数
/// let factory = RouteRegistry::get_factory("CustomRoute");
/// assert!(factory.is_some());
/// ```
pub struct RouteRegistry;

/// 注册表错误类型
#[derive(Debug, thiserror::Error)]
pub enum RouteRegistryError {
    /// 路由类型已存在
    #[error("Route type '{0}' is already registered")]
    AlreadyRegistered(String),

    /// 路由类型未找到
    #[error("Route type '{0}' not found in registry")]
    NotFound(String),

    /// 反序列化失败
    #[error("Failed to deserialize route: {0}")]
    DeserializationError(String),
}

// 使用 once_cell 的 Lazy 实现全局静态注册表
// 在这个版本中，我们使用 std::sync::OnceLock (Rust 1.70+) 或者手动实现懒初始化
use std::sync::OnceLock;

static REGISTRY: OnceLock<RwLock<HashMap<String, RouteFactory>>> = OnceLock::new();

impl RouteRegistry {
    /// 获取注册表实例
    fn get_registry() -> &'static RwLock<HashMap<String, RouteFactory>> {
        REGISTRY.get_or_init(|| {
            // 初始化时注册 SimpleRoute
            let mut map: HashMap<String, RouteFactory> = HashMap::new();
            map.insert("SimpleRoute".to_string(), super::SimpleRoute::from_serializable);
            RwLock::new(map)
        })
    }

    /// 注册一个新的路由类型
    ///
    /// # 参数
    ///
    /// * `route_type` - 路由类型标识符（如 "SimpleRoute", "CustomRoute"）
    /// * `factory` - 工厂函数，用于从序列化数据创建路由实例
    ///
    /// # 返回
    ///
    /// 如果注册成功，返回 `Ok(())`；如果类型已存在，返回 `Err(RouteRegistryError::AlreadyRegistered)`
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::core::route_registry::RouteRegistry;
    ///
    /// fn my_factory(data: dynamic_route_actix::SerializableRoute) -> Box<dyn dynamic_route_actix::RouteEntry> {
    ///     Box::new(dynamic_route_actix::SimpleRoute::new(data.body, data.content_type))
    /// }
    ///
    /// RouteRegistry::register("MyRoute", my_factory);
    /// ```
    pub fn register(route_type: &str, factory: RouteFactory) -> Result<(), RouteRegistryError> {
        let mut registry = Self::get_registry().write().unwrap();

        if registry.contains_key(route_type) {
            return Err(RouteRegistryError::AlreadyRegistered(route_type.to_string()));
        }

        registry.insert(route_type.to_string(), factory);
        Ok(())
    }

    /// 获取指定类型的工厂函数
    ///
    /// # 参数
    ///
    /// * `route_type` - 路由类型标识符
    ///
    /// # 返回
    ///
    /// 如果类型存在，返回 `Some(RouteFactory)`；否则返回 `None`
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::core::route_registry::RouteRegistry;
    ///
    /// let factory = RouteRegistry::get_factory("SimpleRoute");
    /// assert!(factory.is_some());
    /// ```
    pub fn get_factory(route_type: &str) -> Option<RouteFactory> {
        let registry = Self::get_registry().read().unwrap();
        registry.get(route_type).copied()
    }

    /// 检查指定类型是否已注册
    ///
    /// # 参数
    ///
    /// * `route_type` - 路由类型标识符
    ///
    /// # 返回
    ///
    /// 如果类型已注册，返回 `true`；否则返回 `false`
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::core::route_registry::RouteRegistry;
    ///
    /// assert!(RouteRegistry::is_registered("SimpleRoute"));
    /// assert!(!RouteRegistry::is_registered("NonExistentRoute"));
    /// ```
    pub fn is_registered(route_type: &str) -> bool {
        let registry = Self::get_registry().read().unwrap();
        registry.contains_key(route_type)
    }

    /// 从序列化数据创建路由实例
    ///
    /// # 参数
    ///
    /// * `data` - 序列化的路由数据
    ///
    /// # 返回
    ///
    /// 如果类型已注册，返回 `Ok(Box<dyn RouteEntry>)`；否则返回 `Err(RouteRegistryError::NotFound)`
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::core::route_registry::RouteRegistry;
    /// use dynamic_route_actix::SerializableRoute;
    ///
    /// let data = SerializableRoute {
    ///     route_type: "SimpleRoute".to_string(),
    ///     body: "Hello".to_string(),
    ///     content_type: "text/plain".to_string(),
    /// };
    ///
    /// let route = RouteRegistry::create_route(data);
    /// assert!(route.is_ok());
    /// ```
    pub fn create_route(data: SerializableRoute) -> Result<Box<dyn RouteEntry>, RouteRegistryError> {
        let factory = Self::get_factory(&data.route_type)
            .ok_or_else(|| RouteRegistryError::NotFound(data.route_type.clone()))?;

        Ok(factory(data))
    }

    /// 获取所有已注册的路由类型
    ///
    /// # 返回
    ///
    /// 返回包含所有已注册类型标识符的 `Vec<String>`
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::core::route_registry::RouteRegistry;
    ///
    /// let types = RouteRegistry::list_types();
    /// assert!(types.contains(&"SimpleRoute".to_string()));
    /// ```
    pub fn list_types() -> Vec<String> {
        let registry = Self::get_registry().read().unwrap();
        registry.keys().cloned().collect()
    }

    /// 注销指定类型的路由
    ///
    /// # 参数
    ///
    /// * `route_type` - 要注销的路由类型标识符
    ///
    /// # 返回
    ///
    /// 如果类型存在并被成功注销，返回 `true`；否则返回 `false`
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::core::route_registry::RouteRegistry;
    ///
    /// RouteRegistry::unregister("CustomRoute");
    /// ```
    pub fn unregister(route_type: &str) -> bool {
        let mut registry = Self::get_registry().write().unwrap();
        registry.remove(route_type).is_some()
    }

    /// 清空注册表（主要用于测试）
    ///
    /// # 警告
    ///
    /// 此方法会清空所有已注册的类型，包括默认的 `SimpleRoute`。
    /// 仅用于测试目的。
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::core::route_registry::RouteRegistry;
    ///
    /// // 清空注册表
    /// RouteRegistry::clear();
    /// ```
    #[cfg(test)]
    pub fn clear() {
        let mut registry = Self::get_registry().write().unwrap();
        registry.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_factory(data: SerializableRoute) -> Box<dyn RouteEntry> {
        Box::new(super::super::SimpleRoute::new(data.body, data.content_type))
    }

    #[test]
    fn test_registry_initialization() {
        // 验证 SimpleRoute 默认已注册
        assert!(RouteRegistry::is_registered("SimpleRoute"));
    }

    #[test]
    fn test_register_route_type() {
        let test_type = format!("TestRoute_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos());
        assert!(!RouteRegistry::is_registered(&test_type));

        // 注册新类型
        let result = RouteRegistry::register(&test_type, test_factory);
        assert!(result.is_ok());
        assert!(RouteRegistry::is_registered(&test_type));

        // 清理
        RouteRegistry::unregister(&test_type);
    }

    #[test]
    fn test_register_duplicate_type() {
        let test_type = format!("DuplicateRoute_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos());

        // 第一次注册应该成功
        let result1 = RouteRegistry::register(&test_type, test_factory);
        assert!(result1.is_ok());

        // 第二次注册应该失败
        let result2 = RouteRegistry::register(&test_type, test_factory);
        assert!(matches!(result2, Err(RouteRegistryError::AlreadyRegistered(_))));

        // 清理
        RouteRegistry::unregister(&test_type);
    }

    #[test]
    fn test_get_factory() {
        let test_type = format!("GetTestRoute_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos());
        RouteRegistry::register(&test_type, test_factory).unwrap();

        let factory = RouteRegistry::get_factory(&test_type);
        assert!(factory.is_some());

        let factory = RouteRegistry::get_factory("NonExistentRoute");
        assert!(factory.is_none());

        // 清理
        RouteRegistry::unregister(&test_type);
    }

    #[test]
    fn test_create_route() {
        let test_type = format!("CreateTestRoute_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos());
        RouteRegistry::register(&test_type, test_factory).unwrap();

        let data = SerializableRoute {
            route_type: test_type.clone(),
            body: "Test Body".to_string(),
            content_type: "text/plain".to_string(),
            extra_data: None,
        };

        let route = RouteRegistry::create_route(data);
        assert!(route.is_ok());

        // 清理
        RouteRegistry::unregister(&test_type);
    }

    #[test]
    fn test_create_route_unregistered_type() {
        let data = SerializableRoute {
            route_type: "NonExistentRoute".to_string(),
            body: "Test Body".to_string(),
            content_type: "text/plain".to_string(),
            extra_data: None,
        };

        let route = RouteRegistry::create_route(data);
        assert!(matches!(route, Err(RouteRegistryError::NotFound(_))));
    }

    #[test]
    fn test_list_types() {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let route1 = format!("Route1_{}", timestamp);
        let route2 = format!("Route2_{}", timestamp);
        let route3 = format!("Route3_{}", timestamp);

        RouteRegistry::register(&route1, test_factory).unwrap();
        RouteRegistry::register(&route2, test_factory).unwrap();
        RouteRegistry::register(&route3, test_factory).unwrap();

        let types = RouteRegistry::list_types();
        assert!(types.contains(&route1));
        assert!(types.contains(&route2));
        assert!(types.contains(&route3));

        // 清理
        RouteRegistry::unregister(&route1);
        RouteRegistry::unregister(&route2);
        RouteRegistry::unregister(&route3);
    }

    #[test]
    fn test_clear_registry() {
        let test_type = format!("ClearTestRoute_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos());
        RouteRegistry::register(&test_type, test_factory).unwrap();
        assert!(RouteRegistry::is_registered(&test_type));

        RouteRegistry::clear();
        assert!(!RouteRegistry::is_registered(&test_type));
        assert!(!RouteRegistry::is_registered("SimpleRoute"));

        // 清理后重新注册 SimpleRoute，确保后续测试正常
        RouteRegistry::register("SimpleRoute", crate::SimpleRoute::from_serializable).unwrap();
    }
}