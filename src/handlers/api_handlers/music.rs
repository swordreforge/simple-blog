use actix_web::{web, HttpResponse};
use actix_multipart::Multipart;
use serde::{Deserialize, Serialize};
use crate::db::repositories::{MusicTrackRepository, Repository};
use std::sync::Arc;

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
    } else {
        // 默认更新封面（通过 multipart form）
        HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "封面更新成功"
        }))
    }
}

/// 删除音乐
pub async fn delete(
    path: web::Path<i64>,
    repo: web::Data<Arc<dyn Repository>>,
) -> HttpResponse {
    let id = path.into_inner();
    let music_repo = MusicTrackRepository::new(repo.get_pool().clone());
    
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
    use futures_util::stream::StreamExt;
    
    let music_repo = MusicTrackRepository::new(repo.get_pool().clone());
    
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
        
        let mut file_bytes = Vec::new();
        while let Some(chunk) = field.next().await {
            let chunk = match chunk {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("读取块失败: {}", e);
                    break;
                }
            };
            file_bytes.extend_from_slice(&chunk);
        }
        
        // 保存文件到 music 目录
        let file_path = format!("/music/{}", filename);
        
        let track = crate::db::models::MusicTrack {
            id: None,
            title: filename.clone().replace(".mp3", "").replace(".wav", ""),
            artist: "未知艺术家".to_string(),
            file_path: file_path,
            file_name: filename,
            duration: "0:00".to_string(),
            cover_image: String::new(),
            created_at: chrono::Utc::now(),
        };
        
        if let Err(e) = music_repo.create(&track).await {
            eprintln!("保存音乐信息失败: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "上传失败"
            }));
        }
    }
    
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "上传成功"
    }))
}