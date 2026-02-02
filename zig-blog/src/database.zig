const std = @import("std");
const sqlite = @import("sqlite");
const models = @import("models.zig");

// 数据库连接池
pub const DatabasePool = struct {
    allocator: std.mem.Allocator,
    connections: std.ArrayList(*sqlite.Db),
    max_connections: u32,
    min_idle: u32,
    mutex: std.Thread.Mutex,
    available_count: usize,

    const Self = @This();

    pub fn init(allocator: std.mem.Allocator, db_path: []const u8, max_conn: u32, min_idle: u32) !*Self {
        var pool = try allocator.create(Self);
        pool.* = Self{
            .allocator = allocator,
            .connections = std.ArrayList(*sqlite.Db).init(allocator),
            .max_connections = max_conn,
            .min_idle = min_idle,
            .mutex = std.Thread.Mutex{},
            .available_count = 0,
        };

        // 初始化最小空闲连接
        for (0..min_idle) |_| {
            const db = try openDatabase(allocator, db_path);
            try pool.connections.append(db);
            pool.available_count += 1;
        }

        return pool;
    }

    pub fn deinit(self: *Self) void {
        for (self.connections.items) |conn| {
            conn.deinit();
            self.allocator.destroy(conn);
        }
        self.connections.deinit();
        self.allocator.destroy(self);
    }

    pub fn acquire(self: *Self) !*sqlite.Db {
        self.mutex.lock();
        defer self.mutex.unlock();

        if (self.available_count > 0) {
            self.available_count -= 1;
            return self.connections.items[self.available_count];
        }

        if (self.connections.items.len < self.max_connections) {
            // 创建新连接（这里简化处理，实际需要配置）
            return error.PoolExhausted;
        }

        return error.PoolExhausted;
    }

    pub fn release(self: *Self, conn: *sqlite.Db) void {
        self.mutex.lock();
        defer self.mutex.unlock();

        if (self.available_count < self.connections.items.len) {
            self.connections.items[self.available_count] = conn;
            self.available_count += 1;
        }
    }
};

// 打开数据库
fn openDatabase(allocator: std.mem.Allocator, path: []const u8) !*sqlite.Db {
    const db = try allocator.create(sqlite.Db);
    errdefer allocator.destroy(db);

    try db.init(.{
        .mode = .{ .File = path },
        .open_flags = .{
            .write = true,
            .create = true,
        },
    });

    // 优化数据库设置
    try exec(db, "PRAGMA journal_mode = WAL;");
    try exec(db, "PRAGMA synchronous = NORMAL;");
    try exec(db, "PRAGMA cache_size = -64000;");
    try exec(db, "PRAGMA temp_store = MEMORY;");
    try exec(db, "PRAGMA mmap_size = 268435456;");
    try exec(db, "PRAGMA foreign_keys = ON;");

    return db;
}

// 执行 SQL 语句
pub fn exec(db: *sqlite.Db, sql: []const u8) !void {
    try db.exec(sql, .{}, .{});
}

// 初始化数据库表结构
pub fn initTables(db: *sqlite.Db) !void {
    // 创建 users 表
    try exec(db,
        \\CREATE TABLE IF NOT EXISTS users (
        \\    id INTEGER PRIMARY KEY AUTOINCREMENT,
        \\    username TEXT UNIQUE NOT NULL,
        \\    password TEXT NOT NULL,
        \\    email TEXT UNIQUE NOT NULL,
        \\    role TEXT DEFAULT 'user',
        \\    status TEXT DEFAULT 'active',
        \\    created_at INTEGER DEFAULT (strftime('%s', 'now')),
        \\    updated_at INTEGER DEFAULT (strftime('%s', 'now'))
        \\);
    );

    // 创建 passages 表
    try exec(db,
        \\CREATE TABLE IF NOT EXISTS passages (
        \\    id INTEGER PRIMARY KEY AUTOINCREMENT,
        \\    uuid TEXT UNIQUE NOT NULL,
        \\    title TEXT NOT NULL,
        \\    content TEXT NOT NULL,
        \\    original_content TEXT,
        \\    summary TEXT,
        \\    author TEXT DEFAULT '管理员',
        \\    tags TEXT DEFAULT '[]',
        \\    category TEXT DEFAULT '未分类',
        \\    status TEXT DEFAULT 'published',
        \\    file_path TEXT UNIQUE,
        \\    visibility TEXT DEFAULT 'public',
        \\    is_scheduled INTEGER DEFAULT 0,
        \\    published_at INTEGER,
        \\    cover_image TEXT DEFAULT '/img/passage-cover.webp',
        \\    created_at INTEGER DEFAULT (strftime('%s', 'now')),
        \\    updated_at INTEGER DEFAULT (strftime('%s', 'now'))
        \\);
    );

    // 创建 comments 表
    try exec(db,
        \\CREATE TABLE IF NOT EXISTS comments (
        \\    id INTEGER PRIMARY KEY AUTOINCREMENT,
        \\    username TEXT NOT NULL,
        \\    content TEXT NOT NULL,
        \\    passage_uuid TEXT NOT NULL,
        \\    created_at INTEGER DEFAULT (strftime('%s', 'now')),
        \\    FOREIGN KEY (passage_uuid) REFERENCES passages(uuid) ON DELETE CASCADE
        \\);
    );

    // 创建 settings 表
    try exec(db,
        \\CREATE TABLE IF NOT EXISTS settings (
        \\    id INTEGER PRIMARY KEY AUTOINCREMENT,
        \\    key TEXT UNIQUE NOT NULL,
        \\    value TEXT NOT NULL,
        \\    type TEXT DEFAULT 'string',
        \\    description TEXT,
        \\    category TEXT DEFAULT 'system',
        \\    created_at INTEGER DEFAULT (strftime('%s', 'now')),
        \\    updated_at INTEGER DEFAULT (strftime('%s', 'now'))
        \\);
    );

    // 创建 attachments 表
    try exec(db,
        \\CREATE TABLE IF NOT EXISTS attachments (
        \\    id INTEGER PRIMARY KEY AUTOINCREMENT,
        \\    filename TEXT NOT NULL,
        \\    file_path TEXT NOT NULL,
        \\    file_size INTEGER NOT NULL,
        \\    mime_type TEXT NOT NULL,
        \\    passage_uuid TEXT,
        \\    created_at INTEGER DEFAULT (strftime('%s', 'now')),
        \\    FOREIGN KEY (passage_uuid) REFERENCES passages(uuid) ON DELETE SET NULL
        \\);
    );

    // 创建 article_views 表
    try exec(db,
        \\CREATE TABLE IF NOT EXISTS article_views (
        \\    id INTEGER PRIMARY KEY AUTOINCREMENT,
        \\    passage_uuid TEXT NOT NULL,
        \\    ip TEXT NOT NULL,
        \\    user_agent TEXT,
        \\    country TEXT DEFAULT '',
        \\    city TEXT DEFAULT '',
        \\    region TEXT DEFAULT '',
        \\    view_date TEXT NOT NULL,
        \\    view_time INTEGER DEFAULT (strftime('%s', 'now')),
        \\    duration INTEGER DEFAULT 0,
        \\    created_at INTEGER DEFAULT (strftime('%s', 'now')),
        \\    FOREIGN KEY (passage_uuid) REFERENCES passages(uuid) ON DELETE CASCADE
        \\);
    );

    // 创建索引
    try exec(db, "CREATE INDEX IF NOT EXISTS idx_passages_uuid ON passages(uuid);");
    try exec(db, "CREATE INDEX IF NOT EXISTS idx_passages_status ON passages(status);");
    try exec(db, "CREATE INDEX IF NOT EXISTS idx_passages_category ON passages(category);");
    try exec(db, "CREATE INDEX IF NOT EXISTS idx_comments_passage_uuid ON comments(passage_uuid);");
    try exec(db, "CREATE INDEX IF NOT EXISTS idx_article_views_passage_uuid ON article_views(passage_uuid);");
    try exec(db, "CREATE INDEX IF NOT EXISTS idx_article_views_view_date ON article_views(view_date);");
}

// 初始化默认管理员用户
pub fn initDefaultAdmin(db: *sqlite.Db, allocator: std.mem.Allocator) !void {
    // 检查是否已有管理员
    const query = "SELECT COUNT(*) as count FROM users WHERE role = 'admin';";
    var stmt = try db.prepare(query);
    defer stmt.deinit();

    var count: i64 = 0;
    var iter = try stmt.iterator(struct { count: i64 }, .{});
    if (try iter.next(.{})) |row| {
        count = row.count;
    }

    if (count == 0) {
        // 创建默认管理员: admin / admin123
        // 注意: 这里使用简化的密码哈希，生产环境应该使用 Argon2
        const password_hash = "admin_hash_placeholder"; // 应该使用实际的 Argon2 哈希
        try exec(db,
            \\INSERT INTO users (username, password, email, role)
            \\VALUES ('admin', 'admin_hash_placeholder', 'admin@example.com', 'admin');
        );
    }
}