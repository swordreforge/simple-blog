#ifndef CRYPTO_H
#define CRYPTO_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

/* === 哈希函数 === */

/**
 * SHA256 哈希
 */
void crypto_sha256(const uint8_t *data, size_t len, uint8_t hash[32]);

/**
 * SHA256 哈希（字符串输入）
 */
void crypto_sha256_string(const char *str, char hash_hex[65]);

/* === 密码哈希 === */

/**
 * 生成密码哈希（Argon2id）
 */
int crypto_hash_password(const char *password, char hash[256]);

/**
 * 验证密码
 */
bool crypto_verify_password(const char *password, const char *hash);

/**
 * 生成随机盐
 */
void crypto_generate_salt(uint8_t salt[16]);

/* === 随机数 === */

/**
 * 生成随机字节
 */
void crypto_random_bytes(uint8_t *buf, size_t len);

/**
 * 生成随机字符串
 */
void crypto_random_string(char *buf, size_t len);

/**
 * 生成 UUID
 */
void crypto_generate_uuid(char uuid[37]);

/* === 编码/解码 === */

/**
 * Base64 编码
 */
int crypto_base64_encode(const uint8_t *src, size_t src_len, char *dst, size_t dst_len);

/**
 * Base64 解码
 */
int crypto_base64_decode(const char *src, uint8_t *dst, size_t *dst_len);

/**
 * 十六进制编码
 */
void crypto_hex_encode(const uint8_t *src, size_t src_len, char *dst);

/**
 * 十六进制解码
 */
int crypto_hex_decode(const char *src, uint8_t *dst, size_t *dst_len);

/* === HMAC === */

/**
 * HMAC-SHA256
 */
void crypto_hmac_sha256(const uint8_t *key, size_t key_len,
                        const uint8_t *data, size_t data_len,
                        uint8_t hmac[32]);

#endif /* CRYPTO_H */