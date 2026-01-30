use actix_web::{web, HttpResponse, error};
use actix_multipart::Multipart;
use serde::{Deserialize, Serialize};
use crate::db::repositories::{MusicTrackRepository, Repository};
use crate::audio_metadata::{extract_metadata, format_duration, fallback_metadata};
use std::sync::Arc;
use futures_util::stream::StreamExt;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use std::path::Path;

/// 音乐轨道响应
#[derive(Debug, Serialize)]
pub struct MusicTrackResponse {
    pub id: i64,
    pub title: String,
    pub artist: String,
    pub file_path: String,
    pub file_name: String,
    pub duration: String,
    pub cover_image: String,
}

/// 更新音乐请求
#[derive(Debug, Deserialize)]
pub struct UpdateMusicRequest {
    pub title: Option<String>,
}

/// 获取音乐列表
pub async fn list() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "music": []
    }))
}

/// 获取音乐播放列表
pub async fn playlist(repo: web::Data<Arc<dyn Repository>>) -> HttpResponse {
    let music_repo = MusicTrackRepository::new(repo.get_pool().clone());
    
    match music_repo.get_all_without_pagination().await {
        Ok(tracks) => {
            let data: Vec<MusicTrackResponse> = tracks.into_iter()
                .map(|track| MusicTrackResponse {
                    id: track.id.unwrap_or(0),
                    title: track.title,
                    artist: track.artist,
                    file_path: track.file_path,
                    file_name: track.file_name,
                    duration: track.duration,
                    cover_image: track.cover_image,
                })
                .collect();
            
            HttpResponse::Ok().json(data)
        }
        Err(e) => {
            eprintln!("获取播放列表失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "获取播放列表失败"
            }))
        }
    }
}

/// 播放音乐
pub async fn play(path: web::Path<String>) -> HttpResponse {
    let _id = path.into_inner();
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Music playing"
    }))
}

/// 更新音乐（标题或封面）
pub async fn update(
    path: web::Path<i64>,
    query: web::Query<std::collections::HashMap<String, String>>,
    payload: Option<web::Json<UpdateMusicRequest>>,
    repo: web::Data<Arc<dyn Repository>>,
) -> HttpResponse {
    let id = path.into_inner();
    let action = query.get("action").map(|s| s.as_str());
    let music_repo = MusicTrackRepository::new(repo.get_pool().clone());

    if action == Some("title") {
        // 更新标题
        if let Some(payload) = payload {
            if let Some(ref title) = payload.title {
                if title.trim().is_empty() {
                    return HttpResponse::BadRequest().json(serde_json::json!({
                        "success": false,
                        "message": "标题不能为空"
                    }));
                }

                match music_repo.update_title(id, title).await {
                    Ok(_) => HttpResponse::Ok().json(serde_json::json!({
                        "success": true,
                        "message": "标题更新成功"
                    })),
                    Err(e) => {
                        eprintln!("更新标题失败: {}", e);
                        HttpResponse::InternalServerError().json(serde_json::json!({
                            "success": false,
                            "message": "更新标题失败"
                        }))
                    }
                }
            } else {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "success": false,
                    "message": "缺少标题参数"
                }))
            }
        } else {
            HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "message": "缺少请求体"
            }))
        }
    } else if action == Some("cover") {
        // 默认更新封面（通过 multipart form）
        HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "请使用封面上传接口"
        }))
    } else {
        HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "message": "缺少 action 参数"
        }))
    }
}

/// 上传封面
pub async fn upload_cover(
    path: web::Path<i64>,
    mut payload: Multipart,
    repo: web::Data<Arc<dyn Repository>>,
) -> HttpResponse {
    let id = path.into_inner();
    let music_repo = MusicTrackRepository::new(repo.get_pool().clone());

    // 允许的图片文件类型
    let allowed_extensions = ["jpg", "jpeg", "png", "gif", "webp"];
    const MAX_COVER_SIZE: usize = 5 * 1024 * 1024; // 5MB

    while let Some(field) = payload.next().await {
        let mut field = match field {
            Ok(f) => f,
            Err(e) => {
                eprintln!("获取字段失败: {}", e);
                continue;
            }
        };

        let content_disposition = field.content_disposition();
        let filename = content_disposition
            .and_then(|cd| cd.get_filename().map(|s| s.to_string()))
            .unwrap_or_else(|| "cover.jpg".to_string());

        // 验证文件扩展名
        let extension = Path::new(&filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("jpg")
            .to_lowercase();

        if !allowed_extensions.contains(&extension.as_str()) {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "message": format!("不支持的图片类型: .{}", extension)
            }));
        }

        // 生成唯一封面文件名
        let timestamp = chrono::Utc::now().timestamp();
        let cover_dir = "music/covers";
        let cover_filename = format!("{}_cover.{}", timestamp, extension);
        let cover_path = format!("{}/{}", cover_dir, cover_filename);

        // 确保目录存在
        if let Err(e) = fs::create_dir_all(cover_dir).await {
            eprintln!("创建封面目录失败: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "创建目录失败"
            }));
        }

        // 保存文件到磁盘
        let mut file = match fs::File::create(&cover_path).await {
            Ok(f) => f,
            Err(e) => {
                eprintln!("创建封面文件失败: {}", e);
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "success": false,
                    "message": "创建文件失败"
                }));
            }
        };

        let mut file_size = 0;

        while let Some(chunk) = field.next().await {
            let chunk = match chunk {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("读取块失败: {}", e);
                    let _ = fs::remove_file(&cover_path).await;
                    return HttpResponse::InternalServerError().json(serde_json::json!({
                        "success": false,
                        "message": "读取文件失败"
                    }));
                }
            };

            file_size += chunk.len();
            if file_size > MAX_COVER_SIZE {
                let _ = fs::remove_file(&cover_path).await;
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "success": false,
                    "message": "封面图片过大（最大5MB）"
                }));
            }

            if let Err(e) = file.write_all(&chunk).await {
                eprintln!("写入封面文件失败: {}", e);
                let _ = fs::remove_file(&cover_path).await;
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "success": false,
                    "message": "保存文件失败"
                }));
            }
        }

        // 更新数据库中的封面路径
        let cover_url = format!("/music/covers/{}", cover_filename);
        match music_repo.update_cover(id, &cover_url).await {
            Ok(_) => {
                // 删除旧封面
                if let Ok(track) = music_repo.get_by_id(id).await {
                    let old_cover = track.cover_image;
                    if !old_cover.is_empty() {
                        let old_path = format!(".{}", old_cover);
                        let _ = fs::remove_file(&old_path).await;
                    }
                }

                return HttpResponse::Ok().json(serde_json::json!({
                    "success": true,
                    "message": "封面更新成功",
                    "cover": cover_url
                }));
            }
            Err(e) => {
                eprintln!("更新封面失败: {}", e);
                let _ = fs::remove_file(&cover_path).await;
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "success": false,
                    "message": "更新数据库失败"
                }));
            }
        }
    }

    HttpResponse::BadRequest().json(serde_json::json!({
        "success": false,
        "message": "没有上传文件"
    }))
}

/// 删除音乐
pub async fn delete(
    path: web::Path<i64>,
    repo: web::Data<Arc<dyn Repository>>,
) -> HttpResponse {
    let id = path.into_inner();
    let music_repo = MusicTrackRepository::new(repo.get_pool().clone());

    // 先获取音乐信息，以便删除相关文件
    let track_info = match music_repo.get_by_id(id).await {
        Ok(track) => track,
        Err(e) => {
            eprintln!("获取音乐信息失败: {}", e);
            return HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "message": "音乐不存在"
            }));
        }
    };

    // 删除音频文件
    if !track_info.file_path.is_empty() {
        let file_path = format!(".{}", track_info.file_path);
        if let Err(e) = fs::remove_file(&file_path).await {
            eprintln!("删除音频文件失败 {}: {}", file_path, e);
            // 不中断流程，继续删除数据库记录
        }
    }

    // 检查封面图片是否被其他音乐使用
    if !track_info.cover_image.is_empty() {
        let cover_image = track_info.cover_image.clone();

        // 获取所有音乐，检查是否有其他音乐使用相同的封面
        match music_repo.get_all_without_pagination().await {
            Ok(all_tracks) => {
                // 统计使用此封面的音乐数量（排除当前要删除的音乐）
                let cover_usage_count = all_tracks.iter()
                    .filter(|track| {
                        track.id != track_info.id && track.cover_image == cover_image
                    })
                    .count();

                // 只有当没有其他音乐使用此封面时，才删除封面图片
                if cover_usage_count == 0 {
                    let cover_path = format!(".{}", cover_image);
                    if let Err(e) = fs::remove_file(&cover_path).await {
                        eprintln!("删除封面图片失败 {}: {}", cover_path, e);
                        // 不中断流程，继续删除数据库记录
                    }
                } else {
                    println!("封面图片 {} 被 {} 个其他音乐使用，跳过删除", cover_image, cover_usage_count);
                }
            }
            Err(e) => {
                eprintln!("获取音乐列表失败: {}", e);
                // 不中断流程，继续删除数据库记录
            }
        }
    }

    // 删除数据库记录
    match music_repo.delete(id).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "删除成功"
        })),
        Err(e) => {
            eprintln!("删除音乐失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "删除失败"
            }))
        }
    }
}

/// 上传音乐
pub async fn upload(
    mut payload: Multipart,
    repo: web::Data<Arc<dyn Repository>>,
) -> HttpResponse {
    let music_repo = MusicTrackRepository::new(repo.get_pool().clone());

    // 允许的音频文件类型
    let allowed_extensions = ["mp3", "wav", "ogg", "flac", "m4a"];

    while let Some(field) = payload.next().await {
        let mut field = match field {
            Ok(f) => f,
            Err(e) => {
                eprintln!("获取字段失败: {}", e);
                continue;
            }
        };

        let content_disposition = field.content_disposition();
        let filename = content_disposition
            .and_then(|cd| cd.get_filename().map(|s| s.to_string()))
            .unwrap_or_else(|| "unknown".to_string());

        // 验证文件扩展名
        let extension = Path::new(&filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if !allowed_extensions.contains(&extension.as_str()) {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "message": format!("不支持的文件类型: .{}", extension)
            }));
        }

        // 生成唯一文件名（时间戳_原文件名）
        let timestamp = chrono::Utc::now().timestamp();
        let unique_filename = format!("{}_{}", timestamp, filename);
        let music_dir = "music";

        // 确保目录存在
        if let Err(e) = fs::create_dir_all(music_dir).await {
            eprintln!("创建音乐目录失败: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "创建目录失败"
            }));
        }

        let file_path = format!("{}/{}", music_dir, unique_filename);

        // 保存文件到磁盘
        let mut file = match fs::File::create(&file_path).await {
            Ok(f) => f,
            Err(e) => {
                eprintln!("创建文件失败: {}", e);
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "success": false,
                    "message": "创建文件失败"
                }));
            }
        };

        let mut file_size = 0;
        const MAX_FILE_SIZE: usize = 50 * 1024 * 1024; // 50MB

        while let Some(chunk) = field.next().await {
            let chunk = match chunk {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("读取块失败: {}", e);
                    // 清理已上传的文件
                    let _ = fs::remove_file(&file_path).await;
                    return HttpResponse::InternalServerError().json(serde_json::json!({
                        "success": false,
                        "message": "读取文件失败"
                    }));
                }
            };

            file_size += chunk.len();
            if file_size > MAX_FILE_SIZE {
                let _ = fs::remove_file(&file_path).await;
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "success": false,
                    "message": "文件过大（最大50MB）"
                }));
            }

            if let Err(e) = file.write_all(&chunk).await {
                eprintln!("写入文件失败: {}", e);
                let _ = fs::remove_file(&file_path).await;
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "success": false,
                    "message": "保存文件失败"
                }));
            }
        }

        // 提取音频元数据
        let metadata = extract_metadata(&file_path)
            .unwrap_or_else(|_| fallback_metadata(&unique_filename));

        // 确定标题和艺术家
        let title = metadata.title.unwrap_or_else(|| {
            unique_filename
                .trim_end_matches(&format!(".{}", extension))
                .to_string()
        });
        let artist = metadata.artist.unwrap_or_else(|| "未知艺术家".to_string());

        // 时长暂时使用 "未知"，前端会预加载
        let duration = "未知".to_string();

        // 创建数据库记录
        let track = crate::db::models::MusicTrack {
            id: None,
            title,
            artist,
            file_path: format!("/music/{}", unique_filename),
            file_name: unique_filename,
            duration,
            cover_image: String::new(),
            created_at: chrono::Utc::now(),
        };

        match music_repo.create(&track).await {
            Ok(_) => {
                return HttpResponse::Ok().json(serde_json::json!({
                    "success": true,
                    "message": "上传成功",
                    "track": track
                }));
            }
            Err(e) => {
                eprintln!("保存音乐信息失败: {}", e);
                // 清理已上传的文件
                let _ = fs::remove_file(&file_path).await;
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "success": false,
                    "message": "保存到数据库失败"
                }));
            }
        }
    }

    HttpResponse::BadRequest().json(serde_json::json!({
        "success": false,
        "message": "没有上传文件"
    }))
}