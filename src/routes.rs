use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

use crate::handlers::{
    check_init, delete_wallpaper, get_me, get_random_wallpaper_mo, get_random_wallpaper_pc,
    get_wallpapers, init_admin, login, logout, update_wallpaper_tags, upload_wallpaper,
};
use crate::AppState;

pub fn create_router(state: AppState) -> Router {
    let admin_router = Router::new()
        .route("/check-init", get(check_init))
        .route("/init", post(init_admin))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/me", get(get_me))
        .route("/wallpapers", get(get_wallpapers))
        .route("/upload", post(upload_wallpaper))
        .route("/wallpapers/:id/tags", axum::routing::put(update_wallpaper_tags))
        .route("/wallpapers/:id", axum::routing::delete(delete_wallpaper))
        .with_state(state.clone());

    Router::new()
        .route("/", get(index_handler))
        .route("/about", get(about_handler))
        .route("/admin", get(admin_handler))
        .route("/login", get(login_handler))
        .route("/img/pc", get(get_random_wallpaper_pc))
        .route("/img/mo", get(get_random_wallpaper_mo))
        .nest_service("/wallpaper", ServeDir::new(state.wallpaper_dir.clone()))
        .nest_service("/assets", ServeDir::new("public/assets"))
        .nest_service("/api/admin", admin_router)
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn index_handler() -> axum::response::Html<String> {
    serve_html_file("public/index.html")
}

async fn about_handler() -> axum::response::Html<String> {
    serve_html_file("public/about.html")
}

async fn admin_handler() -> axum::response::Html<String> {
    serve_html_file("public/admin/index.html")
}

async fn login_handler() -> axum::response::Html<String> {
    serve_html_file("public/login.html")
}

fn serve_html_file(path: &str) -> axum::response::Html<String> {
    match std::fs::read_to_string(path) {
        Ok(content) => axum::response::Html(content),
        Err(_) => axum::response::Html(format!(
            r#"<!DOCTYPE html>
<html>
<head><title>404</title></head>
<body><h1>404 - File not found</h1><p>{}</p></body>
</html>"#,
            path
        )),
    }
}