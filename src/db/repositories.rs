use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};

use super::models::*;

/// 文章仓库
pub struct PassageRepository;

impl PassageRepository {
    /// 创建文章
    pub fn create(conn: &rusqlite::Connection, passage: &Passage) -> Result<i64, Box<dyn std::error::Error>> {
        conn.execute(
            "INSERT INTO passages (title, content, original_content, summary, author, tags, category, status, file_path, visibility, is_scheduled, published_at, created_at, updated_at) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
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
                &passage.created_at,
                &passage.updated_at,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// 根据 ID 获取文章
    pub fn get_by_id(conn: &rusqlite::Connection, id: i64) -> Result<Option<Passage>, Box<dyn std::error::Error>> {
        let mut stmt = conn.prepare(
            "SELECT id, title, content, original_content, summary, author, tags, category, status, file_path, visibility, is_scheduled, published_at, created_at, updated_at 
             FROM passages WHERE id = ?"
        )?;
        
        let passage = stmt.query_row(params![id], |row| {
            Ok(Passage {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                content: row.get(2)?,
                original_content: row.get(3)?,
                summary: row.get(4)?,
                author: row.get(5)?,
                tags: row.get(6)?,
                category: row.get(7)?,
                status: row.get(8)?,
                file_path: row.get(9)?,
                visibility: row.get(10)?,
                is_scheduled: row.get(11)?,
                published_at: row.get(12)?,
                created_at: row.get(13)?,
                updated_at: row.get(14)?,
            })
        }).optional()?;
        
        Ok(passage)
    }

    /// 获取所有文章
    pub fn get_all(conn: &rusqlite::Connection, limit: i64, offset: i64) -> Result<Vec<Passage>, Box<dyn std::error::Error>> {
        let mut stmt = conn.prepare(
            "SELECT id, title, content, original_content, summary, author, tags, category, status, file_path, visibility, is_scheduled, published_at, created_at, updated_at 
             FROM passages ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )?;
        
        let passages = stmt.query_map(params![limit, offset], |row| {
            Ok(Passage {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                content: row.get(2)?,
                original_content: row.get(3)?,
                summary: row.get(4)?,
                author: row.get(5)?,
                tags: row.get(6)?,
                category: row.get(7)?,
                status: row.get(8)?,
                file_path: row.get(9)?,
                visibility: row.get(10)?,
                is_scheduled: row.get(11)?,
                published_at: row.get(12)?,
                created_at: row.get(13)?,
                updated_at: row.get(14)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        
        Ok(passages)
    }

    /// 获取已发布的文章
    pub fn get_published(conn: &rusqlite::Connection, limit: i64, offset: i64) -> Result<Vec<Passage>, Box<dyn std::error::Error>> {
        let mut stmt = conn.prepare(
            "SELECT id, title, content, original_content, summary, author, tags, category, status, file_path, visibility, is_scheduled, published_at, created_at, updated_at 
             FROM passages WHERE status = 'published' ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )?;
        
        let passages = stmt.query_map(params![limit, offset], |row| {
            Ok(Passage {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                content: row.get(2)?,
                original_content: row.get(3)?,
                summary: row.get(4)?,
                author: row.get(5)?,
                tags: row.get(6)?,
                category: row.get(7)?,
                status: row.get(8)?,
                file_path: row.get(9)?,
                visibility: row.get(10)?,
                is_scheduled: row.get(11)?,
                published_at: row.get(12)?,
                created_at: row.get(13)?,
                updated_at: row.get(14)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        
        Ok(passages)
    }

    /// 更新文章
    pub fn update(conn: &rusqlite::Connection, passage: &Passage) -> Result<(), Box<dyn std::error::Error>> {
        let id = passage.id.ok_or("文章 ID 不能为空")?;
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
    pub fn delete(conn: &rusqlite::Connection, id: i64) -> Result<(), Box<dyn std::error::Error>> {
        conn.execute("DELETE FROM passages WHERE id = ?", params![id])?;
        Ok(())
    }

    /// 获取文章总数
    pub fn count(conn: &rusqlite::Connection) -> Result<i64, Box<dyn std::error::Error>> {
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM passages", [], |row| row.get(0))?;
        Ok(count)
    }
}

/// 用户仓库
pub struct UserRepository;

impl UserRepository {
    /// 创建用户
    pub fn create(conn: &rusqlite::Connection, user: &User) -> Result<i64, Box<dyn std::error::Error>> {
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

    /// 根据用户名获取用户
    pub fn get_by_username(conn: &rusqlite::Connection, username: &str) -> Result<Option<User>, Box<dyn std::error::Error>> {
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
        }).optional()?;
        
        Ok(user)
    }

    /// 根据 ID 获取用户
    pub fn get_by_id(conn: &rusqlite::Connection, id: i64) -> Result<Option<User>, Box<dyn std::error::Error>> {
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
        }).optional()?;
        
        Ok(user)
    }

    /// 更新用户
    pub fn update(conn: &rusqlite::Connection, user: &User) -> Result<(), Box<dyn std::error::Error>> {
        let id = user.id.ok_or("用户 ID 不能为空")?;
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
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value, type, description, category, created_at, updated_at) 
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![
                &setting.key,
                &setting.value,
                &setting.r#type,
                &setting.description,
                &setting.category,
                &setting.created_at,
                &setting.updated_at,
            ],
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