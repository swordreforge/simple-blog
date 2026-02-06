/* server.h - 服务器核心定义 */
#ifndef SERVER_H
#define SERVER_H

#include "types.h"
#include <sys/types.h>
#include <sys/select.h>

/* BearSSL 头文件 */
#include <bearssl.h>

/* SQLite 前向声明 */
typedef struct sqlite3 sqlite3;

/* === 连接结构 === */

#define MAX_CONNECTIONS 32
#define BUFFER_SIZE 8192

typedef struct {
    int fd;
    ConnectionState state;
    
    /* BearSSL 上下文 */
    br_ssl_server_context sc;
    br_sslio_context ioc;
    
    /* 缓冲区 */
    uint8_t rbuf[BUFFER_SIZE];
    uint8_t wbuf[BUFFER_SIZE];
    size_t rlen;
    size_t wlen;
    
    /* HTTP 解析 */
    HttpRequest request;
    HttpResponse response;
    
    /* 超时管理 */
    time_t last_activity;
    
    /* 统计 */
    size_t bytes_sent;
    size_t bytes_received;
} Connection;

/* === 服务器结构 === */

typedef struct {
    Config config;
    sqlite3 *db;
    
    /* BearSSL */
    br_x509_certificate cert;
    br_rsa_private_key key;
    br_ssl_server_context *sc;
    
    /* Socket */
    int listen_fd;
    
    /* 连接池 */
    Connection *connections[MAX_CONNECTIONS];
    int connection_count;
    
    /* 文件描述符集合 */
    fd_set read_fds;
    fd_set write_fds;
    int max_fd;
    
    /* 运行状态 */
    bool running;
} Server;

/* === 服务器 API === */

/**
 * 初始化服务器
 */
int server_init(Server *srv, const char *config_path);

/**
 * 启动服务器（主事件循环）
 */
int server_run(Server *srv);

/**
 * 停止服务器
 */
void server_stop(Server *srv);

/**
 * 清理服务器资源
 */
void server_cleanup(Server *srv);

/**
 * 接受新连接
 */
int server_accept_connection(Server *srv);

/**
 * 处理连接数据
 */
int server_handle_connection(Server *srv, Connection *conn);

/**
 * 关闭连接
 */
void server_close_connection(Server *srv, Connection *conn);

/**
 * 发送 HTTP 响应
 */
int server_send_response(Server *srv, Connection *conn, HttpResponse *resp);

/**
 * 发送错误响应
 */
int server_send_error(Server *srv, Connection *conn, int status, const char *message);

/**
 * 发送静态文件
 */
int server_send_file(Server *srv, Connection *conn, const char *path);

/**
 * 发送 JSON 响应
 */
int server_send_json(Server *srv, Connection *conn, const char *json);

/**
 * 发送 HTML 响应
 */
int server_send_html(Server *srv, Connection *conn, const char *html);

/**
 * 发送重定向
 */
int server_send_redirect(Server *srv, Connection *conn, const char *location);

/**
 * 发送错误页面（使用模板）
 */
int server_send_error_page(Server *srv, Connection *conn, int status, const char *message, const char *description);

#endif /* SERVER_H */