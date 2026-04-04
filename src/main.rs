mod auth;
mod db;
mod embedded;
mod handlers;
mod image;
mod init;
mod models;
mod routes;

use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::signal;
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
fn parse_args() -> (String, u16) {
    let mut host = String::from("127.0.0.1");
    let mut port = 3000;
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
            _ => {
                eprintln!("Error: Unknown argument: '{}'\n", args[i]);
                print_help();
                std::process::exit(1);
            }
        }
    }

    (host, port)
}

/// Print help message
fn print_help() {
    println!("Usage: staticwallpaper [OPTIONS]");
    println!();
    println!("Options:");
    println!("  -h, --help         Show this help message");
    println!("      --host <HOST>  Server host address (default: 127.0.0.1)");
    println!("  -p, --port <PORT>  Server port number (default: 3000)");
    println!();
    println!("Examples:");
    println!("  staticwallpaper --host 0.0.0.0 --port 8080");
    println!("  staticwallpaper -p 8080");
    println!("  staticwallpaper --help");
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let (host, port) = parse_args();

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
    tokio::fs::create_dir_all(wallpaper_dir.join("pc")).await?;
    tokio::fs::create_dir_all(wallpaper_dir.join("mo")).await?;

    // Initialize database
    let db = Arc::new(Database::new(db_path).await?);
    let auth_manager = Arc::new(AuthManager::new(db.clone()));

    // Initialize database from existing files
    initialize_database(&db, &wallpaper_dir).await?;

    // Create application state
    let app_state = AppState {
        db: db.clone(),
        auth_manager: auth_manager.clone(),
        wallpaper_dir,
    };

    // Create router
    let app = create_router(app_state)
        .layer(TraceLayer::new_for_http());

    // Bind to address
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;

    println!("🦊 Server is running at http://{}\n", addr);

    // Start session cleanup task
    let auth_manager_cleanup = auth_manager.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600));
        loop {
            interval.tick().await;
            auth_manager_cleanup.cleanup_expired_sessions().await;
        }
    });

    // Start server
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Ctrl+C received, shutting down gracefully...");
        },
        _ = terminate => {
            tracing::info!("SIGTERM received, shutting down gracefully...");
        },
    }
}
