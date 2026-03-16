//! Arc 优化的路由条目包装器
//!
//! 该模块提供了一个通用的包装器，可以自动为任何 RouteEntry 实现添加 Arc 优化。
//! 通过使用 Arc 共享路由条目，减少克隆开销，提升多线程环境下的性能。

use super::{RouteEntry, SerializableRoute};
use actix_web::{HttpRequest, HttpResponse};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Arc 优化的路由条目包装器
///
/// 该包装器使用 Arc 包装任何 RouteEntry 实现，提供零成本的克隆操作。
/// 当路由条目需要在多个线程或路由表中共享时，使用此包装器可以显著减少内存开销。
///
/// # 性能优势
///
/// - **零成本克隆**: `clone_box()` 仅增加 Arc 引用计数，不复制实际数据
/// - **内存高效**: 多个路由条目可以共享同一个底层数据
/// - **线程安全**: Arc 提供 Send + Sync 保证，适合多线程环境
///
/// # 使用场景
///
/// - 路由条目需要在多个地方共享
/// - 需要频繁克隆路由条目
/// - 多线程环境下的路由匹配
///
/// # 示例
///
/// ```
/// use dynamic_route_actix::{ArcRouteEntry, SimpleRoute, RouteEntry};
///
/// // 创建一个简单的路由
/// let simple_route = SimpleRoute::new("Hello, World!", "text/plain");
///
/// // 使用 ArcRouteEntry 包装
/// let arc_route = ArcRouteEntry::new(simple_route);
///
/// // 克隆是零成本的（仅增加引用计数）
/// let cloned = arc_route.clone_box();
///
/// // 两者共享相同的数据
/// assert_eq!(arc_route.to_serializable().body, cloned.to_serializable().body);
/// ```
#[derive(Debug)]
pub struct ArcRouteEntry {
    /// 使用 Arc 包装的内部路由条目
    inner: Arc<dyn RouteEntry>,
}

impl ArcRouteEntry {
    /// 创建一个新的 ArcRouteEntry
    ///
    /// # 参数
    ///
    /// * `route` - 要包装的路由条目
    ///
    /// # 返回
    ///
    /// 返回一个新的 ArcRouteEntry 实例
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::{ArcRouteEntry, SimpleRoute};
    ///
    /// let simple_route = SimpleRoute::new("Hello", "text/plain");
    /// let arc_route = ArcRouteEntry::new(simple_route);
    /// ```
    pub fn new<T: RouteEntry + 'static>(route: T) -> Self {
        Self {
            inner: Arc::new(route),
        }
    }

    /// 从 `Box<dyn RouteEntry>` 创建 ArcRouteEntry
    ///
    /// # 参数
    ///
    /// * `boxed_route` - 要包装的 boxed 路由条目
    ///
    /// # 返回
    ///
    /// 返回一个新的 ArcRouteEntry 实例
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::{ArcRouteEntry, RouteEntry, SimpleRoute};
    ///
    /// let boxed: Box<dyn RouteEntry> = Box::new(SimpleRoute::new("Hello", "text/plain"));
    /// let arc_route = ArcRouteEntry::from_boxed(boxed);
    /// ```
    pub fn from_boxed(boxed_route: Box<dyn RouteEntry>) -> Self {
        Self {
            inner: Arc::from(boxed_route),
        }
    }

    /// 获取内部路由条目的引用
    ///
    /// # 返回
    ///
    /// 返回内部 Arc 路由条目的引用
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::{ArcRouteEntry, SimpleRoute, RouteEntry};
    ///
    /// let simple_route = SimpleRoute::new("Hello", "text/plain");
    /// let arc_route = ArcRouteEntry::new(simple_route);
    ///
    /// // 通过 as_any 检查内部类型
    /// if let Some(simple) = arc_route.inner.as_any().downcast_ref::<SimpleRoute>() {
    ///     println!("Body: {}", simple.body);
    /// }
    /// ```
    pub fn inner(&self) -> &Arc<dyn RouteEntry> {
        &self.inner
    }

    /// 获取当前 Arc 的引用计数
    ///
    /// # 返回
    ///
    /// 返回当前的引用计数（至少为 1）
    ///
    /// # 注意
    ///
    /// 这个值主要用于调试和性能分析。在生产环境中，不应该依赖这个值。
    pub fn ref_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }
}

impl Clone for ArcRouteEntry {
    /// ArcRouteEntry 的克隆是零成本的
    ///
    /// 仅增加 Arc 的引用计数，不复制实际数据。
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::{ArcRouteEntry, SimpleRoute};
    ///
    /// let route1 = ArcRouteEntry::new(SimpleRoute::new("Hello", "text/plain"));
    /// let route2 = route1.clone(); // 零成本克隆
    ///
    /// assert_eq!(route1.ref_count(), 2);
    /// assert_eq!(route2.ref_count(), 2);
    /// ```
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl RouteEntry for ArcRouteEntry {
    /// 处理 HTTP 请求
    ///
    /// 委托给内部的路由条目处理请求。
    ///
    /// # 参数
    ///
    /// * `req` - HTTP 请求对象
    ///
    /// # 返回
    ///
    /// 返回一个 Future，该 Future 解析为 `HttpResponse`
    fn handle(&self, req: &HttpRequest) -> Pin<Box<dyn Future<Output = HttpResponse> + Send>> {
        self.inner.handle(req)
    }

    /// 克隆 ArcRouteEntry
    ///
    /// 这是零成本的克隆操作，仅增加 Arc 的引用计数。
    ///
    /// # 返回
    ///
    /// 返回克隆的 ArcRouteEntry
    fn clone_box(&self) -> Box<dyn RouteEntry> {
        Box::new(self.clone())
    }

    /// 将路由序列化为可传输的格式
    ///
    /// 委托给内部的路由条目进行序列化。
    ///
    /// # 返回
    ///
    /// 返回包含路由数据的 `SerializableRoute`
    fn to_serializable(&self) -> SerializableRoute {
        self.inner.to_serializable()
    }

    /// 从序列化数据创建路由实例
    ///
    /// 注意：这个方法会返回一个普通的 RouteEntry，而不是 ArcRouteEntry。
    /// 如果需要 Arc 优化，应该使用 `ArcRouteEntry::new()` 包装结果。
    ///
    /// # 参数
    ///
    /// * `data` - 序列化的路由数据
    ///
    /// # 返回
    ///
    /// 返回创建的路由实例
    fn from_serializable(data: SerializableRoute) -> Box<dyn RouteEntry>
    where
        Self: Sized,
    {
        // ArcRouteEntry 本身不直接支持从序列化数据创建
        // 而是委托给内部类型
        // 注意：这里返回的是内部类型，不是 ArcRouteEntry
        // 如果需要 Arc 优化，调用者应该用 ArcRouteEntry::new() 包装
        match data.route_type.as_str() {
            "SimpleRoute" => {
                use super::SimpleRoute;
                SimpleRoute::from_serializable(data)
            }
            _ => {
                // 尝试从注册表中查找类型
                use super::route_registry::RouteRegistry;
                if let Some(factory) = RouteRegistry::get_factory(&data.route_type) {
                    factory(data)
                } else {
                    // 如果找不到类型，返回一个错误路由
                    use super::SimpleRoute;
                    SimpleRoute::from_serializable(SerializableRoute {
                        route_type: "SimpleRoute".to_string(),
                        body: format!("Error: Unknown route type '{}'", data.route_type),
                        content_type: "text/plain".to_string(),
                        extra_data: None,
                    })
                }
            }
        }
    }

    /// 将路由处理器转换为 `Any` 类型
    ///
    /// # 返回
    ///
    /// 返回 `&dyn Any`，可以通过 `downcast_ref` 转换为具体类型
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SimpleRoute;

    #[test]
    fn test_arc_route_entry_creation() {
        let simple_route = SimpleRoute::new("Hello, World!", "text/plain");
        let arc_route = ArcRouteEntry::new(simple_route);

        let serializable = arc_route.to_serializable();
        assert_eq!(serializable.route_type, "SimpleRoute");
        assert_eq!(serializable.body, "Hello, World!");
    }

    #[test]
    fn test_arc_route_entry_clone() {
        let simple_route = SimpleRoute::new("Test", "text/plain");
        let arc_route1 = ArcRouteEntry::new(simple_route);

        assert_eq!(arc_route1.ref_count(), 1);

        let arc_route2 = arc_route1.clone_box();
        assert_eq!(arc_route1.ref_count(), 2);

        // 验证数据共享
        let s1 = arc_route1.to_serializable();
        let s2 = arc_route2.to_serializable();
        assert_eq!(s1.body, s2.body);
    }

    #[test]
    fn test_arc_route_entry_from_boxed() {
        let boxed: Box<dyn RouteEntry> = Box::new(SimpleRoute::new("Boxed", "application/json"));
        let arc_route = ArcRouteEntry::from_boxed(boxed);

        let serializable = arc_route.to_serializable();
        assert_eq!(serializable.body, "Boxed");
        assert_eq!(serializable.content_type, "application/json");
    }

    #[test]
    fn test_arc_route_entry_ref_count() {
        let simple_route = SimpleRoute::new("RefCount", "text/plain");
        let arc_route1 = ArcRouteEntry::new(simple_route);

        assert_eq!(arc_route1.ref_count(), 1);

        let arc_route2 = arc_route1.clone();
        assert_eq!(arc_route1.ref_count(), 2);
        assert_eq!(arc_route2.ref_count(), 2);

        drop(arc_route1);
        assert_eq!(arc_route2.ref_count(), 1);
    }

    #[test]
    fn test_arc_route_entry_inner_access() {
        let simple_route = SimpleRoute::new("Inner", "text/plain");
        let arc_route = ArcRouteEntry::new(simple_route);

        // 访问内部路由条目
        let inner = arc_route.inner();
        let serializable = inner.to_serializable();
        assert_eq!(serializable.body, "Inner");
    }
}