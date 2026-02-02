const std = @import("std");

// 简单的 UUID 生成器
pub const UuidGenerator = struct {
    rng: std.rand.DefaultPrng,

    pub fn init() UuidGenerator {
        var seed: u64 = undefined;
        try std.os.getrandom(std.mem.asBytes(&seed));
        return UuidGenerator{
            .rng = std.rand.DefaultPrng.init(seed),
        };
    }

    // 生成 UUID (简化版)
    pub fn generate(self: *UuidGenerator, allocator: std.mem.Allocator) ![]const u8 {
        var uuid = std.ArrayList(u8).init(allocator);
        errdefer uuid.deinit();

        const parts = [8]u8{ 0, 0, 0, 0, 0, 0, 0, 0 };
        const values = [_]u32{
            self.rng.random().int(u32),
            self.rng.random().int(u32),
            self.rng.random().int(u32),
            self.rng.random().int(u32),
        };

        try std.fmt.format(uuid.writer(), "{x:0>8}-{x:0>4}-{x:0>4}-{x:0>4}-{x:0>12}", .{
            values[0],
            (values[1] & 0xFFFF0000) >> 16,
            (values[1] & 0x0000FFFF),
            values[2] & 0x0000FFFF,
            values[3],
        });

        return uuid.toOwnedSlice();
    }
};

// 验证 UUID 格式
pub fn isValidUuid(uuid: []const u8) bool {
    if (uuid.len != 36) return false;

    const expected_dashes = [4]usize{ 8, 13, 18, 23 };
    for (expected_dashes) |pos| {
        if (uuid[pos] != '-') return false;
    }

    var i: usize = 0;
    while (i < 36) : (i += 1) {
        if (i == 8 or i == 13 or i == 18 or i == 23) continue;
        const c = uuid[i];
        if (!std.ascii.isHex(c)) return false;
    }

    return true;
}