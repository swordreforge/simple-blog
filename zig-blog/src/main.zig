const std = @import("std");

// 配置
const Config = struct {
    host: []const u8 = "0.0.0.0",
    port: u16 = 8082,
    db_path: []const u8 = "data/blog.db",
};

// HTTP 服务器
const Server = struct {
    allocator: std.mem.Allocator,
    config: Config,

    pub fn init(allocator: std.mem.Allocator, config: Config) Server {
        return Server{
            .allocator = allocator,
            .config = config,
        };
    }

    pub fn run(self: *const Server) !void {
        // 创建 TCP 监听器
        const address = try std.net.Address.parseIp(self.config.host, self.config.port);
        var listener = try address.listen(.{
            .reuse_address = true,
        });
        defer listener.deinit();

        std.log.info("Server listening on {s}:{}\n", .{ self.config.host, self.config.port });

        while (true) {
            const connection = listener.accept() catch |err| {
                std.log.err("Accept error: {}\n", .{err});
                continue;
            };

            _ = std.Thread.spawn(.{}, handleConnection, .{ self.allocator, connection }) catch |err| {
                std.log.err("Thread spawn error: {}\n", .{err});
                connection.stream.close();
            };
        }
    }
};

fn handleConnection(allocator: std.mem.Allocator, connection: std.net.Server.Connection) void {
    defer connection.stream.close();

    var buffer: [4096]u8 = undefined;
    const request_data = connection.stream.read(&buffer) catch |err| {
        std.log.err("Read error: {}\n", .{err});
        return;
    };

    if (request_data == 0) return;

    const request = buffer[0..request_data];

    // 简单解析 HTTP 请求
    var lines = std.mem.splitScalar(u8, request, '\n');
    const first_line = lines.next() orelse return;

    var parts = std.mem.splitScalar(u8, first_line, ' ');
    const method = parts.next() orelse return;
    const path = parts.next() orelse "/";

    std.log.info("{s} {s}\n", .{ method, path });

    // 生成响应
    const response = buildResponse(allocator, path) catch |err| {
        std.log.err("Build response error: {}\n", .{err});
        const error_response = "HTTP/1.1 500 Internal Server Error\r\n\r\n";
        _ = connection.stream.writeAll(error_response) catch {};
        return;
    };
    defer allocator.free(response);

    _ = connection.stream.writeAll(response) catch |err| {
        std.log.err("Write error: {}\n", .{err});
    };
}

fn buildResponse(allocator: std.mem.Allocator, path: []const u8) ![]const u8 {
    if (std.mem.eql(u8, path, "/")) {
        const html =
            \\HTTP/1.1 200 OK\r
            \\Content-Type: text/html\r
            \\\r
            \\<!DOCTYPE html>
            \\<html>
            \\<head><title>Zig Blog</title></head>
            \\<body>
            \\<h1>Welcome to Zig Blog!</h1>
            \\<p>A high-performance blog system written in Zig.</p>
            \\</body>
            \\</html>
        ;
        return allocator.dupe(u8, html);
    } else if (std.mem.eql(u8, path, "/api/passage/list")) {
        const json = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"success\":true,\"data\":[{\"title\":\"Hello Zig\",\"uuid\":\"12345678-1234-1234-1234-123456789012\"}]}"[0..];
        return allocator.dupe(u8, json);
    } else {
        const not_found = "HTTP/1.1 404 Not Found\r\nContent-Type: text/html\r\n\r\n<h1>404 Not Found</h1>"[0..];
        return allocator.dupe(u8, not_found);
    }
}

pub fn main() !void {
    // 初始化分配器
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer {
        const leaked = gpa.deinit();
        if (leaked == .leak) {
            std.log.err("Memory leak detected!\n", .{});
        }
    }
    const allocator = gpa.allocator();

    // 初始化日志
    std.log.info("Starting Zig Blog Server...\n", .{});

    // 配置
    const config = Config{
        .host = "0.0.0.0",
        .port = 8082,
        .db_path = "data/blog.db",
    };

    // 创建数据目录
    try std.fs.cwd().makePath("data");

    // 启动服务器
    const server = Server.init(allocator, config);
    try server.run();
}

// 测试
test "basic functionality" {
    // 测试配置
    const config = Config{};
    try std.testing.expectEqual(config.port, 8082);

    // 测试路径匹配
    try std.testing.expect(std.mem.eql(u8, "/", "/"));
    try std.testing.expect(std.mem.eql(u8, "/api/passage/list", "/api/passage/list"));
}