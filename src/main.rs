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
use std::path::PathBuf;
use std::sync::Arc;
use tokio::signal;
use tower_http::trace::TraceLayer;
use tracing::Level;
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

use crate::auth::AuthManager;
use crate::db::Database;
use crate::handlers::AppState;
use crate::init::initialize_database;
use crate::routes::create_router;

#[tokio::main]
async fn main() -> Result<()> {
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
        .layer(TraceLayer::new_for_http())
        .layer(axum::extract::Extension(auth::AuthState {
            auth_manager: auth_manager.clone(),
        }));

    // Bind to address
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
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