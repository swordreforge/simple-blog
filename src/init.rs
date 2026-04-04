use anyhow::Result;
use std::collections::HashSet;
use std::path::Path;
use tokio::fs;

use crate::db::Database;
use crate::image::{convert_to_webp, is_image_file};

pub async fn initialize_database(db: &Database, wallpaper_dir: &Path, max_size: usize) -> Result<()> {
    println!("🚀 开始初始化数据库...\n");

    sync_wallpapers(db, wallpaper_dir, "pc", max_size).await?;
    cleanup_orphaned_records(db, wallpaper_dir, "pc").await?;

    sync_wallpapers(db, wallpaper_dir, "mo", max_size).await?;
    cleanup_orphaned_records(db, wallpaper_dir, "mo").await?;

    println!("\n✅ 数据库初始化完成!\n");
    Ok(())
}

async fn scan_directory(dir: &Path) -> Result<Vec<String>> {
    let mut entries = fs::read_dir(dir).await?;
    let mut files = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_file() {
            if let Some(filename) = path.file_name() {
                if let Some(filename_str) = filename.to_str() {
                    if is_image_file(filename_str) && !filename_str.starts_with("temp_") {
                        files.push(filename_str.to_string());
                    }
                }
            }
        }
    }

    Ok(files)
}

async fn sync_wallpapers(db: &Database, wallpaper_dir: &Path, r#type: &str, max_size: usize) -> Result<()> {
    let dir = wallpaper_dir.join(r#type);
    let files = scan_directory(&dir).await.unwrap_or_default();

    println!("📁 扫描 {} 目录: 找到 {} 个图片文件", r#type.to_uppercase(), files.len());

    let mut added = 0;
    let mut skipped = 0;
    let mut compressed = 0;
    let mut error = 0;

    for filename in &files {
        match db.get_wallpaper_by_filename(filename, crate::models::WallpaperType::from_str(r#type).unwrap()).await {
            Ok(Some(_)) => {
                // File already exists in database, check if it needs compression
                if max_size > 0 {
                    let file_path = dir.join(filename);
                    if let Ok(metadata) = fs::metadata(&file_path).await {
                        let file_size = metadata.len();
                        if file_size > max_size as u64 {
                            println!("  🔄 压缩: {} ({} KB -> 目标 {} KB)",
                                filename,
                                file_size / 1024,
                                max_size / 1024
                            );

                            match convert_to_webp(&file_path, &file_path, max_size).await {
                                Ok(_) => {
                                    compressed += 1;
                                    // Get new file size
                                    if let Ok(new_metadata) = fs::metadata(&file_path).await {
                                        println!("     ✅ 压缩完成: {} KB", new_metadata.len() / 1024);
                                    }
                                }
                                Err(e) => {
                                    println!("     ❌ 压缩失败: {}", e);
                                }
                            }
                        }
                    }
                }
                skipped += 1;
                println!("  ⏭️  跳过: {} (已存在)", filename);
            }
            Ok(None) => {
                let file_path = dir.join(filename);
                let metadata = fs::metadata(&file_path).await;
                let created_at = match metadata {
                    Ok(m) => {
                        let created = m.created().ok();
                        let modified = m.modified().ok();
                        created
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_millis() as i64)
                            .or_else(|| {
                                modified.and_then(|t| {
                                    t.duration_since(std::time::UNIX_EPOCH)
                                        .ok()
                                        .map(|d| d.as_millis() as i64)
                                })
                            })
                            .unwrap_or_else(|| chrono::Utc::now().timestamp_millis())
                    }
                    Err(_) => chrono::Utc::now().timestamp_millis(),
                };

                // Compress if file size exceeds max_size
                if max_size > 0 {
                    if let Ok(m) = fs::metadata(&file_path).await {
                        if m.len() > max_size as u64 {
                            println!("  🔄 压缩: {} ({} KB -> 目标 {} KB)",
                                filename,
                                m.len() / 1024,
                                max_size / 1024
                            );
                            if let Err(e) = convert_to_webp(&file_path, &file_path, max_size).await {
                                println!("     ❌ 压缩失败: {}", e);
                            }
                        }
                    }
                }

                match db
                    .insert_wallpaper(
                        filename,
                        filename,
                        crate::models::WallpaperType::from_str(r#type).unwrap(),
                        "",
                        created_at,
                    )
                    .await
                {
                    Ok(_) => {
                        added += 1;
                        println!("  ✅ 添加: {}", filename);
                    }
                    Err(e) => {
                        error += 1;
                        println!("  ❌ 错误: {} - {}", filename, e);
                    }
                }
            }
            Err(e) => {
                error += 1;
                println!("  ❌ 错误: {} - {}", filename, e);
            }
        }
    }

    println!("\n📊 {} 目录同步完成:", r#type.to_uppercase());
    println!("   ✅ 新增: {}", added);
    println!("   ⏭️  跳过: {}", skipped);
    if max_size > 0 && compressed > 0 {
        println!("   🔄 压缩: {}", compressed);
    }
    println!("   ❌ 错误: {}", error);

    Ok(())
}

async fn cleanup_orphaned_records(db: &Database, wallpaper_dir: &Path, r#type: &str) -> Result<()> {
    let dir = wallpaper_dir.join(r#type);
    let files = scan_directory(&dir).await.unwrap_or_default();
    let file_set: HashSet<String> = files.into_iter().collect();

    let wallpapers = db
        .get_all_wallpapers(Some(crate::models::WallpaperType::from_str(r#type).unwrap()))
        .await
        .unwrap_or_default();

    let mut removed = 0;

    for wallpaper in wallpapers {
        if !file_set.contains(&wallpaper.filename) {
            match db.delete_wallpaper(wallpaper.id).await {
                Ok(_) => {
                    removed += 1;
                    println!("  🗑️  删除: {} (文件不存在)", wallpaper.filename);
                }
                Err(e) => {
                    println!("  ❌ 删除失败: {} - {}", wallpaper.filename, e);
                }
            }
        }
    }

    if removed > 0 {
        println!("\n🧹 {} 目录清理完成: 删除了 {} 条无效记录", r#type.to_uppercase(), removed);
    }

    Ok(())
}