const std = @import("std");

// 简单的模板引擎
pub const TemplateEngine = struct {
    allocator: std.mem.Allocator,
    templates_dir: []const u8,

    pub fn init(allocator: std.mem.Allocator, templates_dir: []const u8) TemplateEngine {
        return TemplateEngine{
            .allocator = allocator,
            .templates_dir = templates_dir,
        };
    }

    // 渲染模板
    pub fn render(self: *const TemplateEngine, template_name: []const u8, context: anytype) ![]const u8 {
        // 构建模板文件路径
        var template_path = std.ArrayList(u8).init(self.allocator);
        defer template_path.deinit();

        try template_path.appendSlice(self.templates_dir);
        try template_path.append('/');
        try template_path.appendSlice(template_name);

        // 读取模板文件
        const template_content = try std.fs.cwd().readFileAlloc(self.allocator, template_path.items, 1024 * 1024);
        defer self.allocator.free(template_content);

        // 简单的变量替换
        var result = std.ArrayList(u8).init(self.allocator);
        errdefer result.deinit();

        var pos: usize = 0;
        while (pos < template_content.len) {
            const start = std.mem.indexOfPos(u8, template_content, pos, "{{") orelse {
                try result.appendSlice(template_content[pos..]);
                break;
            };

            try result.appendSlice(template_content[pos..start]);

            const end = std.mem.indexOfPos(u8, template_content, start, "}}") orelse {
                try result.appendSlice(template_content[pos..]);
                break;
            };

            const var_name = std.mem.trim(u8, template_content[start + 2 .. end], &std.ascii.whitespace);
            const value = try self.getContextValue(var_name, context);
            try result.appendSlice(value);

            pos = end + 2;
        }

        return result.toOwnedSlice();
    }

    // 从上下文中获取值（简化版）
    fn getContextValue(self: *const TemplateEngine, var_name: []const u8, context: anytype) ![]const u8 {
        // 这里简化处理，实际应该使用反射或类型检查
        if (std.mem.eql(u8, var_name, "title")) {
            return self.allocator.dupe(u8, "Zig Blog");
        }
        if (std.mem.eql(u8, var_name, "year")) {
            const year = std.time.timestamp();
            const time = std.time.epoch.EpochSeconds{ .secs = year };
            const year_val = time.getEpochYear();
            return std.fmt.allocPrint(self.allocator, "{d}", .{year_val});
        }

        return self.allocator.dupe(u8, "");
    }
};

// 模板设置
pub const TemplateSettings = struct {
    name: []const u8 = "Zig Blog",
    greeting: []const u8 = "Welcome to Zig Blog",
    year: []const u8 = "2026",
    footer: []const u8 = "Powered by Zig",

    background_image: []const u8 = "/img/passage-cover.webp",
    global_opacity: f64 = 0.9,
    blur_amount: u32 = 0,
    saturate_amount: u32 = 100,

    navbar_glass_color: []const u8 = "rgba(0, 0, 0, 0.7)",
    card_glass_color: []const u8 = "rgba(0, 0, 0, 0.5)",
    footer_glass_color: []const u8 = "rgba(0, 0, 0, 0.7)",

    floating_text_enabled: bool = false,
    live2d_enabled: bool = false,
    sponsor_enabled: bool = false,
    external_link_warning: bool = true,

    allocator: std.mem.Allocator,

    pub fn init(allocator: std.mem.Allocator) TemplateSettings {
        return TemplateSettings{
            .name = allocator.dupe(u8, "Zig Blog") catch "",
            .greeting = allocator.dupe(u8, "Welcome to Zig Blog") catch "",
            .year = allocator.dupe(u8, "2026") catch "",
            .footer = allocator.dupe(u8, "Powered by Zig") catch "",
            .background_image = allocator.dupe(u8, "/img/passage-cover.webp") catch "",
            .navbar_glass_color = allocator.dupe(u8, "rgba(0, 0, 0, 0.7)") catch "",
            .card_glass_color = allocator.dupe(u8, "rgba(0, 0, 0, 0.5)") catch "",
            .footer_glass_color = allocator.dupe(u8, "rgba(0, 0, 0, 0.7)") catch "",
            .allocator = allocator,
        };
    }

    pub fn deinit(self: *TemplateSettings) void {
        self.allocator.free(self.name);
        self.allocator.free(self.greeting);
        self.allocator.free(self.year);
        self.allocator.free(self.footer);
        self.allocator.free(self.background_image);
        self.allocator.free(self.navbar_glass_color);
        self.allocator.free(self.card_glass_color);
        self.allocator.free(self.footer_glass_color);
    }
};