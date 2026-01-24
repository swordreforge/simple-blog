use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use tokio::sync::RwLock;

use super::models::*;
use super::repositories::*;

/// 全局数据库连接池
static DB_POOL: tokio::sync::OnceCell<Pool<SqliteConnectionManager>> = tokio::sync::OnceCell::const_new();

/// 初始化数据库
pub fn init_db(db_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 创建数据库目录
    if let Some(parent) = std::path::Path::new(db_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    // 创建连接池
    let manager = SqliteConnectionManager::file(db_path);
    let pool = Pool::builder()
        .max_size(15)
        .min_idle(Some(5))
        .build(manager)?;

    // 获取连接并初始化表结构
    {
        let conn = pool.get()?;
        create_tables(&conn)?;
        seed_default_data(&conn)?;
    }

    // 保存连接池到全局变量
    DB_POOL.set(pool).map_err(|_| "数据库已初始化")?;

    println!("✅ 数据库初始化成功: {}", db_path);
    Ok(())
}

/// 获取数据库连接池
pub async fn get_db_pool() -> Result<Pool<SqliteConnectionManager>, String> {
    DB_POOL.get()
        .cloned()
        .ok_or_else(|| "数据库未初始化".to_string())
}

/// 创建所有数据库表
fn create_tables(conn: &rusqlite::Connection) -> Result<(), Box<dyn std::error::Error>> {
    // 创建文章表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS passages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            original_content TEXT,
            summary TEXT,
            author TEXT DEFAULT '管理员',
            tags TEXT DEFAULT '[]',
            category TEXT DEFAULT '未分类',
            status TEXT DEFAULT 'published',
            file_path TEXT,
            visibility TEXT DEFAULT 'public',
            is_scheduled INTEGER DEFAULT 0,
            published_at DATETIME,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // 创建文章表索引
    conn.execute("CREATE UNIQUE INDEX IF NOT EXISTS idx_passages_file_path ON passages(file_path)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_passages_status ON passages(status)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_passages_category ON passages(category)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_passages_created_at ON passages(created_at)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_passages_status_created ON passages(status, created_at DESC)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_passages_category_status ON passages(category, status)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_passages_visibility ON passages(visibility)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_passages_published_at ON passages(published_at)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_passages_scheduled ON passages(is_scheduled, published_at)", [])?;

    // 创建用户表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
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
    conn.execute("CREATE INDEX IF NOT EXISTS idx_users_username ON users(username)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)", [])?;

    // 创建访客表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS visitors (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            ip TEXT NOT NULL,
            user_agent TEXT,
            visit_date TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    conn.execute("CREATE UNIQUE INDEX IF NOT EXISTS idx_visitors_ip_date ON visitors(ip, visit_date)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_visitors_date ON visitors(visit_date)", [])?;

    // 创建文章阅读记录表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS article_views (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            passage_id INTEGER NOT NULL,
            ip TEXT NOT NULL,
            user_agent TEXT,
            country TEXT DEFAULT '',
            city TEXT DEFAULT '',
            region TEXT DEFAULT '',
            view_date TEXT NOT NULL,
            view_time DATETIME DEFAULT CURRENT_TIMESTAMP,
            duration INTEGER DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (passage_id) REFERENCES passages(id) ON DELETE CASCADE
        )",
        [],
    )?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_article_views_passage_id ON article_views(passage_id)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_article_views_passage_date ON article_views(passage_id, view_date)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_article_views_ip_date ON article_views(ip, view_date)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_article_views_date ON article_views(view_date)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_article_views_country ON article_views(country)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_article_views_city_region ON article_views(city, region)", [])?;

    // 创建评论表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS comments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL,
            content TEXT NOT NULL,
            passage_id INTEGER NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (passage_id) REFERENCES passages(id) ON DELETE CASCADE
        )",
        [],
    )?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_comments_passage_id ON comments(passage_id)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_comments_passage_created ON comments(passage_id, created_at DESC)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_comments_created_at ON comments(created_at)", [])?;

    // 创建设置表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
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
    conn.execute("CREATE UNIQUE INDEX IF NOT EXISTS idx_settings_key ON settings(key)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_settings_category ON settings(category)", [])?;

    // 创建关于页面主卡片表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS about_main_cards (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
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
    conn.execute("CREATE INDEX IF NOT EXISTS idx_main_cards_sort ON about_main_cards(sort_order)", [])?;

    // 创建关于页面次卡片表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS about_sub_cards (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
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
    conn.execute("CREATE INDEX IF NOT EXISTS idx_sub_cards_main_id ON about_sub_cards(main_card_id)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_sub_cards_sort ON about_sub_cards(sort_order)", [])?;

    // 创建分类表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
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
    conn.execute("CREATE UNIQUE INDEX IF NOT EXISTS idx_categories_name ON categories(name)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_categories_sort ON categories(sort_order)", [])?;

    // 创建标签表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
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
    conn.execute("CREATE UNIQUE INDEX IF NOT EXISTS idx_tags_name ON tags(name)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_tags_category ON tags(category_id)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_tags_sort ON tags(sort_order)", [])?;

    // 创建附件表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS attachments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_name TEXT NOT NULL,
            stored_name TEXT NOT NULL,
            file_path TEXT NOT NULL,
            file_type TEXT NOT NULL,
            content_type TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            passage_id INTEGER,
            visibility TEXT DEFAULT 'public',
            show_in_passage INTEGER DEFAULT 1,
            uploaded_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (passage_id) REFERENCES passages(id) ON DELETE CASCADE
        )",
        [],
    )?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_attachments_passage_id ON attachments(passage_id)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_attachments_type ON attachments(file_type)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_attachments_visibility ON attachments(visibility)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_attachments_uploaded_at ON attachments(uploaded_at)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_attachments_passage_visibility ON attachments(passage_id, visibility)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_attachments_show_in_passage ON attachments(show_in_passage)", [])?;

    // 创建音乐表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS music_tracks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
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
    conn.execute("CREATE INDEX IF NOT EXISTS idx_music_tracks_created_at ON music_tracks(created_at)", [])?;

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
        let password_hash = argon2.hash_password(b"admin123", &salt)
            .map_err(|e| format!("密码哈希失败: {}", e))?
            .to_string();

        conn.execute(
            "INSERT INTO users (username, password, email, role, status) VALUES (?, ?, ?, ?, ?)",
            ["admin", &password_hash, "admin@example.com", "admin", "active"],
        )?;
        println!("✅ 默认管理员用户已创建 (用户名: admin, 密码: admin123)");
    }

    // 检查是否已有设置
    let setting_count: i64 = conn.query_row("SELECT COUNT(*) FROM settings", [], |row| row.get(0))?;
    
    if setting_count == 0 {
        // 插入默认设置
        let default_settings = vec![
            ("background_image", "/img/test.webp", "string", "页面背景图片路径", "appearance"),
            ("global_opacity", "0.15", "number", "全局透明度 (0-1)", "appearance"),
            ("background_size", "cover", "string", "背景图片尺寸", "appearance"),
            ("background_position", "center", "string", "背景图片位置", "appearance"),
            ("background_repeat", "no-repeat", "string", "背景图片重复方式", "appearance"),
            ("background_attachment", "fixed", "string", "背景图片滚动方式", "appearance"),
            ("blur_amount", "20px", "string", "背景模糊程度", "appearance"),
            ("saturate_amount", "180%", "string", "背景饱和度", "appearance"),
            ("dark_mode_enabled", "false", "boolean", "是否启用暗色模式", "appearance"),
            ("navbar_glass_color", "rgba(220, 138, 221, 0.15)", "string", "导航栏毛玻璃颜色", "appearance"),
            ("navbar_text_color", "#333333", "string", "导航栏文字颜色", "appearance"),
            ("card_glass_color", "rgba(220, 138, 221, 0.2)", "string", "页面卡片毛玻璃颜色", "appearance"),
            ("footer_glass_color", "rgba(220, 138, 221, 0.25)", "string", "底栏毛玻璃颜色", "appearance"),
            ("floating_text_enabled", "false", "boolean", "是否启用飘字效果", "appearance"),
            ("floating_texts", "[\"perfect\",\"good\",\"excellent\"]", "json", "飘字效果文本列表", "appearance"),
            ("mobile_background_image", "/img/mobile-test.webp", "string", "移动端背景图片", "appearance"),
            ("template_name", "欢迎来到我的博客", "string", "个人主页标题", "template"),
            ("template_greting", "这是一个使用 Rust 语言构建的个人博客系统", "string", "首页欢迎语", "template"),
            ("template_year", "2026", "string", "版权年份", "template"),
            ("template_foods", "我的博客", "string", "页脚信息", "template"),
            ("live2d_enabled", "false", "boolean", "是否启用 Live2D 看板娘", "template"),
            ("live2d_model_id", "1", "string", "Live2D 模型 ID", "template"),
            ("live2d_model_path", "", "string", "Live2D 自定义模型路径", "template"),
            ("live2d_cdn_path", "https://unpkg.com/live2d-widget-model@1.0.5/", "string", "Live2D CDN 路径", "template"),
            ("live2d_position", "right", "string", "Live2D 显示位置", "template"),
            ("live2d_width", "280px", "string", "Live2D 宽度", "template"),
            ("live2d_height", "250px", "string", "Live2D 高度", "template"),
            ("music_enabled", "false", "boolean", "是否启用音乐播放器", "appearance"),
            ("music_auto_play", "false", "boolean", "音乐是否自动播放", "appearance"),
            ("music_control_size", "medium", "string", "音乐控件大小", "appearance"),
            ("music_player_color", "rgba(66, 133, 244, 0.9)", "string", "音乐播放器颜色", "appearance"),
            ("music_position", "bottom-right", "string", "音乐播放器显示位置", "template"),
        ];

        for (key, value, setting_type, description, category) in default_settings {
            conn.execute(
                "INSERT INTO settings (key, value, type, description, category) VALUES (?, ?, ?, ?, ?)",
                [key, value, setting_type, description, category],
            )?;
        }
        println!("✅ 默认设置已插入");
    }

    println!("✅ 默认数据插入完成");
    Ok(())
}
