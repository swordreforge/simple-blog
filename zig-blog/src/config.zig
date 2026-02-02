const std = @import("std");

// 配置结构体
pub const Config = struct {
    server: ServerConfig,
    database: DatabaseConfig,
    templates: TemplateConfig,
    static_files: StaticConfig,
    jwt: JwtConfig,
};

pub const ServerConfig = struct {
    host: []const u8 = "0.0.0.0",
    port: u16 = 8080,
    worker_count: usize = 4,
};

pub const DatabaseConfig = struct {
    path: []const u8 = "data/blog.db",
    max_connections: u32 = 20,
    min_idle: u32 = 5,
    connection_timeout: u64 = 30,
    idle_timeout: u64 = 600,
    max_lifetime: u64 = 1800,
};

pub const TemplateConfig = struct {
    dir: []const u8 = "templates",
    cache_enabled: bool = true,
};

pub const StaticConfig = struct {
    dir: []const u8 = "static",
    cache_max_age: u64 = 3600,
};

pub const JwtConfig = struct {
    secret: []const u8 = "your-secret-key-change-in-production",
    expire_hours: i64 = 24,
    issuer: []const u8 = "zig-blog",
};

// 全局配置实例
var global_config: Config = undefined;

// 初始化配置
pub fn init(allocator: std.mem.Allocator) !Config {
    global_config = Config{
        .server = ServerConfig{
            .host = try allocator.dupe(u8, "0.0.0.0"),
            .port = 8080,
            .worker_count = 4,
        },
        .database = DatabaseConfig{
            .path = try allocator.dupe(u8, "data/blog.db"),
            .max_connections = 20,
            .min_idle = 5,
            .connection_timeout = 30,
            .idle_timeout = 600,
            .max_lifetime = 1800,
        },
        .templates = TemplateConfig{
            .dir = try allocator.dupe(u8, "templates"),
            .cache_enabled = true,
        },
        .static_files = StaticConfig{
            .dir = try allocator.dupe(u8, "static"),
            .cache_max_age = 3600,
        },
        .jwt = JwtConfig{
            .secret = try allocator.dupe(u8, "your-secret-key-change-in-production"),
            .expire_hours = 24,
            .issuer = try allocator.dupe(u8, "zig-blog"),
        },
    };

    return global_config;
}

// 获取全局配置
pub fn get() *Config {
    return &global_config;
}

// 释放配置资源
pub fn deinit(config: *Config, allocator: std.mem.Allocator) void {
    allocator.free(config.server.host);
    allocator.free(config.database.path);
    allocator.free(config.templates.dir);
    allocator.free(config.static_files.dir);
    allocator.free(config.jwt.secret);
    allocator.free(config.jwt.issuer);
}