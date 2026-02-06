/* ssl.c - BearSSL 封装 */
#include "include/ssl.h"
#include "include/common.h"
#include <bearssl.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>

/* === 读取 DER 格式证书/私钥 === */

static int read_der_file(const char *path, void **data, size_t *len) {
    FILE *f = fopen(path, "rb");
    if (!f) {
        LOG_ERROR("无法打开文件: %s", path);
        return -1;
    }
    
    fseek(f, 0, SEEK_END);
    *len = ftell(f);
    fseek(f, 0, SEEK_SET);
    
    *data = SAFE_MALLOC(*len);
    if (fread(*data, 1, *len, f) != *len) {
        LOG_ERROR("读取文件失败: %s", path);
        free(*data);
        fclose(f);
        return -1;
    }
    
    fclose(f);
    return 0;
}

/* === 初始化 BearSSL === */

int ssl_init(Server *srv) {
    /* 加载证书 */
    void *cert_data = NULL;
    void *key_data = NULL;
    size_t cert_len = 0;
    size_t key_len = 0;
    
    if (read_der_file(srv->config.tls.cert_path, &cert_data, &cert_len) < 0) {
        LOG_ERROR("加载证书失败: %s", srv->config.tls.cert_path);
        return -1;
    }
    
    if (read_der_file(srv->config.tls.key_path, &key_data, &key_len) < 0) {
        LOG_ERROR("加载私钥失败: %s", srv->config.tls.key_path);
        free(cert_data);
        return -1;
    }
    
    /* 解析证书 */
    br_x509_certificate *xc = &srv->cert;
    xc->data = cert_data;
    xc->data_len = cert_len;
    
    /* 解析私钥（假设是 RSA） */
    br_skey_decoder_context kc;
    br_skey_decoder_init(&kc);
    br_skey_decoder_push(&kc, key_data, key_len);
    
    if (!br_skey_decoder_last_error(&kc)) {
        const br_rsa_private_key *rk = br_skey_decoder_get_rsa(&kc);
        if (rk) {
            /* 复制 RSA 私钥 */
            srv->key = *rk;
            srv->key.n_bitlen = rk->n_bitlen;
        } else {
            LOG_ERROR("私钥格式不支持（仅支持 RSA）");
            free(cert_data);
            free(key_data);
            return -1;
        }
    } else {
        LOG_ERROR("解析私钥失败: 0x%04X", br_skey_decoder_last_error(&kc));
        free(cert_data);
        free(key_data);
        return -1;
    }
    
    /* 证书数据需要保留到程序结束 */
    /* 注意：实际应该将 cert_data 存储在 Server 结构中 */
    
    LOG_INFO("BearSSL 初始化完成");
    return 0;
}

/* === 加载证书和私钥 === */

int ssl_load_certificate(Server *srv, const char *cert_path, const char *key_path) {
    STR_COPY(srv->config.tls.cert_path, cert_path);
    STR_COPY(srv->config.tls.key_path, key_path);
    return ssl_init(srv);
}

/* === 初始化 SSL 服务器上下文 === */

int ssl_init_server_context(Server *srv, Connection *conn) {
    /* 清空 SSL 上下文 */
    memset(&conn->sc, 0, sizeof(br_ssl_server_context));
    
    /* 初始化 SSL 服务器上下文 */
    br_ssl_server_init_full_rsa(&conn->sc, &srv->cert, 1, &srv->key);
    
    /* 设置缓冲区 - 使用单独的缓冲区给 SSL 引擎 */
    br_ssl_engine_set_buffer(&conn->sc.eng, conn->rbuf, sizeof(conn->rbuf), 1);
    
    /* 重置 SSL 引擎 */
    br_ssl_server_reset(&conn->sc);
    
    conn->state = CONN_STATE_HANDSHAKE;
    return 0;
}

/* === 处理 SSL 握手 === */

int ssl_handshake(Connection *conn) {
    unsigned char *buf;
    size_t len;
    int st;
    
    while (1) {
        /* 检查当前状态 */
        st = br_ssl_engine_current_state(&conn->sc.eng);
        
        /* 如果连接关闭，表示握手失败 */
        if (st & BR_SSL_CLOSED) {
            unsigned int err = br_ssl_engine_last_error(&conn->sc.eng);
            LOG_ERROR("SSL 握手失败: 错误码 0x%04X", err);
            return -1;
        }
        
        /* 检查握手是否完成 */
        if (st & BR_SSL_SENDAPP) {
            conn->state = CONN_STATE_READING;
            LOG_INFO("SSL 握手完成: fd=%d", conn->fd);
            return 1;  /* 握手完成 */
        }
        
        /* 如果引擎有记录要发送 */
        if (st & BR_SSL_SENDREC) {
            buf = br_ssl_engine_sendrec_buf(&conn->sc.eng, &len);
            LOG_DEBUG("SSL 握手: 发送 %zu 字节", len);
            if (len > 0) {
                ssize_t n = send(conn->fd, buf, len, 0);
                if (n < 0) {
                    if (errno == EAGAIN || errno == EWOULDBLOCK) {
                        return 0;  /* 需要等待可写 */
                    }
                    LOG_ERROR("SSL 发送失败: %s", strerror(errno));
                    return -1;
                }
                br_ssl_engine_sendrec_ack(&conn->sc.eng, n);
                LOG_DEBUG("SSL 握手: 已发送 %zd 字节", n);
            }
        }
        
        /* 如果引擎需要接收记录 */
        if (st & BR_SSL_RECVREC) {
            buf = br_ssl_engine_recvrec_buf(&conn->sc.eng, &len);
            LOG_DEBUG("SSL 握手: 接收缓冲区 %zu 字节", len);
            if (len > 0) {
                ssize_t n = recv(conn->fd, buf, len, 0);
                if (n < 0) {
                    if (errno == EAGAIN || errno == EWOULDBLOCK) {
                        return 0;  /* 需要等待可读 */
                    }
                    LOG_ERROR("SSL 接收失败: %s", strerror(errno));
                    return -1;
                } else if (n == 0) {
                    LOG_ERROR("SSL 连接关闭: fd=%d", conn->fd);
                    return -1;
                }
                br_ssl_engine_recvrec_ack(&conn->sc.eng, n);
                LOG_DEBUG("SSL 握手: 已接收 %zd 字节", n);
            }
        }
        
        /* 如果既不能发送也不能接收，可能是处理中 */
        if (!(st & (BR_SSL_SENDREC | BR_SSL_RECVREC))) {
            LOG_DEBUG("SSL 握手: 状态 0x%X, 等待", st);
            /* 引擎可能在处理数据，再次检查 */
            return 0;
        }
    }
}

/* === 检查是否完成握手 === */

bool ssl_is_handshake_complete(Connection *conn) {
    int st = br_ssl_engine_current_state(&conn->sc.eng);
    /* 握手完成当引擎处于应用数据发送状态 */
    return (st & BR_SSL_SENDAPP) != 0;
}

/* === 读取加密数据 === */

int ssl_read(Connection *conn, void *buf, size_t len) {
    unsigned char *src;
    size_t src_len;
    int st;
    
    while (1) {
        st = br_ssl_engine_current_state(&conn->sc.eng);
        
        /* 如果连接关闭 */
        if (st & BR_SSL_CLOSED) {
            return -1;
        }
        
        /* 如果有应用数据可读 */
        if (st & BR_SSL_RECVAPP) {
            src = br_ssl_engine_recvapp_buf(&conn->sc.eng, &src_len);
            if (src_len > 0) {
                size_t to_copy = src_len < len ? src_len : len;
                memcpy(buf, src, to_copy);
                br_ssl_engine_recvapp_ack(&conn->sc.eng, to_copy);
                return (int)to_copy;
            }
        }
        
        /* 如果需要接收记录 */
        if (st & BR_SSL_RECVREC) {
            src = br_ssl_engine_recvrec_buf(&conn->sc.eng, &src_len);
            if (src_len > 0) {
                ssize_t n = recv(conn->fd, src, src_len, 0);
                if (n < 0) {
                    if (errno == EAGAIN || errno == EWOULDBLOCK) {
                        return 0;  /* 需要等待可读 */
                    }
                    return -1;
                } else if (n == 0) {
                    return -1;  /* 连接关闭 */
                }
                br_ssl_engine_recvrec_ack(&conn->sc.eng, n);
            }
        }
        
        /* 没有数据可读，等待 */
        return 0;
    }
}

/* === 写入加密数据 === */

int ssl_write(Connection *conn, const void *buf, size_t len) {
    unsigned char *dst;
    size_t dst_len;
    size_t written = 0;
    const unsigned char *src = (const unsigned char *)buf;
    
    while (written < len) {
        int st = br_ssl_engine_current_state(&conn->sc.eng);
        
        /* 如果连接关闭 */
        if (st & BR_SSL_CLOSED) {
            return -1;
        }
        
        /* 如果可以写入应用数据 */
        if (st & BR_SSL_SENDAPP) {
            dst = br_ssl_engine_sendapp_buf(&conn->sc.eng, &dst_len);
            if (dst_len > 0) {
                size_t to_write = (len - written) < dst_len ? (len - written) : dst_len;
                memcpy(dst, src + written, to_write);
                br_ssl_engine_sendapp_ack(&conn->sc.eng, to_write);
                written += to_write;
            }
        }
        
        /* 如果需要发送记录 */
        if (st & BR_SSL_SENDREC) {
            dst = br_ssl_engine_sendrec_buf(&conn->sc.eng, &dst_len);
            if (dst_len > 0) {
                ssize_t n = send(conn->fd, dst, dst_len, 0);
                if (n < 0) {
                    if (errno == EAGAIN || errno == EWOULDBLOCK) {
                        return 0;  /* 需要等待可写 */
                    }
                    return -1;
                }
                br_ssl_engine_sendrec_ack(&conn->sc.eng, n);
            }
        }
    }
    
    /* 刷新所有待发送的数据 */
    while (1) {
        int st = br_ssl_engine_current_state(&conn->sc.eng);
        if (st & BR_SSL_SENDREC) {
            dst = br_ssl_engine_sendrec_buf(&conn->sc.eng, &dst_len);
            if (dst_len > 0) {
                ssize_t n = send(conn->fd, dst, dst_len, 0);
                if (n < 0) {
                    if (errno == EAGAIN || errno == EWOULDBLOCK) {
                        return 0;  /* 需要等待可写 */
                    }
                    return -1;
                }
                br_ssl_engine_sendrec_ack(&conn->sc.eng, n);
            }
        } else {
            break;
        }
    }
    
    return (int)written;
}

/* === 获取 SSL 错误信息 === */

const char *ssl_get_error(int error_code) {
    static char err_buf[256];
    
    switch (error_code) {
        case BR_ERR_OK:
            return "No error";
        case BR_ERR_IO:
            return "I/O error";
        case BR_ERR_BAD_VERSION:
            return "Bad protocol version";
        case BR_ERR_BAD_CIPHER_SUITE:
            return "Bad cipher suite";
        case BR_ERR_BAD_ALERT:
            return "Bad alert";
        case BR_ERR_BAD_LENGTH:
            return "Bad length";
        case BR_ERR_BAD_STATE:
            return "Bad state";
        case BR_ERR_UNKNOWN_TYPE:
            return "Unknown type";
        case BR_ERR_UNEXPECTED:
            return "Unexpected message";
        default:
            snprintf(err_buf, sizeof(err_buf), "Unknown error: %d", error_code);
            return err_buf;
    }
}