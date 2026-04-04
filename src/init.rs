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
    use std::sync::Arc;
    use tokio::sync::{Mutex, Semaphore};

    let dir = wallpaper_dir.join(r#type);
    let files = scan_directory(&dir).await.unwrap_or_default();

    println!("📁 扫描 {} 目录: 找到 {} 个图片文件", r#type.to_uppercase(), files.len());

    // 共享计数器
    let added = Arc::new(Mutex::new(0));
    let skipped = Arc::new(Mutex::new(0));
    let compressed = Arc::new(Mutex::new(0));
    let error = Arc::new(Mutex::new(0));

    // 并发控制：限制同时进行的任务数量
    // 使用 CPU 核心数，但最多不超过 8 个
    let max_concurrent = std::cmp::min(num_cpus::get(), 8);
    let semaphore = Arc::new(Semaphore::new(max_concurrent));
    println!("🚀 使用 {} 个并发任务处理文件", max_concurrent);

    // 收集所有任务
    let mut tasks = Vec::new();

    for filename in files {
        let filename = filename.clone();
        let db = db.clone();
        let dir = dir.clone();
        let type_str = r#type.to_string();
        let semaphore = semaphore.clone();
        let added_counter = added.clone();
        let skipped_counter = skipped.clone();
        let compressed_counter = compressed.clone();
        let error_counter = error.clone();

        let task = tokio::spawn(async move {
            // 获取信号量许可
            let _permit = semaphore.acquire().await.unwrap();

            match db.get_wallpaper_by_filename(&filename, crate::models::WallpaperType::from_str(&type_str).unwrap()).await {
                Ok(Some(_)) => {
                    // File already exists in database, check if it needs compression
                    if max_size > 0 {
                        let file_path = dir.join(&filename);
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
                                        let mut c = compressed_counter.lock().await;
                                        *c += 1;
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
                    let mut s = skipped_counter.lock().await;
                    *s += 1;
                    println!("  ⏭️  跳过: {} (已存在)", filename);
                }
                Ok(None) => {
                    let file_path = dir.join(&filename);
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
                            &filename,
                            &filename,
                            crate::models::WallpaperType::from_str(&type_str).unwrap(),
                            "",
                            created_at,
                        )
                        .await
                    {
                        Ok(_) => {
                            let mut a = added_counter.lock().await;
                            *a += 1;
                            println!("  ✅ 添加: {}", filename);
                        }
                        Err(_e) => {
                            let mut e = error_counter.lock().await;
                            *e += 1;
                            println!("  ❌ 错误: {} - {}", filename, _e);
                        }
                    }
                }
                Err(_e) => {
                    let mut e = error_counter.lock().await;
                    *e += 1;
                    println!("  ❌ 错误: {} - {}", filename, _e);
                }
            }
        });

        tasks.push(task);
    }

    // 等待所有任务完成
    for task in tasks {
        let _ = task.await;
    }

    // 读取最终统计
    let added_final = *added.lock().await;
    let skipped_final = *skipped.lock().await;
    let compressed_final = *compressed.lock().await;
    let error_final = *error.lock().await;

    println!("\n📊 {} 目录同步完成:", r#type.to_uppercase());
    println!("   ✅ 新增: {}", added_final);
    println!("   ⏭️  跳过: {}", skipped_final);
    if max_size > 0 && compressed_final > 0 {
        println!("   🔄 压缩: {}", compressed_final);
    }
    println!("   ❌ 错误: {}", error_final);

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