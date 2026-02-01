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
mod jwt;
mod http3_server;

use actix_web::{App, HttpServer, middleware as actix_middleware, web};
use clap::Parser;
use config::{AppConfig, CliArgs};
use routes::configure_routes;
use middleware::logging::LoggingMiddleware;
use std::path::Path;

/// 检查首次运行所需的文件和目录
fn check_first_run(args: &CliArgs) {
    println!("🔍 检查运行环境...");
    
    let mut issues = Vec::new();
    let mut warnings = Vec::new();

    // 检查数据库文件
    let db_path = Path::new(&args.db_path);
    if !db_path.exists() {
        warnings.push(format!("数据库文件不存在: {} (将在首次运行时自动创建)", args.db_path));
    }

    // 检查 GeoIP 数据库
    let geoip_path = Path::new(&args.geoip_db_path);
    if !geoip_path.exists() {
        warnings.push(format!("GeoIP 数据库不存在: {} (地理位置查询将返回 'unknown')", args.geoip_db_path));
    }

    // 检查模板目录
    let templates_dir = Path::new(&args.templates_dir);
    if !templates_dir.exists() {
        warnings.push(format!("模板目录不存在: {} (将使用嵌入的模板)", args.templates_dir));
    }

    // 检查静态文件目录
    let static_dir = Path::new(&args.static_dir);
    if !static_dir.exists() {
        warnings.push(format!("静态文件目录不存在: {} (将使用嵌入的静态文件)", args.static_dir));
    }

    // 检查 TLS 证书
    if args.enable_tls {
        if let Some(ref cert) = args.tls_cert {
            if !Path::new(cert).exists() {
                issues.push(format!("TLS 证书文件不存在: {}", cert));
            }
        } else {
            issues.push("启用了 TLS 但未指定证书文件".to_string());
        }
        
        if let Some(ref key) = args.tls_key {
            if !Path::new(key).exists() {
                issues.push(format!("TLS 私钥文件不存在: {}", key));
            }
        } else {
            issues.push("启用了 TLS 但未指定私钥文件".to_string());
        }
    }

    // 输出检查结果
    if !warnings.is_empty() {
        println!("⚠️  警告:");
        for warning in &warnings {
            println!("    - {}", warning);
        }
    }

    if !issues.is_empty() {
        println!("❌ 发现以下问题:");
        for issue in &issues {
            println!("    - {}", issue);
        }
        println!("\n💡 提示:");
        println!("    - 确保 data、markdown、attachments 等目录具有写入权限");
        println!("    - 如需 GeoIP 功能，请下载 GeoLite2-City.mmdb 并放置到 data/ 目录");
        println!("    - 查看 README.md 了解更多配置信息");
        std::process::exit(1);
    } else {
        println!("✅ 环境检查通过");
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 解析命令行参数
    let mut args = CliArgs::parse();

    // 如果指定了配置文件，则加载配置文件
    if let Some(ref config_path) = args.config {
        println!("📄 加载配置文件: {}", config_path);
        match CliArgs::load_from_config_file(config_path) {
            Ok(config) => {
                args.merge_with_config(config);
                println!("✅ 配置文件加载成功");
            }
            Err(e) => {
                eprintln!("❌ 加载配置文件失败: {}", e);
                std::process::exit(1);
            }
        }
    }

    // 解析路径
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

    // 检查首次运行
    check_first_run(&args);

    // 释放嵌入的资源并创建必要的目录
    println!("📦 资源初始化...");
    let base_dir = args.get_base_dir();
    if let Err(e) = embedded::extract_embedded_resources(base_dir) {
        eprintln!("⚠️  资源释放失败: {}", e);
    }

    // 创建必要的目录
    create_directories(base_dir);

    // 初始化数据库
    println!("🗄️  初始化数据库...");
    if let Err(e) = db::init_db(&args.db_path) {
        eprintln!("❌ 数据库初始化失败: {}", e);
        return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()));
    }

    // 初始化 JWT 服务
    println!("🔐 初始化 JWT 服务...");
    let jwt_secret = jwt::init_jwt_secret(base_dir, args.jwt_secret.as_deref());
    jwt::init_jwt_service(&jwt_secret);

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
    
    // 启动 HTTP/1.1/HTTP/2 服务器
    let http_server = HttpServer::new(move || {
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
            // 支持 Gzip、Deflate、Brotli，优先使用 Brotli
            .wrap(actix_middleware::Compress::default().exclude("/img/").exclude("/music/").exclude("/attachments/"))
    })
    .bind((config.server.host.as_str(), config.server.port))?
    .run();

    // 如果启用了 TLS，同时启动 HTTP/3 服务器
    let http3_task = if args.enable_tls && args.enable_http3 {
        if let (Some(cert), Some(key)) = (args.tls_cert, args.tls_key) {
            let http3_config = http3_server::Http3ServerConfig {
                cert_path: cert,
                key_path: key,
                bind_addr: format!("{}:443", config.server.host),
                forward_addr: format!("http://{}:{}", config.server.host, config.server.port),
            };
            Some(tokio::spawn(async move {
                if let Err(e) = http3_server::start_http3_server(http3_config).await {
                    eprintln!("❌ HTTP/3 服务器启动失败: {}", e);
                }
            }))
        } else {
            None
        }
    } else {
        None
    };

    // 等待 HTTP/1.1/HTTP/2 服务器完成
    let http_result = http_server.await;

    // 如果 HTTP/3 服务器正在运行，等待它完成
    if let Some(task) = http3_task {
        task.abort();
    }

    http_result
}

/// 创建必要的目录
fn create_directories(base_dir: &Path) {
    let dirs = vec![
        "img",
        "music",
        "music/covers",
        "attachments",
        "markdown",
        "data",
    ];

    for dir in dirs {
        let dir_path = base_dir.join(dir);
        std::fs::create_dir_all(&dir_path).unwrap_or_else(|e| {
            eprintln!("创建目录 {} 失败: {}", dir_path.display(), e);
        });
    }
}