use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use chrono::Utc;
use rusqlite::{params, OptionalExtension};
use std::sync::Arc;

use super::models::*;

/// 生成唯一的 machine ID（基于主机名或随机数）
pub fn get_machine_id() -> [u8; 6] {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    // 尝试获取主机名并哈希
    let hostname = std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "default-host".to_string());
    
    let mut hasher = DefaultHasher::new();
    hostname.hash(&mut hasher);
    let hash = hasher.finish();
    
    // 取哈希值的低6个字节作为 machine ID
    let bytes = hash.to_be_bytes();
    [bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]]
}

/// 仓库 Trait
#[async_trait::async_trait]
pub trait Repository: Send + Sync {
    fn get_pool(&self) -> Arc<Pool<SqliteConnectionManager>>;
}

/// 创建 Repository 实例
pub fn create_repository(pool: Pool<SqliteConnectionManager>) -> Arc<dyn Repository> {
    Arc::new(PassageRepository::new(Arc::new(pool)))
}

/// 文章仓库
pub struct PassageRepository {
    pool: Arc<Pool<SqliteConnectionManager>>,
}

#[async_trait::async_trait]
impl Repository for PassageRepository {
    fn get_pool(&self) -> Arc<Pool<SqliteConnectionManager>> {
        self.pool.clone()
    }
}

impl PassageRepository {
    pub fn new(pool: Arc<Pool<SqliteConnectionManager>>) -> Self {
        Self { pool }
    }

    /// 创建文章
    pub async fn create(&self, passage: &Passage) -> Result<i64, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        
        // 生成 Flake UUID（使用基于主机名的唯一 machine ID）
        let machine_id = get_machine_id();
        let mut flaker = flaker::Flaker::new(machine_id, flaker::Endianness::LittleEndian);
        let uuid = flaker.get_id().map_err(|e| format!("Failed to generate UUID: {:?}", e))?.to_string();
        
        let _ = conn.execute(
            "INSERT INTO passages (uuid, title, content, original_content, summary, author, tags, category, status, file_path, visibility, is_scheduled, published_at, created_at, updated_at) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                &uuid,
                &passage.title,
                &passage.content,
                &passage.original_content,
                &passage.summary,
                &passage.author,
                &passage.tags,
                &passage.category,
                &passage.status,
                &passage.file_path,
                &passage.visibility,
                &passage.is_scheduled,
                &passage.published_at,
                &passage.created_at,
                &passage.updated_at,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// 根据 ID 获取文章
    pub async fn get_by_id(&self, id: i64) -> Result<Passage, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, uuid, title, content, original_content, summary, author, tags, category, status, file_path, visibility, is_scheduled, published_at, created_at, updated_at 
             FROM passages WHERE id = ?"
        )?;
        
        let passage = stmt.query_row(params![id], |row| {
            Ok(Passage {
                id: Some(row.get(0)?),
                uuid: Some(row.get(1)?),
                title: row.get(2)?,
                content: row.get(3)?,
                original_content: row.get(4)?,
                summary: row.get(5)?,
                author: row.get(6)?,
                tags: row.get(7)?,
                category: row.get(8)?,
                status: row.get(9)?,
                file_path: row.get(10)?,
                visibility: row.get(11)?,
                is_scheduled: row.get(12)?,
                published_at: row.get(13)?,
                created_at: row.get(14)?,
                updated_at: row.get(15)?,
            })
        })?;
        
        Ok(passage)
    }

    /// 根据 UUID 获取文章
    pub async fn get_by_uuid(&self, uuid: &str) -> Result<Passage, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, uuid, title, content, original_content, summary, author, tags, category, status, file_path, visibility, is_scheduled, published_at, created_at, updated_at 
             FROM passages WHERE uuid = ?"
        )?;
        
        let passage = stmt.query_row(params![uuid], |row| {
            Ok(Passage {
                id: Some(row.get(0)?),
                uuid: Some(row.get(1)?),
                title: row.get(2)?,
                content: row.get(3)?,
                original_content: row.get(4)?,
                summary: row.get(5)?,
                author: row.get(6)?,
                tags: row.get(7)?,
                category: row.get(8)?,
                status: row.get(9)?,
                file_path: row.get(10)?,
                visibility: row.get(11)?,
                is_scheduled: row.get(12)?,
                published_at: row.get(13)?,
                created_at: row.get(14)?,
                updated_at: row.get(15)?,
            })
        })?;
        
        Ok(passage)
    }

    /// 根据文件路径获取文章
    pub async fn get_by_file_path(&self, file_path: &str) -> Result<Passage, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, uuid, title, content, original_content, summary, author, tags, category, status, file_path, visibility, is_scheduled, published_at, created_at, updated_at 
             FROM passages WHERE file_path = ?"
        )?;
        
        let passage = stmt.query_row(params![file_path], |row| {
            Ok(Passage {
                id: Some(row.get(0)?),
                uuid: Some(row.get(1)?),
                title: row.get(2)?,
                content: row.get(3)?,
                original_content: row.get(4)?,
                summary: row.get(5)?,
                author: row.get(6)?,
                tags: row.get(7)?,
                category: row.get(8)?,
                status: row.get(9)?,
                file_path: row.get(10)?,
                visibility: row.get(11)?,
                is_scheduled: row.get(12)?,
                published_at: row.get(13)?,
                created_at: row.get(14)?,
                updated_at: row.get(15)?,
            })
        })?;
        
        Ok(passage)
    }

    /// 获取所有文章
    pub async fn get_all(&self, limit: i64, offset: i64) -> Result<Vec<Passage>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, uuid, title, content, original_content, summary, author, tags, category, status, file_path, visibility, is_scheduled, published_at, created_at, updated_at 
             FROM passages ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )?;
        
        let passages = stmt.query_map(params![limit, offset], |row| {
            Ok(Passage {
                id: Some(row.get(0)?),
                uuid: Some(row.get(1)?),
                title: row.get(2)?,
                content: row.get(3)?,
                original_content: row.get(4)?,
                summary: row.get(5)?,
                author: row.get(6)?,
                tags: row.get(7)?,
                category: row.get(8)?,
                status: row.get(9)?,
                file_path: row.get(10)?,
                visibility: row.get(11)?,
                is_scheduled: row.get(12)?,
                published_at: row.get(13)?,
                created_at: row.get(14)?,
                updated_at: row.get(15)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        
        Ok(passages)
    }

    /// 获取已发布的文章
    pub async fn get_published(&self, limit: i64, offset: i64) -> Result<Vec<Passage>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, uuid, title, content, original_content, summary, author, tags, category, status, file_path, visibility, is_scheduled, published_at, created_at, updated_at 
             FROM passages WHERE status = 'published' ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )?;
        
        let passages = stmt.query_map(params![limit, offset], |row| {
            Ok(Passage {
                id: Some(row.get(0)?),
                uuid: Some(row.get(1)?),
                title: row.get(2)?,
                content: row.get(3)?,
                original_content: row.get(4)?,
                summary: row.get(5)?,
                author: row.get(6)?,
                tags: row.get(7)?,
                category: row.get(8)?,
                status: row.get(9)?,
                file_path: row.get(10)?,
                visibility: row.get(11)?,
                is_scheduled: row.get(12)?,
                published_at: row.get(13)?,
                created_at: row.get(14)?,
                updated_at: row.get(15)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        
        Ok(passages)
    }

    /// 更新文章
    pub async fn update(&self, passage: &Passage) -> Result<(), Box<dyn std::error::Error>> {
        let id = passage.id.ok_or("文章 ID 不能为空")?;
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE passages SET title = ?, content = ?, original_content = ?, summary = ?, author = ?, tags = ?, category = ?, status = ?, file_path = ?, visibility = ?, is_scheduled = ?, published_at = ?, updated_at = ? 
             WHERE id = ?",
            params![
                &passage.title,
                &passage.content,
                &passage.original_content,
                &passage.summary,
                &passage.author,
                &passage.tags,
                &passage.category,
                &passage.status,
                &passage.file_path,
                &passage.visibility,
                &passage.is_scheduled,
                &passage.published_at,
                &passage.updated_at,
                id,
            ],
        )?;
        Ok(())
    }

    /// 删除文章
    pub async fn delete(&self, id: i64) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM passages WHERE id = ?", params![id])?;
        Ok(())
    }

    /// 获取文章总数
    pub async fn count(&self) -> Result<i64, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM passages", [], |row| row.get(0))?;
        Ok(count)
    }

    /// 获取已发布文章总数
    pub async fn count_published(&self) -> Result<i64, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM passages WHERE status = 'published'", [], |row| row.get(0))?;
        Ok(count)
    }

    /// 获取所有分类
    pub async fn get_all_categories(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT DISTINCT category FROM passages WHERE category IS NOT NULL AND category != '' ORDER BY category")?;
        let categories = stmt.query_map([], |row| row.get(0))?.collect::<Result<Vec<_>, _>>()?;
        Ok(categories)
    }
}

/// 评论仓库
pub struct CommentRepository {
    pool: Arc<Pool<SqliteConnectionManager>>,
}

impl CommentRepository {
    pub fn new(pool: Arc<Pool<SqliteConnectionManager>>) -> Self {
        Self { pool }
    }

    /// 创建评论
    pub async fn create(&self, comment: &Comment) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO comments (username, content, passage_uuid, created_at) VALUES (?, ?, ?, ?)",
            params![
                &comment.username,
                &comment.content,
                &comment.passage_uuid,
                &comment.created_at,
            ],
        )?;
        Ok(())
    }

    /// 根据 ID 获取评论
    pub async fn get_by_id(&self, id: i64) -> Result<Comment, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, username, content, passage_uuid, created_at FROM comments WHERE id = ?"
        )?;
        
        let comment = stmt.query_row(params![id], |row| {
            Ok(Comment {
                id: Some(row.get(0)?),
                username: row.get(1)?,
                content: row.get(2)?,
                passage_uuid: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;
        
        Ok(comment)
    }

    /// 根据文章 UUID 获取评论
    pub async fn get_by_passage_uuid(&self, passage_uuid: &str, limit: i64, offset: i64) -> Result<Vec<Comment>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, username, content, passage_uuid, created_at FROM comments WHERE passage_uuid = ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )?;
        
        let comments = stmt.query_map(params![passage_uuid, limit, offset], |row| {
            Ok(Comment {
                id: Some(row.get(0)?),
                username: row.get(1)?,
                content: row.get(2)?,
                passage_uuid: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        
        Ok(comments)
    }

    /// 获取所有评论
    pub async fn get_all(&self, limit: i64, offset: i64) -> Result<Vec<Comment>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, username, content, passage_uuid, created_at FROM comments ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )?;
        
        let comments = stmt.query_map(params![limit, offset], |row| {
            Ok(Comment {
                id: Some(row.get(0)?),
                username: row.get(1)?,
                content: row.get(2)?,
                passage_uuid: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        
        Ok(comments)
    }

    /// 删除评论
    pub async fn delete(&self, id: i64) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM comments WHERE id = ?", params![id])?;
        Ok(())
    }

    /// 获取评论总数
    pub async fn count(&self) -> Result<i64, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM comments", [], |row| row.get(0))?;
        Ok(count)
    }

    /// 根据文章 UUID 获取评论数
    pub async fn count_by_passage_uuid(&self, passage_uuid: &str) -> Result<i64, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM comments WHERE passage_uuid = ?", params![passage_uuid], |row| row.get(0))?;
        Ok(count)
    }
}

/// 文章阅读记录仓库
pub struct ArticleViewRepository {
    pool: Arc<Pool<SqliteConnectionManager>>,
}

impl ArticleViewRepository {
    pub fn new(pool: Arc<Pool<SqliteConnectionManager>>) -> Self {
        Self { pool }
    }

    /// 记录文章阅读
    pub async fn record_view(&self, passage_uuid: &str, ip: &str, user_agent: Option<&str>, country: &str, city: &str, region: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let now = Utc::now();
        let view_date = now.format("%Y-%m-%d").to_string();
        
        conn.execute(
            "INSERT INTO article_views (passage_uuid, ip, user_agent, country, city, region, view_date, view_time, created_at) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                passage_uuid,
                ip,
                user_agent,
                country,
                city,
                region,
                view_date,
                now,
                now,
            ],
        )?;
        Ok(())
    }

    /// 获取最多阅读的文章
    pub async fn get_most_viewed_articles(&self, limit: i64) -> Result<Vec<PopularArticleStats>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT p.id, p.title, p.author, COUNT(av.id) as view_count FROM passages p 
             LEFT JOIN article_views av ON p.uuid = av.passage_uuid 
             GROUP BY p.id ORDER BY view_count DESC LIMIT ?"
        )?;
        
        let articles = stmt.query_map(params![limit], |row| {
            Ok(PopularArticleStats {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                author: row.get(2)?,
                view_count: row.get(3)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        
        Ok(articles)
    }

    /// 获取阅读来源（按国家统计）
    pub async fn get_view_sources(&self, days: i64) -> Result<Vec<ViewSourceStats>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT country, COUNT(*) as count FROM article_views 
             WHERE view_date >= date('now', ? || ' days') 
             GROUP BY country ORDER BY count DESC"
        )?;
        
        let sources = stmt.query_map(params![-days], |row| {
            Ok(ViewSourceStats {
                country: row.get(0)?,
                count: row.get(1)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        
        Ok(sources)
    }

    /// 获取阅读趋势
    pub async fn get_view_trend(&self, days: i64) -> Result<Vec<ViewTrendStats>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT view_date, COUNT(*) as count FROM article_views 
             WHERE view_date >= date('now', ? || ' days') 
             GROUP BY view_date ORDER BY view_date"
        )?;
        
        let trend = stmt.query_map(params![-days], |row| {
            Ok(ViewTrendStats {
                date: row.get(0)?,
                count: row.get(1)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        
        Ok(trend)
    }

    /// 获取单篇文章的统计信息
    pub async fn get_article_stats(&self, passage_uuid: &str, days: i64) -> Result<ArticleStatsData, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        
        // 获取文章信息
        let passage = self.pool.get()?.query_row(
            "SELECT id, title FROM passages WHERE uuid = ?",
            params![passage_uuid],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        )?;
        
        // 获取总浏览量
        let total_views: i64 = conn.query_row(
            "SELECT COUNT(*) FROM article_views WHERE passage_uuid = ? AND view_date >= date('now', ? || ' days')",
            params![passage_uuid, -days],
            |row| row.get(0)
        )?;
        
        // 获取独立访客数
        let unique_visitors: i64 = conn.query_row(
            "SELECT COUNT(DISTINCT ip) FROM article_views WHERE passage_uuid = ? AND view_date >= date('now', ? || ' days')",
            params![passage_uuid, -days],
            |row| row.get(0)
        )?;
        
        // 获取平均停留时间
        let avg_duration: f64 = conn.query_row(
            "SELECT AVG(duration) FROM article_views WHERE passage_uuid = ? AND view_date >= date('now', ? || ' days')",
            params![passage_uuid, -days],
            |row| row.get(0)
        ).unwrap_or(0.0);
        
        Ok(ArticleStatsData {
            article_id: passage.0,
            title: passage.1,
            total_views,
            unique_visitors,
            avg_duration,
        })
    }

    /// 获取按城市统计的阅读数据
    pub async fn get_view_by_city(&self, days: i64) -> Result<Vec<CityStatsData>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT city, country, COUNT(*) as count FROM article_views 
             WHERE view_date >= date('now', ? || ' days') 
             GROUP BY city, country ORDER BY count DESC"
        )?;
        
        let cities = stmt.query_map(params![-days], |row| {
            Ok(CityStatsData {
                city: row.get(0)?,
                country: row.get(1)?,
                count: row.get(2)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        
        Ok(cities)
    }

    /// 获取按IP统计的访问数据
    pub async fn get_view_by_ip(&self, days: i64) -> Result<Vec<IPStatsData>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT ip, country, city, region, COUNT(*) as count, 
                    MIN(view_time) as first_visit, MAX(view_time) as last_visit 
             FROM article_views 
             WHERE view_date >= date('now', ? || ' days') 
             GROUP BY ip, country, city, region ORDER BY count DESC LIMIT 100"
        )?;
        
        let ips = stmt.query_map(params![-days], |row| {
            Ok(IPStatsData {
                ip: row.get(0)?,
                country: row.get(1)?,
                city: row.get(2)?,
                region: row.get(3)?,
                count: row.get(4)?,
                first_visit: row.get(5)?,
                last_visit: row.get(6)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        
        Ok(ips)
    }
}

/// 统计数据结构
#[derive(Debug)]
pub struct PopularArticleStats {
    pub id: Option<i64>,
    pub title: String,
    pub author: Option<String>,
    pub view_count: i64,
}

#[derive(Debug)]
pub struct ViewSourceStats {
    pub country: String,
    pub count: i64,
}

#[derive(Debug)]
pub struct ViewTrendStats {
    pub date: String,
    pub count: i64,
}

#[derive(Debug)]
pub struct ArticleStatsData {
    pub article_id: i64,
    pub title: String,
    pub total_views: i64,
    pub unique_visitors: i64,
    pub avg_duration: f64,
}

#[derive(Debug)]
pub struct CityStatsData {
    pub city: String,
    pub country: String,
    pub count: i64,
}

#[derive(Debug)]
pub struct IPStatsData {
    pub ip: String,
    pub country: String,
    pub city: String,
    pub region: String,
    pub count: i64,
    pub first_visit: String,
    pub last_visit: String,
}

/// 设置仓库
pub struct SettingRepository;

impl SettingRepository {
    /// 获取设置值
    pub fn get(conn: &rusqlite::Connection, key: &str) -> Result<Option<Setting>, Box<dyn std::error::Error>> {
        let mut stmt = conn.prepare(
            "SELECT id, key, value, type, description, category, created_at, updated_at 
             FROM settings WHERE key = ?"
        )?;
        
        let setting = stmt.query_row(params![key], |row| {
            Ok(Setting {
                id: Some(row.get(0)?),
                key: row.get(1)?,
                value: row.get(2)?,
                r#type: row.get(3)?,
                description: row.get(4)?,
                category: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        }).optional()?;
        
        Ok(setting)
    }

    /// 设置值
    pub fn set(conn: &rusqlite::Connection, setting: &Setting) -> Result<(), Box<dyn std::error::Error>> {
        // 使用 query_row 执行 INSERT OR REPLACE，因为它可能返回结果
        let _ = conn.query_row(
            "INSERT OR REPLACE INTO settings (key, value, type, description, category, created_at, updated_at) 
             VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING 1",
            params![
                &setting.key,
                &setting.value,
                &setting.r#type,
                &setting.description,
                &setting.category,
                &setting.created_at,
                &setting.updated_at,
            ],
            |_| Ok(()),
        )?;
        Ok(())
    }

    /// 获取所有设置
    pub fn get_all(conn: &rusqlite::Connection) -> Result<Vec<Setting>, Box<dyn std::error::Error>> {
        let mut stmt = conn.prepare(
            "SELECT id, key, value, type, description, category, created_at, updated_at 
             FROM settings ORDER BY category, key"
        )?;
        
        let settings = stmt.query_map([], |row| {
            Ok(Setting {
                id: Some(row.get(0)?),
                key: row.get(1)?,
                value: row.get(2)?,
                r#type: row.get(3)?,
                description: row.get(4)?,
                category: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        
        Ok(settings)
    }

    /// 根据分类获取设置
    pub fn get_by_category(conn: &rusqlite::Connection, category: &str) -> Result<Vec<Setting>, Box<dyn std::error::Error>> {
        let mut stmt = conn.prepare(
            "SELECT id, key, value, type, description, category, created_at, updated_at 
             FROM settings WHERE category = ? ORDER BY key"
        )?;
        
        let settings = stmt.query_map(params![category], |row| {
            Ok(Setting {
                id: Some(row.get(0)?),
                key: row.get(1)?,
                value: row.get(2)?,
                r#type: row.get(3)?,
                description: row.get(4)?,
                category: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        
        Ok(settings)
    }
}

/// 分类仓库
pub struct CategoryRepository {
    pool: Arc<Pool<SqliteConnectionManager>>,
}

impl CategoryRepository {
    pub fn new(pool: Arc<Pool<SqliteConnectionManager>>) -> Self {
        Self { pool }
    }

    /// 创建分类
    pub async fn create(&self, category: &Category) -> Result<i64, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO categories (name, description, icon, sort_order, is_enabled, created_at, updated_at) 
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![
                &category.name,
                &category.description,
                &category.icon,
                &category.sort_order,
                &category.is_enabled,
                &category.created_at,
                &category.updated_at,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// 根据 ID 获取分类
    pub async fn get_by_id(&self, id: i64) -> Result<Category, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, icon, sort_order, is_enabled, created_at, updated_at 
             FROM categories WHERE id = ?"
        )?;
        
        let category = stmt.query_row(params![id], |row| {
            Ok(Category {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
                icon: row.get(3)?,
                sort_order: row.get(4)?,
                is_enabled: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;
        
        Ok(category)
    }

    /// 获取所有分类
    pub async fn get_all(&self, limit: i64, offset: i64) -> Result<Vec<Category>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, icon, sort_order, is_enabled, created_at, updated_at 
             FROM categories ORDER BY sort_order ASC, created_at DESC LIMIT ? OFFSET ?"
        )?;
        
        let categories = stmt.query_map(params![limit, offset], |row| {
            Ok(Category {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
                icon: row.get(3)?,
                sort_order: row.get(4)?,
                is_enabled: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        
        Ok(categories)
    }

    /// 获取所有分类（不分页）
    pub async fn get_all_without_pagination(&self) -> Result<Vec<Category>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, icon, sort_order, is_enabled, created_at, updated_at 
             FROM categories ORDER BY sort_order ASC, created_at DESC"
        )?;
        
        let categories = stmt.query_map([], |row| {
            Ok(Category {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
                icon: row.get(3)?,
                sort_order: row.get(4)?,
                is_enabled: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        
        Ok(categories)
    }

    /// 更新分类
    pub async fn update(&self, category: &Category) -> Result<(), Box<dyn std::error::Error>> {
        let id = category.id.ok_or("分类 ID 不能为空")?;
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE categories SET name = ?, description = ?, icon = ?, sort_order = ?, is_enabled = ?, updated_at = ? 
             WHERE id = ?",
            params![
                &category.name,
                &category.description,
                &category.icon,
                &category.sort_order,
                &category.is_enabled,
                &category.updated_at,
                id,
            ],
        )?;
        Ok(())
    }

    /// 删除分类
    pub async fn delete(&self, id: i64) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM categories WHERE id = ?", params![id])?;
        Ok(())
    }

    /// 获取分类总数
    pub async fn count(&self) -> Result<i64, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM categories", [], |row| row.get(0))?;
        Ok(count)
    }
}

/// 标签仓库
pub struct TagRepository {
    pool: Arc<Pool<SqliteConnectionManager>>,
}

impl TagRepository {
    pub fn new(pool: Arc<Pool<SqliteConnectionManager>>) -> Self {
        Self { pool }
    }

    /// 创建标签
    pub async fn create(&self, tag: &Tag) -> Result<i64, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO tags (name, description, color, category_id, sort_order, is_enabled, created_at, updated_at) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                &tag.name,
                &tag.description,
                &tag.color,
                &tag.category_id,
                &tag.sort_order,
                &tag.is_enabled,
                &tag.created_at,
                &tag.updated_at,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// 根据 ID 获取标签
    pub async fn get_by_id(&self, id: i64) -> Result<Tag, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, color, category_id, sort_order, is_enabled, created_at, updated_at 
             FROM tags WHERE id = ?"
        )?;
        
        let tag = stmt.query_row(params![id], |row| {
            Ok(Tag {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
                color: row.get(3)?,
                category_id: row.get(4)?,
                sort_order: row.get(5)?,
                is_enabled: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;
        
        Ok(tag)
    }

    /// 根据名称获取标签
    pub async fn get_by_name(&self, name: &str) -> Result<Tag, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, color, category_id, sort_order, is_enabled, created_at, updated_at 
             FROM tags WHERE name = ?"
        )?;
        
        let tag = stmt.query_row(params![name], |row| {
            Ok(Tag {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
                color: row.get(3)?,
                category_id: row.get(4)?,
                sort_order: row.get(5)?,
                is_enabled: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;
        
        Ok(tag)
    }

    /// 获取所有标签
    pub async fn get_all(&self, limit: i64, offset: i64) -> Result<Vec<Tag>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, color, category_id, sort_order, is_enabled, created_at, updated_at 
             FROM tags ORDER BY sort_order ASC, created_at DESC LIMIT ? OFFSET ?"
        )?;
        
        let tags = stmt.query_map(params![limit, offset], |row| {
            Ok(Tag {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
                color: row.get(3)?,
                category_id: row.get(4)?,
                sort_order: row.get(5)?,
                is_enabled: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        
        Ok(tags)
    }

    /// 获取所有标签（不分页）
    pub async fn get_all_without_pagination(&self) -> Result<Vec<Tag>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, color, category_id, sort_order, is_enabled, created_at, updated_at 
             FROM tags ORDER BY sort_order ASC, created_at DESC"
        )?;
        
        let tags = stmt.query_map([], |row| {
            Ok(Tag {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
                color: row.get(3)?,
                category_id: row.get(4)?,
                sort_order: row.get(5)?,
                is_enabled: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        
        Ok(tags)
    }

    /// 更新标签
    pub async fn update(&self, tag: &Tag) -> Result<(), Box<dyn std::error::Error>> {
        let id = tag.id.ok_or("标签 ID 不能为空")?;
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE tags SET name = ?, description = ?, color = ?, category_id = ?, sort_order = ?, is_enabled = ?, updated_at = ? 
             WHERE id = ?",
            params![
                &tag.name,
                &tag.description,
                &tag.color,
                &tag.category_id,
                &tag.sort_order,
                &tag.is_enabled,
                &tag.updated_at,
                id,
            ],
        )?;
        Ok(())
    }

    /// 删除标签
    pub async fn delete(&self, id: i64) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM tags WHERE id = ?", params![id])?;
        Ok(())
    }

    /// 获取标签总数
    pub async fn count(&self) -> Result<i64, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM tags", [], |row| row.get(0))?;
        Ok(count)
    }
}

/// 用户仓库
pub struct UserRepository {
    pool: Arc<Pool<SqliteConnectionManager>>,
}

impl UserRepository {
    pub fn new(pool: Arc<Pool<SqliteConnectionManager>>) -> Self {
        Self { pool }
    }

    /// 创建用户
    pub async fn create(&self, user: &User) -> Result<i64, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO users (username, password, email, role, status, created_at, updated_at) 
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![
                &user.username,
                &user.password,
                &user.email,
                &user.role,
                &user.status,
                &user.created_at,
                &user.updated_at,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// 根据 ID 获取用户
    pub async fn get_by_id(&self, id: i64) -> Result<User, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, username, password, email, role, status, created_at, updated_at 
             FROM users WHERE id = ?"
        )?;
        
        let user = stmt.query_row(params![id], |row| {
            Ok(User {
                id: Some(row.get(0)?),
                username: row.get(1)?,
                password: row.get(2)?,
                email: row.get(3)?,
                role: row.get(4)?,
                status: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;
        
        Ok(user)
    }

    /// 根据用户名获取用户
    pub async fn get_by_username(&self, username: &str) -> Result<User, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, username, password, email, role, status, created_at, updated_at 
             FROM users WHERE username = ?"
        )?;
        
        let user = stmt.query_row(params![username], |row| {
            Ok(User {
                id: Some(row.get(0)?),
                username: row.get(1)?,
                password: row.get(2)?,
                email: row.get(3)?,
                role: row.get(4)?,
                status: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;
        
        Ok(user)
    }

    /// 获取所有用户
    pub async fn get_all(&self, limit: i64, offset: i64) -> Result<Vec<User>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, username, password, email, role, status, created_at, updated_at 
             FROM users ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )?;
        
        let users = stmt.query_map(params![limit, offset], |row| {
            Ok(User {
                id: Some(row.get(0)?),
                username: row.get(1)?,
                password: row.get(2)?,
                email: row.get(3)?,
                role: row.get(4)?,
                status: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        
        Ok(users)
    }

    /// 更新用户
    pub async fn update(&self, user: &User) -> Result<(), Box<dyn std::error::Error>> {
        let id = user.id.ok_or("用户 ID 不能为空")?;
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE users SET username = ?, password = ?, email = ?, role = ?, status = ?, updated_at = ? 
             WHERE id = ?",
            params![
                &user.username,
                &user.password,
                &user.email,
                &user.role,
                &user.status,
                &user.updated_at,
                id,
            ],
        )?;
        Ok(())
    }

    /// 删除用户
    pub async fn delete(&self, id: i64) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM users WHERE id = ?", params![id])?;
        Ok(())
    }

    /// 获取用户总数
    pub async fn count(&self) -> Result<i64, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))?;
        Ok(count)
    }
}

/// 音乐轨道仓库
pub struct MusicTrackRepository {
    pool: Arc<Pool<SqliteConnectionManager>>,
}

impl MusicTrackRepository {
    pub fn new(pool: Arc<Pool<SqliteConnectionManager>>) -> Self {
        Self { pool }
    }

    pub async fn get_all_without_pagination(&self) -> Result<Vec<MusicTrack>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, artist, file_path, file_name, duration, cover_image, created_at 
             FROM music_tracks ORDER BY created_at DESC"
        )?;
        
        let tracks = stmt.query_map([], |row| {
            Ok(MusicTrack {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                artist: row.get(2)?,
                file_path: row.get(3)?,
                file_name: row.get(4)?,
                duration: row.get(5)?,
                cover_image: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        
        Ok(tracks)
    }

    pub async fn create(&self, track: &MusicTrack) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let _ = conn.execute(
            "INSERT INTO music_tracks (title, artist, file_path, file_name, duration, cover_image, created_at) 
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![
                &track.title,
                &track.artist,
                &track.file_path,
                &track.file_name,
                &track.duration,
                &track.cover_image,
                &track.created_at,
            ],
        )?;
        Ok(())
    }

    pub async fn update_title(&self, id: i64, title: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE music_tracks SET title = ? WHERE id = ?",
            params![title, id],
        )?;
        Ok(())
    }

    pub async fn update_cover(&self, id: i64, cover_image: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE music_tracks SET cover_image = ? WHERE id = ?",
            params![cover_image, id],
        )?;
        Ok(())
    }

    pub async fn update_cover_by_filename(&self, file_name: &str, cover_image: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE music_tracks SET cover_image = ? WHERE file_name = ?",
            params![cover_image, file_name],
        )?;
        Ok(())
    }

    pub async fn get_by_id(&self, id: i64) -> Result<MusicTrack, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, artist, file_path, file_name, duration, cover_image, created_at 
             FROM music_tracks WHERE id = ?"
        )?;
        
        let track = stmt.query_row(params![id], |row| {
            Ok(MusicTrack {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                artist: row.get(2)?,
                file_path: row.get(3)?,
                file_name: row.get(4)?,
                duration: row.get(5)?,
                cover_image: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?;
        
        Ok(track)
    }

    pub async fn delete(&self, id: i64) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM music_tracks WHERE id = ?", params![id])?;
        Ok(())
    }
}

/// 附件仓库
pub struct AttachmentRepository {
    pool: Arc<Pool<SqliteConnectionManager>>,
}

impl AttachmentRepository {
    pub fn new(pool: Arc<Pool<SqliteConnectionManager>>) -> Self {
        Self { pool }
    }

    pub async fn get_all(&self, limit: i64, offset: i64) -> Result<Vec<Attachment>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, file_name, stored_name, file_path, file_type, content_type, file_size, passage_uuid, visibility, show_in_passage, uploaded_at 
             FROM attachments ORDER BY uploaded_at DESC LIMIT ? OFFSET ?"
        )?;
        
        let attachments = stmt.query_map(params![limit, offset], |row| {
            Ok(Attachment {
                id: Some(row.get(0)?),
                file_name: row.get(1)?,
                stored_name: row.get(2)?,
                file_path: row.get(3)?,
                file_type: row.get(4)?,
                content_type: row.get(5)?,
                file_size: row.get(6)?,
                passage_uuid: row.get(7)?,
                visibility: row.get(8)?,
                show_in_passage: row.get(9)?,
                uploaded_at: row.get(10)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        
        Ok(attachments)
    }

    pub async fn create(&self, attachment: &Attachment) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO attachments (file_name, stored_name, file_path, file_type, content_type, file_size, passage_uuid, visibility, show_in_passage, uploaded_at) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                &attachment.file_name,
                &attachment.stored_name,
                &attachment.file_path,
                &attachment.file_type,
                &attachment.content_type,
                &attachment.file_size,
                &attachment.passage_uuid,
                &attachment.visibility,
                &attachment.show_in_passage,
                &attachment.uploaded_at,
            ],
        )?;
        Ok(())
    }

    pub async fn count(&self) -> Result<i64, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM attachments", [], |row| row.get(0))?;
        Ok(count)
    }

    pub async fn get_by_id(&self, id: i64) -> Result<Attachment, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, file_name, stored_name, file_path, file_type, content_type, file_size, passage_uuid, visibility, show_in_passage, uploaded_at 
             FROM attachments WHERE id = ?"
        )?;
        
        let attachment = stmt.query_row(params![id], |row| {
            Ok(Attachment {
                id: Some(row.get(0)?),
                file_name: row.get(1)?,
                stored_name: row.get(2)?,
                file_path: row.get(3)?,
                file_type: row.get(4)?,
                content_type: row.get(5)?,
                file_size: row.get(6)?,
                passage_uuid: row.get(7)?,
                visibility: row.get(8)?,
                show_in_passage: row.get(9)?,
                uploaded_at: row.get(10)?,
            })
        })?;
        
        Ok(attachment)
    }

    pub async fn update(&self, attachment: &Attachment) -> Result<(), Box<dyn std::error::Error>> {
        let id = attachment.id.ok_or("附件 ID 不能为空")?;
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE attachments SET visibility = ?, show_in_passage = ? WHERE id = ?",
            params![&attachment.visibility, &attachment.show_in_passage, id],
        )?;
        Ok(())
    }

    pub async fn delete(&self, id: i64) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM attachments WHERE id = ?", params![id])?;
        Ok(())
    }
}

/// 关于页面主卡片仓库
pub struct AboutMainCardRepository {
    pool: Arc<Pool<SqliteConnectionManager>>,
}

impl AboutMainCardRepository {
    pub fn new(pool: Arc<Pool<SqliteConnectionManager>>) -> Self {
        Self { pool }
    }

    pub async fn get_all(&self) -> Result<Vec<AboutMainCard>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, icon, layout_type, custom_css, sort_order, is_enabled, created_at, updated_at 
             FROM about_main_cards ORDER BY sort_order"
        )?;
        
        let cards = stmt.query_map([], |row| {
            Ok(AboutMainCard {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                icon: row.get(2)?,
                layout_type: row.get(3)?,
                custom_css: row.get(4)?,
                sort_order: row.get(5)?,
                is_enabled: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        
        Ok(cards)
    }

    pub async fn get_by_id(&self, id: i64) -> Result<AboutMainCard, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, icon, layout_type, custom_css, sort_order, is_enabled, created_at, updated_at 
             FROM about_main_cards WHERE id = ?"
        )?;
        
        let card = stmt.query_row(params![id], |row| {
            Ok(AboutMainCard {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                icon: row.get(2)?,
                layout_type: row.get(3)?,
                custom_css: row.get(4)?,
                sort_order: row.get(5)?,
                is_enabled: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;
        
        Ok(card)
    }

    pub async fn create(&self, card: &AboutMainCard) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO about_main_cards (title, icon, layout_type, custom_css, sort_order, is_enabled, created_at, updated_at) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                &card.title,
                &card.icon,
                &card.layout_type,
                &card.custom_css,
                &card.sort_order,
                &card.is_enabled,
                &card.created_at,
                &card.updated_at,
            ],
        )?;
        Ok(())
    }

    pub async fn update(&self, card: &AboutMainCard) -> Result<(), Box<dyn std::error::Error>> {
        let id = card.id.ok_or("主卡片 ID 不能为空")?;
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE about_main_cards SET title = ?, icon = ?, layout_type = ?, custom_css = ?, sort_order = ?, is_enabled = ?, updated_at = ? 
             WHERE id = ?",
            params![
                &card.title,
                &card.icon,
                &card.layout_type,
                &card.custom_css,
                &card.sort_order,
                &card.is_enabled,
                &card.updated_at,
                id,
            ],
        )?;
        Ok(())
    }

    pub async fn delete(&self, id: i64) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM about_main_cards WHERE id = ?", params![id])?;
        Ok(())
    }
}

/// 关于页面次卡片仓库
pub struct AboutSubCardRepository {
    pool: Arc<Pool<SqliteConnectionManager>>,
}

impl AboutSubCardRepository {
    pub fn new(pool: Arc<Pool<SqliteConnectionManager>>) -> Self {
        Self { pool }
    }

    pub async fn get_all(&self) -> Result<Vec<AboutSubCard>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, main_card_id, title, description, icon, link_url, layout_type, custom_css, sort_order, is_enabled, created_at, updated_at 
             FROM about_sub_cards ORDER BY sort_order"
        )?;
        
        let cards = stmt.query_map([], |row| {
            Ok(AboutSubCard {
                id: Some(row.get(0)?),
                main_card_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                icon: row.get(4)?,
                link_url: row.get(5)?,
                layout_type: row.get(6)?,
                custom_css: row.get(7)?,
                sort_order: row.get(8)?,
                is_enabled: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        
        Ok(cards)
    }

    pub async fn get_by_id(&self, id: i64) -> Result<AboutSubCard, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, main_card_id, title, description, icon, link_url, layout_type, custom_css, sort_order, is_enabled, created_at, updated_at 
             FROM about_sub_cards WHERE id = ?"
        )?;
        
        let card = stmt.query_row(params![id], |row| {
            Ok(AboutSubCard {
                id: Some(row.get(0)?),
                main_card_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                icon: row.get(4)?,
                link_url: row.get(5)?,
                layout_type: row.get(6)?,
                custom_css: row.get(7)?,
                sort_order: row.get(8)?,
                is_enabled: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })?;
        
        Ok(card)
    }

    pub async fn create(&self, card: &AboutSubCard) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO about_sub_cards (main_card_id, title, description, icon, link_url, layout_type, custom_css, sort_order, is_enabled, created_at, updated_at) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                &card.main_card_id,
                &card.title,
                &card.description,
                &card.icon,
                &card.link_url,
                &card.layout_type,
                &card.custom_css,
                &card.sort_order,
                &card.is_enabled,
                &card.created_at,
                &card.updated_at,
            ],
        )?;
        Ok(())
    }

    pub async fn update(&self, card: &AboutSubCard) -> Result<(), Box<dyn std::error::Error>> {
        let id = card.id.ok_or("次卡片 ID 不能为空")?;
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE about_sub_cards SET main_card_id = ?, title = ?, description = ?, icon = ?, link_url = ?, layout_type = ?, custom_css = ?, sort_order = ?, is_enabled = ?, updated_at = ? 
             WHERE id = ?",
            params![
                &card.main_card_id,
                &card.title,
                &card.description,
                &card.icon,
                &card.link_url,
                &card.layout_type,
                &card.custom_css,
                &card.sort_order,
                &card.is_enabled,
                &card.updated_at,
                id,
            ],
        )?;
        Ok(())
    }

    pub async fn delete(&self, id: i64) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM about_sub_cards WHERE id = ?", params![id])?;
        Ok(())
    }
}