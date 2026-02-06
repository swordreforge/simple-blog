#ifndef JWT_H
#define JWT_H

#include "types.h"
#include <stdbool.h>

/* === JWT 配置 === */

typedef struct {
    char secret[128];
    int expiry;  // 过期时间（秒）
} JwtConfig;

/* === JWT Claims === */

typedef struct {
    char sub[64];      // Subject (用户ID)
    char username[64]; // 用户名
    char role[32];     // 角色
    int64_t iat;       // Issued At
    int64_t exp;       // Expiration
} JwtClaims;

/* === JWT API === */

/**
 * 初始化 JWT
 */
void jwt_init(const char *secret, int expiry);

/**
 * 创建 JWT Token
 */
char *jwt_create(const JwtClaims *claims);

/**
 * 验证 JWT Token
 */
bool jwt_verify(const char *token, JwtClaims *claims);

/**
 * 解析 JWT Token（不验证签名）
 */
bool jwt_parse(const char *token, JwtClaims *claims);

/**
 * 从 HTTP 请求中提取 Token
 */
char *jwt_extract_token(const char *auth_header);

/**
 * 检查 Token 是否过期
 */
bool jwt_is_expired(const JwtClaims *claims);

/**
 * 刷新 Token
 */
char *jwt_refresh(const char *token);

/**
 * 获取用户角色
 */
const char *jwt_get_role(const char *token);

/**
 * 检查是否为管理员
 */
bool jwt_is_admin(const char *token);

#endif /* JWT_H */