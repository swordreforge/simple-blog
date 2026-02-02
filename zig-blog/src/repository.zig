const std = @import("std");
const sqlite = @import("sqlite");
const models = @import("models.zig");

// 用户仓库
pub const UserRepository = struct {
    db: *sqlite.Db,
    allocator: std.mem.Allocator,

    pub fn init(db: *sqlite.Db, allocator: std.mem.Allocator) UserRepository {
        return UserRepository{
            .db = db,
            .allocator = allocator,
        };
    }

    // 根据用户名查找用户
    pub fn findByUsername(self: *const UserRepository, username: []const u8) !?models.User {
        const query = "SELECT id, username, password, email, role, status, created_at, updated_at FROM users WHERE username = ?;";
        var stmt = try self.db.prepare(query);
        defer stmt.deinit();

        var iter = try stmt.iterator(struct {
            id: i64,
            username: []const u8,
            password: []const u8,
            email: []const u8,
            role: []const u8,
            status: []const u8,
            created_at: i64,
            updated_at: i64,
        }, .{ .username = username });

        if (try iter.next(.{})) |row| {
            return models.User{
                .id = row.id,
                .username = try self.allocator.dupe(u8, row.username),
                .password = try self.allocator.dupe(u8, row.password),
                .email = try self.allocator.dupe(u8, row.email),
                .role = try self.allocator.dupe(u8, row.role),
                .status = try self.allocator.dupe(u8, row.status),
                .created_at = row.created_at,
                .updated_at = row.updated_at,
            };
        }

        return null;
    }

    // 根据邮箱查找用户
    pub fn findByEmail(self: *const UserRepository, email: []const u8) !?models.User {
        const query = "SELECT id, username, password, email, role, status, created_at, updated_at FROM users WHERE email = ?;";
        var stmt = try self.db.prepare(query);
        defer stmt.deinit();

        var iter = try stmt.iterator(struct {
            id: i64,
            username: []const u8,
            password: []const u8,
            email: []const u8,
            role: []const u8,
            status: []const u8,
            created_at: i64,
            updated_at: i64,
        }, .{ .email = email });

        if (try iter.next(.{})) |row| {
            return models.User{
                .id = row.id,
                .username = try self.allocator.dupe(u8, row.username),
                .password = try self.allocator.dupe(u8, row.password),
                .email = try self.allocator.dupe(u8, row.email),
                .role = try self.allocator.dupe(u8, row.role),
                .status = try self.allocator.dupe(u8, row.status),
                .created_at = row.created_at,
                .updated_at = row.updated_at,
            };
        }

        return null;
    }

    // 根据ID查找用户
    pub fn findById(self: *const UserRepository, id: i64) !?models.User {
        const query = "SELECT id, username, password, email, role, status, created_at, updated_at FROM users WHERE id = ?;";
        var stmt = try self.db.prepare(query);
        defer stmt.deinit();

        var iter = try stmt.iterator(struct {
            id: i64,
            username: []const u8,
            password: []const u8,
            email: []const u8,
            role: []const u8,
            status: []const u8,
            created_at: i64,
            updated_at: i64,
        }, .{ .id = id });

        if (try iter.next(.{})) |row| {
            return models.User{
                .id = row.id,
                .username = try self.allocator.dupe(u8, row.username),
                .password = try self.allocator.dupe(u8, row.password),
                .email = try self.allocator.dupe(u8, row.email),
                .role = try self.allocator.dupe(u8, row.role),
                .status = try self.allocator.dupe(u8, row.status),
                .created_at = row.created_at,
                .updated_at = row.updated_at,
            };
        }

        return null;
    }

    // 创建用户
    pub fn create(self: *const UserRepository, username: []const u8, password_hash: []const u8, email: []const u8) !i64 {
        const query = "INSERT INTO users (username, password, email) VALUES (?, ?, ?);";
        try self.db.exec(query, .{}, .{ username, password_hash, email });
        return self.db.getLastInsertRowId();
    }
};

// 文章仓库
pub const PassageRepository = struct {
    db: *sqlite.Db,
    allocator: std.mem.Allocator,

    pub fn init(db: *sqlite.Db, allocator: std.mem.Allocator) PassageRepository {
        return PassageRepository{
            .db = db,
            .allocator = allocator,
        };
    }

    // 根据 UUID 查找文章
    pub fn findByUuid(self: *const UserRepository, uuid: []const u8) !?models.Passage {
        const query =
            \\SELECT id, uuid, title, content, original_content, summary, author, tags, category, status,
            \\       file_path, visibility, is_scheduled, published_at, cover_image, created_at, updated_at
            \\FROM passages WHERE uuid = ?;
        ;
        var stmt = try self.db.prepare(query);
        defer stmt.deinit();

        var iter = try stmt.iterator(struct {
            id: i64,
            uuid: []const u8,
            title: []const u8,
            content: []const u8,
            original_content: ?[]const u8,
            summary: ?[]const u8,
            author: []const u8,
            tags: []const u8,
            category: []const u8,
            status: []const u8,
            file_path: ?[]const u8,
            visibility: []const u8,
            is_scheduled: i32,
            published_at: ?i64,
            cover_image: []const u8,
            created_at: i64,
            updated_at: i64,
        }, .{ .uuid = uuid });

        if (try iter.next(.{})) |row| {
            return models.Passage{
                .id = row.id,
                .uuid = try self.allocator.dupe(u8, row.uuid),
                .title = try self.allocator.dupe(u8, row.title),
                .content = try self.allocator.dupe(u8, row.content),
                .original_content = if (row.original_content) |content| try self.allocator.dupe(u8, content) else null,
                .summary = if (row.summary) |summary| try self.allocator.dupe(u8, summary) else null,
                .author = try self.allocator.dupe(u8, row.author),
                .tags = try self.allocator.dupe(u8, row.tags),
                .category = try self.allocator.dupe(u8, row.category),
                .status = try self.allocator.dupe(u8, row.status),
                .file_path = if (row.file_path) |path| try self.allocator.dupe(u8, path) else null,
                .visibility = try self.allocator.dupe(u8, row.visibility),
                .is_scheduled = row.is_scheduled,
                .published_at = row.published_at,
                .cover_image = try self.allocator.dupe(u8, row.cover_image),
                .created_at = row.created_at,
                .updated_at = row.updated_at,
                .allocator = self.allocator,
            };
        }

        return null;
    }

    // 获取已发布文章列表
    pub fn findPublished(self: *const PassageRepository, offset: usize, limit: usize) !std.ArrayList(models.Passage) {
        const query =
            \\SELECT id, uuid, title, content, original_content, summary, author, tags, category, status,
            \\       file_path, visibility, is_scheduled, published_at, cover_image, created_at, updated_at
            \\FROM passages WHERE status = 'published' AND visibility = 'public'
            \\ORDER BY published_at DESC LIMIT ? OFFSET ?;
        ;
        var stmt = try self.db.prepare(query);
        defer stmt.deinit();

        var passages = std.ArrayList(models.Passage).init(self.allocator);
        errdefer {
            for (passages.items) |*p| p.deinit();
            passages.deinit();
        }

        var iter = try stmt.iterator(struct {
            id: i64,
            uuid: []const u8,
            title: []const u8,
            content: []const u8,
            original_content: ?[]const u8,
            summary: ?[]const u8,
            author: []const u8,
            tags: []const u8,
            category: []const u8,
            status: []const u8,
            file_path: ?[]const u8,
            visibility: []const u8,
            is_scheduled: i32,
            published_at: ?i64,
            cover_image: []const u8,
            created_at: i64,
            updated_at: i64,
        }, .{ .limit = limit, .offset = offset });

        while (try iter.next(.{})) |row| {
            try passages.append(models.Passage{
                .id = row.id,
                .uuid = try self.allocator.dupe(u8, row.uuid),
                .title = try self.allocator.dupe(u8, row.title),
                .content = try self.allocator.dupe(u8, row.content),
                .original_content = if (row.original_content) |content| try self.allocator.dupe(u8, content) else null,
                .summary = if (row.summary) |summary| try self.allocator.dupe(u8, summary) else null,
                .author = try self.allocator.dupe(u8, row.author),
                .tags = try self.allocator.dupe(u8, row.tags),
                .category = try self.allocator.dupe(u8, row.category),
                .status = try self.allocator.dupe(u8, row.status),
                .file_path = if (row.file_path) |path| try self.allocator.dupe(u8, path) else null,
                .visibility = try self.allocator.dupe(u8, row.visibility),
                .is_scheduled = row.is_scheduled,
                .published_at = row.published_at,
                .cover_image = try self.allocator.dupe(u8, row.cover_image),
                .created_at = row.created_at,
                .updated_at = row.updated_at,
                .allocator = self.allocator,
            });
        }

        return passages;
    }

    // 创建文章
    pub fn create(self: *const PassageRepository, passage: *const models.Passage) !i64 {
        const query =
            \\INSERT INTO passages (uuid, title, content, original_content, summary, author, tags, category,
            \\                       status, file_path, visibility, is_scheduled, published_at, cover_image)
            \\VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);
        ;
        try self.db.exec(query, .{}, .{
            passage.uuid,
            passage.title,
            passage.content,
            passage.original_content,
            passage.summary,
            passage.author,
            passage.tags,
            passage.category,
            passage.status,
            passage.file_path,
            passage.visibility,
            passage.is_scheduled,
            passage.published_at,
            passage.cover_image,
        });
        return self.db.getLastInsertRowId();
    }

    // 更新文章
    pub fn update(self: *const PassageRepository, passage: *const models.Passage) !void {
        const query =
            \\UPDATE passages SET title = ?, content = ?, original_content = ?, summary = ?, tags = ?,
            \\                    category = ?, status = ?, visibility = ?, cover_image = ?, updated_at = strftime('%s', 'now')
            \\WHERE uuid = ?;
        ;
        try self.db.exec(query, .{}, .{
            passage.title,
            passage.content,
            passage.original_content,
            passage.summary,
            passage.tags,
            passage.category,
            passage.status,
            passage.visibility,
            passage.cover_image,
            passage.uuid,
        });
    }

    // 删除文章
    pub fn delete(self: *const PassageRepository, uuid: []const u8) !void {
        const query = "DELETE FROM passages WHERE uuid = ?;";
        try self.db.exec(query, .{}, .{ uuid });
    }
};

// 评论仓库
pub const CommentRepository = struct {
    db: *sqlite.Db,
    allocator: std.mem.Allocator,

    pub fn init(db: *sqlite.Db, allocator: std.mem.Allocator) CommentRepository {
        return CommentRepository{
            .db = db,
            .allocator = allocator,
        };
    }

    // 获取文章评论
    pub fn findByPassageUuid(self: *const CommentRepository, passage_uuid: []const u8) !std.ArrayList(models.Comment) {
        const query = "SELECT id, username, content, passage_uuid, created_at FROM comments WHERE passage_uuid = ? ORDER BY created_at ASC;";
        var stmt = try self.db.prepare(query);
        defer stmt.deinit();

        var comments = std.ArrayList(models.Comment).init(self.allocator);
        errdefer {
            for (comments.items) |*c| c.deinit();
            comments.deinit();
        }

        var iter = try stmt.iterator(struct {
            id: i64,
            username: []const u8,
            content: []const u8,
            passage_uuid: []const u8,
            created_at: i64,
        }, .{ .passage_uuid = passage_uuid });

        while (try iter.next(.{})) |row| {
            try comments.append(models.Comment{
                .id = row.id,
                .username = try self.allocator.dupe(u8, row.username),
                .content = try self.allocator.dupe(u8, row.content),
                .passage_uuid = try self.allocator.dupe(u8, row.passage_uuid),
                .created_at = row.created_at,
                .allocator = self.allocator,
            });
        }

        return comments;
    }

    // 创建评论
    pub fn create(self: *const CommentRepository, comment: *const models.Comment) !i64 {
        const query = "INSERT INTO comments (username, content, passage_uuid) VALUES (?, ?, ?);";
        try self.db.exec(query, .{}, .{ comment.username, comment.content, comment.passage_uuid });
        return self.db.getLastInsertRowId();
    }

    // 删除评论
    pub fn delete(self: *const CommentRepository, id: i64) !void {
        const query = "DELETE FROM comments WHERE id = ?;";
        try self.db.exec(query, .{}, .{ id });
    }
};