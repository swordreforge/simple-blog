use anyhow::{Context, Result};
use sqlx::{sqlite::SqlitePool, Pool, Sqlite};
use std::path::PathBuf;

use crate::models::{UserWithPasswordHash, Wallpaper, WallpaperType};

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
                UNIQUE(filename, type)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

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

    pub async fn create_user(
        &self,
        username: &str,
        password_hash: &str,
    ) -> Result<i64> {
        let result = sqlx::query(
            "INSERT INTO users (username, password_hash, created_at) VALUES (?, ?, ?)"
        )
        .bind(username)
        .bind(password_hash)
        .bind(chrono::Utc::now().timestamp_millis())
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<UserWithPasswordHash>> {
        let user = sqlx::query_as::<_, UserWithPasswordHash>(
            "SELECT * FROM users WHERE username = ?"
        )
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
    ) -> Result<i64> {
        let result = sqlx::query(
            "INSERT INTO wallpapers (filename, original_filename, type, tags, created_at) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(filename)
        .bind(original_filename)
        .bind(wallpaper_type.as_str())
        .bind(tags)
        .bind(created_at)
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
                let tag_conditions: Vec<String> = tags.iter().map(|_| "tags LIKE ?".to_string()).collect();
                q.push_str(&format!(" AND ({})", tag_conditions.join(" OR ")));
            }
            q
        } else {
            "SELECT * FROM wallpapers WHERE type = ?".to_string()
        };

        let mut query_builder = sqlx::query_as::<_, Wallpaper>(&query);
        query_builder = query_builder.bind(wallpaper_type.as_str());

        if let Some(ref tags) = tag_filter {
            for tag in tags {
                query_builder = query_builder.bind(format!("%{}%", tag));
            }
        }

        let wallpapers: Vec<Wallpaper> = query_builder.fetch_all(&self.pool).await?;

        if wallpapers.is_empty() {
            return Ok(None);
        }

        use rand::seq::SliceRandom;
        Ok(wallpapers.choose(&mut rand::thread_rng()).cloned())
    }

    pub async fn get_all_wallpapers(
        &self,
        wallpaper_type: Option<WallpaperType>,
    ) -> Result<Vec<Wallpaper>> {
        let wallpapers = if let Some(wt) = wallpaper_type {
            sqlx::query_as::<_, Wallpaper>("SELECT * FROM wallpapers WHERE type = ? ORDER BY created_at DESC")
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
            "SELECT * FROM wallpapers WHERE filename = ? AND type = ?"
        )
        .bind(filename)
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
}