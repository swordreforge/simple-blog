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
mod id_generator;
mod profiling;
mod json_adapter;
mod lock_monitor;
mod audit;
mod error;
mod logging;
mod services;
mod app_state;
mod utils;

use actix_web::{App, HttpServer, middleware as actix_middleware, web, http::KeepAlive};
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

    // 初始化日志系统
    let log_dir = Path::new(&args.db_path).parent().map(|p| p.join("logs"));
    logging::init_logging(log_dir.as_deref(), &args.log_level);

    // 如果指定了配置文件，则加载配置文件
    if let Some(ref config_path) = args.config {
        println!("📄 加载配置文件: {}", config_path);
        match CliArgs::load_from_config_file(config_path) {
            Ok(config) => {
                args.merge_with_config(config);
                println!("✅ 配置文件加载成功");
            }
            Err(e) => {
                tracing::error!("加载配置文件失败: {}", e);
                std::process::exit(1);
            }
        }
    }

    // 解析路径
    args.resolve_paths();

    // 验证配置
    println!("🔍 验证配置...");
    let validation_result = args.validate();
    
    // 输出验证警告
    if !validation_result.warnings.is_empty() {
        println!("⚠️  配置警告:");
        for warning in &validation_result.warnings {
            println!("    - {}", warning);
        }
    }
    
    // 检查验证错误
    if !validation_result.is_valid() {
        println!("❌ 配置验证失败:");
        for error in &validation_result.errors {
            println!("    - {}", error);
        }
        println!("\n💡 请修复上述配置错误后重试");
        std::process::exit(1);
    }
    
    println!("✅ 配置验证通过");

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
    println!("💾 应用缓存: {}", if args.enable_cache { "启用" } else { "禁用" });

    // 显示服务器性能配置
    let cpu_count = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    let workers = config.server.workers.unwrap_or(cpu_count);
    let keep_alive = config.server.keep_alive.unwrap_or(75);
    let max_connections = config.server.max_connections.unwrap_or(10000);
    println!("⚡ Worker 线程数: {} (CPU 核心: {})", workers, cpu_count);
    println!("🔄 Keep-alive 超时: {} 秒", keep_alive);
    println!("🔗 最大并发连接: {}", max_connections);

    // 检查首次运行
    check_first_run(&args);

    // 初始化性能分析
    #[cfg(feature = "profiling")]
    let mut profiling_manager = profiling::ProfilingManager::new(std::path::PathBuf::from("./profiling"));
    if args.enable_profiling {
        #[cfg(feature = "profiling")]
        {
            if let Err(e) = profiling_manager.enable() {
                tracing::warn!("启用性能分析失败: {}", e);
            }
        }
        #[cfg(not(feature = "profiling"))]
        {
            tracing::warn!("性能分析功能未启用，请使用 --features profiling 编译");
        }
    }

    // 释放嵌入的资源并创建必要的目录
    println!("📦 资源初始化...");
    let base_dir = args.get_base_dir();
    if let Err(e) = embedded::extract_embedded_resources(base_dir) {
        tracing::warn!("资源释放失败: {}", e);
    }

    // 创建必要的目录
    create_directories(base_dir);

    // 初始化数据库
    println!("🗄️  初始化数据库...");
    if let Err(e) = db::init_db(&args.db_path) {
        tracing::error!("数据库初始化失败: {}", e);
        return Err(std::io::Error::other(e.to_string()));
    }

    // 初始化 JWT 服务
    println!("🔐 初始化 JWT 服务...");
    let jwt_secret = jwt::init_jwt_secret(base_dir, args.jwt_secret.as_deref());
    let _ = jwt::init_jwt_service(&jwt_secret);

    // 初始化 GeoIP 数据库
    println!("🌍 加载 GeoIP 数据库...");
    if !geoip::is_database_loaded() {
        tracing::warn!("GeoIP 数据库未找到，地理位置查询将返回 'unknown'");
    }
    
    // 获取数据库连接池
    let db_pool = db::get_db_pool().await.map_err(|e| {
        eprintln!("❌ 获取数据库连接池失败: {}", e);
        std::io::Error::other(e)
    })?;
    
    // 创建 Repository 实例
    let repository = db::repositories::create_repository(db_pool.clone());
    
    // 初始化应用缓存
    println!("💾 初始化应用缓存...");
    let cache_config = cache::CacheConfig::new(args.cache_ttl, args.cache_fallback);
    let mut app_cache = cache::AppCache::new(cache_config.clone());
    
    // 如果启用了缓存，初始化缓存管理器
    if args.enable_cache {
        println!("🚀 缓存模式: {}", args.cache_backend);
        let valkey_url = args.valkey_url.as_deref();
        
        if let Err(e) = app_cache.init_manager(&args.cache_backend, valkey_url, cache_config.clone()).await {
            eprintln!("⚠️  缓存初始化失败: {}", e);
            println!("📊 缓存功能将不可用");
        } else {
            if let Some(manager) = app_cache.manager() {
                let stats = manager.get_stats();
                println!("✅ 缓存已启用 (TTL: {}秒)", stats.default_ttl);
                if stats.has_fallback && stats.fallback_enabled {
                    println!("🔄 自动降级已启用");
                }
            }
            
            // 如果设置了清除缓存，清除所有缓存
            if args.clear_cache {
                println!("🧹 正在清除旧缓存...");
                app_cache.clear_all().await;
            }
        }
    } else {
        println!("⚠️  缓存未启用");
    }
    
    let app_cache = std::sync::Arc::new(app_cache);
    
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
    // 创建应用状态（依赖注入容器）
    let app_state = app_state::AppState::new(
        repository.clone(),
        app_cache.clone(),
        view_batch_processor.clone(),
    );

    let mut server = HttpServer::new(move || {
        App::new()
            // 注入应用状态（依赖注入容器）
            .app_data(web::Data::new(app_state.clone()))
            // 配置所有路由
            .configure(configure_routes)
            // 添加中间件
            .wrap(LoggingMiddleware)
            // 优化的压缩中间件（已压缩内容不会再次压缩）
            // 支持 Gzip、Deflate、Brotli，优先使用 Brotli
            .wrap(actix_middleware::Compress::default())
            // 默认服务：处理所有未匹配的路由，显示友好的状态码页面
            .default_service(web::route().to(handlers::page_handlers::handle_default_status))
    });

    // 应用 keep-alive 和性能优化配置
    if let Some(workers) = config.server.workers {
        server = server.workers(workers);
    }

    if let Some(keep_alive) = config.server.keep_alive {
        // 使用 KeepAlive::Timeout 设置具体的超时时间（秒）
        server = server.keep_alive(KeepAlive::Timeout(std::time::Duration::from_secs(keep_alive)));
    }

    if let Some(max_connections) = config.server.max_connections {
        server = server.max_connections(max_connections);
    }

    if let Some(max_connection_rate) = config.server.max_connection_rate {
        server = server.max_connection_rate(max_connection_rate);
    }

    // 绑定地址并运行
    let server_result = server
        .bind((config.server.host.as_str(), config.server.port))?
        .run()
        .await;

    // 程序结束时生成性能分析报告
    #[cfg(feature = "profiling")]
    {
        if args.enable_profiling {
            if let Err(e) = profiling_manager.disable_and_generate_report() {
                eprintln!("⚠️  生成性能分析报告失败: {}", e);
            }
        }
    }

    server_result
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