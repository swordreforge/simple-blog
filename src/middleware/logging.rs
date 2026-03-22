use actix_web::{
    Error,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use futures_util::future::LocalBoxFuture;
use std::sync::Arc;
use std::time::Instant;

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

    #[inline]
    fn new_transform(&self, service: S) -> Self::Future {
        futures_util::future::ready(Ok(LoggingMiddlewareService {
            service: Arc::new(service),
        }))
    }
}

pub struct LoggingMiddlewareService<S> {
    service: Arc<S>,
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
        let service = Arc::clone(&self.service);
        let start_time = Instant::now();
        let method = req.method().clone();
        let path = req.path().to_string();
        let query = req.query_string().to_string();

        Box::pin(async move {
            let res = service.call(req).await;

            let duration = start_time.elapsed();
            let status = res
                .as_ref()
                .map(|r| r.status())
                .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
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
                String::from(&path)
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
            } else {
                println!("{}", log_message);
            }

            res
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{
        dev::ServiceResponse,
        http::{header, StatusCode},
        test, web, App, HttpResponse,
    };
    use actix_web::body::MessageBody;

    #[actix_web::test]
    async fn test_logging_middleware_basic() {
        // 创建一个简单的应用来测试日志中间件
        let app = test::init_service(
            App::new()
                .wrap(LoggingMiddleware)
                .route("/test", web::get().to(|| async { HttpResponse::Ok().body("test") })),
        )
        .await;

        // 发送请求
        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;

        // 验证响应
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.status().as_u16(), 200);
    }

    #[actix_web::test]
    async fn test_logging_middleware_with_query() {
        let app = test::init_service(
            App::new()
                .wrap(LoggingMiddleware)
                .route("/test", web::get().to(|| async { HttpResponse::Ok().body("test") })),
        )
        .await;

        // 发送带查询参数的请求
        let req = test::TestRequest::get()
            .uri("/test?param=value&another=123")
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_logging_middleware_different_methods() {
        let app = test::init_service(
            App::new()
                .wrap(LoggingMiddleware)
                .route("/test", web::get().to(|| async { HttpResponse::Ok().body("get") }))
                .route("/test", web::post().to(|| async { HttpResponse::Ok().body("post") }))
                .route("/test", web::put().to(|| async { HttpResponse::Ok().body("put") }))
                .route("/test", web::delete().to(|| async { HttpResponse::Ok().body("delete") })),
        )
        .await;

        // 测试不同的HTTP方法
        let methods = vec![
            test::TestRequest::get().uri("/test"),
            test::TestRequest::post().uri("/test"),
            test::TestRequest::put().uri("/test"),
            test::TestRequest::delete().uri("/test"),
        ];

        for req in methods {
            let req = req.to_request();
            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), StatusCode::OK);
        }
    }

    #[actix_web::test]
    async fn test_logging_middleware_error_status() {
        let app = test::init_service(
            App::new()
                .wrap(LoggingMiddleware)
                .route("/notfound", web::get().to(|| async {
                    HttpResponse::NotFound().body("not found")
                }))
                .route("/error", web::get().to(|| async {
                    HttpResponse::InternalServerError().body("error")
                })),
        )
        .await;

        // 测试404错误
        let req = test::TestRequest::get().uri("/notfound").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        assert_eq!(resp.status().as_u16(), 404);

        // 测试500错误
        let req = test::TestRequest::get().uri("/error").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(resp.status().as_u16(), 500);
    }

    #[actix_web::test]
    async fn test_logging_middleware_multiple_requests() {
        let app = test::init_service(
            App::new()
                .wrap(LoggingMiddleware)
                .route("/test", web::get().to(|| async { HttpResponse::Ok().body("test") })),
        )
        .await;

        // 发送多个请求
        for i in 0..10 {
            let req = test::TestRequest::get()
                .uri(&format!("/test?id={}", i))
                .to_request();
            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), StatusCode::OK);
        }
    }

    #[actix_web::test]
    async fn test_logging_middleware_with_headers() {
        let app = test::init_service(
            App::new()
                .wrap(LoggingMiddleware)
                .route("/test", web::get().to(|| async { HttpResponse::Ok().body("test") })),
        )
        .await;

        // 发送带头部的请求
        let req = test::TestRequest::get()
            .uri("/test")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .insert_header((header::USER_AGENT, "test-agent"))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_logging_middleware_path_variants() {
        let app = test::init_service(
            App::new()
                .wrap(LoggingMiddleware)
                .route("/", web::get().to(|| async { HttpResponse::Ok().body("root") }))
                .route("/api/v1", web::get().to(|| async { HttpResponse::Ok().body("api") }))
                .route("/path/with/slashes", web::get().to(|| async {
                    HttpResponse::Ok().body("slashes")
                })),
        )
        .await;

        // 测试不同路径
        let paths = ["/", "/api/v1", "/path/with/slashes"];

        for path in paths {
            let req = test::TestRequest::get().uri(path).to_request();
            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), StatusCode::OK);
        }
    }

    #[actix_web::test]
    async fn test_logging_middleware_response_body() {
        let app = test::init_service(
            App::new()
                .wrap(LoggingMiddleware)
                .route("/test", web::get().to(|| async {
                    HttpResponse::Ok()
                        .content_type("text/plain")
                        .body("test response")
                })),
        )
        .await;

        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
        
        // 验证响应体
        let body = test::read_body(resp).await;
        assert_eq!(body, "test response");
    }

    #[actix_web::test]
    async fn test_logging_middleware_timing() {
        let app = test::init_service(
            App::new()
                .wrap(LoggingMiddleware)
                .route("/fast", web::get().to(|| async {
                    HttpResponse::Ok().body("fast")
                }))
                .route("/slow", web::get().to(|| async {
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                    HttpResponse::Ok().body("slow")
                })),
        )
        .await;

        // 测试快速请求
        let start = std::time::Instant::now();
        let req = test::TestRequest::get().uri("/fast").to_request();
        let _resp = test::call_service(&app, req).await;
        let fast_duration = start.elapsed();
        
        // 测试慢速请求
        let start = std::time::Instant::now();
        let req = test::TestRequest::get().uri("/slow").to_request();
        let _resp = test::call_service(&app, req).await;
        let slow_duration = start.elapsed();

        // 慢速请求应该比快速请求慢
        assert!(slow_duration > fast_duration);
    }

    #[actix_web::test]
    async fn test_logging_middleware_concurrent_requests() {
        let app = test::init_service(
            App::new()
                .wrap(LoggingMiddleware)
                .route("/test", web::get().to(|| async {
                    HttpResponse::Ok().body("test")
                })),
        )
        .await;

        // 并发发送多个请求
        let handles: Vec<_> = (0..10)
            .map(|i| {
                let app = app.clone();
                tokio::spawn(async move {
                    let req = test::TestRequest::get()
                        .uri(&format!("/test?id={}", i))
                        .to_request();
                    test::call_service(&app, req).await
                })
            })
            .collect();

        // 等待所有请求完成
        for handle in handles {
            let resp = handle.await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
        }
    }

    #[actix_web::test]
    async fn test_logging_middleware_empty_query() {
        let app = test::init_service(
            App::new()
                .wrap(LoggingMiddleware)
                .route("/test", web::get().to(|| async {
                    HttpResponse::Ok().body("test")
                })),
        )
        .await;

        // 测试空查询字符串
        let req = test::TestRequest::get().uri("/test?").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_logging_middleware_special_characters() {
        let app = test::init_service(
            App::new()
                .wrap(LoggingMiddleware)
                .route("/test", web::get().to(|| async {
                    HttpResponse::Ok().body("test")
                })),
        )
        .await;

        // 测试包含特殊字符的查询参数
        let req = test::TestRequest::get()
            .uri("/test?name=John%20Doe&email=test%40example.com")
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_logging_middleware_redirect_status() {
        let app = test::init_service(
            App::new()
                .wrap(LoggingMiddleware)
                .route("/redirect", web::get().to(|| async {
                    HttpResponse::PermanentRedirect()
                        .insert_header((header::LOCATION, "/new-location"))
                        .finish()
                })),
        )
        .await;

        let req = test::TestRequest::get().uri("/redirect").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::PERMANENT_REDIRECT);
        assert_eq!(resp.status().as_u16(), 308);
    }
}
