/* jwt.c - JWT 实现 */
#include "include/jwt.h"
#include "include/crypto.h"
#include "include/common.h"
#include <string.h>
#include <stdlib.h>

static JwtConfig g_jwt_config;

/* === 初始化 JWT === */

void jwt_init(const char *secret, int expiry) {
    STR_COPY(g_jwt_config.secret, secret);
    g_jwt_config.expiry = expiry;
}

/* === 创建 JWT Token === */

char *jwt_create(const JwtClaims *claims) {
    /* 创建 Header */
    const char *header = "{\"alg\":\"HS256\",\"typ\":\"JWT\"}";
    
    /* 创建 Payload */
    char payload[512];
    snprintf(payload, sizeof(payload),
             "{\"sub\":\"%s\",\"username\":\"%s\",\"role\":\"%s\",\"iat\":%lld,\"exp\":%lld}",
             claims->sub,
             claims->username,
             claims->role,
             (long long)claims->iat,
             (long long)claims->exp);
    
    /* Base64 编码 */
    char header_b64[128], payload_b64[128];
    crypto_base64_encode((uint8_t*)header, strlen(header), header_b64, sizeof(header_b64));
    crypto_base64_encode((uint8_t*)payload, strlen(payload), payload_b64, sizeof(payload_b64));
    
    /* 移除 padding */
    for (char *p = header_b64; *p; p++) if (*p == '=') *p = '\0';
    for (char *p = payload_b64; *p; p++) if (*p == '=') *p = '\0';
    
    /* 计算 Signature */
    char signing_input[512];
    snprintf(signing_input, sizeof(signing_input), "%s.%s", header_b64, payload_b64);
    
    uint8_t hmac[32];
    crypto_hmac_sha256((uint8_t*)g_jwt_config.secret, strlen(g_jwt_config.secret),
                       (uint8_t*)signing_input, strlen(signing_input), hmac);
    
    char signature_b64[64];
    crypto_base64_encode(hmac, 32, signature_b64, sizeof(signature_b64));
    for (char *p = signature_b64; *p; p++) if (*p == '=') *p = '\0';
    
    /* 组合 Token */
    char *token = SAFE_MALLOC(512);
    snprintf(token, 512, "%s.%s.%s", header_b64, payload_b64, signature_b64);
    
    return token;
}

/* === 验证 JWT Token === */

bool jwt_verify(const char *token, JwtClaims *claims) {
    if (!token) return false;
    
    /* 分割 Token */
    char *parts[3];
    char token_copy[512];
    strncpy(token_copy, token, sizeof(token_copy) - 1);
    token_copy[sizeof(token_copy) - 1] = '\0';
    
    char *p = token_copy;
    for (int i = 0; i < 3 && p; i++) {
        parts[i] = p;
        p = strchr(p, '.');
        if (p) *p++ = '\0';
    }
    
    if (!parts[2]) return false;  /* 缺少部分 */
    
    /* 重新计算签名 */
    char signing_input[512];
    snprintf(signing_input, sizeof(signing_input), "%s.%s", parts[0], parts[1]);
    
    uint8_t computed_hmac[32];
    crypto_hmac_sha256((uint8_t*)g_jwt_config.secret, strlen(g_jwt_config.secret),
                       (uint8_t*)signing_input, strlen(signing_input), computed_hmac);
    
    char computed_sig[64];
    crypto_base64_encode(computed_hmac, 32, computed_sig, sizeof(computed_sig));
    for (char *p = computed_sig; *p; p++) if (*p == '=') *p = '\0';
    
    /* 比较签名 */
    if (strcmp(computed_sig, parts[2]) != 0) {
        return false;
    }
    
    /* 解析 Claims */
    return jwt_parse(token, claims);
}

/* === 解析 JWT Token === */

bool jwt_parse(const char *token, JwtClaims *claims) {
    if (!token || !claims) return false;
    
    /* 分割 Token */
    char *parts[3];
    char token_copy[512];
    strncpy(token_copy, token, sizeof(token_copy) - 1);
    token_copy[sizeof(token_copy) - 1] = '\0';
    
    char *p = token_copy;
    for (int i = 0; i < 3 && p; i++) {
        parts[i] = p;
        p = strchr(p, '.');
        if (p) *p++ = '\0';
    }
    
    if (!parts[1]) return false;
    
    /* Base64 解码 Payload */
    uint8_t payload[256];
    size_t payload_len = sizeof(payload);
    if (crypto_base64_decode(parts[1], payload, &payload_len) < 0) {
        return false;
    }
    payload[payload_len] = '\0';
    
    /* 简化版：直接字符串查找提取值 */
    /* 实际项目中应该使用真正的 JSON 解析器 */
    
    const char *sub = strstr((char*)payload, "\"sub\":");
    if (sub) {
        sub += 6;
        while (*sub == ' ' || *sub == '"') sub++;
        const char *end = strchr(sub, '"');
        if (end) {
            size_t len = end - sub;
            if (len < sizeof(claims->sub)) {
                strncpy(claims->sub, sub, len);
                claims->sub[len] = '\0';
            }
        }
    }
    
    const char *username = strstr((char*)payload, "\"username\":");
    if (username) {
        username += 11;
        while (*username == ' ' || *username == '"') username++;
        const char *end = strchr(username, '"');
        if (end) {
            size_t len = end - username;
            if (len < sizeof(claims->username)) {
                strncpy(claims->username, username, len);
                claims->username[len] = '\0';
            }
        }
    }
    
    const char *role = strstr((char*)payload, "\"role\":");
    if (role) {
        role += 7;
        while (*role == ' ' || *role == '"') role++;
        const char *end = strchr(role, '"');
        if (end) {
            size_t len = end - role;
            if (len < sizeof(claims->role)) {
                strncpy(claims->role, role, len);
                claims->role[len] = '\0';
            }
        }
    }
    
    const char *iat = strstr((char*)payload, "\"iat\":");
    if (iat) {
        claims->iat = atoll(iat + 6);
    }
    
    const char *exp = strstr((char*)payload, "\"exp\":");
    if (exp) {
        claims->exp = atoll(exp + 6);
    }
    
    return true;
}

/* === 从 HTTP 请求中提取 Token === */

char *jwt_extract_token(const char *auth_header) {
    if (!auth_header) return NULL;
    
    /* 检查 "Bearer " 前缀 */
    if (strncmp(auth_header, "Bearer ", 7) == 0) {
        char *token = SAFE_MALLOC(strlen(auth_header) - 6);
        strcpy(token, auth_header + 7);
        return token;
    }
    
    /* 假设直接是 Token */
    char *token = SAFE_MALLOC(strlen(auth_header) + 1);
    strcpy(token, auth_header);
    return token;
}

/* === 检查 Token 是否过期 === */

bool jwt_is_expired(const JwtClaims *claims) {
    if (!claims) return true;
    return claims->exp < NOW();
}

/* === 刷新 Token === */

char *jwt_refresh(const char *token) {
    JwtClaims claims;
    if (!jwt_parse(token, &claims)) {
        return NULL;
    }
    
    /* 更新过期时间 */
    claims.iat = NOW();
    claims.exp = claims.iat + g_jwt_config.expiry;
    
    return jwt_create(&claims);
}

/* === 获取用户角色 === */

const char *jwt_get_role(const char *token) {
    JwtClaims claims;
    if (!jwt_parse(token, &claims)) {
        return NULL;
    }
    return claims.role;
}

/* === 检查是否为管理员 === */

bool jwt_is_admin(const char *token) {
    const char *role = jwt_get_role(token);
    return role && strcmp(role, "admin") == 0;
}
