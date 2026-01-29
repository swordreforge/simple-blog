mod config;
mod routes;
mod handlers;
mod templates;
mod r#static;
mod db;
mod middleware;
mod audio_metadata;
mod music_sync;

use actix_web::{App, HttpServer, middleware as actix_middleware, web};
use config::AppConfig;
use routes::configure_routes;
use middleware::logging::LoggingMiddleware;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 加载配置
    let config = AppConfig::default();
    
    println!("🚀 启动 RustBlog 服务器...");
    println!("📡 访问地址: http://{}:{}", config.server.host, config.server.port);
    println!("📁 模板目录: {}", config.templates.dir);
    println!("📁 静态文件目录: {}", config.static_files.dir);
    println!("💾 模板缓存: {}", if config.templates.cache_enabled { "启用" } else { "禁用" });
    
    // 创建必要的目录
    create_directories();
    
    // 初始化数据库
    println!("🗄️  初始化数据库...");
    if let Err(e) = db::init_db("data/blog.db") {
        eprintln!("❌ 数据库初始化失败: {}", e);
        return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()));
    }
    
    // 获取数据库连接池
    let db_pool = db::get_db_pool().await.map_err(|e| {
        eprintln!("❌ 获取数据库连接池失败: {}", e);
        std::io::Error::new(std::io::ErrorKind::Other, e)
    })?;
    
    // 创建 Repository 实例
    let repository = db::repositories::create_repository(db_pool.clone());
    
    // 同步音乐文件到数据库
    println!("🎵 同步音乐文件...");
    let music_sync_service = music_sync::MusicSyncService::new(repository.clone());
    match music_sync_service.sync_music_files_to_db().await {
        Ok(result) => {
            println!("✅ {}", result.message);
        }
        Err(e) => {
            eprintln!("⚠️  音乐同步失败: {}", e);
        }
    }
    
    // 同步 markdown 文件到数据库
    println!("📝 同步 Markdown 文件...");
    let passage_repo = db::repositories::PassageRepository::new(repository.get_pool().clone());
    match handlers::api_handlers::sync::sync_directory_internal(&passage_repo).await {
        Ok(result) => {
            println!("✅ {}", result.message);
        }
        Err(e) => {
            eprintln!("⚠️  文章同步失败: {}", e);
        }
    }
    
    HttpServer::new(move || {
        App::new()
            // 注入数据库连接池
            .app_data(web::Data::new(repository.clone()))
            // 配置所有路由
            .configure(configure_routes)
            // 添加中间件
            .wrap(LoggingMiddleware)
            .wrap(actix_middleware::Compress::default())
            .wrap(actix_middleware::Condition::new(
                config.static_files.cache_max_age > 0,
                actix_middleware::DefaultHeaders::new().add(("Cache-Control", 
                    format!("public, max-age={}", config.static_files.cache_max_age)))
            ))
    })
    .bind((config.server.host.as_str(), config.server.port))?
    .run()
    .await
}

/// 创建必要的目录
fn create_directories() {
    let dirs = vec![
        "templates",
        "templates/css",
        "templates/js",
        "img",
        "music",
        "attachments",
        "markdown",
        "data",
    ];
    
    for dir in dirs {
        std::fs::create_dir_all(dir).unwrap_or_else(|e| {
            eprintln!("创建目录 {} 失败: {}", dir, e);
        });
    }
}