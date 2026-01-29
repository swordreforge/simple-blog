use std::collections::HashMap;
use std::fs;
use std::path::Path;
use crate::db::repositories::{MusicTrackRepository, Repository};
use crate::audio_metadata::extract_metadata;
use std::sync::Arc;

/// 音乐同步服务
pub struct MusicSyncService {
    repo: Arc<dyn Repository>,
}

impl MusicSyncService {
    pub fn new(repo: Arc<dyn Repository>) -> Self {
        Self { repo }
    }

    /// 同步 music 目录中的文件到数据库
    pub async fn sync_music_files_to_db(&self) -> Result<SyncResult, String> {
        let music_dir = "music";
        let covers_dir = format!("{}/covers", music_dir);

        // 检查目录是否存在
        if !Path::new(music_dir).exists() {
            return Ok(SyncResult {
                synced_count: 0,
                updated_count: 0,
                deleted_count: 0,
                message: "Music directory does not exist, skipping sync".to_string(),
            });
        }

        // 读取 covers 目录中的所有封面文件
        let covers_map = self.read_covers_map(&covers_dir).await?;

        // 读取音乐目录中的所有文件
        let entries = match fs::read_dir(music_dir) {
            Ok(e) => e,
            Err(e) => return Err(format!("Failed to read music directory: {}", e)),
        };

        // 获取数据库中已存在的文件
        let music_repo = MusicTrackRepository::new(self.repo.get_pool().clone());
        let existing_files = self.get_existing_files(&music_repo).await?;

        let mut synced_count = 0;
        let mut updated_count = 0;

        // 遍历文件，同步到数据库
        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("Failed to read directory entry: {}", e);
                    continue;
                }
            };

            let file_path = entry.path();
            let file_name = match entry.file_name().into_string() {
                Ok(name) => name,
                Err(_) => continue,
            };

            // 跳过目录和非音频文件
            let file_type = match entry.file_type() {
                Ok(ft) => ft,
                Err(_) => continue,
            };
            if file_type.is_dir() || !Self::is_audio_file(&file_name) {
                continue;
            }

            // 提取时间戳（格式：timestamp_filename.ext）
            let timestamp = Self::extract_timestamp(&file_name);

            // 查找匹配的封面
            let cover_image = if let Some(ts) = &timestamp {
                covers_map.get(ts).map(|cover| format!("/music/covers/{}", cover))
            } else {
                None
            }.unwrap_or_default();

            // 如果文件已存在于数据库中，更新封面信息
            if existing_files.contains(&file_name) {
                if !cover_image.is_empty() {
                    match music_repo.update_cover_by_filename(&file_name, &cover_image).await {
                        Ok(_) => {
                            updated_count += 1;
                            println!("Updated cover for: {} -> {}", file_name, cover_image);
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to update cover for {}: {}", file_name, e);
                        }
                    }
                }
                continue;
            }

            // 提取元数据
            let full_path = file_path.to_string_lossy().to_string();
            let metadata = match extract_metadata(&full_path) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("Warning: Failed to extract metadata for {}: {}", file_name, e);
                    crate::audio_metadata::fallback_metadata(&file_name)
                }
            };

            // 准备标题和艺术家
            let title = metadata.title.unwrap_or_else(|| {
                Self::clean_title(&file_name)
            });
            let artist = metadata.artist.unwrap_or_else(|| "未知艺术家".to_string());
            let duration = "未知".to_string();

            // 插入数据库
            let track = crate::db::models::MusicTrack {
                id: None,
                title,
                artist,
                file_path: format!("/music/{}", file_name),
                file_name: file_name.clone(),
                duration,
                cover_image,
                created_at: chrono::Utc::now(),
            };

            match music_repo.create(&track).await {
                Ok(_) => {
                    synced_count += 1;
                    println!("Synced music file: {} - {}", track.title, track.artist);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to insert track {}: {}", file_name, e);
                }
            }
        }

        // 清理数据库中不存在的文件记录
        let deleted_count = self.cleanup_orphaned_files(&music_repo, music_dir).await?;

        Ok(SyncResult {
            synced_count,
            updated_count,
            deleted_count,
            message: format!(
                "Music sync completed: {} files synced, {} covers updated, {} orphaned records removed",
                synced_count, updated_count, deleted_count
            ),
        })
    }

    /// 读取 covers 目录中的所有封面文件，建立时间戳到文件名的映射
    async fn read_covers_map(&self, covers_dir: &str) -> Result<HashMap<String, String>, String> {
        let mut covers_map = HashMap::new();

        if !Path::new(covers_dir).exists() {
            return Ok(covers_map);
        }

        let entries = match fs::read_dir(covers_dir) {
            Ok(e) => e,
            Err(e) => return Err(format!("Failed to read covers directory: {}", e)),
        };

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let file_type = match entry.file_type() {
                Ok(ft) => ft,
                Err(_) => continue,
            };
            if file_type.is_dir() {
                continue;
            }

            let cover_name = match entry.file_name().into_string() {
                Ok(name) => name,
                Err(_) => continue,
            };

            // 提取时间戳（格式：timestamp_cover.ext）
            if let Some(timestamp) = Self::extract_cover_timestamp(&cover_name) {
                covers_map.insert(timestamp, cover_name);
            }
        }

        Ok(covers_map)
    }

    /// 获取数据库中已存在的文件
    async fn get_existing_files(&self, music_repo: &MusicTrackRepository) -> Result<std::collections::HashSet<String>, String> {
        let tracks = music_repo.get_all_without_pagination().await
            .map_err(|e| format!("Failed to get existing files: {}", e))?;

        let mut files = std::collections::HashSet::new();
        for track in tracks {
            files.insert(track.file_name);
        }

        Ok(files)
    }

    /// 清理数据库中不存在的文件记录
    async fn cleanup_orphaned_files(&self, music_repo: &MusicTrackRepository, music_dir: &str) -> Result<usize, String> {
        let entries = match fs::read_dir(music_dir) {
            Ok(e) => e,
            Err(e) => return Err(format!("Failed to read music directory: {}", e)),
        };

        // 构建当前目录中的文件集合
        let mut files_in_dir = std::collections::HashSet::new();
        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let file_type = match entry.file_type() {
                Ok(ft) => ft,
                Err(_) => continue,
            };
            if !file_type.is_dir() {
                if let Ok(name) = entry.file_name().into_string() {
                    files_in_dir.insert(name);
                }
            }
        }

        // 获取数据库中的所有文件
        let tracks = music_repo.get_all_without_pagination().await
            .map_err(|e| format!("Failed to get tracks: {}", e))?;

        let mut deleted_count = 0;
        for track in tracks {
            if !files_in_dir.contains(&track.file_name) {
                if let Some(id) = track.id {
                    match music_repo.delete(id).await {
                        Ok(_) => {
                            deleted_count += 1;
                            println!("Removed orphaned music record: {}", track.file_name);
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to delete orphaned record {}: {}", track.file_name, e);
                        }
                    }
                }
            }
        }

        Ok(deleted_count)
    }

    /// 检查文件是否是音频文件
    fn is_audio_file(file_name: &str) -> bool {
        let ext = Path::new(file_name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        matches!(ext.as_str(), "mp3" | "wav" | "ogg" | "m4a" | "flac" | "aac" | "wma")
    }

    /// 提取文件名中的时间戳（格式：timestamp_filename.ext）
    fn extract_timestamp(file_name: &str) -> Option<String> {
        if let Some(pos) = file_name.find('_') {
            let timestamp_str = &file_name[..pos];
            // 验证是否是有效的时间戳（数字）
            if timestamp_str.chars().all(|c| c.is_ascii_digit()) {
                return Some(timestamp_str.to_string());
            }
        }
        None
    }

    /// 提取封面文件名中的时间戳（格式：timestamp_cover.ext）
    fn extract_cover_timestamp(cover_name: &str) -> Option<String> {
        if let Some(pos) = cover_name.find("_cover") {
            let timestamp_str = &cover_name[..pos];
            if timestamp_str.chars().all(|c| c.is_ascii_digit()) {
                return Some(timestamp_str.to_string());
            }
        }
        None
    }

    /// 净化标题，移除时间戳和下划线前缀
    fn clean_title(title: &str) -> String {
        // 检查是否以数字开头，后面跟着下划线
        if let Some(pos) = title.find('_') {
            let timestamp_part = &title[..pos];
            // 验证前缀是否全是数字
            if timestamp_part.chars().all(|c| c.is_ascii_digit()) {
                // 移除时间戳和下划线
                if let Some((_, rest)) = title.split_once('_') {
                    return rest.to_string();
                }
            }
        }
        // 移除文件扩展名
        Path::new(title)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(title)
            .to_string()
    }
}

/// 同步结果
#[derive(Debug)]
pub struct SyncResult {
    pub synced_count: usize,
    pub updated_count: usize,
    pub deleted_count: usize,
    pub message: String,
}