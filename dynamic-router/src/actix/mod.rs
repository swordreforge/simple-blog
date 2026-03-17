//! Actix-Web 集成模块
//!
//! 提供与 Actix-Web 框架的深度集成，包括万能路由处理器和管理端点。

pub mod middleware;

use crate::core::{RouteTable, SerializableRoute, SimpleRoute};
use std::sync::Arc;
use actix_web::{delete, get, post, web, web::Path, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};

pub use middleware::{AuthMiddleware, RateLimiter, RequestLogger};

/// 万能路由处理器
///
/// 根据请求路径从路由表中查找对应的路由处理器并执行。
/// 如果路由不存在，返回 404 Not Found。
///
/// # Arguments
///
/// * `req` - HTTP 请求对象
/// * `table` - 路由表，通过 Actix 的依赖注入系统传递
///
/// # Returns
///
/// 返回路由处理器的响应，或 404 错误
///
/// # Examples
///
/// ```no_run
/// use actix_web::{App, web, HttpServer};
/// use dynamic_route_actix::{RouteTable, actix::universal_handler};
///
/// #[actix_web::main]
/// async fn main() -> std::io::Result<()> {
///     let table = RouteTable::new();
///
///     HttpServer::new(move || {
///         App::new()
///             .app_data(web::Data::new(table.clone()))
///             .route("/{tail:.*}", web::get().to(universal_handler))
///             .route("/{tail:.*}", web::post().to(universal_handler))
///     })
///     .bind("127.0.0.1:8080")?
///     .run()
///     .await
/// }
/// ```
pub async fn universal_handler(req: HttpRequest, table: web::Data<Arc<RouteTable>>) -> HttpResponse {
    // 获取请求的路径
    let path = req.path();

    // 从路由表中查找路由
    if let Some(route) = table.get_clone(path) {
        // 调用路由处理器
        route.handle(&req).await
    } else {
        // 路由不存在，返回 404
        HttpResponse::NotFound().body(format!("Route not found: {}", path))
    }
}

/// 添加路由的请求数据
#[derive(Debug, Serialize, Deserialize)]
pub struct AddRouteRequest {
    /// 路由路径
    pub path: String,
    /// 响应体内容
    pub body: String,
    /// Content-Type
    pub content_type: String,
}

/// 添加路由
///
/// POST /admin/routes
///
/// 向路由表中添加一个新的路由。
///
/// # Arguments
///
/// * `req` - 添加路由的请求
/// * `table` - 路由表
///
/// # Returns
///
/// 返回 201 Created 表示成功，或 400 Bad Request 表示参数错误
///
/// # Examples
///
/// ```no_run
/// use actix_web::{App, web, HttpServer};
/// use dynamic_route_actix::{RouteTable, actix::admin_routes};
///
/// #[actix_web::main]
/// async fn main() -> std::io::Result<()> {
///     let table = RouteTable::new();
///
///     HttpServer::new(move || {
///         App::new()
///             .app_data(web::Data::new(table.clone()))
///             .configure(admin_routes)
///     })
///     .bind("127.0.0.1:8080")?
///     .run()
///     .await
/// }
/// ```
#[post("/admin/routes")]
pub async fn add_route(
    req: web::Json<AddRouteRequest>,
    table: web::Data<Arc<RouteTable>>,
) -> HttpResponse {
    let AddRouteRequest {
        path,
        body,
        content_type,
    } = req.into_inner();

    // 验证路径格式
    if path.is_empty() || !path.starts_with('/') {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Path must be non-empty and start with '/'"
        }));
    }

    // 创建 SimpleRoute 并插入路由表
    let route = SimpleRoute::new(body, content_type);
    table.insert(path.clone(), Box::new(route));

    HttpResponse::Created().json(serde_json::json!({
        "message": "Route added successfully",
        "path": path
    }))
}

/// 删除路由
///
/// DELETE /admin/routes/{path}
///
/// 从路由表中删除指定路径的路由。
///
/// # Arguments
///
/// * `path` - 要删除的路由路径
/// * `table` - 路由表
///
/// # Returns
///
/// 返回 200 OK 表示成功，或 404 Not Found 表示路由不存在
#[delete("/admin/routes/{path:.*}")]
pub async fn delete_route(path: Path<String>, table: web::Data<Arc<RouteTable>>) -> HttpResponse {
    let mut route_path = path.into_inner();
    
    // 确保路径只有一个前导斜杠，避免双斜杠问题
    while route_path.starts_with("//") {
        route_path = route_path.replacen("//", "/", 1);
    }
    
    // 如果路径不以 / 开头，添加一个
    if !route_path.starts_with('/') {
        route_path = format!("/{}", route_path);
    }

    if table.remove(&route_path) {
        HttpResponse::Ok().json(serde_json::json!({
            "message": "Route deleted successfully",
            "path": route_path
        }))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({
            "error": "Route not found",
            "path": route_path
        }))
    }
}

/// 列出所有路由
///
/// GET /admin/routes
///
/// 返回路由表中所有路由的路径列表。
///
/// # Arguments
///
/// * `table` - 路由表
///
/// # Returns
///
/// 返回包含所有路由路径的 JSON 数组
#[get("/admin/routes")]
pub async fn list_routes(table: web::Data<Arc<RouteTable>>) -> HttpResponse {
    let paths = table.list_paths();

    HttpResponse::Ok().json(paths)
}

/// 路由信息响应
#[derive(Debug, Serialize, Deserialize)]
pub struct RouteInfo {
    /// 路由路径
    pub path: String,
    /// 路由详情
    pub route: SerializableRoute,
}

/// 获取路由详情
///
/// GET /admin/routes/{path}
///
/// 返回指定路径的路由详情。
///
/// # Arguments
///
/// * `path` - 路由路径
/// * `table` - 路由表
///
/// # Returns
///
/// 返回路由详情，或 404 Not Found
#[get("/admin/routes/{path:.*}")]
pub async fn get_route(path: Path<String>, table: web::Data<Arc<RouteTable>>) -> HttpResponse {
    let mut route_path = path.into_inner();
    
    // 确保路径只有一个前导斜杠，避免双斜杠问题
    while route_path.starts_with("//") {
        route_path = route_path.replacen("//", "/", 1);
    }
    
    // 如果路径不以 / 开头，添加一个
    if !route_path.starts_with('/') {
        route_path = format!("/{}", route_path);
    }

    if let Some(serializable) = table.get_with(&route_path, |route| route.to_serializable()) {
        HttpResponse::Ok().json(RouteInfo {
            path: route_path,
            route: serializable,
        })
    } else {
        HttpResponse::NotFound().json(serde_json::json!({
            "error": "Route not found",
            "path": route_path
        }))
    }
}

/// 配置管理端点
///
/// 创建一个 Actix 服务配置，包含所有管理端点。
///
/// # Returns
///
/// 返回 Actix 服务配置
///
/// # Examples
///
/// ```no_run
/// use actix_web::{App, HttpServer, web};
/// use dynamic_route_actix::{RouteTable, actix::admin_routes};
///
/// #[actix_web::main]
/// async fn main() -> std::io::Result<()> {
///     let table = RouteTable::new();
///
///     HttpServer::new(move || {
///         App::new()
///             .app_data(web::Data::new(table.clone()))
///             .configure(admin_routes)
///     })
///     .bind("127.0.0.1:8080")?
///     .run()
///     .await
/// }
/// ```
pub fn admin_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(add_route)
        .service(delete_route)
        .service(list_routes)
        .service(get_route);
}

/// 配置动态路由
///
/// 为 App 配置动态路由处理器，支持所有 HTTP 方法。
///
/// # Arguments
///
/// * `table` - 路由表
///
/// # Examples
///
/// ```no_run
/// use actix_web::{App, HttpServer, web};
/// use dynamic_route_actix::{RouteTable, actix::configure_dynamic_routes, actix::admin_routes};
///
/// #[actix_web::main]
/// async fn main() -> std::io::Result<()> {
///     let table = RouteTable::new();
///
///     HttpServer::new(move || {
///         App::new()
///             .app_data(web::Data::new(table.clone()))
///             .configure(admin_routes)
///             .configure(configure_dynamic_routes)
///     })
///     .bind("127.0.0.1:8080")?
///     .run()
///     .await
/// }
/// ```
pub fn configure_dynamic_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/{tail:.*}", web::get().to(universal_handler))
        .route("/{tail:.*}", web::post().to(universal_handler))
        .route("/{tail:.*}", web::put().to(universal_handler))
        .route("/{tail:.*}", web::delete().to(universal_handler))
        .route("/{tail:.*}", web::patch().to(universal_handler));
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use actix_web::{http::StatusCode, test, web, App};

    #[actix_web::test]
    async fn test_universal_handler() {
        let table = RouteTable::new();
        table.insert(
            "/hello".into(),
            Box::new(SimpleRoute::new("world", "text/plain")),
        );

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(table.clone()))
                .route("/{tail:.*}", web::get().to(universal_handler)),
        )
        .await;

        let req = test::TestRequest::get().uri("/hello").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let body = test::read_body(resp).await;
        assert_eq!(body, "world");
    }

    #[actix_web::test]
    async fn test_universal_handler_not_found() {
        let table = RouteTable::new();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(table.clone()))
                .route("/{tail:.*}", web::get().to(universal_handler)),
        )
        .await;

        let req = test::TestRequest::get().uri("/nonexistent").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        let body = test::read_body(resp).await;
        assert!(String::from_utf8_lossy(&body).contains("Route not found"));
    }

    #[actix_web::test]
    async fn test_admin_add_route() {
        let table = RouteTable::new();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(table.clone()))
                .configure(admin_routes),
        )
        .await;

        let payload = r#"{"path": "/test", "body": "content", "content_type": "text/plain"}"#;
        let req = test::TestRequest::post()
            .uri("/admin/routes")
            .insert_header(("content-type", "application/json"))
            .set_payload(payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);

        // 验证路由已添加
        assert!(table.contains("/test"));
    }

    #[actix_web::test]
    async fn test_admin_add_route_invalid_path() {
        let table = RouteTable::new();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(table.clone()))
                .configure(admin_routes),
        )
        .await;

        // 测试空路径
        let payload = r#"{"path": "", "body": "content", "content_type": "text/plain"}"#;
        let req = test::TestRequest::post()
            .uri("/admin/routes")
            .insert_header(("content-type", "application/json"))
            .set_payload(payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        // 测试不以 / 开头的路径
        let payload = r#"{"path": "invalid", "body": "content", "content_type": "text/plain"}"#;
        let req = test::TestRequest::post()
            .uri("/admin/routes")
            .insert_header(("content-type", "application/json"))
            .set_payload(payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    async fn test_admin_delete_route() {
        let table = RouteTable::new();
        table.insert(
            "/test".into(),
            Box::new(SimpleRoute::new("content", "text/plain")),
        );

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(table.clone()))
                .configure(admin_routes),
        )
        .await;

        let req = test::TestRequest::delete()
            .uri("/admin/routes/test")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        // 验证路由已删除
        assert!(!table.contains("/test"));
    }

    #[actix_web::test]
    async fn test_admin_delete_route_not_found() {
        let table = RouteTable::new();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(table.clone()))
                .configure(admin_routes),
        )
        .await;

        let req = test::TestRequest::delete()
            .uri("/admin/routes/nonexistent")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    async fn test_admin_list_routes() {
        let table = RouteTable::new();
        table.insert(
            "/route1".into(),
            Box::new(SimpleRoute::new("body1", "text/plain")),
        );
        table.insert(
            "/route2".into(),
            Box::new(SimpleRoute::new("body2", "text/plain")),
        );

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(table.clone()))
                .configure(admin_routes),
        )
        .await;

        let req = test::TestRequest::get().uri("/admin/routes").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let body = test::read_body(resp).await;
        let routes: Vec<String> = serde_json::from_slice(&body).unwrap();
        assert_eq!(routes.len(), 2);
        assert!(routes.contains(&"/route1".to_string()));
        assert!(routes.contains(&"/route2".to_string()));
    }

    #[actix_web::test]
    async fn test_admin_get_route() {
        let table = RouteTable::new();
        table.insert(
            "/test".into(),
            Box::new(SimpleRoute::new("test body", "text/plain")),
        );

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(table.clone()))
                .configure(admin_routes),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/admin/routes/test")
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let body = test::read_body(resp).await;
        let route_info: RouteInfo = serde_json::from_slice(&body).unwrap();
        assert_eq!(route_info.path, "/test");
        assert_eq!(route_info.route.body, "test body");
    }

    #[actix_web::test]
    async fn test_admin_get_route_not_found() {
        let table = RouteTable::new();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(table.clone()))
                .configure(admin_routes),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/admin/routes/nonexistent")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    async fn test_dynamic_route_app_trait() {
        let table = RouteTable::new();
        table.insert(
            "/hello".into(),
            Box::new(SimpleRoute::new("world", "text/plain")),
        );

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(table.clone()))
                .configure(admin_routes)
                .configure(configure_dynamic_routes),
        )
        .await;

        // 测试动态路由
        let req = test::TestRequest::get().uri("/hello").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        // 测试管理端点
        let req = test::TestRequest::get().uri("/admin/routes").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_multiple_http_methods() {
        let table = RouteTable::new();
        table.insert(
            "/api".into(),
            Box::new(SimpleRoute::new("API response", "application/json")),
        );

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(table.clone()))
                .configure(admin_routes)
                .configure(configure_dynamic_routes),
        )
        .await;

        // 测试 GET
        let req = test::TestRequest::get().uri("/api").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        // 测试 POST
        let req = test::TestRequest::post().uri("/api").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        // 测试 PUT
        let req = test::TestRequest::put().uri("/api").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        // 测试 DELETE
        let req = test::TestRequest::delete().uri("/api").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
