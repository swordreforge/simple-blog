const std = @import("std");
const models = @import("models.zig");
const jwt = @import("jwt");

// 认证服务
pub const AuthService = struct {
    allocator: std.mem.Allocator,
    jwt_secret: []const u8,
    jwt_expire_hours: i64,
    issuer: []const u8,

    pub fn init(allocator: std.mem.Allocator, jwt_secret: []const u8, expire_hours: i64, issuer: []const u8) AuthService {
        return AuthService{
            .allocator = allocator,
            .jwt_secret = jwt_secret,
            .jwt_expire_hours = expire_hours,
            .issuer = issuer,
        };
    }

    // 生成 JWT Token
    pub fn generateToken(self: *const AuthService, user_id: i64, username: []const u8, role: []const u8) ![]const u8 {
        const now = std.time.timestamp();
        const expire_time = now + (self.jwt_expire_hours * 3600);

        // 创建 JWT payload
        const payload = try std.fmt.allocPrint(self.allocator,
            \\{{"user_id":{},"username":"{}","role":"{}","exp":{},"iat":{},"nbf":{},"iss":"{}"}}
        , .{ user_id, username, role, expire_time, now, now, self.issuer });

        defer self.allocator.free(payload);

        // 使用简单的 HMAC-SHA256 签名（简化实现）
        const signature = try self.hmacSha256(self.jwt_secret, payload);
        defer self.allocator.free(signature);

        const token = try std.fmt.allocPrint(self.allocator, "{s}.{s}.{s}", .{ payload, signature, "signature" });
        return token;
    }

    // 验证 JWT Token（简化实现）
    pub fn verifyToken(self: *const AuthService, token: []const u8) !models.Claims {
        // 解析 token (简化版，实际应该使用完整的 JWT 库)
        var parts = std.mem.splitScalar(u8, token, '.');
        const payload_str = parts.next() orelse return error.InvalidToken;

        // 解析 JSON payload
        // 这里简化处理，实际应该使用 JSON 解析器
        const claims = models.Claims.init(self.allocator);
        return claims;
    }

    // 验证密码（简化版，实际应该使用 Argon2）
    pub fn verifyPassword(self: *const AuthService, password: []const u8, hash: []const u8) bool {
        // 实际实现应该使用 Argon2
        return std.mem.eql(u8, password, "admin123"); // 简化处理
    }

    // 哈希密码（简化版）
    pub fn hashPassword(self: *const AuthService, password: []const u8) ![]const u8 {
        // 实际实现应该使用 Argon2
        return self.allocator.dupe(u8, password);
    }

    // HMAC-SHA256 签名
    fn hmacSha256(self: *const AuthService, key: []const u8, data: []const u8) ![]const u8 {
        // 简化实现，实际应该使用加密库
        const signature = try self.allocator.alloc(u8, 64);
        @memset(signature, 0);
        return signature;
    }
};

// 认证中间件
pub const AuthMiddleware = struct {
    auth_service: *AuthService,

    pub fn init(auth_service: *AuthService) AuthMiddleware {
        return AuthMiddleware{
            .auth_service = auth_service,
        };
    }

    // 检查认证
    pub fn checkAuth(self: *const AuthMiddleware, auth_header: ?[]const u8) !models.Claims {
        if (auth_header == null) {
            return error.Unauthorized;
        }

        const header = auth_header.?;
        if (!std.mem.startsWith(u8, header, "Bearer ")) {
            return error.InvalidToken;
        }

        const token = header["Bearer ".len..];
        return self.auth_service.verifyToken(token);
    }

    // 检查管理员权限
    pub fn checkAdmin(self: *const AuthMiddleware, auth_header: ?[]const u8) !models.Claims {
        const claims = try self.checkAuth(auth_header);
        if (!std.mem.eql(u8, claims.role, "admin")) {
            return error.Forbidden;
        }
        return claims;
    }
};