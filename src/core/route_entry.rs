use actix_web::{HttpRequest, HttpResponse};
use std::future::Future;
use std::pin::Pin;

/// 可序列化的路由数据
///
/// 用于路由的持久化和传输。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SerializableRoute {
    /// 路由类型标识符
    pub route_type: String,
    /// 响应体内容
    pub body: String,
    /// Content-Type
    pub content_type: String,
}

/// 路由处理器 trait
///
/// 定义了所有路由处理器必须实现的接口。
/// 每个 `RouteEntry` 负责处理匹配到的 HTTP 请求并返回响应。
///
/// # 线程安全
///
/// `RouteEntry` 要求实现 `Send + Sync + 'static`，确保可以在多线程环境中安全使用。
///
/// # 示例
///
/// ```
/// use actix_web::{HttpRequest, HttpResponse};
/// use dynamic_route_actix::{RouteEntry, SerializableRoute};
/// use std::future::Future;
/// use std::pin::Pin;
///
/// #[derive(Debug)]
/// struct CustomRoute {
///     message: String,
/// }
///
/// impl RouteEntry for CustomRoute {
///     fn handle(&self, _req: &HttpRequest) -> Pin<Box<dyn Future<Output = HttpResponse> + Send>> {
///         let message = self.message.clone();
///         Box::pin(async move {
///             HttpResponse::Ok()
///                 .content_type("text/plain")
///                 .body(message)
///         })
///     }
///
///     fn clone_box(&self) -> Box<dyn RouteEntry> {
///         Box::new(CustomRoute {
///             message: self.message.clone(),
///         })
///     }
///
///     fn to_serializable(&self) -> SerializableRoute {
///         SerializableRoute {
///             route_type: "CustomRoute".to_string(),
///             body: self.message.clone(),
///             content_type: "text/plain".to_string(),
///         }
///     }
///
///     fn from_serializable(data: SerializableRoute) -> Box<dyn RouteEntry>
///     where
///         Self: Sized,
///     {
///         Box::new(CustomRoute {
///             message: data.body,
///         })
///     }
/// }
/// ```
pub trait RouteEntry: Send + Sync + 'static + std::fmt::Debug {
    /// 处理 HTTP 请求并返回响应
    ///
    /// # 参数
    ///
    /// * `req` - HTTP 请求对象，包含请求的元数据和方法、路径等信息
    ///
    /// # 返回
    ///
    /// 返回一个 Future，该 Future 解析为 `HttpResponse`
    ///
    /// # 示例
    ///
    /// ```
    /// use actix_web::{HttpRequest, HttpResponse};
    /// use dynamic_route_actix::RouteEntry;
    /// use std::pin::Pin;
    ///
    /// let route = dynamic_route_actix::SimpleRoute::new("Hello", "text/plain");
    /// // route.handle(&req).await; // 在异步上下文中调用
    /// ```
    fn handle(&self, req: &HttpRequest) -> Pin<Box<dyn Future<Output = HttpResponse> + Send>>;

    /// 克隆路由处理器
    ///
    /// 返回一个新的 `Box<dyn RouteEntry>`，包含与当前处理器相同的数据。
    ///
    /// # 返回
    ///
    /// 返回克隆的路由处理器
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::{RouteEntry, SimpleRoute};
    ///
    /// let route = SimpleRoute::new("Hello", "text/plain");
    /// let cloned = route.clone_box();
    /// ```
    fn clone_box(&self) -> Box<dyn RouteEntry>;

    /// 将路由序列化为可传输的格式
    ///
    /// # 返回
    ///
    /// 返回包含路由数据的 `SerializableRoute`
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::{RouteEntry, SimpleRoute};
    ///
    /// let route = SimpleRoute::new("Hello, World!", "text/plain");
    /// let serializable = route.to_serializable();
    /// assert_eq!(serializable.route_type, "SimpleRoute");
    /// ```
    fn to_serializable(&self) -> SerializableRoute;

    /// 从序列化数据创建路由实例
    ///
    /// # 参数
    ///
    /// * `data` - 序列化的路由数据
    ///
    /// # 返回
    ///
    /// 返回创建的路由实例
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::{RouteEntry, SimpleRoute, SerializableRoute};
    ///
    /// let data = SerializableRoute {
    ///     route_type: "SimpleRoute".to_string(),
    ///     body: "Hello!".to_string(),
    ///     content_type: "text/plain".to_string(),
    /// };
    /// let route = SimpleRoute::from_serializable(data);
    /// ```
    fn from_serializable(data: SerializableRoute) -> Box<dyn RouteEntry>
    where
        Self: Sized;
}
