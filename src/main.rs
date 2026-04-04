mod auth;
mod db;
mod embedded;
mod handlers;
mod image;
mod init;
mod models;
mod routes;

use tokio::fs;

use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

use crate::auth::AuthManager;
use crate::db::Database;
use crate::handlers::AppState;
use crate::init::initialize_database;
use crate::routes::create_router;

/// Parse command line arguments
fn parse_args() -> (String, u16, usize) {
    let mut host = String::from("127.0.0.1");
    let mut port = 3000;
    let mut max_size = 300 * 1024; // Default 300KB
    let args: Vec<String> = std::env::args().collect();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            "--host" => {
                if i + 1 < args.len() {
                    host = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --host requires a value\n");
                    print_help();
                    std::process::exit(1);
                }
            }
            "-p" | "--port" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u16>() {
                        Ok(p) => port = p,
                        Err(_) => {
                            eprintln!("Error: Invalid port number '{}'\n", args[i + 1]);
                            print_help();
                            std::process::exit(1);
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("Error: --port requires a value\n");
                    print_help();
                    std::process::exit(1);
                }
            }
            "--max-size" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<usize>() {
                        Ok(size) => max_size = size,
                        Err(_) => {
                            eprintln!("Error: Invalid max-size value '{}'\n", args[i + 1]);
                            print_help();
                            std::process::exit(1);
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("Error: --max-size requires a value\n");
                    print_help();
                    std::process::exit(1);
                }
            }
            _ => {
                eprintln!("Error: Unknown argument: '{}'\n", args[i]);
                print_help();
                std::process::exit(1);
            }
        }
    }

    (host, port, max_size)
}

/// Print help message
fn print_help() {
    println!("Usage: staticwallpaper [OPTIONS]");
    println!();
    println!("Options:");
    println!("  -h, --help           Show this help message");
    println!("      --host <HOST>    Server host address (default: 127.0.0.1)");
    println!("  -p, --port <PORT>    Server port number (default: 3000)");
    println!("      --max-size <KB>  Maximum WebP file size in KB (default: 300, 0 = no limit)");
    println!();
    println!("Examples:");
    println!("  staticwallpaper --host 0.0.0.0 --port 8080");
    println!("  staticwallpaper -p 8080 --max-size 500");
    println!("  staticwallpaper --max-size 0  # No size limit");
    println!("  staticwallpaper --help");
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let (host, port, max_size) = parse_args();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "staticwallpaper=debug,tower_http=debug,axum=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Get current working directory
    let current_dir = std::env::current_dir()?;
    let wallpaper_dir = current_dir.join("wallpaper");
    let db_path = current_dir.join("wallpapers.db");

    // Create wallpaper directories
    fs::create_dir_all(wallpaper_dir.join("pc")).await?;
    fs::create_dir_all(wallpaper_dir.join("mo")).await?;

    // Initialize database
    let db = Arc::new(Database::new(db_path).await?);
    let auth_manager = Arc::new(AuthManager::new(db.clone()));

    // Initialize database from existing files
    initialize_database(&db, &wallpaper_dir, max_size).await?;

    // Migrate missing hash values for existing records
    db.migrate_missing_hashes(&wallpaper_dir).await?;

    // Create application state
    let app_state = AppState {
        db: db.clone(),
        auth_manager: auth_manager.clone(),
        wallpaper_dir,
        max_size,
    };

    // Create router
    let app = create_router(app_state)
        .layer(TraceLayer::new_for_http());

    // Bind to address
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;

    println!("🦊 Server is running at http://{}\n", addr);
    println!("📏 Maximum WebP size: {} KB\n", max_size / 1024);

    // Start session cleanup task
    let auth_manager_cleanup = auth_manager.clone();
    tokio::spawn(async move {
        // 每10分钟清理一次过期会话，避免长时间运行后内存累积
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(600));
        loop {
            interval.tick().await;
            let before_count = auth_manager_cleanup.get_session_count().await;
            auth_manager_cleanup.cleanup_expired_sessions().await;
            let after_count = auth_manager_cleanup.get_session_count().await;

            if before_count > 0 {
                tracing::info!(
                    "Cleaned up {} expired session(s), {} active session(s) remaining",
                    before_count - after_count,
                    after_count
                );
            }
        }
    });

    // Start server using axum with tokio
    axum::serve(listener, app).await?;

    Ok(())
}