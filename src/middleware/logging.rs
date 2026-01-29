use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use std::time::{Duration, Instant};
use std::rc::Rc;

/// 自定义日志中间件
pub struct LoggingMiddleware;

impl<S, B> Transform<S, ServiceRequest> for LoggingMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = LoggingMiddlewareService<S>;
    type InitError = ();
    type Future = futures_util::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        futures_util::future::ready(Ok(LoggingMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct LoggingMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for LoggingMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let start_time = Instant::now();
        let method = req.method().clone();
        let path = req.path().to_string();
        let query = req.query_string().to_string();

        Box::pin(async move {
            let res = service.call(req).await;

            let duration = start_time.elapsed();
            let status = res.as_ref().map(|r| r.status()).unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
            let status_code = status.as_u16();

            // 格式化延迟
            let duration_ms = duration.as_millis();
            let duration_str = if duration_ms < 1 {
                format!("{}μs", duration.as_micros())
            } else if duration_ms < 1000 {
                format!("{}ms", duration_ms)
            } else {
                format!("{:.2}s", duration.as_secs_f64())
            };

            // 获取错误信息（如果有）
            let error_info = if res.is_err() {
                match res.as_ref().err() {
                    Some(e) => format!(" - 错误: {}", e),
                    None => String::new(),
                }
            } else {
                String::new()
            };

            // 构建完整的查询字符串
            let full_path = if query.is_empty() {
                path.clone()
            } else {
                format!("{}?{}", path, query)
            };

            // 根据状态码使用不同的颜色（在终端中）
            let status_color = if status_code < 300 {
                "\x1b[32m" // 绿色
            } else if status_code < 400 {
                "\x1b[33m" // 黄色
            } else if status_code < 500 {
                "\x1b[31m" // 红色
            } else {
                "\x1b[35m" // 紫色
            };
            let reset_color = "\x1b[0m";

            // 构建日志消息
            let log_message = format!(
                "[{}] {} {} -> {}{}{} - {}{}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                method,
                full_path,
                status_color,
                status_code,
                reset_color,
                duration_str,
                error_info
            );

            // 根据状态码选择日志级别
            if status_code >= 500 {
                eprintln!("{}", log_message);
            } else if status_code >= 400 {
                println!("{}", log_message);
            } else {
                println!("{}", log_message);
            }

            res
        })
    }
}