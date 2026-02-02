const std = @import("std");

// 用户模型
pub const User = struct {
    id: i64,
    username: []const u8,
    password: []const u8,
    email: []const u8,
    role: []const u8 = "user",
    status: []const u8 = "active",
    created_at: i64,
    updated_at: i64,
};

// 文章模型
pub const Passage = struct {
    id: i64,
    uuid: []const u8,
    title: []const u8,
    content: []const u8,
    original_content: ?[]const u8,
    summary: ?[]const u8,
    author: []const u8 = "管理员",
    tags: []const u8 = "[]",
    category: []const u8 = "未分类",
    status: []const u8 = "published",
    file_path: ?[]const u8,
    visibility: []const u8 = "public",
    is_scheduled: i32 = 0,
    published_at: ?i64,
    cover_image: []const u8 = "/img/passage-cover.webp",
    created_at: i64,
    updated_at: i64,

    // 分配器
    allocator: std.mem.Allocator,

    // 初始化函数
    pub fn init(allocator: std.mem.Allocator) Passage {
        return Passage{
            .id = 0,
            .uuid = "",
            .title = "",
            .content = "",
            .original_content = null,
            .summary = null,
            .author = "管理员",
            .tags = "[]",
            .category = "未分类",
            .status = "published",
            .file_path = null,
            .visibility = "public",
            .is_scheduled = 0,
            .published_at = null,
            .cover_image = "/img/passage-cover.webp",
            .created_at = 0,
            .updated_at = 0,
            .allocator = allocator,
        };
    }

    // 释放资源
    pub fn deinit(self: *Passage) void {
        self.allocator.free(self.uuid);
        self.allocator.free(self.title);
        self.allocator.free(self.content);
        if (self.original_content) |content| {
            self.allocator.free(content);
        }
        if (self.summary) |summary| {
            self.allocator.free(summary);
        }
        self.allocator.free(self.author);
        self.allocator.free(self.tags);
        self.allocator.free(self.category);
        self.allocator.free(self.status);
        if (self.file_path) |path| {
            self.allocator.free(path);
        }
        self.allocator.free(self.visibility);
        self.allocator.free(self.cover_image);
    }
};

// 评论模型
pub const Comment = struct {
    id: i64,
    username: []const u8,
    content: []const u8,
    passage_uuid: []const u8,
    created_at: i64,

    allocator: std.mem.Allocator,

    pub fn init(allocator: std.mem.Allocator) Comment {
        return Comment{
            .id = 0,
            .username = "",
            .content = "",
            .passage_uuid = "",
            .created_at = 0,
            .allocator = allocator,
        };
    }

    pub fn deinit(self: *Comment) void {
        self.allocator.free(self.username);
        self.allocator.free(self.content);
        self.allocator.free(self.passage_uuid);
    }
};

// 设置模型
pub const Setting = struct {
    id: i64,
    key: []const u8,
    value: []const u8,
    type: []const u8 = "string",
    description: ?[]const u8,
    category: []const u8 = "system",
    created_at: i64,
    updated_at: i64,

    allocator: std.mem.Allocator,

    pub fn init(allocator: std.mem.Allocator) Setting {
        return Setting{
            .id = 0,
            .key = "",
            .value = "",
            .type = "string",
            .description = null,
            .category = "system",
            .created_at = 0,
            .updated_at = 0,
            .allocator = allocator,
        };
    }

    pub fn deinit(self: *Setting) void {
        self.allocator.free(self.key);
        self.allocator.free(self.value);
        self.allocator.free(self.type);
        if (self.description) |desc| {
            self.allocator.free(desc);
        }
        self.allocator.free(self.category);
    }
};

// 附件模型
pub const Attachment = struct {
    id: i64,
    filename: []const u8,
    file_path: []const u8,
    file_size: i64,
    mime_type: []const u8,
    passage_uuid: ?[]const u8,
    created_at: i64,

    allocator: std.mem.Allocator,

    pub fn init(allocator: std.mem.Allocator) Attachment {
        return Attachment{
            .id = 0,
            .filename = "",
            .file_path = "",
            .file_size = 0,
            .mime_type = "",
            .passage_uuid = null,
            .created_at = 0,
            .allocator = allocator,
        };
    }

    pub fn deinit(self: *Attachment) void {
        self.allocator.free(self.filename);
        self.allocator.free(self.file_path);
        self.allocator.free(self.mime_type);
        if (self.passage_uuid) |uuid| {
            self.allocator.free(uuid);
        }
    }
};

// 文章阅读统计模型
pub const ArticleView = struct {
    id: i64,
    passage_uuid: []const u8,
    ip: []const u8,
    user_agent: ?[]const u8,
    country: []const u8 = "",
    city: []const u8 = "",
    region: []const u8 = "",
    view_date: []const u8,
    view_time: i64,
    duration: i32 = 0,
    created_at: i64,

    allocator: std.mem.Allocator,

    pub fn init(allocator: std.mem.Allocator) ArticleView {
        return ArticleView{
            .id = 0,
            .passage_uuid = "",
            .ip = "",
            .user_agent = null,
            .country = "",
            .city = "",
            .region = "",
            .view_date = "",
            .view_time = 0,
            .duration = 0,
            .created_at = 0,
            .allocator = allocator,
        };
    }

    pub fn deinit(self: *ArticleView) void {
        self.allocator.free(self.passage_uuid);
        self.allocator.free(self.ip);
        if (self.user_agent) |ua| {
            self.allocator.free(ua);
        }
        self.allocator.free(self.country);
        self.allocator.free(self.city);
        self.allocator.free(self.region);
        self.allocator.free(self.view_date);
    }
};

// JWT Claims
pub const Claims = struct {
    user_id: i64,
    username: []const u8,
    role: []const u8,
    exp: i64,
    iat: i64,
    nbf: i64,
    iss: []const u8,

    allocator: std.mem.Allocator,

    pub fn init(allocator: std.mem.Allocator) Claims {
        return Claims{
            .user_id = 0,
            .username = "",
            .role = "",
            .exp = 0,
            .iat = 0,
            .nbf = 0,
            .iss = "",
            .allocator = allocator,
        };
    }

    pub fn deinit(self: *Claims) void {
        self.allocator.free(self.username);
        self.allocator.free(self.role);
        self.allocator.free(self.iss);
    }
};