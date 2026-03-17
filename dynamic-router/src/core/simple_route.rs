use super::{RouteEntry, SerializableRoute};
use actix_web::{HttpRequest, HttpResponse};
use std::future::Future;
use std::pin::Pin;

/// 简单的路由处理器
///
/// 返回固定内容的路由，适用于静态响应场景。
///
/// # 字段
///
/// * `body` - 响应体内容（使用 Arc 共享，减少克隆开销）
/// * `content_type` - 响应的 Content-Type（使用 Arc 共享，减少克隆开销）
///
/// # 示例
///
/// ```
/// use dynamic_route_actix::SimpleRoute;
///
/// let route = SimpleRoute::new("Hello, World!", "text/plain");
/// assert_eq!(&*route.body, "Hello, World!");
/// assert_eq!(&*route.content_type, "text/plain");
/// ```
#[derive(Debug, Clone)]
pub struct SimpleRoute {
    /// 响应体内容（使用 Arc 共享，减少克隆开销）
    pub body: std::sync::Arc<str>,
    /// 内容类型（使用 Arc 共享，减少克隆开销）
    pub content_type: std::sync::Arc<str>,
}

impl SimpleRoute {
    /// 创建一个新的简单路由
    ///
    /// # 参数
    ///
    /// * `body` - 响应体内容
    /// * `content_type` - Content-Type 头的值，例如 "text/plain" 或 "application/json"
    ///
    /// # 返回
    ///
    /// 返回一个新的 `SimpleRoute` 实例
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::SimpleRoute;
    ///
    /// let route = SimpleRoute::new("Hello", "text/plain");
    /// ```
    pub fn new<S: Into<String>, C: Into<String>>(body: S, content_type: C) -> Self {
        Self {
            body: body.into().into(),
            content_type: content_type.into().into(),
        }
    }
}

impl RouteEntry for SimpleRoute {
    /// 处理 HTTP 请求并返回预定义的响应
    ///
    /// 总是返回状态码 200 OK，包含预定义的 body 和 content_type。
    ///
    /// # 参数
    ///
    /// * `_req` - HTTP 请求对象（在简单路由中未使用）
    ///
    /// # 返回
    ///
    /// 返回一个 Future，该 Future 解析为包含预定义内容的 `HttpResponse`
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::{SimpleRoute, RouteEntry};
    /// use actix_web::HttpRequest;
    /// // 在实际使用中，需要一个真实的 HttpRequest
    /// // let response = route.handle(&req).await;
    /// ```
    fn handle(&self, _req: &HttpRequest) -> Pin<Box<dyn Future<Output = HttpResponse> + Send>> {
        // 使用 Arc::clone 而非字符串克隆，显著减少开销
        let body = std::sync::Arc::clone(&self.body);
        let content_type = std::sync::Arc::clone(&self.content_type);
        Box::pin(async move {
            HttpResponse::Ok()
                .content_type(content_type.as_ref())
                .body(body.as_ref().to_string())
        })
    }

    /// 克隆 SimpleRoute
    ///
    /// 返回一个新的 SimpleRoute 实例，包含相同的 body 和 content_type。
    /// 使用 Arc::clone 增加引用计数，避免深拷贝。
    fn clone_box(&self) -> Box<dyn RouteEntry> {
        Box::new(SimpleRoute {
            body: std::sync::Arc::clone(&self.body),
            content_type: std::sync::Arc::clone(&self.content_type),
        })
    }

    /// 将 SimpleRoute 序列化为 SerializableRoute
    ///
    /// # 返回
    ///
    /// 返回包含 SimpleRoute 数据的 SerializableRoute
    fn to_serializable(&self) -> SerializableRoute {
        SerializableRoute {
            route_type: "SimpleRoute".to_string(),
            body: self.body.to_string(),
            content_type: self.content_type.to_string(),
            extra_data: None, // SimpleRoute 不需要额外数据
        }
    }

    /// 从 SerializableRoute 创建 SimpleRoute 实例
    ///
    /// # 参数
    ///
    /// * `data` - 序列化的路由数据
    ///
    /// # 返回
    ///
    /// 返回创建的 SimpleRoute 实例
    fn from_serializable(data: SerializableRoute) -> Box<dyn RouteEntry>
    where
        Self: Sized,
    {
        Box::new(SimpleRoute {
            body: data.body.into(),
            content_type: data.content_type.into(),
        })
    }

    /// 将 SimpleRoute 转换为 Any 类型
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_route_creation() {
        let route = SimpleRoute::new("test body", "text/plain");
        assert_eq!(&*route.body, "test body");
        assert_eq!(&*route.content_type, "text/plain");
    }

    #[test]
    fn test_simple_route_creation_with_str() {
        let route = SimpleRoute::new("hello", "application/json");
        assert_eq!(&*route.body, "hello");
        assert_eq!(&*route.content_type, "application/json");
    }

    #[test]
    fn test_simple_route_clone() {
        let route1 = SimpleRoute::new("original", "text/plain");
        let route2 = route1.clone();
        assert_eq!(&*route1.body, &*route2.body);
        assert_eq!(&*route1.content_type, &*route2.content_type);
    }

    #[test]
    fn test_simple_route_empty_body() {
        let route = SimpleRoute::new("", "text/plain");
        assert_eq!(&*route.body, "");
    }
}
