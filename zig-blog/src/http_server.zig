const std = @import("std");
const httpz = @import("httpz");

// HTTP 服务器
pub const Server = struct {
    allocator: std.mem.Allocator,
    host: []const u8,
    port: u16,

    pub fn init(allocator: std.mem.Allocator, host: []const u8, port: u16) Server {
        return Server{
            .allocator = allocator,
            .host = host,
            .port = port,
        };
    }

    pub fn run(self: *const Server) !void {
        var server = try httpz.Server(*Context).init(self.allocator, .{
            .address = self.host,
            .port = self.port,
            .request = .{
                .max_body_size = 100 * 1024 * 1024, // 100MB
            },
        }, .{
            .pool_size = 10,
        });
        defer server.deinit();

        // 配置路由
        try self.configureRoutes(&server);

        std.log.info("Server listening on {s}:{}\n", .{ self.host, self.port });
        server.listen() catch |err| {
            std.log.err("Server error: {}\n", .{err});
            return err;
        };
    }

    fn configureRoutes(self: *const Server, server: *httpz.Server(*Context)) !void {
        // 页面路由
        var router = server.router();

        // 主页
        router.get("/", getHomePage);
        router.get("/passage", getPassageListPage);
        router.get("/passage/:uuid", getPassageDetailPage);
        router.get("/collect", getCollectPage);
        router.get("/about", getAboutPage);
        router.get("/admin", getAdminPage);

        // API 路由
        // 认证 API
        router.post("/api/login", postLogin);
        router.post("/api/register", postRegister);
        router.get("/api/check", getCheckAuth);

        // 文章 API
        router.get("/api/passage/list", getPassageList);
        router.get("/api/passage/:uuid", getPassageByUuid);
        router.post("/api/passage", createPassage);
        router.put("/api/passage/:uuid", updatePassage);
        router.delete("/api/passage/:uuid", deletePassage);

        // 评论 API
        router.get("/api/comments", getComments);
        router.post("/api/comments", createComment);
        router.delete("/api/comments/:id", deleteComment);

        // 静态文件
        router.get("/static/*", serveStaticFile);

        // 状态页
        router.get("/status/404", getStatus404);
        router.get("/status/500", getStatus500);
    }
};

// 请求上下文
pub const Context = struct {
    allocator: std.mem.Allocator,
};

// 路由处理函数
fn getHomePage(req: *httpz.Request, res: *httpz.Response, ctx: *Context) !void {
    res.status = 200;
    res.content_type = .HTML;
    try res.body.append("<!-- Main Page -->");
    try res.body.append("<h1>Zig Blog</h1>");
    try res.body.append("<p>Welcome to Zig Blog!</p>");
}

fn getPassageListPage(req: *httpz.Request, res: *httpz.Response, ctx: *Context) !void {
    res.status = 200;
    res.content_type = .HTML;
    try res.body.append("<h1>Passage List</h1>");
}

fn getPassageDetailPage(req: *httpz.Request, res: *httpz.Response, ctx: *Context) !void {
    const uuid = req.param("uuid").?;
    res.status = 200;
    res.content_type = .HTML;
    try res.body.append("<h1>Passage Detail: ");
    try res.body.append(uuid);
    try res.body.append("</h1>");
}

fn getCollectPage(req: *httpz.Request, res: *httpz.Response, ctx: *Context) !void {
    res.status = 200;
    res.content_type = .HTML;
    try res.body.append("<h1>Collect / Archive</h1>");
}

fn getAboutPage(req: *httpz.Request, res: *httpz.Response, ctx: *Context) !void {
    res.status = 200;
    res.content_type = .HTML;
    try res.body.append("<h1>About</h1>");
}

fn getAdminPage(req: *httpz.Request, res: *httpz.Response, ctx: *Context) !void {
    res.status = 200;
    res.content_type = .HTML;
    try res.body.append("<h1>Admin Panel</h1>");
}

// API 处理函数
fn postLogin(req: *httpz.Request, res: *httpz.Response, ctx: *Context) !void {
    const body = try req.body();
    std.log.info("Login request: {s}\n", .{body});

    res.status = 200;
    res.content_type = .JSON;
    try res.body.append("{\"success\":true,\"message\":\"Login successful\"}");
}

fn postRegister(req: *httpz.Request, res: *httpz.Response, ctx: *Context) !void {
    const body = try req.body();
    std.log.info("Register request: {s}\n", .{body});

    res.status = 200;
    res.content_type = .JSON;
    try res.body.append("{\"success\":true,\"message\":\"Registration successful\"}");
}

fn getCheckAuth(req: *httpz.Request, res: *httpz.Response, ctx: *Context) !void {
    res.status = 200;
    res.content_type = .JSON;
    try res.body.append("{\"authenticated\":false}");
}

fn getPassageList(req: *httpz.Request, res: *httpz.Response, ctx: *Context) !void {
    const offset = req.queryParam("offset") orelse "0";
    const limit = req.queryParam("limit") orelse "10";

    res.status = 200;
    res.content_type = .JSON;
    try res.body.append("{\"success\":true,\"data\":[]}");
}

fn getPassageByUuid(req: *httpz.Request, res: *httpz.Response, ctx: *Context) !void {
    const uuid = req.param("uuid").?;

    res.status = 200;
    res.content_type = .JSON;
    try res.body.append("{\"success\":true,\"data\":{\"uuid\":\"");
    try res.body.append(uuid);
    try res.body.append("\"}}");
}

fn createPassage(req: *httpz.Request, res: *httpz.Response, ctx: *Context) !void {
    res.status = 201;
    res.content_type = .JSON;
    try res.body.append("{\"success\":true,\"message\":\"Passage created\"}");
}

fn updatePassage(req: *httpz.Request, res: *httpz.Response, ctx: *Context) !void {
    const uuid = req.param("uuid").?;

    res.status = 200;
    res.content_type = .JSON;
    try res.body.append("{\"success\":true,\"message\":\"Passage updated\"}");
}

fn deletePassage(req: *httpz.Request, res: *httpz.Response, ctx: *Context) !void {
    const uuid = req.param("uuid").?;

    res.status = 200;
    res.content_type = .JSON;
    try res.body.append("{\"success\":true,\"message\":\"Passage deleted\"}");
}

fn getComments(req: *httpz.Request, res: *httpz.Response, ctx: *Context) !void {
    res.status = 200;
    res.content_type = .JSON;
    try res.body.append("{\"success\":true,\"data\":[]}");
}

fn createComment(req: *httpz.Request, res: *httpz.Response, ctx: *Context) !void {
    res.status = 201;
    res.content_type = .JSON;
    try res.body.append("{\"success\":true,\"message\":\"Comment created\"}");
}

fn deleteComment(req: *httpz.Request, res: *httpz.Response, ctx: *Context) !void {
    const id = req.param("id").?;

    res.status = 200;
    res.content_type = .JSON;
    try res.body.append("{\"success\":true,\"message\":\"Comment deleted\"}");
}

fn serveStaticFile(req: *httpz.Request, res: *httpz.Response, ctx: *Context) !void {
    const path = req.param("0").?;
    res.status = 404;
    res.content_type = .TEXT;
    try res.body.append("Static file not found: ");
    try res.body.append(path);
}

fn getStatus404(req: *httpz.Request, res: *httpz.Response, ctx: *Context) !void {
    res.status = 404;
    res.content_type = .HTML;
    try res.body.append("<h1>404 Not Found</h1>");
}

fn getStatus500(req: *httpz.Request, res: *httpz.Response, ctx: *Context) !void {
    res.status = 500;
    res.content_type = .HTML;
    try res.body.append("<h1>500 Internal Server Error</h1>");
}