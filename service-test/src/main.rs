use actix_files as fs;
use actix_web::{web, App, HttpServer, Responder};
use dynamic_route_actix::{RouteTable, actix::{admin_routes, configure_dynamic_routes}, core::SimpleRoute};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::path::Path;

/// 服务器状态
struct AppState {
    route_table: Arc<RouteTable>,
    file_route_table: Arc<RouteTable>,
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

/// 文件路由首页响应
async fn index_file() -> impl Responder {
    match tokio::fs::read("./static/index-file.html").await {
        Ok(content) => actix_web::HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(content),
        Err(_) => actix_web::HttpResponse::NotFound().body("index-file.html not found"),
    }
}
#[derive(Deserialize)]
struct DemoRoute {
    name: String,
    path: String,
    body: String,
    content_type: String,
}

/// 文件路由添加（不包含 name 字段）
#[derive(Deserialize)]
struct FileRoute {
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

// ==================== 文件路由管理接口 ====================

/// 文件路由添加
async fn add_file_route(
    route: web::Json<FileRoute>,
    data: web::Data<Arc<RouteTable>>,
) -> impl Responder {
    let route = route.into_inner();
    let simple_route = SimpleRoute::new(&route.body, &route.content_type);
    
    // 添加到内存路由表
    data.insert(route.path.clone(), Box::new(simple_route.clone()));
    
    // 保存到文件
    if let Err(e) = save_route_to_file(&route.path, &route.body, &route.content_type) {
        return actix_web::HttpResponse::InternalServerError().json(serde_json::json!({
            "status": "error",
            "message": format!("路由添加失败: {}", e)
        }));
    }

    actix_web::HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": format!("路由已添加到路径 '{}'（已保存到文件）", route.path),
        "path": route.path
    }))
}

/// 获取文件路由列表
async fn get_file_routes() -> impl Responder {
    match list_file_routes() {
        Ok(routes) => actix_web::HttpResponse::Ok().json(routes),
        Err(e) => actix_web::HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("读取文件路由失败: {}", e)
        }))
    }
}

/// 删除文件路由
async fn delete_file_route(
    path: web::Path<String>,
    data: web::Data<Arc<RouteTable>>,
) -> impl Responder {
    let path_str = path.into_inner();
    
    // 从内存中删除
    data.remove(&path_str);
    
    // 从文件中删除
    if let Err(e) = delete_route_from_file(&path_str) {
        return actix_web::HttpResponse::InternalServerError().json(serde_json::json!({
            "status": "error",
            "message": format!("路由删除失败: {}", e)
        }));
    }

    actix_web::HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": format!("路由 '{}' 已删除", path_str)
    }))
}

/// 查看文件路由详情
async fn view_file_route(
    path: web::Path<String>,
    data: web::Data<Arc<RouteTable>>,
) -> impl Responder {
    let path_str = path.into_inner();
    
    // Data<Arc<RouteTable>> 需要先解包才能使用
    let route_table = data.as_ref().as_ref();
    
    match route_table.get_arc(&path_str) {
        Some(route) => {
            let serializable = route.to_serializable();
            actix_web::HttpResponse::Ok().json(serde_json::json!({
                "path": path_str,
                "route": serializable
            }))
        }
        None => actix_web::HttpResponse::NotFound().json(serde_json::json!({
            "error": "路由不存在"
        }))
    }
}

/// 批量添加演示文件路由
async fn add_all_file_routes(data: web::Data<Arc<RouteTable>>) -> impl Responder {
    // 清空现有路由
    data.clear();
    if let Err(e) = clear_file_routes() {
        return actix_web::HttpResponse::InternalServerError().json(serde_json::json!({
            "status": "error",
            "message": format!("清空文件路由失败: {}", e)
        }));
    }

    // 添加演示路由
    let routes = vec![
        ("/", "欢迎使用文件路由系统！<br><br>这是一个基于 Rust 和 Actix-Web 的高性能动态路由库。<br>所有路由都会持久化存储到文件系统中。", "text/html"),
        ("/api/status", r#"{"status":"ok","version":"1.0.0","service":"file-route-service"}"#, "application/json"),
        ("/api/user", r#"{"id":1,"name":"张三","email":"zhangsan@example.com","role":"admin"}"#, "application/json"),
        ("/api/products", r#"{"products":[{"id":1,"name":"产品A","price":99.99},{"id":2,"name":"产品B","price":149.99}]}"#, "application/json"),
        ("/about", "<h1>关于我们</h1><p>这是一个基于文件存储的动态路由系统演示项目</p>", "text/html"),
    ];

    let count = routes.len();

    for (path, body, content_type) in &routes {
        let route = SimpleRoute::new(*body, *content_type);
        data.insert(path.to_string(), Box::new(route.clone()));
        
        // 保存到文件
        if let Err(e) = save_route_to_file(path, body, content_type) {
            return actix_web::HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": format!("保存路由 '{}' 到文件失败: {}", path, e)
            }));
        }
    }

    actix_web::HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": "已添加 5 个演示文件路由",
        "count": count
    }))
}

/// 清空所有文件路由
async fn clear_all_file_routes(data: web::Data<Arc<RouteTable>>) -> impl Responder {
    data.clear();
    
    if let Err(e) = clear_file_routes() {
        return actix_web::HttpResponse::InternalServerError().json(serde_json::json!({
            "status": "error",
            "message": format!("清空文件路由失败: {}", e)
        }));
    }

    actix_web::HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": "已清空所有文件路由"
    }))
}

// ==================== 文件操作辅助函数 ====================

/// 获取路由文件目录
fn get_routes_dir() -> std::io::Result<String> {
    let routes_dir = "./routes";
    if !Path::new(routes_dir).exists() {
        std::fs::create_dir_all(routes_dir)?;
    }
    Ok(routes_dir.to_string())
}

/// 将路径转换为安全的文件名
fn path_to_filename(path: &str) -> String {
    path.replace('/', "_")
        .replace('.', "_")
        .replace(' ', "_")
}

/// 保存路由到文件
fn save_route_to_file(path: &str, body: &str, content_type: &str) -> std::io::Result<()> {
    let routes_dir = get_routes_dir()?;
    let filename = path_to_filename(path);
    let file_path = format!("{}/{}.json", routes_dir, filename);
    
    let route_data = serde_json::json!({
        "path": path,
        "body": body,
        "content_type": content_type,
        "created_at": chrono::Utc::now().to_rfc3339()
    });
    
    std::fs::write(&file_path, serde_json::to_string_pretty(&route_data)?)?;
    Ok(())
}

/// 从文件中删除路由
fn delete_route_from_file(path: &str) -> std::io::Result<()> {
    let routes_dir = get_routes_dir()?;
    let filename = path_to_filename(path);
    let file_path = format!("{}/{}.json", routes_dir, filename);
    
    if Path::new(&file_path).exists() {
        std::fs::remove_file(&file_path)?;
    }
    Ok(())
}

/// 列出所有文件路由
fn list_file_routes() -> std::io::Result<Vec<String>> {
    let routes_dir = get_routes_dir()?;
    let mut routes = Vec::new();
    
    for entry in std::fs::read_dir(&routes_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(path_str) = data.get("path").and_then(|p| p.as_str()) {
                        routes.push(path_str.to_string());
                    }
                }
            }
        }
    }
    
    routes.sort();
    Ok(routes)
}

/// 清空所有文件路由
fn clear_file_routes() -> std::io::Result<()> {
    let routes_dir = get_routes_dir()?;
    
    for entry in std::fs::read_dir(&routes_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
            std::fs::remove_file(&path)?;
        }
    }
    
    Ok(())
}

/// 从文件加载所有路由到内存
fn load_routes_from_file(route_table: &RouteTable) -> std::io::Result<()> {
    let routes_dir = get_routes_dir()?;
    
    for entry in std::fs::read_dir(&routes_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let (Some(path_str), Some(body), Some(content_type)) = (
                        data.get("path").and_then(|p| p.as_str()),
                        data.get("body").and_then(|b| b.as_str()),
                        data.get("content_type").and_then(|c| c.as_str())
                    ) {
                        let route = SimpleRoute::new(body, content_type);
                        route_table.insert(path_str.to_string(), Box::new(route));
                    }
                }
            }
        }
    }
    
    Ok(())
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
    let file_route_table = Arc::new(RouteTable::new());

    // 添加一些初始路由到内存路由表
    route_table.insert(
        "/".into(),
        Box::new(SimpleRoute::new(
            "欢迎使用动态路由系统！请访问 /admin 来管理路由",
            "text/plain",
        )),
    );

    // 从文件加载路由到文件路由表
    if let Err(e) = load_routes_from_file(&file_route_table) {
        eprintln!("警告: 从文件加载路由失败: {}", e);
    }

    // 如果文件路由表为空，添加默认路由
    if file_route_table.count() == 0 {
        file_route_table.insert(
            "/".into(),
            Box::new(SimpleRoute::new(
                "欢迎使用文件路由系统！请访问 /file 来管理路由",
                "text/plain",
            )),
        );
    }

    println!("服务器已启动！");
    println!("  访问地址: http://127.0.0.1:8080");
    println!("  管理界面（内存路由）: http://127.0.0.1:8080/");
    println!("  管理界面（文件路由）: http://127.0.0.1:8080/file");
    println!("  API文档（内存路由）: http://127.0.0.1:8080/admin/routes");
    println!("  API文档（文件路由）: http://127.0.0.1:8080/admin-file/routes");
    println!();
    println!("功能特性:");
    println!("  ✓ 动态路由管理（添加、删除、查询）");
    println!("  ✓ 支持多种 HTTP 方法（GET, POST, PUT, DELETE, PATCH）");
    println!("  ✓ 实时路由更新");
    println!("  ✓ 路由验证");
    println!("  ✓ 高性能路由匹配");
    println!("  ✓ 文件持久化存储");
    println!();
    println!("按 Ctrl+C 停止服务器");
    println!();

    // 启动 HTTP 服务器
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(route_table.clone()))
            .app_data(web::Data::new(file_route_table.clone()))
            // 静态文件服务
            .service(fs::Files::new("/static", "./static").show_files_listing())
            // 主页
            .route("/", web::get().to(index))
            .route("/file", web::get().to(index_file))
            // 演示端点（内存路由）
            .route("/demo/routes", web::get().to(get_demo_routes))
            .route("/demo/add", web::post().to(add_demo_route))
            .route("/demo/add-all", web::post().to(add_all_demo_routes))
            .route("/demo/clear", web::post().to(clear_all_routes))
            .route("/demo/stats", web::get().to(get_stats))
            // 演示端点（文件路由）
            .route("/demo-file/routes", web::get().to(get_demo_routes))
            .route("/demo-file/add", web::post().to(add_file_route))
            .route("/demo-file/add-all", web::post().to(add_all_file_routes))
            .route("/demo-file/clear", web::post().to(clear_all_file_routes))
            .route("/demo-file/stats", web::get().to(get_stats))
            // 管理端点（内存路由）
            .configure(admin_routes)
            // 管理端点（文件路由）
            .service(
                web::scope("/admin-file")
                    .route("/routes", web::get().to(get_file_routes))
                    .route("/routes", web::post().to(add_file_route))
                    .route("/routes{tail:.*}", web::get().to(view_file_route))
                    .route("/routes{tail:.*}", web::delete().to(delete_file_route))
            )
            // 动态路由（内存路由，必须放在最后，因为它有通配符）
            .configure(configure_dynamic_routes)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}