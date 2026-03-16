//! 中间件模块
//!
//! 提供常用的中间件实现，包括请求日志、认证和限流。

use actix_web::{
    body::{EitherBody, MessageBody},
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures_util::future::{ok, LocalBoxFuture, Ready};
use std::{
    collections::HashMap,
    net::IpAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;

/// 请求日志中间件
///
/// 记录每个请求的基本信息，包括方法、路径、状态码和响应时间。
///
/// # 示例
///
/// ```no_run
/// use actix_web::{App, web, HttpServer};
/// use dynamic_route_actix::actix::middleware::RequestLogger;
///
/// #[actix_web::main]
/// async fn main() -> std::io::Result<()> {
///     HttpServer::new(|| {
///         App::new()
///             .wrap(RequestLogger)
///             .route("/", web::get().to(|| async { "Hello" }))
///     })
///     .bind("127.0.0.1:8080")?
///     .run()
///     .await
/// }
/// ```
pub struct RequestLogger;

impl<S, B> Transform<S, ServiceRequest> for RequestLogger
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestLoggerMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RequestLoggerMiddleware { service })
    }
}

pub struct RequestLoggerMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RequestLoggerMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start = Instant::now();
        let method = req.method().clone();
        let path = req.path().to_string();

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            let duration = start.elapsed();
            let status = res.status();

            // 日志输出
            println!(
                "[{}] {} {} - {} ({:.2?})",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                method,
                path,
                status.as_u16(),
                duration
            );

            Ok(res.map_into_left_body())
        })
    }
}

/// 认证中间件
///
/// 验证请求是否包含有效的认证令牌。
///
/// # 示例
///
/// ```no_run
/// use actix_web::{App, web, HttpServer};
/// use dynamic_route_actix::actix::middleware::AuthMiddleware;
///
/// #[actix_web::main]
/// async fn main() -> std::io::Result<()> {
///     HttpServer::new(|| {
///         App::new()
///             .wrap(AuthMiddleware::from_token("secret-token"))
///             .route("/protected", web::get().to(|| async { "Protected content" }))
///     })
///     .bind("127.0.0.1:8080")?
///     .run()
///     .await
/// }
/// ```
pub struct AuthMiddleware {
    valid_tokens: Vec<String>,
}

impl AuthMiddleware {
    /// 创建新的认证中间件
    ///
    /// # 参数
    ///
    /// * `tokens` - 有效的认证令牌列表
    pub fn new(tokens: Vec<String>) -> Self {
        Self {
            valid_tokens: tokens,
        }
    }

    /// 从单个令牌创建认证中间件
    pub fn from_token(token: &str) -> Self {
        Self {
            valid_tokens: vec![token.to_string()],
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareImpl<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareImpl {
            service,
            valid_tokens: self.valid_tokens.clone(),
        })
    }
}

pub struct AuthMiddlewareImpl<S> {
    service: S,
    valid_tokens: Vec<String>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareImpl<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let valid_tokens = self.valid_tokens.clone();

        // 从请求头中获取令牌
        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok());

        let is_valid = match auth_header {
            Some(header) if header.starts_with("Bearer ") => {
                let token = &header[7..];
                valid_tokens.iter().any(|t| t == token)
            }
            _ => false,
        };

        if !is_valid {
            let (req, _payload) = req.into_parts();
            let response = HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized",
                "message": "Invalid or missing authentication token"
            }));
            return Box::pin(async move {
                Ok(ServiceResponse::new(req, response).map_into_right_body())
            });
        }

        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res.map_into_left_body())
        })
    }
}

/// 限流中间件
///
/// 基于令牌桶算法实现请求限流，防止服务被滥用。
///
/// # 示例
///
/// ```no_run
/// use actix_web::{App, web, HttpServer};
/// use dynamic_route_actix::actix::middleware::RateLimiter;
///
/// #[actix_web::main]
/// async fn main() -> std::io::Result<()> {
///     HttpServer::new(|| {
///         App::new()
///             .wrap(RateLimiter::new(100, 60)) // 每分钟 100 个请求
///             .route("/", web::get().to(|| async { "Hello" }))
///     })
///     .bind("127.0.0.1:8080")?
///     .run()
///     .await
/// }
/// ```
pub struct RateLimiter {
    max_requests: usize,
    window_duration: Duration,
    state: Arc<RwLock<HashMap<IpAddr, ClientState>>>,
}

impl RateLimiter {
    /// 创建新的限流中间件
    ///
    /// # 参数
    ///
    /// * `max_requests` - 时间窗口内允许的最大请求数
    /// * `window_seconds` - 时间窗口的秒数
    pub fn new(max_requests: usize, window_seconds: u64) -> Self {
        Self {
            max_requests,
            window_duration: Duration::from_secs(window_seconds),
            state: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 清理过期的客户端状态
    pub async fn cleanup_expired(&self) {
        let mut state = self.state.write().await;
        let now = Instant::now();

        state.retain(|_, client_state| {
            now.duration_since(client_state.window_start) < self.window_duration * 2
        });
    }
}

/// 客户端状态
#[derive(Debug, Clone)]
struct ClientState {
    request_count: usize,
    window_start: Instant,
}

impl<S, B> Transform<S, ServiceRequest> for RateLimiter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimiterImpl<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RateLimiterImpl {
            service: std::sync::Arc::new(service),
            max_requests: self.max_requests,
            window_duration: self.window_duration,
            state: self.state.clone(),
        })
    }
}

pub struct RateLimiterImpl<S> {
    service: std::sync::Arc<S>,
    max_requests: usize,
    window_duration: Duration,
    state: Arc<RwLock<HashMap<IpAddr, ClientState>>>,
}

impl<S, B> Service<ServiceRequest> for RateLimiterImpl<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let client_ip = req
            .connection_info()
            .peer_addr()
            .and_then(|addr| addr.parse::<IpAddr>().ok())
            .unwrap_or_else(|| IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)));

        let state = self.state.clone();
        let max_requests = self.max_requests;
        let window_duration = self.window_duration;
        let service = self.service.clone();

        Box::pin(async move {
            let mut state_guard = state.write().await;
            let now = Instant::now();

            let client_state = state_guard.entry(client_ip).or_insert_with(|| ClientState {
                request_count: 0,
                window_start: now,
            });

            // 检查是否需要重置计数器
            if now.duration_since(client_state.window_start) >= window_duration {
                client_state.request_count = 0;
                client_state.window_start = now;
            }

            // 检查是否超过限制
            if client_state.request_count >= max_requests {
                let (req, _payload) = req.into_parts();
                let response = HttpResponse::TooManyRequests().json(serde_json::json!({
                    "error": "Rate limit exceeded",
                    "message": format!("Maximum {} requests per {:?} allowed", max_requests, window_duration)
                }));
                return Ok(ServiceResponse::new(req, response).map_into_right_body());
            }

            client_state.request_count += 1;
            drop(state_guard);

            let fut = service.call(req);
            let res = fut.await?;
            Ok(res.map_into_left_body())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};

    #[actix_web::test]
    async fn test_request_logger() {
        let app = test::init_service(
            App::new()
                .wrap(RequestLogger)
                .route("/test", web::get().to(|| async { "Hello" })),
        )
        .await;

        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 200);
    }

    #[actix_web::test]
    async fn test_auth_middleware_valid_token() {
        let app = test::init_service(
            App::new()
                .wrap(AuthMiddleware::from_token("valid-token"))
                .route("/protected", web::get().to(|| async { "Protected" })),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/protected")
            .insert_header(("Authorization", "Bearer valid-token"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }

    #[actix_web::test]
    async fn test_auth_middleware_invalid_token() {
        let app = test::init_service(
            App::new()
                .wrap(AuthMiddleware::from_token("valid-token"))
                .route("/protected", web::get().to(|| async { "Protected" })),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/protected")
            .insert_header(("Authorization", "Bearer invalid-token"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401);
    }

    #[actix_web::test]
    async fn test_auth_middleware_missing_token() {
        let app = test::init_service(
            App::new()
                .wrap(AuthMiddleware::from_token("valid-token"))
                .route("/protected", web::get().to(|| async { "Protected" })),
        )
        .await;

        let req = test::TestRequest::get().uri("/protected").to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401);
    }

    #[actix_web::test]
    async fn test_rate_limiter() {
        let app = test::init_service(
            App::new()
                .wrap(RateLimiter::new(2, 60)) // 每分钟 2 个请求
                .route("/test", web::get().to(|| async { "Hello" })),
        )
        .await;

        // 前 2 个请求应该成功
        for _ in 0..2 {
            let req = test::TestRequest::get().uri("/test").to_request();
            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), 200);
        }

        // 第 3 个请求应该被限流
        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 429);
    }
}
