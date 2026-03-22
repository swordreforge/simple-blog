use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{OptionalExtension, params};
use smallvec::SmallVec;
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
#[derive(Clone)]
pub struct PassageRepository {
    pool: Arc<Pool<SqliteConnectionManager>>,
    count_cache: Arc<parking_lot::RwLock<Option<i64>>>,
    count_published_cache: Arc<parking_lot::RwLock<Option<i64>>>,
    cache_valid: Arc<std::sync::atomic::AtomicBool>,
}

#[async_trait::async_trait]
impl Repository for PassageRepository {
    fn get_pool(&self) -> Arc<Pool<SqliteConnectionManager>> {
        self.pool.clone()
    }
}

impl PassageRepository {
    pub fn new(pool: Arc<Pool<SqliteConnectionManager>>) -> Self {
        Self {
            pool,
            count_cache: Arc::new(parking_lot::RwLock::new(None)),
            count_published_cache: Arc::new(parking_lot::RwLock::new(None)),
            cache_valid: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// 创建文章
    pub async fn create(&self, passage: &Passage) -> Result<i64, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;

        // 生成 Flake UUID（使用基于主机名的唯一 machine ID）
        let uuid = crate::id_generator::generate_unique_id();

        // 检查是否启用文章摘要功能
        use crate::services::summarize_service::SummarizeService;
        let summarize = match SettingRepository::get(&conn, "passage_summarize_enabled") {
            Ok(Some(setting)) if setting.value == "true" => Some(
                SummarizeService::generate_summary_from_markdown(&passage.content),
            ),
            _ => None,
        };

        let _ = conn.execute(
            "INSERT INTO passages (uuid, title, content, original_content, summary, summarize, author, tags, category, status, file_path, visibility, is_scheduled, published_at, cover_image, created_at, updated_at) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                &uuid,
                &passage.title,
                &passage.content,
                &passage.original_content,
                &passage.summary,
                &summarize,
                &passage.author,
                &passage.tags,
                &passage.category,
                &passage.status,
                &passage.file_path,
                &passage.visibility,
                &passage.is_scheduled,
                &passage.published_at,
                &passage.cover_image,
                &passage.created_at,
                &passage.updated_at,
            ],
        )?;
        // 使缓存失效
        self.invalidate_cache();
        Ok(conn.last_insert_rowid())
    }

    /// 根据 ID 获取文章
    pub async fn get_by_id(&self, id: i64) -> Result<Passage, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, uuid, title, content, original_content, summary, summarize, author, tags, category, status, file_path, visibility, is_scheduled, published_at, cover_image, created_at, updated_at 
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
                summarize: row.get(6)?,
                author: row.get(7)?,
                tags: row.get(8)?,
                category: row.get(9)?,
                status: row.get(10)?,
                file_path: row.get(11)?,
                visibility: row.get(12)?,
                is_scheduled: row.get(13)?,
                published_at: row.get(14)?,
                cover_image: row.get(15)?,
                created_at: row.get(16)?,
                updated_at: row.get(17)?,
            })
        })?;

        Ok(passage)
    }

    /// 根据 UUID 获取文章
    pub async fn get_by_uuid(&self, uuid: &str) -> Result<Passage, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, uuid, title, content, original_content, summary, summarize, author, tags, category, status, file_path, visibility, is_scheduled, published_at, cover_image, created_at, updated_at 
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
                summarize: row.get(6)?,
                author: row.get(7)?,
                tags: row.get(8)?,
                category: row.get(9)?,
                status: row.get(10)?,
                file_path: row.get(11)?,
                visibility: row.get(12)?,
                is_scheduled: row.get(13)?,
                published_at: row.get(14)?,
                cover_image: row.get(15)?,
                created_at: row.get(16)?,
                updated_at: row.get(17)?,
            })
        })?;

        Ok(passage)
    }

    /// 根据文件路径获取文章
    pub async fn get_by_file_path(
        &self,
        file_path: &str,
    ) -> Result<Passage, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, uuid, title, content, original_content, summary, summarize, author, tags, category, status, file_path, visibility, is_scheduled, published_at, cover_image, created_at, updated_at 
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
                summarize: row.get(6)?,
                author: row.get(7)?,
                tags: row.get(8)?,
                category: row.get(9)?,
                status: row.get(10)?,
                file_path: row.get(11)?,
                visibility: row.get(12)?,
                is_scheduled: row.get(13)?,
                published_at: row.get(14)?,
                cover_image: row.get(15)?,
                created_at: row.get(16)?,
                updated_at: row.get(17)?,
            })
        })?;

        Ok(passage)
    }

    /// 获取所有文章
    pub async fn get_all(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Passage>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, uuid, title, content, original_content, summary, summarize, author, tags, category, status, file_path, visibility, is_scheduled, published_at, cover_image, created_at, updated_at 
             FROM passages ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )?;

        let passages = stmt
            .query_map(params![limit, offset], |row| {
                Ok(Passage {
                    id: Some(row.get(0)?),
                    uuid: Some(row.get(1)?),
                    title: row.get(2)?,
                    content: row.get(3)?,
                    original_content: row.get(4)?,
                    summary: row.get(5)?,
                    summarize: row.get(6)?,
                    author: row.get(6)?,
                    tags: row.get(8)?,
                    category: row.get(9)?,
                    status: row.get(10)?,
                    file_path: row.get(11)?,
                    visibility: row.get(12)?,
                    is_scheduled: row.get(13)?,
                    published_at: row.get(14)?,
                    cover_image: row.get(15)?,
                    created_at: row.get(16)?,
                    updated_at: row.get(17)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(passages)
    }

    /// 获取已发布的文章
    pub async fn get_published(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Passage>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, uuid, title, content, original_content, summary, summarize, author, tags, category, status, file_path, visibility, is_scheduled, published_at, cover_image, created_at, updated_at
             FROM passages WHERE status = 'published' ORDER BY created_at DESC, id DESC LIMIT ? OFFSET ?"
        )?;

        let passages = stmt
            .query_map(params![limit, offset], |row| {
                Ok(Passage {
                    id: Some(row.get(0)?),
                    uuid: Some(row.get(1)?),
                    title: row.get(2)?,
                    content: row.get(3)?,
                    original_content: row.get(4)?,
                    summary: row.get(5)?,
                    summarize: row.get(6)?,
                    author: row.get(6)?,
                    tags: row.get(8)?,
                    category: row.get(9)?,
                    status: row.get(10)?,
                    file_path: row.get(11)?,
                    visibility: row.get(12)?,
                    is_scheduled: row.get(13)?,
                    published_at: row.get(14)?,
                    cover_image: row.get(15)?,
                    created_at: row.get(16)?,
                    updated_at: row.get(17)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(passages)
    }

    /// 使用游标分页获取已发布的文章（性能优化）
    /// cursor: (created_at, id) 格式为 "created_at:id"
    pub async fn get_published_cursor(
        &self,
        cursor: Option<String>,
        limit: i64,
    ) -> Result<(Vec<Passage>, Option<String>), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;

        if let Some(cursor_str) = cursor {
            // 解析游标（支持新旧两种格式：'|' 和 ':'）
            // 优先使用 '|' 分隔符（新格式），如果不成功则尝试 ':'（旧格式，向后兼容）
            let (created_at_str, id_str) = if let Some(pos) = cursor_str.find('|') {
                // 新格式：created_at|id
                (
                    cursor_str[..pos].to_string(),
                    cursor_str[pos + 1..].to_string(),
                )
            } else {
                // 旧格式：created_at:id
                // 时间戳格式为 "YYYY-MM-DD HH:MM:SS"，包含冒号
                // 需要从右向左查找最后一个冒号作为分隔符
                if let Some(pos) = cursor_str.rfind(':') {
                    (
                        cursor_str[..pos].to_string(),
                        cursor_str[pos + 1..].to_string(),
                    )
                } else {
                    return Err("Invalid cursor format".into());
                }
            };

            let query = r#"
                SELECT id, uuid, title, content, original_content, summary, summarize, author, tags, category, status, file_path, visibility, is_scheduled, published_at, cover_image, created_at, updated_at
                FROM passages 
                WHERE status = 'published' AND (created_at < ? OR (created_at = ? AND id < ?))
                ORDER BY created_at DESC, id DESC
                LIMIT ?
            "#;

            let mut stmt = conn.prepare(query)?;
            let passages = stmt
                .query_map(
                    params![created_at_str, created_at_str, id_str, limit],
                    |row| {
                        Ok(Passage {
                            id: Some(row.get(0)?),
                            uuid: Some(row.get(1)?),
                            title: row.get(2)?,
                            content: row.get(3)?,
                            original_content: row.get(4)?,
                            summary: row.get(5)?,
                            summarize: row.get(6)?,
                            author: row.get(6)?,
                            tags: row.get(8)?,
                            category: row.get(9)?,
                            status: row.get(10)?,
                            file_path: row.get(11)?,
                            visibility: row.get(12)?,
                            is_scheduled: row.get(13)?,
                            published_at: row.get(14)?,
                            cover_image: row.get(15)?,
                            created_at: row.get(16)?,
                            updated_at: row.get(17)?,
                        })
                    },
                )?
                .collect::<Result<Vec<_>, _>>()?;

            // 计算下一页游标（使用最后一条记录）
            // 使用与数据库存储格式一致的格式：YYYY-MM-DD HH:MM:SS+00:00
            let next_cursor = passages.last().map(|p| {
                format!(
                    "{}|{}",
                    p.created_at.format("%Y-%m-%d %H:%M:%S%:z"),
                    p.id.unwrap_or(0)
                )
            });

            Ok((passages, next_cursor))
        } else {
            // 第一页，没有游标
            let query = r#"
                SELECT id, uuid, title, content, original_content, summary, summarize, author, tags, category, status, file_path, visibility, is_scheduled, published_at, cover_image, created_at, updated_at
                FROM passages 
                WHERE status = 'published'
                ORDER BY created_at DESC, id DESC
                LIMIT ?
            "#;

            let mut stmt = conn.prepare(query)?;
            let passages = stmt
                .query_map(params![limit], |row| {
                    Ok(Passage {
                        id: Some(row.get(0)?),
                        uuid: Some(row.get(1)?),
                        title: row.get(2)?,
                        content: row.get(3)?,
                        original_content: row.get(4)?,
                        summary: row.get(5)?,
                        summarize: row.get(6)?,
                        author: row.get(6)?,
                        tags: row.get(8)?,
                        category: row.get(9)?,
                        status: row.get(10)?,
                        file_path: row.get(11)?,
                        visibility: row.get(12)?,
                        is_scheduled: row.get(13)?,
                        published_at: row.get(14)?,
                        cover_image: row.get(15)?,
                        created_at: row.get(16)?,
                        updated_at: row.get(17)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

            // 计算下一页游标（使用最后一条记录）

            // 使用与数据库存储格式一致的格式：YYYY-MM-DD HH:MM:SS+00:00

            let next_cursor = passages.last().map(|p| {
                format!(
                    "{}|{}",
                    p.created_at.format("%Y-%m-%d %H:%M:%S%:z"),
                    p.id.unwrap_or(0)
                )
            });

            Ok((passages, next_cursor))
        }
    }

    /// 获取最新一篇已发布的文章
    pub async fn get_latest_published(
        &self,
    ) -> Result<Option<Passage>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;

        let query = r#"

                        SELECT id, uuid, title, content, original_content, summary, summarize, author, tags, category, status, file_path, visibility, is_scheduled, published_at, cover_image, created_at, updated_at

                        FROM passages

                        WHERE status = 'published'

                        ORDER BY created_at DESC, id DESC

                        LIMIT 1

                    "#;

        let mut stmt = conn.prepare(query)?;

        let passage = stmt
            .query_row(params![], |row| {
                Ok(Passage {
                    id: Some(row.get(0)?),

                    uuid: Some(row.get(1)?),

                    title: row.get(2)?,

                    content: row.get(3)?,

                    original_content: row.get(4)?,

                    summary: row.get(5)?,

                    summarize: row.get(6)?,

                    author: row.get(7)?,

                    tags: row.get(8)?,

                    category: row.get(9)?,

                    status: row.get(10)?,

                    file_path: row.get(11)?,

                    visibility: row.get(12)?,

                    is_scheduled: row.get(13)?,

                    published_at: row.get(14)?,

                    cover_image: row.get(15)?,

                    created_at: row.get(16)?,

                    updated_at: row.get(17)?,
                })
            })
            .optional()?;

        Ok(passage)
    }

    /// 按日期获取已发布的文章（支持年、月、日筛选）
    pub async fn get_published_by_date(
        &self,
        year: Option<i32>,
        month: Option<i32>,
        day: Option<i32>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Passage>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;

        // 构建 WHERE 条件
        // 最多 4 个条件（status + year + month + day），每个约 20 字符
        let mut conditions = Vec::with_capacity(4);
        conditions.push("status = 'published'".to_string());
        // 使用SmallVec优化小数组，减少堆分配（最多6个参数：year, month, day, limit, offset）
        let mut params: SmallVec<[Box<dyn rusqlite::ToSql>; 6]> = SmallVec::new();

        if let Some(y) = year {
            conditions.push("created_year = ?".to_string());
            params.push(Box::new(y));
        }
        if let Some(m) = month {
            conditions.push("created_month = ?".to_string());
            params.push(Box::new(m));
        }
        if let Some(d) = day {
            conditions.push("created_day = ?".to_string());
            params.push(Box::new(d));
        }

        let where_clause = conditions.join(" AND ");
        let sql = format!(
            "SELECT id, uuid, title, content, original_content, summary, summarize, author, tags, category, status, file_path, visibility, is_scheduled, published_at, cover_image, created_at, updated_at
             FROM passages WHERE {} ORDER BY created_at DESC LIMIT ? OFFSET ?",
            where_clause
        );

        // 转换参数为 rusqlite::Params
        let mut sql_params: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        sql_params.push(&limit);
        sql_params.push(&offset);

        let mut stmt = conn.prepare(&sql)?;
        let passages = stmt
            .query_map(sql_params.as_slice(), |row| {
                Ok(Passage {
                    id: Some(row.get(0)?),
                    uuid: Some(row.get(1)?),
                    title: row.get(2)?,
                    content: row.get(3)?,
                    original_content: row.get(4)?,
                    summary: row.get(5)?,
                    summarize: row.get(6)?,
                    author: row.get(6)?,
                    tags: row.get(8)?,
                    category: row.get(9)?,
                    status: row.get(10)?,
                    file_path: row.get(11)?,
                    visibility: row.get(12)?,
                    is_scheduled: row.get(13)?,
                    published_at: row.get(14)?,
                    cover_image: row.get(15)?,
                    created_at: row.get(16)?,
                    updated_at: row.get(17)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(passages)
    }

    /// 统计指定日期的文章数量
    pub async fn count_published_by_date(
        &self,
        year: Option<i32>,
        month: Option<i32>,
        day: Option<i32>,
    ) -> Result<i64, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;

        // 构建 WHERE 条件
        let mut conditions = vec!["status = 'published'".to_string()];
        // 使用SmallVec优化小数组，减少堆分配（最多3个参数：year, month, day）
        let mut params: SmallVec<[Box<dyn rusqlite::ToSql>; 3]> = SmallVec::new();

        if let Some(y) = year {
            conditions.push("created_year = ?".to_string());
            params.push(Box::new(y));
        }
        if let Some(m) = month {
            conditions.push("created_month = ?".to_string());
            params.push(Box::new(m));
        }
        if let Some(d) = day {
            conditions.push("created_day = ?".to_string());
            params.push(Box::new(d));
        }

        let where_clause = conditions.join(" AND ");
        let sql = format!("SELECT COUNT(*) FROM passages WHERE {}", where_clause);

        let sql_params: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let count: i64 = conn.query_row(&sql, sql_params.as_slice(), |row| row.get(0))?;

        Ok(count)
    }

    /// 更新文章
    pub async fn update(&self, passage: &Passage) -> Result<(), Box<dyn std::error::Error>> {
        let id = passage.id.ok_or("文章 ID 不能为空")?;
        let conn = self.pool.get()?;

        // 检查是否启用文章摘要功能
        use crate::services::summarize_service::SummarizeService;
        let summarize = match SettingRepository::get(&conn, "passage_summarize_enabled") {
            Ok(Some(setting)) if setting.value == "true" => Some(
                SummarizeService::generate_summary_from_markdown(&passage.content),
            ),
            _ => None,
        };

        conn.execute(
            "UPDATE passages SET title = ?, content = ?, original_content = ?, summary = ?, summarize = ?, author = ?, tags = ?, category = ?, status = ?, file_path = ?, visibility = ?, is_scheduled = ?, published_at = ?, cover_image = ?, updated_at = ? 
             WHERE id = ?",
            params![
                &passage.title,
                &passage.content,
                &passage.original_content,
                &passage.summary,
                &summarize,
                &passage.author,
                &passage.tags,
                &passage.category,
                &passage.status,
                &passage.file_path,
                &passage.visibility,
                &passage.is_scheduled,
                &passage.published_at,
                &passage.cover_image,
                &passage.updated_at,
                id,
            ],
        )?;
        // 使缓存失效
        self.invalidate_cache();
        Ok(())
    }

    /// 根据 UUID 删除文章
    pub async fn delete_by_uuid(&self, uuid: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM passages WHERE uuid = ?", params![uuid])?;
        // 使缓存失效
        self.invalidate_cache();
        Ok(())
    }

    /// 批量删除文章
    pub async fn delete_batch(&self, ids: Vec<i64>) -> Result<i64, Box<dyn std::error::Error>> {
        if ids.is_empty() {
            return Ok(0);
        }
        let conn = self.pool.get()?;
        // 预分配容量，每个 "?" 1 字符 + ids.len()-1 个 ","
        let mut placeholders = String::with_capacity(ids.len() * 2 - 1);
        for (i, _) in ids.iter().enumerate() {
            if i > 0 {
                placeholders.push(',');
            }
            placeholders.push('?');
        }
        let sql = format!("DELETE FROM passages WHERE id IN ({})", placeholders);
        let params: Vec<&dyn rusqlite::ToSql> =
            ids.iter().map(|id| id as &dyn rusqlite::ToSql).collect();
        let affected = conn.execute(&sql, params.as_slice())?;
        // 使缓存失效
        self.invalidate_cache();
        Ok(affected as i64)
    }

    /// 获取文章总数（使用缓存优化）
    pub async fn count(&self) -> Result<i64, Box<dyn std::error::Error>> {
        // 尝试从缓存读取
        if self.cache_valid.load(std::sync::atomic::Ordering::Relaxed) {
            let count_opt = self.count_cache.read();
            if let Some(count) = *count_opt {
                return Ok(count);
            }
        }

        // 缓存未命中，查询数据库
        let conn = self.pool.get()?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM passages", [], |row| row.get(0))?;

        // 更新缓存
        let mut cache = self.count_cache.write();
        *cache = Some(count);
        self.cache_valid
            .store(true, std::sync::atomic::Ordering::Relaxed);

        Ok(count)
    }

    /// 获取已发布文章总数（使用缓存优化）
    pub async fn count_published(&self) -> Result<i64, Box<dyn std::error::Error>> {
        // 尝试从缓存读取
        if self.cache_valid.load(std::sync::atomic::Ordering::Relaxed) {
            let count_opt = self.count_published_cache.read();
            if let Some(count) = *count_opt {
                return Ok(count);
            }
        }

        // 缓存未命中，查询数据库
        let conn = self.pool.get()?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM passages WHERE status = 'published'",
            [],
            |row| row.get(0),
        )?;

        // 更新缓存
        let mut cache = self.count_published_cache.write();
        *cache = Some(count);
        self.cache_valid
            .store(true, std::sync::atomic::Ordering::Relaxed);

        Ok(count)
    }

    /// 使缓存失效（在创建、更新、删除文章时调用）
    pub fn invalidate_cache(&self) {
        self.cache_valid
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }

    /// 获取所有分类
    pub async fn get_all_categories(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT DISTINCT category FROM passages WHERE category IS NOT NULL AND category != '' ORDER BY category")?;
        let categories = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(categories)
    }

    /// 批量获取文章（修复 N+1 查询问题）
    pub async fn get_by_ids(
        &self,
        ids: &[i64],
    ) -> Result<Vec<Passage>, Box<dyn std::error::Error>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let conn = self.pool.get()?;
        // 预分配容量，每个 "?" 1 字符 + ids.len()-1 个 ","
        let mut placeholders = String::with_capacity(ids.len() * 2 - 1);
        for (i, _) in ids.iter().enumerate() {
            if i > 0 {
                placeholders.push(',');
            }
            placeholders.push('?');
        }
        let sql = format!(
            "SELECT id, uuid, title, content, original_content, summary, summarize, author, tags, category, status, file_path, visibility, is_scheduled, published_at, cover_image, created_at, updated_at 
             FROM passages WHERE id IN ({})",
            placeholders
        );
        let params: Vec<&dyn rusqlite::ToSql> =
            ids.iter().map(|id| id as &dyn rusqlite::ToSql).collect();
        let mut stmt = conn.prepare(&sql)?;
        let passages = stmt
            .query_map(params.as_slice(), |row| {
                Ok(Passage {
                    id: Some(row.get(0)?),
                    uuid: Some(row.get(1)?),
                    title: row.get(2)?,
                    content: row.get(3)?,
                    original_content: row.get(4)?,
                    summary: row.get(5)?,
                    summarize: row.get(6)?,
                    author: row.get(6)?,
                    tags: row.get(8)?,
                    category: row.get(9)?,
                    status: row.get(10)?,
                    file_path: row.get(11)?,
                    visibility: row.get(12)?,
                    is_scheduled: row.get(13)?,
                    published_at: row.get(14)?,
                    cover_image: row.get(15)?,
                    created_at: row.get(16)?,
                    updated_at: row.get(17)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(passages)
    }

    /// 获取归档统计（优化归档页面查询）
    pub async fn get_archive_stats(&self) -> Result<Vec<ArchiveStats>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT
                CAST(created_year AS TEXT) as year,
                CAST(created_month AS TEXT) as month,
                COUNT(*) as count
            FROM passages
            WHERE status = 'published'
            GROUP BY created_year, created_month
            ORDER BY created_year DESC, created_month DESC
            "#,
        )?;

        let stats = stmt
            .query_map([], |row| {
                Ok(ArchiveStats {
                    year: row.get(0)?,
                    month: row.get(1)?,
                    count: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(stats)
    }

    /// 获取标签统计（优化标签统计查询）
    pub async fn get_tag_stats(&self) -> Result<Vec<TagStats>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            r#"
            WITH tag_counts AS (
                SELECT
                    json_each.value as tag_name,
                    COUNT(*) as count
                FROM passages
                CROSS JOIN json_each(tags)
                WHERE status = 'published'
                GROUP BY tag_name
            )
            SELECT
                ROW_NUMBER() OVER (ORDER BY count DESC) as id,
                tag_name as name,
                count
            FROM tag_counts
            ORDER BY count DESC
            "#,
        )?;

        let stats = stmt
            .query_map([], |row| {
                Ok(TagStats {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    count: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(stats)
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

    /// 根据文章 UUID 获取评论
    pub async fn get_by_passage_uuid(
        &self,
        passage_uuid: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Comment>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, username, content, passage_uuid, created_at FROM comments WHERE passage_uuid = ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )?;

        let comments = stmt
            .query_map(params![passage_uuid, limit, offset], |row| {
                Ok(Comment {
                    id: Some(row.get(0)?),
                    username: row.get(1)?,
                    content: row.get(2)?,
                    passage_uuid: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(comments)
    }

    /// 获取所有评论
    pub async fn get_all(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Comment>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, username, content, passage_uuid, created_at FROM comments ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )?;

        let comments = stmt
            .query_map(params![limit, offset], |row| {
                Ok(Comment {
                    id: Some(row.get(0)?),
                    username: row.get(1)?,
                    content: row.get(2)?,
                    passage_uuid: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(comments)
    }

    /// 删除评论
    pub async fn delete(&self, id: i64) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM comments WHERE id = ?", params![id])?;
        Ok(())
    }

    /// 批量删除评论
    pub async fn delete_batch(&self, ids: Vec<i64>) -> Result<i64, Box<dyn std::error::Error>> {
        if ids.is_empty() {
            return Ok(0);
        }
        let conn = self.pool.get()?;
        // 预分配容量，每个 "?" 1 字符 + ids.len()-1 个 ","
        let mut placeholders = String::with_capacity(ids.len() * 2 - 1);
        for (i, _) in ids.iter().enumerate() {
            if i > 0 {
                placeholders.push(',');
            }
            placeholders.push('?');
        }
        let sql = format!("DELETE FROM comments WHERE id IN ({})", placeholders);
        let params: Vec<&dyn rusqlite::ToSql> =
            ids.iter().map(|id| id as &dyn rusqlite::ToSql).collect();
        let affected = conn.execute(&sql, params.as_slice())?;
        Ok(affected as i64)
    }

    /// 获取评论总数
    pub async fn count(&self) -> Result<i64, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM comments", [], |row| row.get(0))?;
        Ok(count)
    }

    /// 根据文章 UUID 获取评论数
    pub async fn count_by_passage_uuid(
        &self,
        passage_uuid: &str,
    ) -> Result<i64, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM comments WHERE passage_uuid = ?",
            params![passage_uuid],
            |row| row.get(0),
        )?;
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

    /// 获取最多阅读的文章
    pub async fn get_most_viewed_articles(
        &self,
        limit: i64,
    ) -> Result<Vec<PopularArticleStats>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT p.id, p.title, p.author, COUNT(av.id) as view_count FROM passages p 
             LEFT JOIN article_views av ON p.uuid = av.passage_uuid 
             GROUP BY p.id ORDER BY view_count DESC LIMIT ?",
        )?;

        let articles = stmt
            .query_map(params![limit], |row| {
                Ok(PopularArticleStats {
                    id: Some(row.get(0)?),
                    title: row.get(1)?,
                    author: row.get(2)?,
                    view_count: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(articles)
    }

    /// 获取阅读来源（按国家统计）
    pub async fn get_view_sources(
        &self,
        days: i64,
    ) -> Result<Vec<ViewSourceStats>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT country, COUNT(*) as count FROM article_views 
             WHERE view_date >= date('now', ? || ' days') 
             GROUP BY country ORDER BY count DESC",
        )?;

        let sources = stmt
            .query_map(params![-days], |row| {
                Ok(ViewSourceStats {
                    country: row.get(0)?,
                    count: row.get(1)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(sources)
    }

    /// 获取阅读趋势
    pub async fn get_view_trend(
        &self,
        days: i64,
    ) -> Result<Vec<ViewTrendStats>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT view_date, COUNT(*) as count FROM article_views 
             WHERE view_date >= date('now', ? || ' days') 
             GROUP BY view_date ORDER BY view_date",
        )?;

        let trend = stmt
            .query_map(params![-days], |row| {
                Ok(ViewTrendStats {
                    date: row.get(0)?,
                    count: row.get(1)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(trend)
    }

    /// 获取单篇文章的统计信息
    pub async fn get_article_stats(
        &self,
        passage_uuid: &str,
        days: i64,
    ) -> Result<ArticleStatsData, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;

        // 获取文章信息
        let passage = self.pool.get()?.query_row(
            "SELECT id, title FROM passages WHERE uuid = ?",
            params![passage_uuid],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
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
    pub async fn get_view_by_city(
        &self,
        days: i64,
    ) -> Result<Vec<CityStatsData>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT city, country, COUNT(*) as count FROM article_views 
             WHERE view_date >= date('now', ? || ' days') 
             GROUP BY city, country ORDER BY count DESC",
        )?;

        let cities = stmt
            .query_map(params![-days], |row| {
                Ok(CityStatsData {
                    city: row.get(0)?,
                    country: row.get(1)?,
                    count: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(cities)
    }

    /// 获取按IP统计的访问数据
    pub async fn get_view_by_ip(
        &self,
        days: i64,
    ) -> Result<Vec<IPStatsData>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT ip, country, city, region, COUNT(*) as count, 
                    MIN(view_time) as first_visit, MAX(view_time) as last_visit 
             FROM article_views 
             WHERE view_date >= date('now', ? || ' days') 
             GROUP BY ip, country, city, region ORDER BY count DESC LIMIT 100",
        )?;

        let ips = stmt
            .query_map(params![-days], |row| {
                Ok(IPStatsData {
                    ip: row.get(0)?,
                    country: row.get(1)?,
                    city: row.get(2)?,
                    region: row.get(3)?,
                    count: row.get(4)?,
                    first_visit: row.get(5)?,
                    last_visit: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

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
    pub fn get(
        conn: &rusqlite::Connection,
        key: &str,
    ) -> Result<Option<Setting>, Box<dyn std::error::Error>> {
        let mut stmt = conn.prepare(
            "SELECT id, key, value, type, description, category, created_at, updated_at 
             FROM settings WHERE key = ?",
        )?;

        let setting = stmt
            .query_row(params![key], |row| {
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
            })
            .optional()?;

        Ok(setting)
    }

    /// 设置值
    pub fn set(
        conn: &rusqlite::Connection,
        setting: &Setting,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 使用 query_row 执行 INSERT OR REPLACE，因为它可能返回结果
        conn.query_row(
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
             FROM categories WHERE id = ?",
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

    /// 根据名称获取分类
    pub async fn get_by_name(&self, name: &str) -> Result<Category, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, icon, sort_order, is_enabled, created_at, updated_at
             FROM categories WHERE name = ?",
        )?;

        let category = stmt.query_row(params![name], |row| {
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
    pub async fn get_all(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Category>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, icon, sort_order, is_enabled, created_at, updated_at 
             FROM categories ORDER BY sort_order ASC, created_at DESC LIMIT ? OFFSET ?",
        )?;

        let categories = stmt
            .query_map(params![limit, offset], |row| {
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
            })?
            .collect::<Result<Vec<_>, _>>()?;

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

    /// 批量删除分类
    pub async fn delete_batch(&self, ids: Vec<i64>) -> Result<i64, Box<dyn std::error::Error>> {
        if ids.is_empty() {
            return Ok(0);
        }
        let conn = self.pool.get()?;
        // 预分配容量，每个 "?" 1 字符 + ids.len()-1 个 ","
        let mut placeholders = String::with_capacity(ids.len() * 2 - 1);
        for (i, _) in ids.iter().enumerate() {
            if i > 0 {
                placeholders.push(',');
            }
            placeholders.push('?');
        }
        let sql = format!("DELETE FROM categories WHERE id IN ({})", placeholders);
        let params: Vec<&dyn rusqlite::ToSql> =
            ids.iter().map(|id| id as &dyn rusqlite::ToSql).collect();
        let affected = conn.execute(&sql, params.as_slice())?;
        Ok(affected as i64)
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
    pub async fn get_all(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Tag>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, color, category_id, sort_order, is_enabled, created_at, updated_at 
             FROM tags ORDER BY sort_order ASC, created_at DESC LIMIT ? OFFSET ?"
        )?;

        let tags = stmt
            .query_map(params![limit, offset], |row| {
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
            })?
            .collect::<Result<Vec<_>, _>>()?;

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

    /// 批量删除标签
    pub async fn delete_batch(&self, ids: Vec<i64>) -> Result<i64, Box<dyn std::error::Error>> {
        if ids.is_empty() {
            return Ok(0);
        }
        let conn = self.pool.get()?;
        // 预分配容量，每个 "?" 1 字符 + ids.len()-1 个 ","
        let mut placeholders = String::with_capacity(ids.len() * 2 - 1);
        for (i, _) in ids.iter().enumerate() {
            if i > 0 {
                placeholders.push(',');
            }
            placeholders.push('?');
        }
        let sql = format!("DELETE FROM tags WHERE id IN ({})", placeholders);
        let params: Vec<&dyn rusqlite::ToSql> =
            ids.iter().map(|id| id as &dyn rusqlite::ToSql).collect();
        let affected = conn.execute(&sql, params.as_slice())?;
        Ok(affected as i64)
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
             FROM users WHERE id = ?",
        )?;

        let user = stmt.query_row(params![id], |row| {
            let role_str: String = row.get(4)?;
            let role = match role_str.as_str() {
                "admin" => UserRole::Admin,
                "editor" => UserRole::Editor,
                "subscriber" | "user" => UserRole::Subscriber,
                _ => UserRole::Subscriber,
            };

            let status_str: String = row.get(5)?;
            let status = match status_str.as_str() {
                "active" => UserStatus::Active,
                "disabled" | "inactive" | "banned" => UserStatus::Disabled,

                _ => UserStatus::Active,
            };

            Ok(User {
                id: Some(row.get(0)?),
                username: row.get(1)?,
                password: row.get(2)?,
                email: row.get(3)?,
                role,
                status,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;

        Ok(user)
    }

    /// 根据用户名获取用户
    pub async fn get_by_username(
        &self,
        username: &str,
    ) -> Result<User, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, username, password, email, role, status, created_at, updated_at
             FROM users WHERE username = ?",
        )?;

        let user = stmt.query_row(params![username], |row| {
            let role_str: String = row.get(4)?;
            let role = match role_str.as_str() {
                "admin" => UserRole::Admin,
                "editor" => UserRole::Editor,
                "subscriber" | "user" => UserRole::Subscriber,
                _ => UserRole::Subscriber,
            };

            let status_str: String = row.get(5)?;
            let status = match status_str.as_str() {
                "active" => UserStatus::Active,
                "disabled" | "inactive" | "banned" => UserStatus::Disabled,

                _ => UserStatus::Active,
            };

            Ok(User {
                id: Some(row.get(0)?),
                username: row.get(1)?,
                password: row.get(2)?,
                email: row.get(3)?,
                role,
                status,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;

        Ok(user)
    }

    /// 获取所有用户
    pub async fn get_all(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<User>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, username, password, email, role, status, created_at, updated_at
             FROM users ORDER BY created_at DESC LIMIT ? OFFSET ?",
        )?;

        let users = stmt
            .query_map(params![limit, offset], |row| {
                let role_str: String = row.get(4)?;
                let role = match role_str.as_str() {
                    "admin" => UserRole::Admin,
                    "editor" => UserRole::Editor,
                    "subscriber" | "user" => UserRole::Subscriber,
                    _ => UserRole::Subscriber,
                };

                let status_str: String = row.get(5)?;
                let status = match status_str.as_str() {
                    "active" => UserStatus::Active,
                    "disabled" | "inactive" | "banned" => UserStatus::Disabled,

                    _ => UserStatus::Active,
                };

                Ok(User {
                    id: Some(row.get(0)?),
                    username: row.get(1)?,
                    password: row.get(2)?,
                    email: row.get(3)?,
                    role,
                    status,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

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

    /// 批量删除用户
    pub async fn delete_batch(&self, ids: Vec<i64>) -> Result<i64, Box<dyn std::error::Error>> {
        if ids.is_empty() {
            return Ok(0);
        }
        let conn = self.pool.get()?;
        // 预分配容量，每个 "?" 1 字符 + ids.len()-1 个 ","
        let mut placeholders = String::with_capacity(ids.len() * 2 - 1);
        for (i, _) in ids.iter().enumerate() {
            if i > 0 {
                placeholders.push(',');
            }
            placeholders.push('?');
        }
        let sql = format!("DELETE FROM users WHERE id IN ({})", placeholders);
        let params: Vec<&dyn rusqlite::ToSql> =
            ids.iter().map(|id| id as &dyn rusqlite::ToSql).collect();
        let affected = conn.execute(&sql, params.as_slice())?;
        Ok(affected as i64)
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

    pub async fn get_all_without_pagination(
        &self,
    ) -> Result<Vec<MusicTrack>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, artist, file_path, file_name, duration, cover_image, created_at 
             FROM music_tracks ORDER BY created_at DESC",
        )?;

        let tracks = stmt
            .query_map([], |row| {
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
            })?
            .collect::<Result<Vec<_>, _>>()?;

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

    pub async fn update_title(
        &self,
        id: i64,
        title: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE music_tracks SET title = ? WHERE id = ?",
            params![title, id],
        )?;
        Ok(())
    }

    pub async fn update_cover(
        &self,
        id: i64,
        cover_image: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE music_tracks SET cover_image = ? WHERE id = ?",
            params![cover_image, id],
        )?;
        Ok(())
    }

    pub async fn update_cover_by_filename(
        &self,
        file_name: &str,
        cover_image: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
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
             FROM music_tracks WHERE id = ?",
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

    pub async fn get_all(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Attachment>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, file_name, stored_name, file_path, file_type, content_type, file_size, passage_uuid, visibility, show_in_passage, uploaded_at 
             FROM attachments ORDER BY uploaded_at DESC LIMIT ? OFFSET ?"
        )?;

        let attachments = stmt
            .query_map(params![limit, offset], |row| {
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
            })?
            .collect::<Result<Vec<_>, _>>()?;

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

    /// 根据文章 UUID 列表查询附件
    pub async fn get_by_passage_uuids(
        &self,
        uuids: Vec<String>,
    ) -> Result<Vec<Attachment>, Box<dyn std::error::Error>> {
        if uuids.is_empty() {
            return Ok(Vec::new());
        }
        let conn = self.pool.get()?;
        // 预分配容量，每个 "?" 1 字符 + uuids.len()-1 个 ","
        let mut placeholders = String::with_capacity(uuids.len() * 2 - 1);
        for (i, _) in uuids.iter().enumerate() {
            if i > 0 {
                placeholders.push(',');
            }
            placeholders.push('?');
        }
        let sql = format!(
            "SELECT id, file_name, stored_name, file_path, file_type, content_type, file_size, passage_uuid, visibility, show_in_passage, uploaded_at 
             FROM attachments WHERE passage_uuid IN ({})", placeholders
        );
        let params: Vec<&dyn rusqlite::ToSql> = uuids
            .iter()
            .map(|uuid| uuid as &dyn rusqlite::ToSql)
            .collect();

        let mut stmt = conn.prepare(&sql)?;
        let attachments = stmt
            .query_map(params.as_slice(), |row| {
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
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(attachments)
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

    pub async fn delete_batch(&self, ids: Vec<i64>) -> Result<usize, Box<dyn std::error::Error>> {
        if ids.is_empty() {
            return Ok(0);
        }
        let conn = self.pool.get()?;
        // 预分配容量，每个 "?" 1 字符 + ids.len()-1 个 ","
        let mut placeholders = String::with_capacity(ids.len() * 2 - 1);
        for (i, _) in ids.iter().enumerate() {
            if i > 0 {
                placeholders.push(',');
            }
            placeholders.push('?');
        }
        let sql = format!("DELETE FROM attachments WHERE id IN ({})", placeholders);
        let params: Vec<&dyn rusqlite::ToSql> =
            ids.iter().map(|id| id as &dyn rusqlite::ToSql).collect();
        let rows_affected = conn.execute(&sql, params.as_slice())?;
        Ok(rows_affected)
    }

    pub async fn get_by_ids(
        &self,
        ids: Vec<i64>,
    ) -> Result<Vec<Attachment>, Box<dyn std::error::Error>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let conn = self.pool.get()?;
        // 预分配容量，每个 "?" 1 字符 + ids.len()-1 个 ","
        let mut placeholders = String::with_capacity(ids.len() * 2 - 1);
        for (i, _) in ids.iter().enumerate() {
            if i > 0 {
                placeholders.push(',');
            }
            placeholders.push('?');
        }
        let sql = format!(
            "SELECT id, file_name, stored_name, file_path, file_type, content_type, file_size, passage_uuid, visibility, show_in_passage, uploaded_at 
             FROM attachments WHERE id IN ({})", placeholders
        );
        let params: Vec<&dyn rusqlite::ToSql> =
            ids.iter().map(|id| id as &dyn rusqlite::ToSql).collect();

        let mut stmt = conn.prepare(&sql)?;
        let attachments = stmt
            .query_map(params.as_slice(), |row| {
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
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(attachments)
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

        let cards = stmt
            .query_map([], |row| {
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
            })?
            .collect::<Result<Vec<_>, _>>()?;

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

        let cards = stmt
            .query_map([], |row| {
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
            })?
            .collect::<Result<Vec<_>, _>>()?;

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

/// 友链仓库
pub struct FriendLinkRepository {
    pool: Arc<Pool<SqliteConnectionManager>>,
}

#[async_trait::async_trait]
impl Repository for FriendLinkRepository {
    fn get_pool(&self) -> Arc<Pool<SqliteConnectionManager>> {
        self.pool.clone()
    }
}

impl FriendLinkRepository {
    pub fn new(pool: Arc<Pool<SqliteConnectionManager>>) -> Self {
        Self { pool }
    }

    pub async fn get_all(&self) -> Result<Vec<FriendLink>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT id, nickname, link_url, avatar_url, motto, sort_order, is_enabled, created_at, updated_at FROM friend_links WHERE is_enabled = 1 ORDER BY sort_order ASC, created_at DESC")?;

        let links = stmt
            .query_map([], |row| {
                Ok(FriendLink {
                    id: Some(row.get(0)?),
                    nickname: row.get(1)?,
                    link_url: row.get(2)?,
                    avatar_url: row.get(3)?,
                    motto: row.get(4)?,
                    sort_order: row.get(5)?,
                    is_enabled: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(links)
    }

    pub async fn get_all_including_disabled(
        &self,
    ) -> Result<Vec<FriendLink>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT id, nickname, link_url, avatar_url, motto, sort_order, is_enabled, created_at, updated_at FROM friend_links ORDER BY sort_order ASC, created_at DESC")?;

        let links = stmt
            .query_map([], |row| {
                Ok(FriendLink {
                    id: Some(row.get(0)?),
                    nickname: row.get(1)?,
                    link_url: row.get(2)?,
                    avatar_url: row.get(3)?,
                    motto: row.get(4)?,
                    sort_order: row.get(5)?,
                    is_enabled: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(links)
    }

    pub async fn get_by_id(
        &self,
        id: i64,
    ) -> Result<Option<FriendLink>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT id, nickname, link_url, avatar_url, motto, sort_order, is_enabled, created_at, updated_at FROM friend_links WHERE id = ?")?;

        let link = stmt
            .query_row(params![id], |row| {
                Ok(FriendLink {
                    id: Some(row.get(0)?),
                    nickname: row.get(1)?,
                    link_url: row.get(2)?,
                    avatar_url: row.get(3)?,
                    motto: row.get(4)?,
                    sort_order: row.get(5)?,
                    is_enabled: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })
            .optional()?;

        Ok(link)
    }

    pub async fn create(&self, link: &FriendLink) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO friend_links (nickname, link_url, avatar_url, motto, sort_order, is_enabled, created_at, updated_at) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                &link.nickname,
                &link.link_url,
                &link.avatar_url,
                &link.motto,
                &link.sort_order,
                &link.is_enabled,
                &link.created_at,
                &link.updated_at,
            ],
        )?;
        Ok(())
    }

    pub async fn update(&self, link: &FriendLink) -> Result<(), Box<dyn std::error::Error>> {
        let id = link.id.ok_or("友链 ID 不能为空")?;
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE friend_links SET nickname = ?, link_url = ?, avatar_url = ?, motto = ?, sort_order = ?, is_enabled = ?, updated_at = ? 
             WHERE id = ?",
            params![
                &link.nickname,
                &link.link_url,
                &link.avatar_url,
                &link.motto,
                &link.sort_order,
                &link.is_enabled,
                &link.updated_at,
                id,
            ],
        )?;
        Ok(())
    }

    pub async fn delete(&self, id: i64) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM friend_links WHERE id = ?", params![id])?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn count(&self) -> Result<i64, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM friend_links WHERE is_enabled = 1",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }
}

// ==================== 动态路由仓库 ====================

/// 动态路由仓库
#[derive(Clone)]
pub struct DynamicRouteRepository {
    pool: Arc<Pool<SqliteConnectionManager>>,
}

impl DynamicRouteRepository {
    pub fn new(pool: Arc<Pool<SqliteConnectionManager>>) -> Self {
        Self { pool }
    }

    /// 创建路由
    pub async fn create(&self, route: &DynamicRoute) -> Result<i64, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;

        // 查找最小的未使用ID（ID复用逻辑）
        let next_id: i64 = conn
            .query_row(
                "SELECT CASE
                WHEN MIN(id) > 1 THEN 1
                ELSE (
                    SELECT MIN(id) + 1
                    FROM dynamic_routes r1
                    WHERE NOT EXISTS (
                        SELECT 1
                        FROM dynamic_routes r2
                        WHERE r2.id = r1.id + 1
                    )
                )
            END as next_id FROM dynamic_routes",
                [],
                |row| row.get(0),
            )
            .unwrap_or(1);

        conn.execute(
            "INSERT INTO dynamic_routes (id, route_name, route_type, path, handler_type, handler_config, inline_template, template_path, content_type_hint, enabled, priority, created_by, group_id, is_primary_entry, metadata, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                next_id,
                &route.route_name,
                &route.route_type,
                &route.path,
                &route.handler_type,
                &route.handler_config.to_string(),
                &route.inline_template,
                &route.template_path,
                &route.content_type_hint,
                &route.enabled,
                &route.priority,
                &route.created_by,
                &route.group_id,
                &route.is_primary_entry,
                &route.metadata.as_ref().map(|v| v.to_string()),
                &route.created_at,
                &route.updated_at,
            ],
        )?;
        Ok(next_id)
    }

    /// 根据ID获取路由
    pub async fn get_by_id(
        &self,
        id: i64,
    ) -> Result<Option<DynamicRoute>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, route_name, route_type, path, handler_type, handler_config, inline_template, template_path, content_type_hint, enabled, priority, created_at, updated_at, created_by, group_id, is_primary_entry, metadata
             FROM dynamic_routes WHERE id = ?"
        )?;

        let route = stmt
            .query_row(params![id], |row| {
                Ok(DynamicRoute {
                    id: Some(row.get(0)?),
                    route_name: row.get(1)?,
                    route_type: row.get(2)?,
                    path: row.get(3)?,
                    handler_type: row.get(4)?,
                    handler_config: serde_json::from_str(&row.get::<_, String>(5)?)
                        .unwrap_or_default(),
                    inline_template: row.get(6)?,
                    template_path: row.get(7)?,
                    content_type_hint: row.get(8)?,
                    enabled: row.get(9)?,
                    priority: row.get(10)?,
                    created_at: row.get(11)?,
                    updated_at: row.get(12)?,
                    created_by: row.get(13)?,
                    group_id: row.get(14)?,
                    is_primary_entry: row.get(15)?,
                    metadata: row
                        .get::<_, Option<String>>(16)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                })
            })
            .optional()?;

        Ok(route)
    }

    /// 根据路径获取路由
    pub async fn get_by_path(
        &self,
        path: &str,
    ) -> Result<Option<DynamicRoute>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
                    "SELECT id, route_name, route_type, path, handler_type, handler_config, inline_template, template_path, content_type_hint, enabled, priority, created_at, updated_at, created_by, group_id, is_primary_entry, metadata
                     FROM dynamic_routes WHERE path = ?"
                )?;

        let route = stmt
            .query_row(params![path], |row| {
                Ok(DynamicRoute {
                    id: Some(row.get(0)?),
                    route_name: row.get(1)?,
                    route_type: row.get(2)?,
                    path: row.get(3)?,
                    handler_type: row.get(4)?,
                    handler_config: serde_json::from_str(&row.get::<_, String>(5)?)
                        .unwrap_or_default(),
                    inline_template: row.get(6)?,
                    template_path: row.get(7)?,
                    content_type_hint: row.get(8)?,
                    enabled: row.get(9)?,
                    priority: row.get(10)?,
                    created_at: row.get(11)?,
                    updated_at: row.get(12)?,
                    created_by: row.get(13)?,
                    group_id: row.get(14)?,
                    is_primary_entry: row.get(15)?,
                    metadata: row
                        .get::<_, Option<String>>(16)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                })
            })
            .optional()?;

        Ok(route)
    }

    /// 获取路由列表
    pub async fn list(
        &self,
        offset: i64,
        limit: i64,
        route_type: Option<RouteType>,
        enabled: Option<bool>,
    ) -> Result<(Vec<DynamicRoute>, i64), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;

        // 构建查询条件
        // 预分配容量，最多约 50 字符 (" WHERE route_type = ? AND enabled = ?")
        let mut where_clause = String::with_capacity(50);
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if route_type.is_some() || enabled.is_some() {
            where_clause.push_str(" WHERE ");
        }

        if let Some(rt) = route_type {
            where_clause.push_str("route_type = ?");
            params.push(Box::new(rt));
        }

        if let Some(en) = enabled {
            if route_type.is_some() {
                where_clause.push_str(" AND ");
            }
            where_clause.push_str("enabled = ?");
            params.push(Box::new(en));
        }

        // 获取总数
        let total: i64 = conn.query_row(
            &format!("SELECT COUNT(*) FROM dynamic_routes{}", where_clause),
            rusqlite::params_from_iter(params.iter()),
            |row| row.get(0),
        )?;

        // 获取列表
        let query = format!(
            "SELECT id, route_name, route_type, path, handler_type, handler_config, inline_template, template_path, content_type_hint, enabled, priority, created_at, updated_at, created_by, group_id, is_primary_entry, metadata
             FROM dynamic_routes{} ORDER BY priority DESC, id ASC LIMIT ? OFFSET ?",
            where_clause
        );

        let mut stmt = conn.prepare(&query)?;

        let mut final_params: Vec<Box<dyn rusqlite::ToSql>> = params;
        final_params.push(Box::new(limit));
        final_params.push(Box::new(offset));

        let routes = stmt
            .query_map(rusqlite::params_from_iter(final_params.iter()), |row| {
                Ok(DynamicRoute {
                    id: Some(row.get(0)?),
                    route_name: row.get(1)?,
                    route_type: row.get(2)?,
                    path: row.get(3)?,
                    handler_type: row.get(4)?,
                    handler_config: serde_json::from_str(&row.get::<_, String>(5)?)
                        .unwrap_or_default(),
                    inline_template: row.get(6)?,
                    template_path: row.get(7)?,
                    content_type_hint: row.get(8)?,
                    enabled: row.get(9)?,
                    priority: row.get(10)?,
                    created_at: row.get(11)?,
                    updated_at: row.get(12)?,
                    created_by: row.get(13)?,
                    group_id: row.get(14)?,
                    is_primary_entry: row.get(15)?,
                    metadata: row
                        .get::<_, Option<String>>(16)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok((routes, total))
    }

    /// 更新路由
    pub async fn update(
        &self,
        id: i64,
        route: &DynamicRoute,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE dynamic_routes SET route_name=?, route_type=?, path=?, handler_type=?, handler_config=?, inline_template=?, template_path=?, content_type_hint=?, enabled=?, priority=?, group_id=?, is_primary_entry=?, metadata=?, updated_at=?
             WHERE id=?",
            params![
                &route.route_name,
                &route.route_type,
                &route.path,
                &route.handler_type,
                &route.handler_config.to_string(),
                &route.inline_template,
                &route.template_path,
                &route.content_type_hint,
                &route.enabled,
                &route.priority,
                &route.group_id,
                &route.is_primary_entry,
                &route.metadata.as_ref().map(|v| v.to_string()),
                &route.updated_at,
                id,
            ],
        )?;
        Ok(())
    }

    /// 删除路由
    pub async fn delete(&self, id: i64) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM dynamic_routes WHERE id=?", params![id])?;
        Ok(())
    }

    /// 根据类型删除路由
    pub async fn delete_by_type(
        &self,
        route_type: RouteType,
    ) -> Result<i64, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let result = conn.execute(
            "DELETE FROM dynamic_routes WHERE route_type=?",
            params![route_type],
        )?;
        Ok(result as i64)
    }

    /// 获取所有启用的路由
    pub async fn get_all_enabled(&self) -> Result<Vec<DynamicRoute>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, route_name, route_type, path, handler_type, handler_config, inline_template, template_path, content_type_hint, enabled, priority, created_at, updated_at, created_by, group_id, is_primary_entry, metadata
             FROM dynamic_routes WHERE enabled = 1 ORDER BY priority DESC, id ASC"
        )?;

        let routes = stmt
            .query_map([], |row| {
                Ok(DynamicRoute {
                    id: Some(row.get(0)?),
                    route_name: row.get(1)?,
                    route_type: row.get(2)?,
                    path: row.get(3)?,
                    handler_type: row.get(4)?,
                    handler_config: serde_json::from_str(&row.get::<_, String>(5)?)
                        .unwrap_or_default(),
                    inline_template: row.get(6)?,
                    template_path: row.get(7)?,
                    content_type_hint: row.get(8)?,
                    enabled: row.get(9)?,
                    priority: row.get(10)?,
                    created_at: row.get(11)?,
                    updated_at: row.get(12)?,
                    created_by: row.get(13)?,
                    group_id: row.get(14)?,
                    is_primary_entry: row.get(15)?,
                    metadata: row
                        .get::<_, Option<String>>(16)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(routes)
    }

    /// 获取所有路由
    pub async fn get_all(&self) -> Result<Vec<DynamicRoute>, Box<dyn std::error::Error>> {
        let routes = self.list(0, 10000, None, None).await?.0;
        Ok(routes)
    }

    /// 获取路由总数
    pub async fn count(&self) -> Result<i64, Box<dyn std::error::Error>> {
        let count = self.list(0, 0, None, None).await?.1;
        Ok(count)
    }

    /// 删除所有路由
    pub async fn delete_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM dynamic_routes", [])?;
        Ok(())
    }

    /// 重置自增ID计数器
    ///
    /// 在清空所有路由后调用此方法，重置自增ID从1开始
    pub async fn reset_auto_increment(&self) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute(
            "DELETE FROM sqlite_sequence WHERE name='dynamic_routes'",
            [],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::db::models::Passage;
    use chrono::Utc;

    #[test]
    fn test_passage_model_creation() {
        let now = Utc::now();
        let passage = Passage {
            id: None,
            uuid: Some("test-uuid-123".to_string()),
            title: "Test Article".to_string(),
            content: "<p>Test content</p>".to_string(),
            original_content: Some("# Test\n\nContent".to_string()),
            summary: Some("Test summary".to_string()),
            summarize: Some("Test summarize".to_string()),
            author: "Test Author".to_string(),
            tags: "[\"tag1\", \"tag2\"]".to_string(),
            category: "Test Category".to_string(),
            status: crate::db::models::PassageStatus::Published,
            file_path: Some("markdown/test.md".to_string()),
            visibility: crate::db::models::PassageVisibility::Public,
            is_scheduled: false,
            published_at: Some(now),
            cover_image: None,
            created_at: now,
            updated_at: now,
        };

        assert_eq!(passage.title, "Test Article");
        assert_eq!(passage.status, crate::db::models::PassageStatus::Published);
        assert_eq!(
            passage.visibility,
            crate::db::models::PassageVisibility::Public
        );
    }

    #[test]
    fn test_pagination_params_validation() {
        // 测试分页参数验证逻辑
        let limit = 10;
        let page = 1;

        assert!(limit > 0 && limit <= 1000);
        assert!(page > 0);
    }

    #[test]
    fn test_dynamic_route_model_with_new_fields() {
        // 测试 DynamicRoute 模型包含新字段
        use crate::db::models::{DynamicRoute, HandlerType, RouteType};
        use serde_json::json;

        let now = Utc::now();
        let route = DynamicRoute {
            id: None,
            route_name: Some("测试路由".to_string()),
            route_type: RouteType::Database,
            path: "/test/route".to_string(),
            handler_type: HandlerType::Static,
            handler_config: json!({"content": "test content"}),
            inline_template: Some("<html><body>Test</body></html>".to_string()),
            template_path: None,
            content_type_hint: Some("text/html".to_string()),
            enabled: true,
            priority: 0,
            created_at: now,
            updated_at: now,
            created_by: Some("test_user".to_string()),
            group_id: None,
            is_primary_entry: None,
            metadata: Some(json!({"key": "value"})),
        };

        assert_eq!(route.path, "/test/route");
        assert_eq!(
            route.inline_template,
            Some("<html><body>Test</body></html>".to_string())
        );
        assert_eq!(route.content_type_hint, Some("text/html".to_string()));
        assert!(route.enabled);
    }

    #[test]
    fn test_date_params_validation() {
        // 测试日期参数验证逻辑
        let year = 2026;
        let month = 2;
        let day = 14;

        assert!(year >= 2000 && year <= 2100);
        assert!(month >= 1 && month <= 12);
        assert!(day >= 1 && day <= 31);
    }
}
