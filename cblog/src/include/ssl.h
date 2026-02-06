#ifndef SSL_H
#define SSL_H

#include "server.h"
#include <bearssl.h>

/* === BearSSL 初始化 === */

/**
 * 初始化 BearSSL
 */
int ssl_init(Server *srv);

/**
 * 加载证书和私钥
 */
int ssl_load_certificate(Server *srv, const char *cert_path, const char *key_path);

/**
 * 初始化 SSL 服务器上下文
 */
int ssl_init_server_context(Server *srv, Connection *conn);

/**
 * 处理 SSL 握手
 */
int ssl_handshake(Connection *conn);

/**
 * 读取加密数据
 */
int ssl_read(Connection *conn, void *buf, size_t len);

/**
 * 写入加密数据
 */
int ssl_write(Connection *conn, const void *buf, size_t len);

/**
 * 检查是否完成握手
 */
bool ssl_is_handshake_complete(Connection *conn);

/**
 * 获取 SSL 错误信息
 */
const char *ssl_get_error(int error_code);

#endif /* SSL_H */