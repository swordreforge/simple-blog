const std = @import("std");

// Markdown 到 HTML 转换器
pub const MarkdownConverter = struct {
    allocator: std.mem.Allocator,

    pub fn init(allocator: std.mem.Allocator) MarkdownConverter {
        return MarkdownConverter{
            .allocator = allocator,
        };
    }

    // 将 Markdown 转换为 HTML
    pub fn toHtml(self: *const MarkdownConverter, markdown: []const u8) ![]const u8 {
        var html = std.ArrayList(u8).init(self.allocator);
        errdefer html.deinit();

        var lines = std.mem.splitScalar(u8, markdown, '\n');
        var in_code_block = false;
        var in_paragraph = false;

        while (lines.next()) |line| {
            // 处理代码块
            if (std.mem.startsWith(u8, line, "```")) {
                if (in_code_block) {
                    try html.append("</code></pre>\n");
                    in_code_block = false;
                } else {
                    try html.append("<pre><code>");
                    in_code_block = true;
                }
                continue;
            }

            if (in_code_block) {
                try html.appendSlice(line);
                try html.append('\n');
                continue;
            }

            // 空行
            if (line.len == 0 or std.mem.trim(u8, line, &std.ascii.whitespace).len == 0) {
                if (in_paragraph) {
                    try html.append("</p>\n");
                    in_paragraph = false;
                }
                continue;
            }

            // 标题
            if (std.mem.startsWith(u8, line, "#")) {
                if (in_paragraph) {
                    try html.append("</p>\n");
                    in_paragraph = false;
                }

                var level: usize = 0;
                var i: usize = 0;
                while (i < line.len and line[i] == '#') : (i += 1) {
                    level += 1;
                }
                if (level > 6) level = 6;

                const text = std.mem.trim(u8, line[i..], &std.ascii.whitespace);
                try html.print("<h{d}>{s}</h{d}>\n", .{ level, text, level });
                continue;
            }

            // 水平线
            if (std.mem.startsWith(u8, line, "---") or std.mem.startsWith(u8, line, "***")) {
                if (in_paragraph) {
                    try html.append("</p>\n");
                    in_paragraph = false;
                }
                try html.append("<hr>\n");
                continue;
            }

            // 列表项
            if (std.mem.startsWith(u8, line, "- ") or std.mem.startsWith(u8, line, "* ")) {
                if (in_paragraph) {
                    try html.append("</p>\n");
                    in_paragraph = false;
                }

                const text = line[2..];
                try html.print("<li>{s}</li>\n", .{text});
                continue;
            }

            // 链接
            if (std.mem.indexOf(u8, line, "[") != null and std.mem.indexOf(u8, line, "](") != null) {
                const link = try self.parseLink(line);
                defer self.allocator.free(link.text);
                defer self.allocator.free(link.url);

                if (!in_paragraph) {
                    try html.append("<p>");
                    in_paragraph = true;
                }

                try html.print("<a href=\"{s}\">{s}</a>", .{ link.url, link.text });
                continue;
            }

            // 普通段落
            if (!in_paragraph) {
                try html.append("<p>");
                in_paragraph = true;
            } else {
                try html.append(' ');
            }

            try html.appendSlice(line);
        }

        // 关闭打开的标签
        if (in_paragraph) {
            try html.append("</p>\n");
        }
        if (in_code_block) {
            try html.append("</code></pre>\n");
        }

        return html.toOwnedSlice();
    }

    // 解析 Markdown 链接
    const Link = struct {
        text: []const u8,
        url: []const u8,
    };

    fn parseLink(self: *const MarkdownConverter, line: []const u8) !Link {
        const start_bracket = std.mem.indexOf(u8, line, "[").?;
        const end_bracket = std.mem.indexOf(u8, line, "](").?;
        const start_paren = end_bracket + 2;
        const end_paren = std.mem.indexOf(u8, line, ")").?;

        const text = try self.allocator.dupe(u8, line[start_bracket + 1 .. end_bracket]);
        const url = try self.allocator.dupe(u8, line[start_paren .. end_paren]);

        return Link{
            .text = text,
            .url = url,
        };
    }
};

// 生成文章摘要
pub fn generateSummary(allocator: std.mem.Allocator, content: []const u8, max_length: usize) ![]const u8 {
    var result = std.ArrayList(u8).init(allocator);
    errdefer result.deinit();

    var char_count: usize = 0;
    var in_tag = false;

    for (content) |c| {
        if (c == '<') {
            in_tag = true;
            try result.append(c);
            continue;
        }
        if (c == '>') {
            in_tag = false;
            try result.append(c);
            continue;
        }
        if (in_tag) {
            try result.append(c);
            continue;
        }

        if (char_count >= max_length) {
            break;
        }

        try result.append(c);
        char_count += 1;
    }

    try result.append("...");

    return result.toOwnedSlice();
}

// 提取标签（简化版）
pub fn extractTags(allocator: std.mem.Allocator, content: []const u8) ![]const u8 {
    // 简化实现：返回空 JSON 数组
    const tags = "[]";
    return allocator.dupe(u8, tags);
}