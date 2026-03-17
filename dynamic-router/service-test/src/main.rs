mod models;
mod services;
mod handlers;

use actix_files as fs;
use actix_web::{web, App, HttpServer};
use dynamic_route_actix::{RouteTable, actix::{admin_routes, configure_dynamic_routes}, core::SimpleRoute};
use std::sync::Arc;
use models::AppState;
use services::load_routes_from_file;
use handlers::{index, index_file, get_demo_routes, add_demo_route, add_all_demo_routes, clear_all_routes, get_stats, add_file_route, get_file_routes, delete_file_route, view_file_route, add_all_file_routes, clear_all_file_routes};

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