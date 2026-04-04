use anyhow::Result;
use axum::{
    extract::{Multipart, Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use axum::extract::multipart::Field;
use futures_util::StreamExt;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;

use crate::auth::AuthManager;
use crate::db::Database;
use crate::image::{convert_to_webp, generate_timestamped_filename, is_image_file};
use crate::models::{
    ApiResponse, CheckInitResponse, InitAdminRequest, LoginRequest,
    UpdateTagsRequest, User, Wallpaper, WallpaperType,
};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub auth_manager: Arc<AuthManager>,
    pub wallpaper_dir: PathBuf,
    pub max_size: usize,
}

// 鉴权辅助函数
async fn verify_auth(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<User, StatusCode> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    if let Some(auth_header) = auth_header {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            if let Some(user) = state.auth_manager.verify_session_token(token).await {
                return Ok(user);
            }
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}

#[derive(Debug, Deserialize)]
pub struct WallpaperQuery {
    pub list: Option<bool>,
    pub tags: Option<String>,
    pub width: Option<String>,
    pub height: Option<String>,
}

pub async fn check_init(State(state): State<AppState>) -> Json<CheckInitResponse> {
    let needs_init = state.auth_manager.needs_admin_init().await.unwrap_or(true);
    Json(CheckInitResponse { needs_init })
}

pub async fn init_admin(
    State(state): State<AppState>,
    Json(req): Json<InitAdminRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if req.password.len() < 6 {
        return Err(StatusCode::BAD_REQUEST);
    }

    match state.auth_manager.create_admin_account(&req.username, &req.password).await {
        Ok(user) => {
            let token = state
                .auth_manager
                .generate_session_token(user.id, &user.username)
                .await;
            Ok(Json(serde_json::json!({
                "success": true,
                "token": token,
                "user": user
            })))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.auth_manager.login(&req).await {
        Ok(Some((user, token))) => Ok(Json(serde_json::json!({
            "success": true,
            "token": token,
            "user": user
        }))),
        Ok(None) => Err(StatusCode::UNAUTHORIZED),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Json<ApiResponse<()>> {
    if let Some(auth_header) = headers.get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                state.auth_manager.revoke_session_token(token).await;
            }
        }
    }
    Json(ApiResponse::success(()))
}

pub async fn get_me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Json<serde_json::Value> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    if let Some(auth_header) = auth_header {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            if let Some(user) = state.auth_manager.verify_session_token(token).await {
                return Json(serde_json::json!({
                    "success": true,
                    "user": user
                }));
            }
        }
    }

    Json(serde_json::json!({
        "success": false,
        "error": "Unauthorized"
    }))
}

pub async fn get_wallpapers(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<Wallpaper>>, StatusCode> {
    verify_auth(&state, &headers).await?;

    let wallpaper_type = params
        .get("type")
        .and_then(|t| WallpaperType::from_str(t));

    match state.db.get_all_wallpapers(wallpaper_type).await {
        Ok(wallpapers) => Ok(Json(wallpapers)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn upload_wallpaper(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, StatusCode> {
    tracing::info!("=== Starting upload_wallpaper handler ===");
    tracing::info!("Content-Type header: {:?}", headers.get("content-type"));

    verify_auth(&state, &headers).await?;
    tracing::info!("Auth verified successfully");

    let mut wallpaper_type: Option<WallpaperType> = None;
    let mut tags = String::new();
    let mut original_filename: Option<String> = None;
    let mut temp_file_path: Option<std::path::PathBuf> = None;
    let mut field_count = 0;

    // Process all fields
    loop {
        let field_result = multipart.next_field().await;
        field_count += 1;
        tracing::info!("Processing field #{}", field_count);

        let mut field: Field = match field_result {
            Ok(Some(f)) => f,
            Ok(None) => {
                tracing::info!("No more fields to process");
                break;
            }
            Err(e) => {
                tracing::error!("Failed to read multipart field #{}: {:?}", field_count, e);
                tracing::error!("Error details: {}", e);
                return Err(StatusCode::BAD_REQUEST);
            }
        };

        let name = field.name().unwrap_or("");
        let filename = field.file_name().map(|n| n.to_string());
        let content_type = field.content_type().map(|ct| ct.to_string());

        tracing::info!("Field details - name: '{}', filename: {:?}, content_type: {:?}",
            name, filename, content_type);

        match name {
            "type" => {
                let type_str = match field.text().await {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::error!("Failed to read type field: {:?}", e);
                        return Err(StatusCode::BAD_REQUEST);
                    }
                };
                tracing::info!("Type field value: '{}'", type_str);
                wallpaper_type = WallpaperType::from_str(&type_str);
            }
            "tags" => {
                tags = field.text().await.unwrap_or_default();
                tracing::info!("Tags field value: '{}'", tags);
            }
            "file" => {
                if let Some(filename) = filename {
                    original_filename = Some(filename.clone());
                    tracing::info!("Found file field with filename: '{}'", filename);

                    // 使用流式读取，直接写入临时文件，避免将整个文件加载到内存
                    let temp_path = state.wallpaper_dir.join("temp_upload").join(filename);
                    fs::create_dir_all(temp_path.parent().unwrap()).await.map_err(|e| {
                        tracing::error!("Failed to create temp directory: {:?}", e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })?;

                    let mut file = fs::File::create(&temp_path).await.map_err(|e| {
                        tracing::error!("Failed to create temp file: {:?}", e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })?;

                    use tokio::io::AsyncWriteExt;
                    let mut total_size: u64 = 0;
                    const MAX_UPLOAD_SIZE: u64 = 20 * 1024 * 1024; // 20MB 限制

                    // Field 实现了 Stream 特性，可以直接使用 next() 读取数据块
                    while let Some(chunk_result) = field.next().await {
                        match chunk_result {
                            Ok(bytes) => {
                                total_size += bytes.len() as u64;
                                if total_size > MAX_UPLOAD_SIZE {
                                    drop(file);
                                    fs::remove_file(&temp_path).await.ok();
                                    tracing::error!("File size exceeds limit: {} bytes", total_size);
                                    return Err(StatusCode::PAYLOAD_TOO_LARGE);
                                }
                                if let Err(e) = file.write_all(&bytes).await {
                                    drop(file);
                                    fs::remove_file(&temp_path).await.ok();
                                    tracing::error!("Failed to write chunk to temp file: {:?}", e);
                                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                                }
                            }
                            Err(e) => {
                                drop(file);
                                fs::remove_file(&temp_path).await.ok();
                                tracing::error!("Failed to read chunk: {:?}", e);
                                return Err(StatusCode::BAD_REQUEST);
                            }
                        }
                    }

                    // 确保所有数据都写入磁盘
                    if let Err(e) = file.flush().await {
                        drop(file);
                        fs::remove_file(&temp_path).await.ok();
                        tracing::error!("Failed to flush temp file: {:?}", e);
                        return Err(StatusCode::INTERNAL_SERVER_ERROR);
                    }
                    drop(file);
                    tracing::info!("Successfully saved file to temp path: {} KB", total_size / 1024);

                    // 保存临时文件路径用于后续处理
                    temp_file_path = Some(temp_path);
                } else {
                    tracing::warn!("File field has no filename");
                }
            }
            _ => {
                tracing::warn!("Unknown field name: '{}'", name);
            }
        }
    }

    tracing::info!("Processed {} fields total", field_count);
    tracing::info!("Final values - type: {:?}, filename: {:?}, tags: '{}', temp_file: {:?}",
        wallpaper_type, original_filename, tags, temp_file_path.is_some());

    let wallpaper_type = wallpaper_type.ok_or_else(|| {
        tracing::error!("Missing wallpaper_type");
        StatusCode::BAD_REQUEST
    })?;
    let temp_path = temp_file_path.ok_or_else(|| {
        tracing::error!("Missing temp_file_path");
        StatusCode::BAD_REQUEST
    })?;
    let original_filename = original_filename.ok_or_else(|| {
        tracing::error!("Missing original_filename");
        StatusCode::BAD_REQUEST
    })?;

    tracing::info!("Validating file type for '{}'", original_filename);
    if !is_image_file(&original_filename) {
        tracing::error!("File '{}' is not a valid image file", original_filename);
        fs::remove_file(&temp_path).await.ok();
        return Err(StatusCode::BAD_REQUEST);
    }

    tracing::info!("Preparing to convert file to WebP");
    let target_dir = state.wallpaper_dir.join(wallpaper_type.as_str());
    tracing::info!("Target directory: {:?}", target_dir);

    if let Err(e) = fs::create_dir_all(&target_dir).await {
        tracing::error!("Failed to create directory {:?}: {:?}", target_dir, e);
        fs::remove_file(&temp_path).await.ok();
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    tracing::info!("File written successfully, starting WebP conversion");

    let webp_filename = generate_timestamped_filename(&original_filename)
        .replace(&format!(".{}", crate::image::get_file_extension(&original_filename)), ".webp");
    let webp_path = target_dir.join(&webp_filename);
    tracing::info!("WebP output path: {:?}", webp_path);

    convert_to_webp(&temp_path, &webp_path, state.max_size)
        .await
        .map_err(|e| {
            tracing::error!("Failed to convert to WebP: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    tracing::info!("WebP conversion completed successfully");

    fs::remove_file(&temp_path).await.ok();

    // 计算文件哈希值
    let file_hash = tokio::task::block_in_place(|| {
        crate::image::calculate_hash(&webp_path)
    }).map_err(|e| {
        tracing::error!("Failed to calculate file hash: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tracing::info!("File hash: {}", file_hash);

    // 检查是否已存在相同哈希的图片
    if let Ok(Some(existing)) = state.db.get_wallpaper_by_hash(&file_hash, &wallpaper_type).await {
        tracing::info!("Duplicate image detected. Existing file: {}", existing.filename);
        // 删除刚生成的文件
        fs::remove_file(&webp_path).await.ok();
        return Err(StatusCode::CONFLICT);
    }

    tracing::info!("Inserting wallpaper record into database");
    let wallpaper_id = state
        .db
        .insert_wallpaper(
            &webp_filename,
            &original_filename,
            wallpaper_type,
            &tags,
            chrono::Utc::now().timestamp_millis(),
            &file_hash,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to insert wallpaper into database: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    tracing::info!("=== Upload completed successfully ===");
    tracing::info!("Wallpaper ID: {}, Filename: {}", wallpaper_id, webp_filename);

    Ok(Json(serde_json::json!({
        "success": true,
        "id": wallpaper_id,
        "filename": webp_filename
    })))
}

pub async fn update_wallpaper_tags(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(req): Json<UpdateTagsRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    verify_auth(&state, &headers).await?;

    match state.db.update_wallpaper_tags(id, &req.tags).await {
        Ok(()) => Ok(Json(serde_json::json!({ "success": true }))),
        Err(_) => Ok(Json(serde_json::json!({ "success": false, "error": "Failed to update tags" }))),
    }
}

pub async fn delete_wallpaper(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    verify_auth(&state, &headers).await?;

    let wallpapers = state
        .db
        .get_all_wallpapers(None)
        .await
        .unwrap_or_default();

    if let Some(wallpaper) = wallpapers.iter().find(|w| w.id == id) {
        let file_path = state
            .wallpaper_dir
            .join(wallpaper.wallpaper_type.as_str())
            .join(&wallpaper.filename);

        fs::remove_file(&file_path).await.ok();

        match state.db.delete_wallpaper(id).await {
            Ok(()) => Ok(Json(serde_json::json!({ "success": true }))),
            Err(_) => Ok(Json(serde_json::json!({ "success": false, "error": "Failed to delete wallpaper" }))),
        }
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub async fn get_random_wallpaper_pc(
    State(state): State<AppState>,
    Query(params): Query<WallpaperQuery>,
) -> axum::response::Response {
    handle_random_wallpaper(state, WallpaperType::Pc, params).await
}

pub async fn get_random_wallpaper_mo(
    State(state): State<AppState>,
    Query(params): Query<WallpaperQuery>,
) -> axum::response::Response {
    handle_random_wallpaper(state, WallpaperType::Mo, params).await
}

async fn handle_random_wallpaper(
    state: AppState,
    wallpaper_type: WallpaperType,
    params: WallpaperQuery,
) -> axum::response::Response {
    if params.list == Some(true) {
        let wallpapers = state
            .db
            .get_all_wallpapers(Some(wallpaper_type))
            .await
            .unwrap_or_default();
        return Json(serde_json::json!({
            "count": wallpapers.len(),
            "wallpapers": wallpapers
        }))
        .into_response();
    }

    let tags = params.tags.map(|t| {
        t.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    });

    let wallpaper = match state.db.get_random_wallpaper(wallpaper_type, tags).await {
        Ok(Some(w)) => w,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, "No wallpapers available").into_response();
        }
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };

    let file_path = state
        .wallpaper_dir
        .join(wallpaper.wallpaper_type.as_str())
        .join(&wallpaper.filename);

    match fs::read(&file_path).await {
        Ok(bytes) => {
            let mut headers = HeaderMap::new();
            headers.insert("content-type", "image/webp".parse().unwrap());
            headers.insert("cache-control", "public, max-age=3600".parse().unwrap());
            // 使用静态字符串 "auto"，避免每次都创建新的 String
            let width_header = params.width.as_deref().unwrap_or("auto");
            let height_header = params.height.as_deref().unwrap_or("auto");
            headers.insert("x-width", width_header.parse().unwrap());
            headers.insert("x-height", height_header.parse().unwrap());
            headers.insert("x-tags", wallpaper.tags.parse().unwrap());

            (headers, bytes).into_response()
        }
        Err(_) => (StatusCode::NOT_FOUND, "Image file not found").into_response(),
    }
}