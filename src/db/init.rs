use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;

/// 全局数据库连接池
static DB_POOL: tokio::sync::OnceCell<Pool<SqliteConnectionManager>> =
    tokio::sync::OnceCell::const_new();

// 数据库连接池配置常量
const DB_MAX_CONNECTIONS: u32 = 20; // 最大连接数（个人博客场景优化）
const DB_MIN_IDLE: u32 = 5; // 最小空闲连接数
const DB_CONNECTION_TIMEOUT: u64 = 30; // 连接超时（秒）
const DB_IDLE_TIMEOUT: u64 = 600; // 空闲连接超时（秒，10分钟）
const DB_MAX_LIFETIME: u64 = 1800; // 连接最大生命周期（秒，30分钟）

/// 初始化数据库
pub fn init_db(db_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 创建数据库目录
    if let Some(parent) = std::path::Path::new(db_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    // 创建连接池并优化配置
    let manager = SqliteConnectionManager::file(db_path);
    let pool = Pool::builder()
        .max_size(DB_MAX_CONNECTIONS)
        .min_idle(Some(DB_MIN_IDLE))
        .connection_timeout(std::time::Duration::from_secs(DB_CONNECTION_TIMEOUT))
        .idle_timeout(Some(std::time::Duration::from_secs(DB_IDLE_TIMEOUT)))
        .max_lifetime(Some(std::time::Duration::from_secs(DB_MAX_LIFETIME)))
        .test_on_check_out(true) // 获取连接时测试连接是否有效
        .build(manager)?;

    // 获取连接并初始化表结构和优化设置
    {
        let conn = pool.get()?;

        // 设置 busy_timeout，避免数据库锁立即失败
        conn.query_row("PRAGMA busy_timeout = 30000;", [], |_| Ok(()))?; // 30秒超时

        // 启用 WAL 模式以支持更好的并发读写
        conn.query_row("PRAGMA journal_mode = WAL;", [], |row| {
            let mode: String = row.get(0)?;
            Ok(mode)
        })?;

        // 增加 WAL 文件大小限制（默认为 -1，无限制）
        {
            let mut stmt = conn.prepare("PRAGMA wal_autocheckpoint = 1000;")?;
            stmt.query_row([], |_| Ok(())).or_else(|e| {
                if e.to_string().contains("Query returned no rows") {
                    Ok(())
                } else {
                    Err(e)
                }
            })?;
        }
        // 优化 SQLite 性能参数
        {
            let mut stmt = conn.prepare("PRAGMA synchronous = NORMAL;")?;
            stmt.query_row([], |_| Ok(())).or_else(|e| {
                if e.to_string().contains("Query returned no rows") {
                    Ok(())
                } else {
                    Err(e)
                }
            })?;
        }
        {
            let mut stmt = conn.prepare("PRAGMA cache_size = -64000;")?;
            stmt.query_row([], |_| Ok(())).or_else(|e| {
                if e.to_string().contains("Query returned no rows") {
                    Ok(())
                } else {
                    Err(e)
                }
            })?;
        } // 64MB 缓存
        {
            let mut stmt = conn.prepare("PRAGMA temp_store = MEMORY;")?;
            stmt.query_row([], |_| Ok(())).or_else(|e| {
                if e.to_string().contains("Query returned no rows") {
                    Ok(())
                } else {
                    Err(e)
                }
            })?;
        } // 临时表使用内存
        {
            let mut stmt = conn.prepare("PRAGMA mmap_size = 268435456;")?;
            stmt.query_row([], |_| Ok(())).or_else(|e| {
                if e.to_string().contains("Query returned no rows") {
                    Ok(())
                } else {
                    Err(e)
                }
            })?;
        } // 256MB 内存映射

        create_tables(&conn)?;
        seed_default_data(&conn)?;
    }

    // 保存连接池到全局变量
    DB_POOL.set(pool).map_err(|_| "数据库已初始化")?;

    println!("✅ 数据库初始化成功: {}", db_path);
    println!("📊 连接池配置:");
    println!("   - 最大连接数: {}", DB_MAX_CONNECTIONS);
    println!("   - 最小空闲连接: {}", DB_MIN_IDLE);
    println!("   - 连接超时: {}秒", DB_CONNECTION_TIMEOUT);
    println!("   - 空闲超时: {}秒", DB_IDLE_TIMEOUT);
    println!("   - 最大生命周期: {}秒", DB_MAX_LIFETIME);
    println!("   - WAL 模式: 已启用");
    Ok(())
}

/// 获取数据库连接池
pub async fn get_db_pool() -> Result<Pool<SqliteConnectionManager>, String> {
    DB_POOL
        .get()
        .cloned()
        .ok_or_else(|| "数据库未初始化".to_string())
}

/// 同步获取数据库连接池（用于非异步上下文）
pub fn get_db_pool_sync() -> Result<Pool<SqliteConnectionManager>, String> {
    DB_POOL
        .get()
        .cloned()
        .ok_or_else(|| "数据库未初始化".to_string())
}

/// 获取连接池状态信息
pub fn get_pool_status() -> Result<PoolStatus, String> {
    let pool = DB_POOL.get().ok_or_else(|| "数据库未初始化".to_string())?;

    let state = pool.state();

    Ok(PoolStatus {
        max_connections: DB_MAX_CONNECTIONS,
        min_idle: DB_MIN_IDLE,
        current_connections: state.connections,
        idle_connections: state.idle_connections,
        active_connections: state.connections - state.idle_connections,
        connection_utilization: if DB_MAX_CONNECTIONS > 0 {
            ((state.connections - state.idle_connections) as f64 / DB_MAX_CONNECTIONS as f64)
                * 100.0
        } else {
            0.0
        },
    })
}

/// 连接池状态信息
#[derive(Debug, serde::Serialize)]
pub struct PoolStatus {
    pub max_connections: u32,
    pub min_idle: u32,
    pub current_connections: u32,
    pub idle_connections: u32,
    pub active_connections: u32,
    pub connection_utilization: f64, // 连接利用率（百分比）
}

/// 创建所有数据库表
fn create_tables(conn: &rusqlite::Connection) -> Result<(), Box<dyn std::error::Error>> {
    // 检查 passages 表是否存在
    let table_exists = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='passages'",
            [],
            |row| {
                let count: i64 = row.get(0)?;
                Ok(count > 0)
            },
        )
        .unwrap_or(false);

    // 如果表已存在，检查是否有 uuid 列
    if table_exists {
        let has_uuid_column = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('passages') WHERE name = 'uuid'",
                [],
                |row| {
                    let count: i64 = row.get(0)?;
                    Ok(count > 0)
                },
            )
            .unwrap_or(false);

        // 检查是否有 cover_image 列
        let has_cover_image_column = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('passages') WHERE name = 'cover_image'",
                [],
                |row| {
                    let count: i64 = row.get(0)?;
                    Ok(count > 0)
                },
            )
            .unwrap_or(false);

        // 检查是否有 summarize 列
        let has_summarize_column = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('passages') WHERE name = 'summarize'",
                [],
                |row| {
                    let count: i64 = row.get(0)?;
                    Ok(count > 0)
                },
            )
            .unwrap_or(false);

        // 如果表已存在但没有 uuid 列，则添加该列
        if !has_uuid_column {
            println!("⚠️  检测到旧版数据库结构，正在迁移 passages 表...");

            // 先添加 uuid 列（可为空，用于迁移现有数据）
            conn.execute("ALTER TABLE passages ADD COLUMN uuid TEXT", [])?;

            // 为现有文章生成 UUID
            let mut stmt = conn.prepare("SELECT id FROM passages WHERE uuid IS NULL")?;
            let mut rows = stmt.query([])?;

            let mut updated_count = 0;
            while let Some(row) = rows.next()? {
                let id: i64 = row.get(0)?;
                let uuid = crate::id_generator::generate_unique_id();
                conn.execute(
                    "UPDATE passages SET uuid = ? WHERE id = ?",
                    rusqlite::params![&uuid, &id],
                )?;
                updated_count += 1;
            }

            // 将 uuid 列改为 NOT NULL 和 UNIQUE
            conn.execute("UPDATE passages SET uuid = CASE WHEN uuid IS NULL THEN (SELECT hex(randomblob(16))) ELSE uuid END WHERE uuid IS NULL", [])?;

            // SQLite 不支持直接添加 NOT NULL 约束到已有列，需要重建表
            conn.execute(
                "CREATE TABLE passages_new (
                    id INTEGER PRIMARY KEY,
                    uuid TEXT UNIQUE NOT NULL,
                    title TEXT NOT NULL,
                    content TEXT NOT NULL,
                    original_content TEXT,
                    summary TEXT,
                    summarize TEXT,
                    author TEXT DEFAULT '管理员',
                    tags TEXT DEFAULT '[]',
                    category TEXT DEFAULT '未分类',
                    status TEXT DEFAULT 'published',
                    file_path TEXT,
                    visibility TEXT DEFAULT 'public',
                    is_scheduled INTEGER DEFAULT 0,
                    published_at DATETIME,
                    cover_image TEXT DEFAULT '/img/passage-cover.webp',
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
                )",
                [],
            )?;

            // 复制数据
            conn.execute(
                "INSERT INTO passages_new (id, uuid, title, content, original_content, summary, summarize, author, tags, category, status, file_path, visibility, is_scheduled, published_at, cover_image, created_at, updated_at) 
                 SELECT id, uuid, title, content, original_content, summary, summarize, author, tags, category, status, file_path, visibility, is_scheduled, published_at, cover_image, created_at, updated_at 
                 FROM passages",
                [],
            )?;

            // 删除旧表并重命名新表
            conn.execute("DROP TABLE passages", [])?;
            conn.execute("ALTER TABLE passages_new RENAME TO passages", [])?;

            println!("✅ 已为 {} 篇现有文章生成 UUID", updated_count);
        } else if !has_cover_image_column {
            // 如果有 uuid 列但没有 cover_image 列，直接添加 cover_image 列
            println!("⚠️  检测到缺少 cover_image 列，正在添加...");
            conn.execute("ALTER TABLE passages ADD COLUMN cover_image TEXT DEFAULT '/img/passage-cover.webp'", [])?;
            // 为现有文章设置默认封面
            conn.execute("UPDATE passages SET cover_image = '/img/passage-cover.webp' WHERE cover_image IS NULL", [])?;
            println!("✅ 已添加 cover_image 列");
        } else if !has_summarize_column {
            // 如果有 uuid 和 cover_image 列但没有 summarize 列，直接添加 summarize 列
            println!("⚠️  检测到缺少 summarize 列，正在添加...");
            conn.execute("ALTER TABLE passages ADD COLUMN summarize TEXT", [])?;
            println!("✅ 已添加 summarize 列");
        }

        // 添加日期预计算列（优化查询性能）
        let has_created_year_column = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('passages') WHERE name = 'created_year'",
                [],
                |row| row.get::<_, i32>(0),
            )
            .unwrap_or(0)
            > 0;

        if !has_created_year_column {
            println!("⚠️  检测到缺少日期预计算列，正在添加...");
            conn.execute("ALTER TABLE passages ADD COLUMN created_year INTEGER", [])?;
            conn.execute("ALTER TABLE passages ADD COLUMN created_month INTEGER", [])?;
            conn.execute("ALTER TABLE passages ADD COLUMN created_day INTEGER", [])?;

            // 更新现有数据
            conn.execute(
                "UPDATE passages SET 
                 created_year = CAST(strftime('%Y', created_at) AS INTEGER),
                 created_month = CAST(strftime('%m', created_at) AS INTEGER),
                 created_day = CAST(strftime('%d', created_at) AS INTEGER)",
                [],
            )?;

            println!("✅ 已添加日期预计算列并更新现有数据");
        }
    }

    // 创建文章表（如果不存在）
    conn.execute(
        "CREATE TABLE IF NOT EXISTS passages (
            id INTEGER PRIMARY KEY,
            uuid TEXT UNIQUE NOT NULL,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            original_content TEXT,
            summary TEXT,
            summarize TEXT,
            author TEXT DEFAULT '管理员',
            tags TEXT DEFAULT '[]',
            category TEXT DEFAULT '未分类',
            status TEXT DEFAULT 'published',
            file_path TEXT,
            visibility TEXT DEFAULT 'public',
            is_scheduled INTEGER DEFAULT 0,
            published_at DATETIME,
            cover_image TEXT DEFAULT '/img/passage-cover.webp',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // 创建文章表索引
    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_passages_uuid ON passages(uuid)",
        [],
    )?;
    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_passages_file_path ON passages(file_path)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_passages_status ON passages(status)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_passages_uuid ON passages(uuid)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_passages_category ON passages(category)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_passages_created_at ON passages(created_at)",
        [],
    )?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_passages_status_created ON passages(status, created_at DESC)", [])?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_passages_category_status ON passages(category, status)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_passages_visibility ON passages(visibility)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_passages_published_at ON passages(published_at)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_passages_scheduled ON passages(is_scheduled, published_at)",
        [],
    )?;
    // 添加复合索引优化统计查询
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_passages_status_visibility ON passages(status, visibility)",
        [],
    )?;

    // 创建日期预计算列索引（优化日期筛选查询）
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_passages_created_year ON passages(created_year)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_passages_created_month ON passages(created_month)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_passages_created_day ON passages(created_day)",
        [],
    )?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_passages_status_created_year ON passages(status, created_year)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_passages_status_created_ymd ON passages(status, created_year, created_month, created_day)", [])?;

    // 创建触发器：插入时自动计算日期
    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS update_passage_created_date_insert
         AFTER INSERT ON passages
         BEGIN
            UPDATE passages SET
                created_year = CAST(strftime('%Y', NEW.created_at) AS INTEGER),
                created_month = CAST(strftime('%m', NEW.created_at) AS INTEGER),
                created_day = CAST(strftime('%d', NEW.created_at) AS INTEGER)
            WHERE id = NEW.id;
         END",
        [],
    )?;

    // 创建触发器：更新 created_at 时自动计算日期
    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS update_passage_created_date_update
         AFTER UPDATE OF created_at ON passages
         BEGIN
            UPDATE passages SET
                created_year = CAST(strftime('%Y', NEW.created_at) AS INTEGER),
                created_month = CAST(strftime('%m', NEW.created_at) AS INTEGER),
                created_day = CAST(strftime('%d', NEW.created_at) AS INTEGER)
            WHERE id = NEW.id;
         END",
        [],
    )?;

    // 创建用户表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            password TEXT NOT NULL,
            email TEXT UNIQUE NOT NULL,
            role TEXT DEFAULT 'user',
            status TEXT DEFAULT 'active',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_users_username ON users(username)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)",
        [],
    )?;

    // 创建访客表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS visitors (
            id INTEGER PRIMARY KEY,
            ip TEXT NOT NULL,
            user_agent TEXT,
            visit_date TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_visitors_ip_date ON visitors(ip, visit_date)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_visitors_date ON visitors(visit_date)",
        [],
    )?;

    // 创建文章阅读记录表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS article_views (
            id INTEGER PRIMARY KEY,
            passage_uuid TEXT NOT NULL,
            ip TEXT NOT NULL,
            user_agent TEXT,
            country TEXT DEFAULT '',
            city TEXT DEFAULT '',
            region TEXT DEFAULT '',
            view_date TEXT NOT NULL,
            view_time DATETIME DEFAULT CURRENT_TIMESTAMP,
            duration INTEGER DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (passage_uuid) REFERENCES passages(uuid) ON DELETE CASCADE
        )",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_article_views_passage_uuid ON article_views(passage_uuid)",
        [],
    )?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_article_views_passage_date ON article_views(passage_uuid, view_date)", [])?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_article_views_ip_date ON article_views(ip, view_date)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_article_views_date ON article_views(view_date)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_article_views_view_time ON article_views(view_time)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_article_views_country ON article_views(country)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_article_views_city_region ON article_views(city, region)",
        [],
    )?;
    // 添加复合索引优化统计查询
    conn.execute("CREATE INDEX IF NOT EXISTS idx_article_views_date_country ON article_views(view_date, country)", [])?;

    // 创建评论表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS comments (
            id INTEGER PRIMARY KEY,
            username TEXT NOT NULL,
            content TEXT NOT NULL,
            passage_uuid TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (passage_uuid) REFERENCES passages(uuid) ON DELETE CASCADE
        )",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_comments_passage_uuid ON comments(passage_uuid)",
        [],
    )?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_comments_passage_created ON comments(passage_uuid, created_at DESC)", [])?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_comments_created_at ON comments(created_at)",
        [],
    )?;

    // 创建文章版本历史表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS passage_versions (
            id INTEGER PRIMARY KEY,
            passage_id INTEGER NOT NULL,
            passage_uuid TEXT NOT NULL,
            version_number INTEGER NOT NULL,
            
            -- 文件信息
            file_path TEXT NOT NULL,
            file_size INTEGER,
            file_hash TEXT,
            
            -- 只存储原始内容和元数据，不存储派生数据
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            tags TEXT,
            category TEXT,
            cover_image TEXT,
            
            -- 变更信息
            change_type TEXT NOT NULL,
            change_reason TEXT,
            
            -- 操作信息
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            created_by TEXT DEFAULT 'system',
            
            -- Git 风格的树形结构
            parent_version_id INTEGER,
            branch_name TEXT,
            
            FOREIGN KEY (passage_id) REFERENCES passages(id) ON DELETE CASCADE,
            FOREIGN KEY (parent_version_id) REFERENCES passage_versions(id) ON DELETE SET NULL,
            UNIQUE(passage_id, version_number)
        )",
        [],
    )?;
    // passage_versions 表索引
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_passage_versions_passage_id ON passage_versions(passage_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_passage_versions_uuid ON passage_versions(passage_uuid)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_passage_versions_file_hash ON passage_versions(file_hash)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_passage_versions_created ON passage_versions(created_at DESC)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_passage_versions_parent ON passage_versions(parent_version_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_passage_versions_type ON passage_versions(change_type)",
        [],
    )?;

    // 创建设置表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            id INTEGER PRIMARY KEY,
            key TEXT UNIQUE NOT NULL,
            value TEXT NOT NULL,
            type TEXT DEFAULT 'string',
            description TEXT,
            category TEXT DEFAULT 'system',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_settings_key ON settings(key)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_settings_category ON settings(category)",
        [],
    )?;

    // 创建关于页面主卡片表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS about_main_cards (
            id INTEGER PRIMARY KEY,
            title TEXT NOT NULL,
            icon TEXT DEFAULT '',
            layout_type TEXT DEFAULT 'default',
            custom_css TEXT DEFAULT '',
            sort_order INTEGER DEFAULT 0,
            is_enabled BOOLEAN DEFAULT 1,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_main_cards_sort ON about_main_cards(sort_order)",
        [],
    )?;

    // 创建关于页面次卡片表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS about_sub_cards (
            id INTEGER PRIMARY KEY,
            main_card_id INTEGER NOT NULL,
            title TEXT NOT NULL,
            description TEXT DEFAULT '',
            icon TEXT DEFAULT '',
            link_url TEXT DEFAULT '',
            layout_type TEXT DEFAULT 'default',
            custom_css TEXT DEFAULT '',
            sort_order INTEGER DEFAULT 0,
            is_enabled BOOLEAN DEFAULT 1,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (main_card_id) REFERENCES about_main_cards(id) ON DELETE CASCADE
        )",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_sub_cards_main_id ON about_sub_cards(main_card_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_sub_cards_sort ON about_sub_cards(sort_order)",
        [],
    )?;

    // 创建分类表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS categories (
            id INTEGER PRIMARY KEY,
            name TEXT UNIQUE NOT NULL,
            description TEXT DEFAULT '',
            icon TEXT DEFAULT '',
            sort_order INTEGER DEFAULT 0,
            is_enabled BOOLEAN DEFAULT 1,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_categories_name ON categories(name)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_categories_sort ON categories(sort_order)",
        [],
    )?;

    // 创建标签表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY,
            name TEXT UNIQUE NOT NULL,
            description TEXT DEFAULT '',
            color TEXT DEFAULT '#007bff',
            category_id INTEGER DEFAULT 0,
            sort_order INTEGER DEFAULT 0,
            is_enabled BOOLEAN DEFAULT 1,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_tags_name ON tags(name)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_tags_category ON tags(category_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_tags_sort ON tags(sort_order)",
        [],
    )?;

    // 创建附件表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS attachments (
            id INTEGER PRIMARY KEY,
            file_name TEXT NOT NULL,
            stored_name TEXT NOT NULL,
            file_path TEXT NOT NULL,
            file_type TEXT NOT NULL,
            content_type TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            passage_uuid TEXT,
            visibility TEXT DEFAULT 'public',
            show_in_passage INTEGER DEFAULT 1,
            uploaded_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (passage_uuid) REFERENCES passages(uuid) ON DELETE CASCADE
        )",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_attachments_passage_uuid ON attachments(passage_uuid)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_attachments_type ON attachments(file_type)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_attachments_visibility ON attachments(visibility)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_attachments_uploaded_at ON attachments(uploaded_at)",
        [],
    )?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_attachments_passage_visibility ON attachments(passage_uuid, visibility)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_attachments_show_in_passage ON attachments(show_in_passage)", [])?;

    // 创建音乐表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS music_tracks (
            id INTEGER PRIMARY KEY,
            title TEXT NOT NULL,
            artist TEXT NOT NULL,
            file_path TEXT NOT NULL,
            file_name TEXT NOT NULL,
            duration TEXT DEFAULT '',
            cover_image TEXT DEFAULT '',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_music_tracks_created_at ON music_tracks(created_at)",
        [],
    )?;

    // 创建友链表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS friend_links (
            id INTEGER PRIMARY KEY,
            nickname TEXT NOT NULL,
            link_url TEXT NOT NULL,
            avatar_url TEXT DEFAULT '',
            motto TEXT DEFAULT '',
            sort_order INTEGER DEFAULT 0,
            is_enabled BOOLEAN DEFAULT 1,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_friend_links_sort ON friend_links(sort_order)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_friend_links_enabled ON friend_links(is_enabled)",
        [],
    )?;

    // 检查 dynamic_routes 表是否存在
    let dynamic_routes_table_exists = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='dynamic_routes'",
            [],
            |row| {
                let count: i64 = row.get(0)?;
                Ok(count > 0)
            },
        )
        .unwrap_or(false);

    // 如果表已存在，检查是否有新字段
    if dynamic_routes_table_exists {
        // 检查是否有 route_name 列
        let has_route_name_column = conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('dynamic_routes') WHERE name = 'route_name'",
            [],
            |row| {
                let count: i64 = row.get(0)?;
                Ok(count > 0)
            }
        ).unwrap_or(false);

        // 检查是否有 inline_template 列
        let has_inline_template_column = conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('dynamic_routes') WHERE name = 'inline_template'",
            [],
            |row| {
                let count: i64 = row.get(0)?;
                Ok(count > 0)
            }
        ).unwrap_or(false);

        // 检查是否有 template_path 列
        let has_template_path_column = conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('dynamic_routes') WHERE name = 'template_path'",
            [],
            |row| {
                let count: i64 = row.get(0)?;
                Ok(count > 0)
            }
        ).unwrap_or(false);

        // 检查是否有 content_type_hint 列
        let has_content_type_hint_column = conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('dynamic_routes') WHERE name = 'content_type_hint'",
            [],
            |row| {
                let count: i64 = row.get(0)?;
                Ok(count > 0)
            }
        ).unwrap_or(false);

        // 检查是否有 group_id 列
        let has_group_id_column = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('dynamic_routes') WHERE name = 'group_id'",
                [],
                |row| {
                    let count: i64 = row.get(0)?;
                    Ok(count > 0)
                },
            )
            .unwrap_or(false);

        // 检查是否有 is_primary_entry 列
        let has_is_primary_entry_column = conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('dynamic_routes') WHERE name = 'is_primary_entry'",
            [],
            |row| {
                let count: i64 = row.get(0)?;
                Ok(count > 0)
            }
        ).unwrap_or(false);

        // 如果表已存在但没有 route_name 列，则添加该列
        if !has_route_name_column {
            println!("⚠️  检测到旧版 dynamic_routes 表结构，正在添加 route_name 列...");
            conn.execute("ALTER TABLE dynamic_routes ADD COLUMN route_name TEXT", [])?;
            println!("✅ 已添加 route_name 列");
        }

        // 如果表已存在但没有 inline_template 列，则添加该列
        if !has_inline_template_column {
            println!("⚠️  检测到旧版 dynamic_routes 表结构，正在添加 inline_template 列...");
            conn.execute(
                "ALTER TABLE dynamic_routes ADD COLUMN inline_template TEXT",
                [],
            )?;
            println!("✅ 已添加 inline_template 列");
        }

        // 如果表已存在但没有 template_path 列，则添加该列
        if !has_template_path_column {
            println!("⚠️  检测到旧版 dynamic_routes 表结构，正在添加 template_path 列...");
            conn.execute(
                "ALTER TABLE dynamic_routes ADD COLUMN template_path TEXT",
                [],
            )?;
            println!("✅ 已添加 template_path 列");
        }

        // 如果表已存在但没有 content_type_hint 列，则添加该列
        if !has_content_type_hint_column {
            println!("⚠️  检测到旧版 dynamic_routes 表结构，正在添加 content_type_hint 列...");
            conn.execute(
                "ALTER TABLE dynamic_routes ADD COLUMN content_type_hint TEXT",
                [],
            )?;
            println!("✅ 已添加 content_type_hint 列");
        }

        // 如果表已存在但没有 group_id 列，则添加该列
        if !has_group_id_column {
            println!("⚠️  检测到旧版 dynamic_routes 表结构，正在添加 group_id 列...");
            conn.execute("ALTER TABLE dynamic_routes ADD COLUMN group_id TEXT", [])?;
            conn.execute("CREATE INDEX IF NOT EXISTS idx_dynamic_routes_group_id ON dynamic_routes(group_id)", [])?;
            println!("✅ 已添加 group_id 列和索引");
        }

        // 如果表已存在但没有 is_primary_entry 列，则添加该列
        if !has_is_primary_entry_column {
            println!("⚠️  检测到旧版 dynamic_routes 表结构，正在添加 is_primary_entry 列...");
            conn.execute(
                "ALTER TABLE dynamic_routes ADD COLUMN is_primary_entry BOOLEAN",
                [],
            )?;
            conn.execute("CREATE INDEX IF NOT EXISTS idx_dynamic_routes_is_primary_entry ON dynamic_routes(is_primary_entry)", [])?;
            println!("✅ 已添加 is_primary_entry 列和索引");
        }
    }

    // 创建动态路由表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS dynamic_routes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            route_name TEXT,
            route_type TEXT NOT NULL CHECK(route_type IN ('memory', 'file', 'database')),
            path TEXT NOT NULL UNIQUE,
            handler_type TEXT NOT NULL CHECK(handler_type IN ('redirect', 'static', 'template', 'proxy', 'custom')),
            handler_config TEXT NOT NULL,
            inline_template TEXT,
            template_path TEXT,
            content_type_hint TEXT,
            enabled BOOLEAN DEFAULT 1,
            priority INTEGER DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            created_by TEXT,
            group_id TEXT,
            is_primary_entry BOOLEAN,
            metadata TEXT
        )",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_dynamic_routes_path ON dynamic_routes(path)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_dynamic_routes_type ON dynamic_routes(route_type)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_dynamic_routes_enabled ON dynamic_routes(enabled)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_dynamic_routes_priority ON dynamic_routes(priority DESC)",
        [],
    )?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_dynamic_routes_inline_template ON dynamic_routes(inline_template)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_dynamic_routes_template_path ON dynamic_routes(template_path)", [])?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_dynamic_routes_group_id ON dynamic_routes(group_id)",
        [],
    )?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_dynamic_routes_is_primary_entry ON dynamic_routes(is_primary_entry)", [])?;

    // 创建动态路由表更新时间戳触发器
    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS update_dynamic_routes_timestamp
        AFTER UPDATE ON dynamic_routes
        FOR EACH ROW
        BEGIN
            UPDATE dynamic_routes SET updated_at = datetime('now') WHERE id = NEW.id;
        END",
        [],
    )?;

    println!("✅ 数据库表结构创建完成");
    Ok(())
}

/// 插入默认数据
fn seed_default_data(conn: &rusqlite::Connection) -> Result<(), Box<dyn std::error::Error>> {
    // 检查是否已有用户
    let user_count: i64 = conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))?;

    if user_count == 0 {
        // 使用 Argon2 哈希默认密码
        use argon2::password_hash::{PasswordHasher, SaltString};
        let argon2 = argon2::Argon2::default();
        let salt = SaltString::generate(&mut rand::thread_rng());
        let password_hash = argon2
            .hash_password(b"admin123", &salt)
            .map_err(|e| format!("密码哈希失败: {}", e))?
            .to_string();

        let _ = conn.execute(
            "INSERT INTO users (username, password, email, role, status) VALUES (?, ?, ?, ?, ?)",
            [
                "admin",
                &password_hash,
                "admin@example.com",
                "admin",
                "active",
            ],
        )?;
        println!("✅ 默认管理员用户已创建 (用户名: admin, 密码: admin123)");
    }

    // 检查是否已有设置
    let setting_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM settings", [], |row| row.get(0))?;

    if setting_count == 0 {
        // 插入默认设置（表为空时）
        let default_settings = vec![
            // 外观设置
            (
                "background_image",
                "/img/test.webp",
                "string",
                "页面背景图片路径",
                "appearance",
            ),
            (
                "mobile_background_image",
                "/img/mobile-test.webp",
                "string",
                "移动端背景图片",
                "appearance",
            ),
            (
                "global_opacity",
                "0.15",
                "number",
                "全局透明度 (0-1)",
                "appearance",
            ),
            (
                "background_size",
                "cover",
                "string",
                "背景图片尺寸 (cover, contain, auto)",
                "appearance",
            ),
            (
                "background_position",
                "center",
                "string",
                "背景图片位置",
                "appearance",
            ),
            (
                "background_repeat",
                "no-repeat",
                "string",
                "背景图片重复方式",
                "appearance",
            ),
            (
                "background_attachment",
                "fixed",
                "string",
                "背景图片滚动方式",
                "appearance",
            ),
            (
                "blur_amount",
                "20px",
                "string",
                "背景模糊程度",
                "appearance",
            ),
            (
                "saturate_amount",
                "180%",
                "string",
                "背景饱和度",
                "appearance",
            ),
            (
                "dark_mode_enabled",
                "false",
                "boolean",
                "是否启用暗色模式",
                "appearance",
            ),
            (
                "navbar_glass_color",
                "rgba(60, 60, 60, 0.6)",
                "string",
                "导航栏毛玻璃颜色",
                "appearance",
            ),
            (
                "navbar_text_color",
                "#ffffff",
                "string",
                "导航栏文字颜色",
                "appearance",
            ),
            (
                "card_glass_color",
                "rgba(220, 138, 221, 0.2)",
                "string",
                "页面卡片毛玻璃颜色",
                "appearance",
            ),
            (
                "footer_glass_color",
                "rgba(220, 138, 221, 0.25)",
                "string",
                "底栏毛玻璃颜色",
                "appearance",
            ),
            (
                "floating_text_enabled",
                "false",
                "boolean",
                "是否启用飘字效果",
                "appearance",
            ),
            (
                "floating_texts",
                "[\"perfect\",\"good\",\"excellent\",\"extraordinary\",\"legend\"]",
                "json",
                "飘字效果文本列表",
                "appearance",
            ),
            // 模板设置
            (
                "template_name",
                "欢迎来到我的博客",
                "string",
                "个人主页标题",
                "template",
            ),
            (
                "template_greting",
                "这是一个使用 Rust 语言构建的个人博客系统，支持文章管理、数据分析等功能。",
                "string",
                "首页欢迎语",
                "template",
            ),
            ("template_year", "2026", "string", "版权年份", "template"),
            (
                "template_foods",
                "我的博客",
                "string",
                "页脚信息",
                "template",
            ),
            (
                "template_article_title",
                "true",
                "boolean",
                "是否显示文章标题",
                "template",
            ),
            (
                "template_article_title_prefix",
                "文章",
                "string",
                "文章标题前缀",
                "template",
            ),
            (
                "template_switch_notice",
                "true",
                "boolean",
                "是否显示切换界面提示",
                "template",
            ),
            (
                "template_switch_notice_text",
                "回来继续阅读",
                "string",
                "切换标签页时显示的提示文字",
                "template",
            ),
            (
                "external_link_warning",
                "true",
                "boolean",
                "是否启用外部链接跳转警告",
                "template",
            ),
            (
                "external_link_whitelist",
                "github.com,gitee.com,stackoverflow.com",
                "string",
                "外部链接白名单（逗号分隔的域名）",
                "template",
            ),
            (
                "external_link_warning_text",
                "您即将离开本站，前往外部链接",
                "string",
                "外部链接警告提示文字",
                "template",
            ),
            (
                "passage_summarize_enabled",
                "true",
                "boolean",
                "是否启用文章摘要功能",
                "template",
            ),
            // Live2D 设置
            (
                "live2d_enabled",
                "false",
                "boolean",
                "是否启用 Live2D 看板娘",
                "template",
            ),
            (
                "live2d_show_on_index",
                "true",
                "boolean",
                "是否在首页显示 Live2D",
                "template",
            ),
            (
                "live2d_show_on_passage",
                "true",
                "boolean",
                "是否在文章页显示 Live2D",
                "template",
            ),
            (
                "live2d_show_on_collect",
                "true",
                "boolean",
                "是否在归档页显示 Live2D",
                "template",
            ),
            (
                "live2d_show_on_about",
                "true",
                "boolean",
                "是否在关于页显示 Live2D",
                "template",
            ),
            (
                "live2d_show_on_admin",
                "false",
                "boolean",
                "是否在管理页显示 Live2D",
                "template",
            ),
            (
                "live2d_model_id",
                "1",
                "string",
                "Live2D 模型 ID",
                "template",
            ),
            (
                "live2d_model_path",
                "",
                "string",
                "Live2D 自定义模型路径（留空使用 CDN）",
                "template",
            ),
            (
                "live2d_cdn_path",
                "https://unpkg.com/live2d-widget-model@1.0.5/",
                "string",
                "Live2D CDN 路径",
                "template",
            ),
            (
                "live2d_position",
                "right",
                "string",
                "Live2D 显示位置（left/right）",
                "template",
            ),
            ("live2d_width", "280px", "string", "Live2D 宽度", "template"),
            (
                "live2d_height",
                "250px",
                "string",
                "Live2D 高度",
                "template",
            ),
            // 赞助设置
            (
                "sponsor_enabled",
                "false",
                "boolean",
                "是否启用赞助功能",
                "template",
            ),
            (
                "sponsor_title",
                "感谢您的支持",
                "string",
                "赞助模态框标题",
                "template",
            ),
            (
                "sponsor_image",
                "/img/avatar.webp",
                "string",
                "赞助图片路径",
                "template",
            ),
            (
                "sponsor_description",
                "如果您觉得这个博客对您有帮助，欢迎赞助支持！",
                "string",
                "赞助描述文字",
                "template",
            ),
            (
                "sponsor_button_text",
                "❤️ 赞助支持",
                "string",
                "赞助按钮文字",
                "template",
            ),
            // 全局设置
            (
                "global_avatar",
                "/img/avatar.webp",
                "string",
                "全局头像路径",
                "template",
            ),
            // 附件设置
            (
                "attachment_default_visibility",
                "public",
                "string",
                "附件默认可见性",
                "template",
            ),
            (
                "attachment_max_size",
                "524288000",
                "number",
                "附件最大文件大小（字节）",
                "template",
            ),
            (
                "attachment_allowed_types",
                "jpg,jpeg,png,gif,mp4,mp3,pdf,doc,docx,xls,xlsx,ppt,pptx,zip,rar,7z,tar,gz",
                "string",
                "附件允许的文件类型",
                "template",
            ),
            // 音乐设置
            (
                "music_enabled",
                "false",
                "boolean",
                "是否启用音乐播放器",
                "appearance",
            ),
            (
                "music_auto_play",
                "false",
                "boolean",
                "音乐是否自动播放",
                "appearance",
            ),
            (
                "music_control_size",
                "medium",
                "string",
                "音乐控件大小 (small, medium, large)",
                "appearance",
            ),
            (
                "music_custom_css",
                "",
                "string",
                "音乐播放器自定义CSS样式",
                "appearance",
            ),
            (
                "music_player_color",
                "rgba(66, 133, 244, 0.9)",
                "string",
                "音乐播放器颜色 (RGBA格式)",
                "appearance",
            ),
            (
                "music_position",
                "bottom-right",
                "string",
                "音乐播放器显示位置 (top-left, top-right, bottom-left, bottom-right)",
                "template",
            ),
            // 备案设置
            (
                "beian_enabled",
                "false",
                "boolean",
                "是否启用备案信息",
                "template",
            ),
            ("icp_number", "", "string", "ICP 备案号", "template"),
            (
                "police_record_code",
                "",
                "string",
                "公安备案代码（用于链接）",
                "template",
            ),
            (
                "police_record_content",
                "",
                "string",
                "公安备案内容（显示文字）",
                "template",
            ),
            // 文章历史版本管理设置
            (
                "passage_history.enabled",
                "true",
                "boolean",
                "是否启用文章历史版本管理",
                "passage",
            ),
            (
                "passage_history.storage_mode",
                "filesystem",
                "string",
                "存储模式：filesystem（文件系统）或 database（仅数据库）",
                "passage",
            ),
            (
                "passage_history.history_dir",
                "markdown/.history",
                "string",
                "历史版本存储目录",
                "passage",
            ),
            (
                "passage_history.max_versions",
                "50",
                "number",
                "保留历史版本的最大数量（0 表示不限制）",
                "passage",
            ),
            (
                "passage_history.enable_deduplication",
                "true",
                "boolean",
                "是否启用内容去重（相同内容不重复存储）",
                "passage",
            ),
            (
                "passage_history.save_on_title_change",
                "true",
                "boolean",
                "标题变化时自动保存版本",
                "passage",
            ),
            (
                "passage_history.save_on_content_change",
                "true",
                "boolean",
                "内容变化时自动保存版本",
                "passage",
            ),
            (
                "passage_history.save_on_tags_change",
                "true",
                "boolean",
                "标签变化时自动保存版本",
                "passage",
            ),
            (
                "passage_history.save_on_summary_change",
                "true",
                "boolean",
                "摘要变化时自动保存版本",
                "passage",
            ),
            (
                "passage_history.save_on_category_change",
                "false",
                "boolean",
                "分类变化时自动保存版本",
                "passage",
            ),
            (
                "passage_history.save_on_cover_change",
                "false",
                "boolean",
                "封面图片变化时自动保存版本",
                "passage",
            ),
            (
                "passage_history.enable_undo_redo",
                "true",
                "boolean",
                "是否启用撤销/重做功能",
                "passage",
            ),
        ];

        for (key, value, setting_type, description, category) in default_settings {
            let _ = conn.execute(
                "INSERT INTO settings (key, value, type, description, category) VALUES (?, ?, ?, ?, ?)",
                [key, value, setting_type, description, category],
            )?;
        }
        println!("✅ 默认设置已插入");
    } else {
        // 补全缺失的设置项（表不为空时）
        let default_settings = vec![
            // 外观设置
            (
                "background_image",
                "/img/test.webp",
                "string",
                "页面背景图片路径",
                "appearance",
            ),
            (
                "mobile_background_image",
                "/img/mobile-test.webp",
                "string",
                "移动端背景图片",
                "appearance",
            ),
            (
                "global_opacity",
                "0.15",
                "number",
                "全局透明度 (0-1)",
                "appearance",
            ),
            (
                "background_size",
                "cover",
                "string",
                "背景图片尺寸 (cover, contain, auto)",
                "appearance",
            ),
            (
                "background_position",
                "center",
                "string",
                "背景图片位置",
                "appearance",
            ),
            (
                "background_repeat",
                "no-repeat",
                "string",
                "背景图片重复方式",
                "appearance",
            ),
            (
                "background_attachment",
                "fixed",
                "string",
                "背景图片滚动方式",
                "appearance",
            ),
            (
                "blur_amount",
                "20px",
                "string",
                "背景模糊程度",
                "appearance",
            ),
            (
                "saturate_amount",
                "180%",
                "string",
                "背景饱和度",
                "appearance",
            ),
            (
                "dark_mode_enabled",
                "false",
                "boolean",
                "是否启用暗色模式",
                "appearance",
            ),
            (
                "navbar_glass_color",
                "rgba(60, 60, 60, 0.6)",
                "string",
                "导航栏毛玻璃颜色",
                "appearance",
            ),
            (
                "navbar_text_color",
                "#ffffff",
                "string",
                "导航栏文字颜色",
                "appearance",
            ),
            (
                "card_glass_color",
                "rgba(220, 138, 221, 0.2)",
                "string",
                "页面卡片毛玻璃颜色",
                "appearance",
            ),
            (
                "footer_glass_color",
                "rgba(220, 138, 221, 0.25)",
                "string",
                "底栏毛玻璃颜色",
                "appearance",
            ),
            (
                "floating_text_enabled",
                "false",
                "boolean",
                "是否启用飘字效果",
                "appearance",
            ),
            (
                "floating_texts",
                "[\"perfect\",\"good\",\"excellent\",\"extraordinary\",\"legend\"]",
                "json",
                "飘字效果文本列表",
                "appearance",
            ),
            // 模板设置
            (
                "template_name",
                "欢迎来到我的博客",
                "string",
                "个人主页标题",
                "template",
            ),
            (
                "template_greting",
                "这是一个使用 Rust 语言构建的个人博客系统，支持文章管理、数据分析等功能。",
                "string",
                "首页欢迎语",
                "template",
            ),
            ("template_year", "2026", "string", "版权年份", "template"),
            (
                "template_foods",
                "我的博客",
                "string",
                "页脚信息",
                "template",
            ),
            (
                "template_article_title",
                "true",
                "boolean",
                "是否显示文章标题",
                "template",
            ),
            (
                "template_article_title_prefix",
                "文章",
                "string",
                "文章标题前缀",
                "template",
            ),
            (
                "template_switch_notice",
                "true",
                "boolean",
                "是否显示切换界面提示",
                "template",
            ),
            (
                "template_switch_notice_text",
                "回来继续阅读",
                "string",
                "切换标签页时显示的提示文字",
                "template",
            ),
            (
                "external_link_warning",
                "true",
                "boolean",
                "是否启用外部链接跳转警告",
                "template",
            ),
            (
                "external_link_whitelist",
                "github.com,gitee.com,stackoverflow.com",
                "string",
                "外部链接白名单（逗号分隔的域名）",
                "template",
            ),
            (
                "external_link_warning_text",
                "您即将离开本站，前往外部链接",
                "string",
                "外部链接警告提示文字",
                "template",
            ),
            (
                "passage_summarize_enabled",
                "true",
                "boolean",
                "是否启用文章摘要功能",
                "template",
            ),
            // Live2D 设置
            (
                "live2d_enabled",
                "false",
                "boolean",
                "是否启用 Live2D 看板娘",
                "template",
            ),
            (
                "live2d_show_on_index",
                "true",
                "boolean",
                "是否在首页显示 Live2D",
                "template",
            ),
            (
                "live2d_show_on_passage",
                "true",
                "boolean",
                "是否在文章页显示 Live2D",
                "template",
            ),
            (
                "live2d_show_on_collect",
                "true",
                "boolean",
                "是否在归档页显示 Live2D",
                "template",
            ),
            (
                "live2d_show_on_about",
                "true",
                "boolean",
                "是否在关于页显示 Live2D",
                "template",
            ),
            (
                "live2d_show_on_admin",
                "false",
                "boolean",
                "是否在管理页显示 Live2D",
                "template",
            ),
            (
                "live2d_model_id",
                "1",
                "string",
                "Live2D 模型 ID",
                "template",
            ),
            (
                "live2d_model_path",
                "",
                "string",
                "Live2D 自定义模型路径（留空使用 CDN）",
                "template",
            ),
            (
                "live2d_cdn_path",
                "https://unpkg.com/live2d-widget-model@1.0.5/",
                "string",
                "Live2D CDN 路径",
                "template",
            ),
            (
                "live2d_position",
                "right",
                "string",
                "Live2D 显示位置（left/right）",
                "template",
            ),
            ("live2d_width", "280px", "string", "Live2D 宽度", "template"),
            (
                "live2d_height",
                "250px",
                "string",
                "Live2D 高度",
                "template",
            ),
            // 赞助设置
            (
                "sponsor_enabled",
                "false",
                "boolean",
                "是否启用赞助功能",
                "template",
            ),
            (
                "sponsor_title",
                "感谢您的支持",
                "string",
                "赞助模态框标题",
                "template",
            ),
            (
                "sponsor_image",
                "/img/avatar.webp",
                "string",
                "赞助图片路径",
                "template",
            ),
            (
                "sponsor_description",
                "如果您觉得这个博客对您有帮助，欢迎赞助支持！",
                "string",
                "赞助描述文字",
                "template",
            ),
            (
                "sponsor_button_text",
                "❤️ 赞助支持",
                "string",
                "赞助按钮文字",
                "template",
            ),
            // 全局设置
            (
                "global_avatar",
                "/img/avatar.webp",
                "string",
                "全局头像路径",
                "template",
            ),
            // 附件设置
            (
                "attachment_default_visibility",
                "public",
                "string",
                "附件默认可见性",
                "template",
            ),
            (
                "attachment_max_size",
                "524288000",
                "number",
                "附件最大文件大小（字节）",
                "template",
            ),
            (
                "attachment_allowed_types",
                "jpg,jpeg,png,gif,mp4,mp3,pdf,doc,docx,xls,xlsx,ppt,pptx,zip,rar,7z,tar,gz",
                "string",
                "附件允许的文件类型",
                "template",
            ),
            // 音乐设置
            (
                "music_enabled",
                "false",
                "boolean",
                "是否启用音乐播放器",
                "appearance",
            ),
            (
                "music_auto_play",
                "false",
                "boolean",
                "音乐是否自动播放",
                "appearance",
            ),
            (
                "music_control_size",
                "medium",
                "string",
                "音乐控件大小 (small, medium, large)",
                "appearance",
            ),
            (
                "music_custom_css",
                "",
                "string",
                "音乐播放器自定义CSS样式",
                "appearance",
            ),
            (
                "music_player_color",
                "rgba(66, 133, 244, 0.9)",
                "string",
                "音乐播放器颜色 (RGBA格式)",
                "appearance",
            ),
            (
                "music_position",
                "bottom-right",
                "string",
                "音乐播放器显示位置 (top-left, top-right, bottom-left, bottom-right)",
                "template",
            ),
            // 备案设置
            (
                "beian_enabled",
                "false",
                "boolean",
                "是否启用备案信息",
                "template",
            ),
            ("icp_number", "", "string", "ICP 备案号", "template"),
            (
                "police_record_code",
                "",
                "string",
                "公安备案代码（用于链接）",
                "template",
            ),
            (
                "police_record_content",
                "",
                "string",
                "公安备案内容（显示文字）",
                "template",
            ),
            // 文章历史版本管理设置
            (
                "passage_history.enabled",
                "true",
                "boolean",
                "是否启用文章历史版本管理",
                "passage",
            ),
            (
                "passage_history.storage_mode",
                "filesystem",
                "string",
                "存储模式：filesystem（文件系统）或 database（仅数据库）",
                "passage",
            ),
            (
                "passage_history.history_dir",
                "markdown/.history",
                "string",
                "历史版本存储目录",
                "passage",
            ),
            (
                "passage_history.max_versions",
                "50",
                "number",
                "保留历史版本的最大数量（0 表示不限制）",
                "passage",
            ),
            (
                "passage_history.enable_deduplication",
                "true",
                "boolean",
                "是否启用内容去重（相同内容不重复存储）",
                "passage",
            ),
            (
                "passage_history.save_on_title_change",
                "true",
                "boolean",
                "标题变化时自动保存版本",
                "passage",
            ),
            (
                "passage_history.save_on_content_change",
                "true",
                "boolean",
                "内容变化时自动保存版本",
                "passage",
            ),
            (
                "passage_history.save_on_tags_change",
                "true",
                "boolean",
                "标签变化时自动保存版本",
                "passage",
            ),
            (
                "passage_history.save_on_summary_change",
                "true",
                "boolean",
                "摘要变化时自动保存版本",
                "passage",
            ),
            (
                "passage_history.save_on_category_change",
                "false",
                "boolean",
                "分类变化时自动保存版本",
                "passage",
            ),
            (
                "passage_history.save_on_cover_change",
                "false",
                "boolean",
                "封面图片变化时自动保存版本",
                "passage",
            ),
            (
                "passage_history.enable_undo_redo",
                "true",
                "boolean",
                "是否启用撤销/重做功能",
                "passage",
            ),
        ];

        // 获取所有现有设置的键名
        let mut existing_keys = std::collections::HashSet::new();
        let mut stmt = conn.prepare("SELECT key FROM settings")?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let key: String = row.get(0)?;
            existing_keys.insert(key);
        }
        drop(rows);

        // 只插入不存在的设置项
        let mut inserted_count = 0;
        for (key, value, setting_type, description, category) in default_settings {
            if !existing_keys.contains(key) {
                let _ = conn.execute(
                    "INSERT INTO settings (key, value, type, description, category) VALUES (?, ?, ?, ?, ?)",
                    [key, value, setting_type, description, category],
                )?;
                inserted_count += 1;
            }
        }

        if inserted_count > 0 {
            println!("✅ 补全了 {} 个缺失的默认设置", inserted_count);
        }
    }

    // 检查是否已有文章（仅在 debug 模式下插入示例文章）
    #[cfg(debug_assertions)]
    {
        let passage_count: i64 =
            conn.query_row("SELECT COUNT(*) FROM passages", [], |row| row.get(0))?;

        if passage_count == 0 {
            // 插入示例文章
            let sample_passages = vec![
                (
                    "欢迎使用 RustBlog",
                    "# 欢迎使用 RustBlog\n\n这是一个使用 Rust 语言和 Actix-web 框架构建的现代化博客系统。\n\n## 主要特性\n\n- 🚀 高性能：基于 Rust 构建，内存安全且高效\n- 🎨 现代化 UI：支持暗色模式和自定义主题\n- 🔒 安全：ECC 加密、Argon2 密码哈希\n- 📝 Markdown 支持：原生支持 Markdown 编写\n- 🎵 音乐播放器：支持背景音乐播放\n- 💬 评论系统：支持文章评论功能\n\n## 技术栈\n\n- **后端**：Rust + Actix-web\n- **数据库**：SQLite\n- **前端**：原生 JavaScript + CSS\n- **加密**：ECC (P-256) + AES-256\n\n欢迎开始你的博客之旅！",
                    "欢迎使用 RustBlog，这是一个基于 Rust 和 Actix-web 构建的现代化博客系统。",
                    "admin",
                    "[\"Rust\", \"博客\", \"教程\"]",
                    "技术",
                    "published",
                    "markdown/welcome.md",
                    "public",
                ),
                (
                    "Rust 语言入门指南",
                    "# Rust 语言入门指南\n\nRust 是一门系统编程语言，注重安全、并发和性能。\n\n## 为什么选择 Rust？\n\n1. **内存安全**：编译时保证内存安全，无需垃圾回收\n2. **高性能**：与 C++ 相当的性能，无运行时开销\n3. **并发安全**：类型系统防止数据竞争\n4. **现代工具链**：Cargo 包管理器，优秀的文档\n\n## Hello World\n\n```rust\nfn main() {\n    println!(\"Hello, World!\");\n}\n```\n\n## 所有权系统\n\nRust 的核心特性是所有权系统，它让 Rust 在没有垃圾回收的情况下保证内存安全。\n\n```rust\nlet s1 = String::from(\"hello\");\nlet s2 = s1; // s1 的所有权转移给 s2\n// println!(\"{}\", s1); // 错误！s1 不再有效\nprintln!(\"{}\", s2); // 正确\n```\n\n开始你的 Rust 之旅吧！",
                    "Rust 是一门系统编程语言，注重安全、并发和性能。本文介绍了 Rust 的核心特性和入门知识。",
                    "admin",
                    "[\"Rust\", \"编程\", \"入门\"]",
                    "编程",
                    "published",
                    "markdown/rust-guide.md",
                    "public",
                ),
                (
                    "Actix-web 快速上手",
                    "# Actix-web 快速上手\n\nActix-web 是一个强大、实用的 Rust Web 框架。\n\n## 创建新项目\n\n```bash\ncargo new my_api\ncd my_api\ncargo add actix-web\n```\n\n## 基本路由\n\n```rust\nuse actix_web::{web, App, HttpServer, HttpResponse};\n\n#[actix_web::main]\nasync fn main() -> std::io::Result<()> {\n    HttpServer::new(|| {\n        App::new()\n            .route(\"/\", web::get().to(hello))\n    })\n    .bind(\"127.0.0.1:8080\")?\n    .run()\n    .await\n}\n\nasync fn hello() -> HttpResponse {\n    HttpResponse::Ok().body(\"Hello World!\")\n}\n```\n\n## 处理 JSON\n\n```rust\nuse serde::{Deserialize, Serialize};\n\n#[derive(Serialize, Deserialize)]\nstruct User {\n    name: String,\n    age: u32,\n}\n\nasync fn create_user(user: web::Json<User>) -> HttpResponse {\n    HttpResponse::Ok().json(user)\n}\n```\n\nActix-web 是构建高性能 Web 应用的绝佳选择！",
                    "Actix-web 是一个强大、实用的 Rust Web 框架。本文介绍了如何快速上手使用 Actix-web 构建 Web 应用。",
                    "admin",
                    "[\"Rust\", \"Web\", \"框架\"]",
                    "技术",
                    "published",
                    "markdown/actix-web.md",
                    "public",
                ),
            ];

            for (title, content, summary, author, tags, category, status, file_path, visibility) in
                sample_passages
            {
                // 将 Markdown 转换为 HTML
                let html_content = convert_markdown_to_html(content);

                // 生成 UUID
                let uuid = crate::id_generator::generate_unique_id();

                match conn.execute(
                "INSERT OR IGNORE INTO passages (uuid, title, content, original_content, summary, author, tags, category, status, file_path, visibility, created_at, updated_at) 
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                rusqlite::params![
                    &uuid,
                    title,
                    html_content,
                    content,
                    summary,
                    author,
                    tags,
                    category,
                    status,
                    file_path,
                    visibility,
                    chrono::Utc::now(),
                    chrono::Utc::now(),
                ],
            ) {
                Ok(rows_affected) => {
                    if rows_affected == 0 {
                        tracing::info!("跳过已存在的文章: {}", title);
                    }
                },
                Err(e) => {
                    tracing::error!("插入文章 '{}' 失败: {}", title, e);
                    return Err(e.into());
                }
            }
            }

            println!("✅ 已插入 3 篇示例文章");
        }
    }

    // 检查是否已有主卡片数据
    let main_card_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM about_main_cards", [], |row| {
            row.get(0)
        })?;

    if main_card_count == 0 {
        // 插入主卡片示例
        let main_cards = vec![
            ("项目简介", "📖", "default", "", 1, true),
            ("核心特性", "⚡", "grid", "", 2, true),
            ("开发团队", "👥", "grid", "", 3, true),
            ("联系我们", "📞", "flex", "", 4, true),
            ("卡片使用指南", "🎯", "default", "", 5, true),
        ];

        let mut main_card_ids: std::collections::HashMap<String, i64> =
            std::collections::HashMap::new();

        for (title, icon, layout_type, custom_css, sort_order, is_enabled) in &main_cards {
            let now = chrono::Utc::now();
            match conn.execute(
                "INSERT INTO about_main_cards (title, icon, layout_type, custom_css, sort_order, is_enabled, created_at, updated_at) 
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                rusqlite::params![
                    title, icon, layout_type, custom_css, sort_order, is_enabled, now, now
                ],
            ) {
                Ok(_) => {
                    let id = conn.last_insert_rowid();
                    main_card_ids.insert(title.to_string(), id);
                },
                Err(e) => {
                    tracing::error!("插入主卡片 '{}' 失败: {}", title, e);
                    return Err(e.into());
                }
            }
        }

        // 插入次卡片示例
        let sub_cards = vec![
            // 项目简介
            (
                "项目简介",
                "欢迎",
                "欢迎来到我们的网站！这是一个专注于技术分享与知识管理的平台。",
                "",
                "",
                "default",
                "",
                1,
                true,
            ),
            (
                "项目简介",
                "目标",
                "我们的目标是构建一个开放、友好、专业的技术社区。",
                "",
                "",
                "default",
                "",
                2,
                true,
            ),
            // 核心特性
            (
                "核心特性",
                "高性能",
                "采用现代化技术栈，确保网站快速响应。",
                "🚀",
                "",
                "default",
                "",
                1,
                true,
            ),
            (
                "核心特性",
                "安全可靠",
                "多层安全防护机制，保护用户数据隐私。",
                "🔒",
                "",
                "default",
                "",
                2,
                true,
            ),
            (
                "核心特性",
                "全平台",
                "响应式设计，各类设备完美呈现。",
                "📱",
                "",
                "default",
                "",
                3,
                true,
            ),
            (
                "核心特性",
                "开放API",
                "提供完善的API接口，方便集成扩展。",
                "🌐",
                "",
                "default",
                "",
                4,
                true,
            ),
            // 开发团队
            (
                "开发团队",
                "技术总监",
                "负责平台架构设计与技术选型。",
                "swordreforge",
                "",
                "default",
                "",
                1,
                true,
            ),
            (
                "开发团队",
                "前端负责人",
                "专注于用户体验与交互设计。",
                "swordreforge",
                "",
                "default",
                "",
                2,
                true,
            ),
            (
                "开发团队",
                "后端工程师",
                "负责服务器端逻辑与数据库设计。",
                "swordreforge",
                "",
                "default",
                "",
                3,
                true,
            ),
            // 联系我们
            (
                "联系我们",
                "电子邮件",
                "zhujian_20060818@qq.com",
                "📧",
                "mailto:zhujian_20060818@qq.com",
                "default",
                "",
                1,
                true,
            ),
            (
                "联系我们",
                "GitHub",
                "github.com/simpleblog",
                "🐙",
                "https://github.com/simpleblog",
                "default",
                "",
                2,
                true,
            ),
            (
                "联系我们",
                "社交媒体",
                "@nobody",
                "🐦",
                "https://twitter.com/ourproject",
                "default",
                "",
                3,
                true,
            ),
            // 卡片使用指南
            (
                "卡片使用指南",
                "主卡片介绍",
                "主卡片用于组织和分类内容，可以设置标题、图标和布局方式。每个主卡片下可以包含多个次卡片，形成层级结构。",
                "📁",
                "",
                "default",
                "",
                1,
                true,
            ),
            (
                "卡片使用指南",
                "次卡片介绍",
                "次卡片用于展示具体内容，可以包含标题、描述、图标和链接。次卡片归属于某个主卡片，支持自定义布局样式。",
                "📄",
                "",
                "default",
                "",
                2,
                true,
            ),
            (
                "卡片使用指南",
                "布局类型说明",
                "支持三种布局类型：default（默认布局）、grid（网格布局）、flex（弹性布局）。在主卡片或次卡片中设置 layout_type 即可应用不同布局。",
                "🎨",
                "",
                "default",
                "",
                3,
                true,
            ),
            (
                "卡片使用指南",
                "自定义样式",
                "可以通过 custom_css 字段为卡片添加自定义 CSS 样式，实现个性化的视觉效果。支持所有标准 CSS 属性。",
                "✨",
                "",
                "default",
                "",
                4,
                true,
            ),
            (
                "卡片使用指南",
                "排序与启用",
                "使用 sort_order 字段控制卡片显示顺序，数值越小越靠前。通过 is_enabled 字段可以控制卡片的显示与隐藏。",
                "🔢",
                "",
                "default",
                "",
                5,
                true,
            ),
            (
                "卡片使用指南",
                "管理入口",
                "登录管理后台，访问关于页面设置即可管理所有卡片。支持创建、编辑、删除和排序操作，实时预览效果。",
                "⚙️",
                "/admin",
                "default",
                "",
                6,
                true,
            ),
        ];

        for (
            main_card_title,
            title,
            description,
            icon,
            link_url,
            layout_type,
            custom_css,
            sort_order,
            is_enabled,
        ) in &sub_cards
        {
            if let Some(&main_card_id) = main_card_ids.get(*main_card_title) {
                let now = chrono::Utc::now();
                match conn.execute(
                    "INSERT INTO about_sub_cards (main_card_id, title, description, icon, link_url, layout_type, custom_css, sort_order, is_enabled, created_at, updated_at) 
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    rusqlite::params![
                        main_card_id, title, description, icon, link_url, layout_type, custom_css, sort_order, is_enabled, now, now
                    ],
                ) {
                    Ok(_) => {},
                    Err(e) => {
                        tracing::error!("插入次卡片 '{}' 失败: {}", title, e);
                        return Err(e.into());
                    }
                }
            }
        }

        println!("✅ 已插入关于页面卡片示例数据");
    }

    // 检查是否已有友链
    let friend_link_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM friend_links", [], |row| row.get(0))?;

    if friend_link_count == 0 {
        // 插入测试友链
        let sample_friend_links = vec![
            (
                "swordreforge",
                "/img/avatar.webp",
                "https://github.com/swordreforge",
                "Rust 开发者，热爱开源技术",
            ),
            (
                "Rust 官方博客",
                "/img/avatar.webp",
                "https://blog.rust-lang.org/",
                "Rust 编程语言官方博客",
            ),
            (
                "Rust 中文社区",
                "/img/avatar.webp",
                "https://rust-lang-cn.org/",
                "Rust 中文学习社区",
            ),
            (
                "Actix-web",
                "/img/avatar.webp",
                "https://actix.rs/",
                "强大的 Rust Web 框架",
            ),
            (
                "Mozilla Hacks",
                "/img/avatar.webp",
                "https://hacks.mozilla.org/",
                "Mozilla 开发者博客",
            ),
        ];

        for (nickname, avatar_url, link_url, motto) in sample_friend_links {
            conn.execute(
                "INSERT INTO friend_links (nickname, avatar_url, link_url, motto, sort_order, is_enabled, created_at, updated_at) 
                 VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                params![nickname, avatar_url, link_url, motto, 0, true],
            )?;
        }

        println!("✅ 已插入开发者友链示例数据");
    }

    println!("✅ 默认数据插入完成");
    Ok(())
}

/// 将 Markdown 转换为 HTML
#[cfg(debug_assertions)]
fn convert_markdown_to_html(markdown: &str) -> String {
    use pulldown_cmark::{Options, Parser, html};

    // 配置选项，启用表格和其他扩展
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(markdown, options);
    // HTML 通常比 Markdown 大 1.5-2 倍，预分配容量避免重分配
    let mut html_output = String::with_capacity(markdown.len() * 2);
    html::push_html(&mut html_output, parser);

    html_output
}
