use anyhow::Result;
use std::collections::HashSet;
use std::path::Path;

use crate::db::Database;
use crate::image::is_image_file;

pub async fn initialize_database(db: &Database, wallpaper_dir: &Path) -> Result<()> {
    println!("🚀 开始初始化数据库...\n");

    sync_wallpapers(db, wallpaper_dir, "pc").await?;
    cleanup_orphaned_records(db, wallpaper_dir, "pc").await?;

    sync_wallpapers(db, wallpaper_dir, "mo").await?;
    cleanup_orphaned_records(db, wallpaper_dir, "mo").await?;

    println!("\n✅ 数据库初始化完成!\n");
    Ok(())
}

async fn scan_directory(dir: &Path) -> Result<Vec<String>> {
    let mut entries = tokio::fs::read_dir(dir).await?;
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

async fn sync_wallpapers(db: &Database, wallpaper_dir: &Path, r#type: &str) -> Result<()> {
    let dir = wallpaper_dir.join(r#type);
    let files = scan_directory(&dir).await.unwrap_or_default();

    println!("📁 扫描 {} 目录: 找到 {} 个图片文件", r#type.to_uppercase(), files.len());

    let mut added = 0;
    let mut skipped = 0;
    let mut error = 0;

    for filename in &files {
        match db.get_wallpaper_by_filename(filename, crate::models::WallpaperType::from_str(r#type).unwrap()).await {
            Ok(Some(_)) => {
                skipped += 1;
                println!("  ⏭️  跳过: {} (已存在)", filename);
            }
            Ok(None) => {
                let file_path = dir.join(filename);
                let metadata = tokio::fs::metadata(&file_path).await;
                let created_at = match metadata {
                    Ok(m) => {
                        let created = m.created().ok();
                        let modified = m.modified().ok();
                        let timestamp = created
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_millis() as i64)
                            .or_else(|| {
                                modified.and_then(|t| {
                                    t.duration_since(std::time::UNIX_EPOCH)
                                        .ok()
                                        .map(|d| d.as_millis() as i64)
                                })
                            })
                            .unwrap_or_else(|| chrono::Utc::now().timestamp_millis());
                        timestamp
                    }
                    Err(_) => chrono::Utc::now().timestamp_millis(),
                };

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