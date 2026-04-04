use anyhow::{Context, Result};
use sqlx::{sqlite::SqlitePool, Pool, Sqlite};
use std::path::{Path, PathBuf};

use crate::models::{UserWithPasswordHash, Wallpaper, WallpaperType};

#[derive(Clone)]
pub struct Database {
    pool: Pool<Sqlite>,
}

impl Database {
    pub async fn new(db_path: PathBuf) -> Result<Self> {
        // 确保数据库目录存在
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let pool = SqlitePool::connect(&format!("sqlite:{}?mode=rwc", db_path.display()))
            .await
            .context("Failed to connect to database")?;

        let db = Self { pool };
        db.migrate().await?;
        Ok(db)
    }

    async fn migrate(&self) -> Result<()> {
        // 创建壁纸表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS wallpapers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                filename TEXT NOT NULL,
                original_filename TEXT NOT NULL,
                type TEXT NOT NULL CHECK(type IN ('pc', 'mo')),
                tags TEXT DEFAULT '',
                created_at INTEGER NOT NULL,
                hash TEXT NOT NULL,
                UNIQUE(filename, type),
                UNIQUE(hash, type)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // 为已有的数据库添加 hash 字段（如果不存在）
        sqlx::query(
            r#"
            ALTER TABLE wallpapers ADD COLUMN hash TEXT
            "#,
        )
        .execute(&self.pool)
        .await
        .ok(); // 忽略字段已存在的错误

        // 创建用户表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn has_user(&self) -> Result<bool> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) as count FROM users")
            .fetch_one(&self.pool)
            .await?;
        Ok(count.0 > 0)
    }

    pub async fn create_user(&self, username: &str, password_hash: &str) -> Result<i64> {
        let result =
            sqlx::query("INSERT INTO users (username, password_hash, created_at) VALUES (?, ?, ?)")
                .bind(username)
                .bind(password_hash)
                .bind(chrono::Utc::now().timestamp_millis())
                .execute(&self.pool)
                .await?;

        Ok(result.last_insert_rowid())
    }

    pub async fn get_user_by_username(
        &self,
        username: &str,
    ) -> Result<Option<UserWithPasswordHash>> {
        let user =
            sqlx::query_as::<_, UserWithPasswordHash>("SELECT * FROM users WHERE username = ?")
                .bind(username)
                .fetch_optional(&self.pool)
                .await?;
        Ok(user)
    }

    pub async fn insert_wallpaper(
        &self,
        filename: &str,
        original_filename: &str,
        wallpaper_type: WallpaperType,
        tags: &str,
        created_at: i64,
        hash: &str,
    ) -> Result<i64> {
        let result = sqlx::query(
            "INSERT INTO wallpapers (filename, original_filename, type, tags, created_at, hash) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(filename)
        .bind(original_filename)
        .bind(wallpaper_type.as_str())
        .bind(tags)
        .bind(created_at)
        .bind(hash)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    pub async fn get_random_wallpaper(
        &self,
        wallpaper_type: WallpaperType,
        tag_filter: Option<Vec<String>>,
    ) -> Result<Option<Wallpaper>> {
        let query = if tag_filter.is_some() {
            let mut q = "SELECT * FROM wallpapers WHERE type = ?".to_string();
            if let Some(ref tags) = tag_filter {
                let tag_conditions: Vec<String> =
                    tags.iter().map(|_| "tags LIKE ?".to_string()).collect();
                q.push_str(&format!(" AND ({})", tag_conditions.join(" OR ")));
            }
            q
        } else {
            "SELECT * FROM wallpapers WHERE type = ?".to_string()
        };

        // 使用 ORDER BY RANDOM() LIMIT 1 直接查询一条随机记录
        // 避免加载所有壁纸到内存，大幅减少内存占用
        let query = format!("{} ORDER BY RANDOM() LIMIT 1", query);

        let mut query_builder = sqlx::query_as::<_, Wallpaper>(&query);
        query_builder = query_builder.bind(wallpaper_type.as_str());

        if let Some(ref tags) = tag_filter {
            for tag in tags {
                query_builder = query_builder.bind(format!("%{}%", tag));
            }
        }

        // 直接获取一条记录，而不是加载所有记录
        Ok(query_builder.fetch_optional(&self.pool).await?)
    }

    pub async fn get_all_wallpapers(
        &self,
        wallpaper_type: Option<WallpaperType>,
    ) -> Result<Vec<Wallpaper>> {
        let wallpapers = if let Some(wt) = wallpaper_type {
            sqlx::query_as::<_, Wallpaper>(
                "SELECT * FROM wallpapers WHERE type = ? ORDER BY created_at DESC",
            )
            .bind(wt.as_str())
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, Wallpaper>("SELECT * FROM wallpapers ORDER BY created_at DESC")
                .fetch_all(&self.pool)
                .await?
        };
        Ok(wallpapers)
    }

    pub async fn get_wallpaper_by_filename(
        &self,
        filename: &str,
        wallpaper_type: WallpaperType,
    ) -> Result<Option<Wallpaper>> {
        let wallpaper = sqlx::query_as::<_, Wallpaper>(
            "SELECT * FROM wallpapers WHERE filename = ? AND type = ?",
        )
        .bind(filename)
        .bind(wallpaper_type.as_str())
        .fetch_optional(&self.pool)
        .await?;
        Ok(wallpaper)
    }

    pub async fn get_wallpaper_by_hash(
        &self,
        hash: &str,
        wallpaper_type: &WallpaperType,
    ) -> Result<Option<Wallpaper>> {
        let wallpaper =
            sqlx::query_as::<_, Wallpaper>("SELECT * FROM wallpapers WHERE hash = ? AND type = ?")
                .bind(hash)
                .bind(wallpaper_type.as_str())
                .fetch_optional(&self.pool)
                .await?;
        Ok(wallpaper)
    }

    pub async fn update_wallpaper_tags(&self, id: i64, tags: &str) -> Result<()> {
        sqlx::query("UPDATE wallpapers SET tags = ? WHERE id = ?")
            .bind(tags)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_wallpaper(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM wallpapers WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// 为数据库中哈希值为空的记录计算并填充哈希值
    pub async fn migrate_missing_hashes(&self, wallpaper_dir: &Path) -> Result<()> {
        use std::sync::Arc;
        use tokio::sync::Mutex;

        let wallpapers = self.get_all_wallpapers(None).await.unwrap_or_default();

        let missing_count = wallpapers.iter().filter(|w| w.hash.is_empty()).count();

        if missing_count == 0 {
            println!("✅ 所有图片都已包含哈希值，无需迁移");
            return Ok(());
        }

        println!("🔄 开始迁移 {} 个缺少哈希值的图片...", missing_count);

        let updated = Arc::new(Mutex::new(0));
        let error = Arc::new(Mutex::new(0));

        let mut tasks = Vec::new();

        for wallpaper in wallpapers {
            if wallpaper.hash.is_empty() {
                let db = self.clone();
                let wallpaper_dir = wallpaper_dir.to_path_buf();
                let type_str = wallpaper.wallpaper_type.as_str(); // 使用 &str 避免克隆
                let filename = wallpaper.filename.clone(); // 保留 clone，因为要在任务中使用
                let id = wallpaper.id;
                let updated_counter = updated.clone();
                let err_counter = error.clone();

                let task = tokio::spawn(async move {
                    let file_path = wallpaper_dir.join(type_str).join(&filename);

                    match tokio::task::block_in_place(|| crate::image::calculate_hash(&file_path)) {
                        Ok(hash) => {
                            match sqlx::query("UPDATE wallpapers SET hash = ? WHERE id = ?")
                                .bind(&hash)
                                .bind(id)
                                .execute(&db.pool)
                                .await
                            {
                                Ok(_) => {
                                    let mut c = updated_counter.lock().await;
                                    *c += 1;
                                    println!("  ✅ 更新: {}", filename);
                                }
                                Err(_e) => {
                                    let mut e = err_counter.lock().await;
                                    *e += 1;
                                    println!("  ❌ 更新失败: {} - {}", filename, _e);
                                }
                            }
                        }
                        Err(_e) => {
                            let mut e = err_counter.lock().await;
                            *e += 1;
                            println!("  ❌ 计算哈希失败: {} - {}", filename, _e);
                        }
                    }
                });

                tasks.push(task);
            }
        }

        // 等待所有任务完成
        for task in tasks {
            let _ = task.await;
        }

        let updated_final = *updated.lock().await;
        let error_final = *error.lock().await;

        println!("📊 迁移完成:");
        println!("   ✅ 成功: {}", updated_final);
        println!("   ❌ 失败: {}", error_final);

        Ok(())
    }
}
