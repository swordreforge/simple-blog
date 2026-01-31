mod config;
mod routes;
mod handlers;
mod templates;
mod r#static;
mod db;
mod middleware;
mod audio_metadata;
mod music_sync;
mod geoip;
mod embedded;
mod cache;
mod view_batch;

use actix_web::{App, HttpServer, middleware as actix_middleware, web, http};
use clap::Parser;
use config::{AppConfig, CliArgs};
use routes::configure_routes;
use middleware::logging::LoggingMiddleware;

/// 已压缩的内容类型列表（不需要再次压缩）
const COMPRESSED_CONTENT_TYPES: [&str; 6] = [
    "image/",
    "video/",
    "audio/",
    "application/zip",
    "application/x-gzip",
    "application/x-rar-compressed",
];

/// 检查内容类型是否已压缩
fn is_already_compressed(content_type: &str) -> bool {
    COMPRESSED_CONTENT_TYPES.iter().any(|&prefix| content_type.starts_with(prefix))
}

/// 优化的压缩中间件
fn optimized_compress() -> actix_middleware::Condition<actix_middleware::Compress> {
    actix_middleware::Condition::new(true, actix_middleware::Compress::default())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 解析命令行参数
    let mut args = CliArgs::parse();
    args.resolve_paths();

    // 从命令行参数创建配置
    let config = AppConfig::from_cli(args.clone());

    println!("🚀 启动 RustBlog 服务器...");
    println!("📡 访问地址: http://{}:{}", config.server.host, config.server.port);
    println!("📁 模板目录: {}", config.templates.dir);
    println!("📁 静态文件目录: {}", config.static_files.dir);
    println!("📁 数据库路径: {}", args.db_path);
    println!("📁 GeoIP 数据库: {}", args.geoip_db_path);
    println!("💾 模板缓存: {}", if config.templates.cache_enabled { "启用" } else { "禁用" });
    println!("🔒 TLS: {}", if args.enable_tls { "启用" } else { "禁用" });
    println!("📊 日志级别: {}", args.log_level);

    // 释放嵌入的资源并创建必要的目录
    println!("📦 资源初始化...");
    if let Err(e) = embedded::extract_embedded_resources() {
        eprintln!("⚠️  资源释放失败: {}", e);
    }

    // 创建必要的目录
    create_directories();

    // 初始化数据库
    println!("🗄️  初始化数据库...");
    if let Err(e) = db::init_db(&args.db_path) {
        eprintln!("❌ 数据库初始化失败: {}", e);
        return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()));
    }

    // 初始化 GeoIP 数据库
    println!("🌍 加载 GeoIP 数据库...");
    if !geoip::is_database_loaded() {
        eprintln!("⚠️  警告: GeoIP 数据库未找到，地理位置查询将返回 'unknown'");
    }
    
    // 获取数据库连接池
    let db_pool = db::get_db_pool().await.map_err(|e| {
        eprintln!("❌ 获取数据库连接池失败: {}", e);
        std::io::Error::new(std::io::ErrorKind::Other, e)
    })?;
    
    // 创建 Repository 实例
    let repository = db::repositories::create_repository(db_pool.clone());
    
    // 初始化应用缓存
    println!("💾 初始化应用缓存...");
    let cache_config = cache::CacheConfig::default();
    let app_cache = std::sync::Arc::new(cache::AppCache::new(cache_config));
    
    // 初始化阅读记录批量处理器
    println!("📊 初始化阅读记录批量处理器...");
    let view_batch_config = view_batch::BatchConfig::default();
    let view_batch_processor = std::sync::Arc::new(view_batch::ViewBatchProcessor::new(
        repository.get_pool().clone(),
        view_batch_config,
    ));
    
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
            // 注入应用缓存
            .app_data(web::Data::new(app_cache.clone()))
            // 注入阅读记录批量处理器
            .app_data(web::Data::new(view_batch_processor.clone()))
            // 配置所有路由
            .configure(configure_routes)
            // 添加中间件
            .wrap(LoggingMiddleware)
            // 优化的压缩中间件（已压缩内容不会再次压缩）
            .wrap(actix_middleware::Compress::default())
    })
    .bind((config.server.host.as_str(), config.server.port))?
    .run()
    .await
}

/// 创建必要的目录
fn create_directories() {
    let dirs = vec![
        "img",
        "music",
        "music/covers",
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