/* crypto.c - 加密功能实现 */
#include "include/crypto.h"
#include "include/common.h"
#include <bearssl.h>
#include <string.h>
#include <stdlib.h>
#include <time.h>

/* === BearSSL SHA256 上下文 === */

void crypto_sha256(const uint8_t *data, size_t len, uint8_t hash[32]) {
    br_sha256_context ctx;
    br_sha256_init(&ctx);
    br_sha256_update(&ctx, data, len);
    br_sha256_out(&ctx, hash);
}

void crypto_sha256_string(const char *str, char hash_hex[65]) {
    uint8_t hash[32];
    crypto_sha256((const uint8_t*)str, strlen(str), hash);
    
    /* 转换为十六进制字符串 */
    for (int i = 0; i < 32; i++) {
        sprintf(hash_hex + i * 2, "%02x", hash[i]);
    }
    hash_hex[64] = '\0';
}

/* === 密码哈希（简化版，使用 SHA256 + 盐） === */

int crypto_hash_password(const char *password, char hash[256]) {
    uint8_t salt[16];
    crypto_generate_salt(salt);
    
    /* 计算 password + salt 的 SHA256 */
    br_sha256_context ctx;
    uint8_t hash_result[32];
    
    br_sha256_init(&ctx);
    br_sha256_update(&ctx, (const uint8_t*)password, strlen(password));
    br_sha256_update(&ctx, salt, sizeof(salt));
    br_sha256_out(&ctx, hash_result);
    
    /* 多轮迭代（简化版 PBKDF2） */
    for (int i = 0; i < 10000; i++) {
        br_sha256_init(&ctx);
        br_sha256_update(&ctx, hash_result, sizeof(hash_result));
        br_sha256_out(&ctx, hash_result);
    }
    
    /* 格式化为: $sha256$salt$hash */
    char salt_hex[33];
    for (int i = 0; i < 16; i++) {
        sprintf(salt_hex + i * 2, "%02x", salt[i]);
    }
    salt_hex[32] = '\0';
    
    char hash_hex[65];
    for (int i = 0; i < 32; i++) {
        sprintf(hash_hex + i * 2, "%02x", hash_result[i]);
    }
    hash_hex[64] = '\0';
    
    snprintf(hash, 256, "$sha256$%s$%s", salt_hex, hash_hex);
    return 0;
}

bool crypto_verify_password(const char *password, const char *hash) {
    /* 解析哈希格式: $sha256$salt$hash */
    if (strncmp(hash, "$sha256$", 8) != 0) {
        return false;
    }
    
    const char *salt_hex = hash + 8;
    const char *hash_hex = strchr(salt_hex, '$');
    if (!hash_hex) return false;
    
    size_t salt_len = hash_hex - salt_hex;
    hash_hex++;  /* 跳过 '$' */
    
    /* 解码盐 */
    uint8_t salt[16];
    for (int i = 0; i < 16 && i * 2 < (int)salt_len; i++) {
        sscanf(salt_hex + i * 2, "%02hhx", &salt[i]);
    }
    
    /* 计算密码哈希 */
    br_sha256_context ctx;
    uint8_t computed_hash[32];
    
    br_sha256_init(&ctx);
    br_sha256_update(&ctx, (const uint8_t*)password, strlen(password));
    br_sha256_update(&ctx, salt, sizeof(salt));
    br_sha256_out(&ctx, computed_hash);
    
    for (int i = 0; i < 10000; i++) {
        br_sha256_init(&ctx);
        br_sha256_update(&ctx, computed_hash, sizeof(computed_hash));
        br_sha256_out(&ctx, computed_hash);
    }
    
    /* 转换为十六进制 */
    char computed_hex[65];
    for (int i = 0; i < 32; i++) {
        sprintf(computed_hex + i * 2, "%02x", computed_hash[i]);
    }
    computed_hex[64] = '\0';
    
    return strcmp(computed_hex, hash_hex) == 0;
}

/* === 生成随机盐 === */

void crypto_generate_salt(uint8_t salt[16]) {
    br_hmac_drbg_context rng;
    uint8_t seed[32];
    
    /* 使用时间作为种子 */
    int64_t t = time(NULL);
    memcpy(seed, &t, sizeof(t));
    
    br_hmac_drbg_init(&rng, &br_sha256_vtable, seed, sizeof(seed));
    br_hmac_drbg_generate(&rng, salt, 16);
}

/* === 生成随机字节 === */

void crypto_random_bytes(uint8_t *buf, size_t len) {
    br_hmac_drbg_context rng;
    uint8_t seed[32];
    
    /* 使用多种熵源 */
    int64_t t = time(NULL);
    memcpy(seed, &t, sizeof(t));
    
    br_hmac_drbg_init(&rng, &br_sha256_vtable, seed, sizeof(seed));
    br_hmac_drbg_generate(&rng, buf, len);
}

/* === 生成随机字符串 === */

void crypto_random_string(char *buf, size_t len) {
    const char charset[] = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    size_t charset_len = strlen(charset);
    
    for (size_t i = 0; i < len - 1; i++) {
        uint8_t r;
        crypto_random_bytes(&r, 1);
        buf[i] = charset[r % charset_len];
    }
    buf[len - 1] = '\0';
}

/* === 生成 UUID === */

void crypto_generate_uuid(char uuid[37]) {
    uint8_t data[16];
    crypto_random_bytes(data, 16);
    
    /* 设置版本和变体 */
    data[6] = (data[6] & 0x0F) | 0x40;  /* Version 4 */
    data[8] = (data[8] & 0x3F) | 0x80;  /* Variant 1 */
    
    /* 格式化为 UUID 字符串 */
    snprintf(uuid, 37,
             "%02x%02x%02x%02x-%02x%02x-%02x%02x-%02x%02x-%02x%02x%02x%02x%02x%02x",
             data[0], data[1], data[2], data[3],
             data[4], data[5], data[6], data[7],
             data[8], data[9], data[10], data[11],
             data[12], data[13], data[14], data[15]);
}

/* === Base64 编码 === */

static const char base64_table[] = 
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

int crypto_base64_encode(const uint8_t *src, size_t src_len, char *dst, size_t dst_len) {
    char *d = dst;
    size_t i = 0;
    
    for (i = 0; i < src_len; i += 3) {
        size_t remaining = src_len - i;
        uint32_t chunk = src[i] << 16;
        if (remaining > 1) chunk |= src[i + 1] << 8;
        if (remaining > 2) chunk |= src[i + 2];
        
        if ((size_t)(d - dst) + 4 >= dst_len) break;
        
        *d++ = base64_table[(chunk >> 18) & 0x3F];
        *d++ = base64_table[(chunk >> 12) & 0x3F];
        *d++ = (remaining > 1) ? base64_table[(chunk >> 6) & 0x3F] : '=';
        *d++ = (remaining > 2) ? base64_table[chunk & 0x3F] : '=';
    }
    *d = '\0';
    
    return 0;
}

/* === Base64 解码 === */

static int base64_decode_char(char c) {
    if (c >= 'A' && c <= 'Z') return c - 'A';
    if (c >= 'a' && c <= 'z') return c - 'a' + 26;
    if (c >= '0' && c <= '9') return c - '0' + 52;
    if (c == '+') return 62;
    if (c == '/') return 63;
    return -1;
}

int crypto_base64_decode(const char *src, uint8_t *dst, size_t *dst_len) {
    size_t src_len = strlen(src);
    uint8_t *d = dst;
    size_t output_len = 0;
    
    for (size_t i = 0; i < src_len; i += 4) {
        if (i + 4 > src_len) break;
        
        int a = base64_decode_char(src[i]);
        int b = base64_decode_char(src[i + 1]);
        int c = base64_decode_char(src[i + 2]);
        int d0 = base64_decode_char(src[i + 3]);
        
        if (a < 0 || b < 0) return -1;
        
        if (output_len + 3 > *dst_len) return -1;
        
        *d++ = (a << 2) | (b >> 4);
        output_len++;
        
        if (c >= 0) {
            *d++ = ((b & 0xF) << 4) | (c >> 2);
            output_len++;
            
            if (d0 >= 0 && src[i + 2] != '=') {
                *d++ = ((c & 0x3) << 6) | d0;
                output_len++;
            }
        }
    }
    
    *dst_len = output_len;
    return 0;
}

/* === 十六进制编码 === */

void crypto_hex_encode(const uint8_t *src, size_t src_len, char *dst) {
    for (size_t i = 0; i < src_len; i++) {
        sprintf(dst + i * 2, "%02x", src[i]);
    }
    dst[src_len * 2] = '\0';
}

/* === 十六进制解码 === */

int crypto_hex_decode(const char *src, uint8_t *dst, size_t *dst_len) {
    size_t src_len = strlen(src);
    if (src_len % 2 != 0) return -1;
    
    size_t output_len = src_len / 2;
    if (output_len > *dst_len) return -1;
    
    for (size_t i = 0; i < output_len; i++) {
        unsigned int val;
        if (sscanf(src + i * 2, "%02x", &val) != 1) {
            return -1;
        }
        dst[i] = (uint8_t)val;
    }
    
    *dst_len = output_len;
    return 0;
}

/* === HMAC-SHA256 === */

void crypto_hmac_sha256(const uint8_t *key, size_t key_len,
                        const uint8_t *data, size_t data_len,
                        uint8_t hmac[32]) {
    br_hmac_key_context kc;
    br_hmac_context ctx;
    
    br_hmac_key_init(&kc, &br_sha256_vtable, key, key_len);
    br_hmac_init(&ctx, &kc, 0);
    br_hmac_update(&ctx, data, data_len);
    br_hmac_out(&ctx, hmac);
}