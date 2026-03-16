use actix_files as fs;
use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use dynamic_route_actix::{RouteTable, actix::{admin_routes, configure_dynamic_routes}, core::SimpleRoute};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 服务器状态
struct AppState {
    route_table: Arc<RouteTable>,
}

/// 首页响应
async fn index() -> impl Responder {
    match tokio::fs::read("./static/index.html").await {
        Ok(content) => actix_web::HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(content),
        Err(_) => actix_web::HttpResponse::NotFound().body("index.html not found"),
    }
}

/// 演示路由添加
#[derive(Deserialize)]
struct DemoRoute {
    name: String,
    path: String,
    body: String,
    content_type: String,
}

async fn add_demo_route(
    route: web::Json<DemoRoute>,
    data: web::Data<Arc<RouteTable>>,
) -> impl Responder {
    let route = route.into_inner();
    let simple_route = SimpleRoute::new(&route.body, &route.content_type);
    data.insert(route.path.clone(), Box::new(simple_route));

    actix_web::HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": format!("路由 '{}' 已添加到路径 '{}'", route.name, route.path),
        "path": route.path
    }))
}

/// 获取演示路由列表
async fn get_demo_routes() -> impl Responder {
    let demo_routes = vec![
        DemoRouteInfo {
            name: "首页".to_string(),
            path: "/".to_string(),
            description: "欢迎页面".to_string(),
            method: "GET".to_string(),
        },
        DemoRouteInfo {
            name: "API状态".to_string(),
            path: "/api/status".to_string(),
            description: "返回API状态信息".to_string(),
            method: "GET, POST, PUT, DELETE".to_string(),
        },
        DemoRouteInfo {
            name: "用户信息".to_string(),
            path: "/api/user".to_string(),
            description: "用户数据接口".to_string(),
            method: "GET, POST".to_string(),
        },
        DemoRouteInfo {
            name: "产品列表".to_string(),
            path: "/api/products".to_string(),
            description: "产品列表接口".to_string(),
            method: "GET".to_string(),
        },
        DemoRouteInfo {
            name: "关于我们".to_string(),
            path: "/about".to_string(),
            description: "关于页面".to_string(),
            method: "GET".to_string(),
        },
    ];

    actix_web::HttpResponse::Ok().json(demo_routes)
}

#[derive(Serialize)]
struct DemoRouteInfo {
    name: String,
    path: String,
    description: String,
    method: String,
}

/// 批量添加演示路由
async fn add_all_demo_routes(data: web::Data<Arc<RouteTable>>) -> impl Responder {
    // 清空现有路由
    data.clear();

    // 添加演示路由
    let routes = vec![
        ("/", "欢迎使用动态路由系统！<br><br>这是一个基于 Rust 和 Actix-Web 的高性能动态路由库。<br>所有路由都可以通过管理界面动态添加、修改和删除。", "text/html"),
        ("/api/status", r#"{"status":"ok","version":"1.0.0","service":"dynamic-route-service"}"#, "application/json"),
        ("/api/user", r#"{"id":1,"name":"张三","email":"zhangsan@example.com","role":"admin"}"#, "application/json"),
        ("/api/products", r#"{"products":[{"id":1,"name":"产品A","price":99.99},{"id":2,"name":"产品B","price":149.99}]}"#, "application/json"),
        ("/about", "<h1>关于我们</h1><p>这是一个动态路由系统演示项目</p>", "text/html"),
    ];

    let count = routes.len();

    for (path, body, content_type) in &routes {
        let route = SimpleRoute::new(*body, *content_type);
        data.insert(path.to_string(), Box::new(route));
    }

    actix_web::HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": "已添加 5 个演示路由",
        "count": count
    }))
}

/// 清空所有路由
async fn clear_all_routes(data: web::Data<Arc<RouteTable>>) -> impl Responder {
    data.clear();

    actix_web::HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": "已清空所有路由"
    }))
}

/// 获取路由统计信息
async fn get_stats(data: web::Data<Arc<RouteTable>>) -> impl Responder {
    let count = data.count();
    let paths = data.list_paths();

    actix_web::HttpResponse::Ok().json(serde_json::json!({
        "total_routes": count,
        "routes": paths
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    println!("=== 动态路由服务测试 ===");
    println!("正在启动服务器...");
    println!();

    // 创建路由表
    let route_table = Arc::new(RouteTable::new());

    // 添加一些初始路由
    route_table.insert(
        "/".into(),
        Box::new(SimpleRoute::new(
            "欢迎使用动态路由系统！请访问 /admin 来管理路由",
            "text/plain",
        )),
    );

    println!("服务器已启动！");
    println!("  访问地址: http://127.0.0.1:8080");
    println!("  管理界面: http://127.0.0.1:8080/");
    println!("  API文档: http://127.0.0.1:8080/admin/routes");
    println!();
    println!("功能特性:");
    println!("  ✓ 动态路由管理（添加、删除、查询）");
    println!("  ✓ 支持多种 HTTP 方法（GET, POST, PUT, DELETE, PATCH）");
    println!("  ✓ 实时路由更新");
    println!("  ✓ 路由验证");
    println!("  ✓ 高性能路由匹配");
    println!();
    println!("按 Ctrl+C 停止服务器");
    println!();

    // 启动 HTTP 服务器
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(route_table.clone()))
            // 静态文件服务
            .service(fs::Files::new("/static", "./static").show_files_listing())
            // 主页
            .route("/", web::get().to(index))
            // 演示端点
            .route("/demo/routes", web::get().to(get_demo_routes))
            .route("/demo/add", web::post().to(add_demo_route))
            .route("/demo/add-all", web::post().to(add_all_demo_routes))
            .route("/demo/clear", web::post().to(clear_all_routes))
            .route("/demo/stats", web::get().to(get_stats))
            // 管理端点
            .configure(admin_routes)
            // 动态路由（必须放在最后，因为它有通配符）
            .configure(configure_dynamic_routes)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}