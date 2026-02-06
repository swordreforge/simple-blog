/* server.c - 服务器核心实现 (BearSSL + select) */
#include "include/server.h"
#include "include/ssl.h"
#include "include/http.h"
#include "include/router.h"
#include "include/database.h"
#include "include/template.h"
#include "include/common.h"
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <fcntl.h>
#include <errno.h>
#include <string.h>

/* === 创建非阻塞 Socket === */

static int create_nonblocking_socket(void) {
    int fd = socket(AF_INET, SOCK_STREAM, 0);
    if (fd < 0) {
        LOG_ERROR("创建 socket 失败: %s", strerror(errno));
        return -1;
    }
    
    int flags = fcntl(fd, F_GETFL, 0);
    if (flags < 0 || fcntl(fd, F_SETFL, flags | O_NONBLOCK) < 0) {
        LOG_ERROR("设置非阻塞模式失败: %s", strerror(errno));
        close(fd);
        return -1;
    }
    
    int opt = 1;
    setsockopt(fd, SOL_SOCKET, SO_REUSEADDR, &opt, sizeof(opt));
    
    return fd;
}

/* === 绑定监听端口 === */

static int bind_listen_socket(const char *host, int port) {
    int fd = create_nonblocking_socket();
    if (fd < 0) return -1;
    
    struct sockaddr_in addr;
    memset(&addr, 0, sizeof(addr));
    addr.sin_family = AF_INET;
    addr.sin_port = htons(port);
    
    if (host && strlen(host) > 0) {
        if (inet_pton(AF_INET, host, &addr.sin_addr) <= 0) {
            LOG_ERROR("无效的主机地址: %s", host);
            close(fd);
            return -1;
        }
    } else {
        addr.sin_addr.s_addr = htonl(INADDR_ANY);
    }
    
    if (bind(fd, (struct sockaddr*)&addr, sizeof(addr)) < 0) {
        LOG_ERROR("绑定端口失败: %s", strerror(errno));
        close(fd);
        return -1;
    }
    
    if (listen(fd, 128) < 0) {
        LOG_ERROR("监听失败: %s", strerror(errno));
        close(fd);
        return -1;
    }
    
    return fd;
}

/* === 创建新连接 === */

static Connection *create_connection(Server *srv, int fd) {
    Connection *conn = SAFE_CALLOC(1, sizeof(Connection));
    
    conn->fd = fd;
    conn->state = CONN_STATE_ACCEPTING;
    conn->last_activity = time(NULL);
    conn->rlen = 0;
    conn->wlen = 0;
    
    /* 初始化 HTTP 请求/响应 */
    memset(&conn->request, 0, sizeof(conn->request));
    memset(&conn->response, 0, sizeof(conn->response));
    
    /* 如果启用了 TLS，初始化 SSL */
    if (srv->config.tls.enabled) {
        if (ssl_init_server_context(srv, conn) < 0) {
            LOG_ERROR("初始化 SSL 上下文失败");
            free(conn);
            return NULL;
        }
    }
    
    return conn;
}

/* === 服务器初始化 === */

int server_init(Server *srv, const char *config_path) {
    memset(srv, 0, sizeof(Server));
    srv->running = false;
    
    /* TODO: 加载配置文件 */
    /* 暂时使用默认配置 */
    STR_COPY(srv->config.server.host, "0.0.0.0");
    srv->config.server.port = 8080;
    srv->config.server.max_connections = MAX_CONNECTIONS;
    srv->config.server.timeout = 30;
    
    STR_COPY(srv->config.database.path, "data/blog.db");
    srv->config.database.cache_size = 2000;
    srv->config.database.page_size = 1024;
    
    STR_COPY(srv->config.tls.cert_path, "data/cert.der");
    STR_COPY(srv->config.tls.key_path, "data/key.der");
    srv->config.tls.enabled = false;  /* 默认禁用 TLS，从配置文件读取 */
    
    /* TODO: 实际上应该从配置文件读取这些值 */
    /* 暂时禁用 TLS 用于测试 */
    
    /* 初始化数据库 */
    LOG_INFO("初始化数据库: %s", srv->config.database.path);
    if (database_init(&srv->db, srv->config.database.path) < 0) {
        LOG_ERROR("数据库初始化失败");
        return -1;
    }
    
    /* 检查数据库表是否存在 */
    /* TODO: 实现更完善的检查 */
    
    /* 如果启用了 TLS，加载证书 */
    if (srv->config.tls.enabled) {
        LOG_INFO("加载 TLS 证书: %s", srv->config.tls.cert_path);
        if (ssl_init(srv) < 0) {
            LOG_ERROR("TLS 初始化失败");
            return -1;
        }
    }
    
    /* 创建监听 Socket */
    LOG_INFO("绑定监听端口: %s:%d", 
             srv->config.server.host, srv->config.server.port);
    srv->listen_fd = bind_listen_socket(srv->config.server.host, 
                                        srv->config.server.port);
    if (srv->listen_fd < 0) {
        return -1;
    }
    
    /* 初始化文件描述符集合 */
    FD_ZERO(&srv->read_fds);
    FD_ZERO(&srv->write_fds);
    FD_SET(srv->listen_fd, &srv->read_fds);
    srv->max_fd = srv->listen_fd;
    
    srv->running = true;
    LOG_INFO("服务器初始化完成");
    
    return 0;
}

/* === 服务器运行（主事件循环） === */

int server_run(Server *srv) {
    LOG_INFO("启动事件循环...");
    
    while (srv->running) {
        fd_set read_fds = srv->read_fds;
        fd_set write_fds = srv->write_fds;
        
        /* 设置超时 */
        struct timeval tv;
        tv.tv_sec = 1;
        tv.tv_usec = 0;
        
        /* 等待事件 */
        int ready = select(srv->max_fd + 1, &read_fds, &write_fds, NULL, &tv);
        if (ready < 0) {
            if (errno == EINTR) continue;
            LOG_ERROR("select 失败: %s", strerror(errno));
            break;
        }
        
        /* 检查新连接 */
        if (FD_ISSET(srv->listen_fd, &read_fds)) {
            server_accept_connection(srv);
        }
        
        /* 处理现有连接 */
        time_t now = time(NULL);
        for (int i = 0; i < srv->config.server.max_connections; i++) {
            Connection *conn = srv->connections[i];
            if (!conn) continue;
            
            /* 检查超时 */
            if (now - conn->last_activity > srv->config.server.timeout) {
                LOG_INFO("连接超时: fd=%d", conn->fd);
                server_close_connection(srv, conn);
                continue;
            }
            
            /* 处理读写事件 */
            if (FD_ISSET(conn->fd, &read_fds) || FD_ISSET(conn->fd, &write_fds)) {
                if (server_handle_connection(srv, conn) < 0) {
                    server_close_connection(srv, conn);
                }
            }
        }
    }
    
    return 0;
}

/* === 接受新连接 === */

int server_accept_connection(Server *srv) {
    struct sockaddr_in client_addr;
    socklen_t addr_len = sizeof(client_addr);
    
    int fd = accept(srv->listen_fd, (struct sockaddr*)&client_addr, &addr_len);
    if (fd < 0) {
        if (errno != EAGAIN && errno != EWOULDBLOCK) {
            LOG_ERROR("accept 失败: %s", strerror(errno));
        }
        return -1;
    }
    
    /* 查找空闲槽位 */
    int slot = -1;
    for (int i = 0; i < srv->config.server.max_connections; i++) {
        if (srv->connections[i] == NULL) {
            slot = i;
            break;
        }
    }
    
    if (slot < 0) {
        LOG_WARN("连接数已达上限，拒绝连接");
        close(fd);
        return -1;
    }
    
    /* 创建连接 */
    Connection *conn = create_connection(srv, fd);
    if (!conn) {
        close(fd);
        return -1;
    }
    
    srv->connections[slot] = conn;
    srv->connection_count++;
    
    /* 添加到 fd_set */
    FD_SET(fd, &srv->read_fds);
    if (fd > srv->max_fd) {
        srv->max_fd = fd;
    }
    
    char client_ip[INET_ADDRSTRLEN];
    inet_ntop(AF_INET, &client_addr.sin_addr, client_ip, sizeof(client_ip));
    LOG_INFO("新连接: %s:%d (fd=%d)", client_ip, ntohs(client_addr.sin_port), fd);
    
    return 0;
}

/* === 处理连接 === */

int server_handle_connection(Server *srv, Connection *conn) {
    conn->last_activity = time(NULL);
    
    /* 如果需要 SSL 握手 */
    if (srv->config.tls.enabled && !ssl_is_handshake_complete(conn)) {
        int ret = ssl_handshake(conn);
        if (ret < 0) {
            LOG_ERROR("SSL 握手失败");
            return -1;
        }
        if (ret == 0) {
            /* 握手未完成，继续等待 */
            return 0;
        }
        LOG_INFO("SSL 握手完成: fd=%d", conn->fd);
    }
    
    /* 读取数据 */
    if (conn->rlen < BUFFER_SIZE) {
        ssize_t n;
        if (srv->config.tls.enabled) {
            n = ssl_read(conn, conn->rbuf + conn->rlen, BUFFER_SIZE - conn->rlen);
        } else {
            n = recv(conn->fd, conn->rbuf + conn->rlen, BUFFER_SIZE - conn->rlen, 0);
        }
        
        if (n < 0) {
            if (errno != EAGAIN && errno != EWOULDBLOCK) {
                LOG_ERROR("读取失败: %s", strerror(errno));
                return -1;
            }
        } else if (n == 0) {
            LOG_INFO("连接关闭: fd=%d", conn->fd);
            return -1;
        } else {
            conn->rlen += n;
            conn->bytes_received += n;
            
            /* 尝试解析 HTTP 请求 */
            if (conn->rlen > 0) {
                conn->rbuf[conn->rlen] = '\0';
                if (http_parse_request((char*)conn->rbuf, conn->rlen, &conn->request) >= 0) {
                    /* 请求解析成功，处理请求 */
                    router_handle_request(srv, conn, &conn->request);
                    conn->rlen = 0;  /* 清空缓冲区 */
                    return 0;
                }
            }
        }
    }
    
    return 0;
}

/* === 关闭连接 === */

void server_close_connection(Server *srv, Connection *conn) {
    if (!conn) return;
    
    LOG_INFO("关闭连接: fd=%d", conn->fd);
    
    /* 从 fd_set 中移除 */
    FD_CLR(conn->fd, &srv->read_fds);
    FD_CLR(conn->fd, &srv->write_fds);
    
    /* 关闭 socket */
    close(conn->fd);
    
    /* 从连接池中移除 */
    for (int i = 0; i < srv->config.server.max_connections; i++) {
        if (srv->connections[i] == conn) {
            srv->connections[i] = NULL;
            srv->connection_count--;
            break;
        }
    }
    
    /* 释放内存 */
    free(conn);
}

/* === 发送 HTTP 响应 === */

int server_send_response(Server *srv, Connection *conn, HttpResponse *resp) {
    char buf[MAX_RESPONSE_SIZE];
    int len = http_build_response(resp, buf, sizeof(buf));
    
    if (len <= 0) {
        LOG_ERROR("构建响应失败");
        return -1;
    }
    
    ssize_t n;
    if (srv->config.tls.enabled) {
        n = ssl_write(conn, (uint8_t*)buf, len);
    } else {
        n = send(conn->fd, buf, len, 0);
    }
    
    if (n < 0) {
        LOG_ERROR("发送响应失败: %s", strerror(errno));
        return -1;
    }
    
    conn->bytes_sent += n;
    return 0;
}

/* === 发送错误响应 === */

int server_send_error(Server *srv, Connection *conn, int status, const char *message) {
    HttpResponse resp;
    http_response_init(&resp, status);
    http_set_header(&resp, "Content-Type", "application/json");

    char *body = http_build_error_response(status, message);
    http_set_body(&resp, body, strlen(body));

    int ret = server_send_response(srv, conn, &resp);
    free(body);

    return ret;
}

/* === 发送错误页面（使用模板） === */

int server_send_error_page(Server *srv, Connection *conn, int status, const char *message, const char *description) {
    TemplateContext *ctx = template_context_create();
    if (!ctx) {
        return server_send_error(srv, conn, status, message);
    }

    /* 设置模板变量 */
    char status_str[16];
    snprintf(status_str, sizeof(status_str), "%d", status);
    template_set_var(ctx, "status", status_str);
    template_set_var(ctx, "message", message ? message : "Error");
    template_set_var(ctx, "description", description ? description : "服务器发生错误，请稍后重试");

    /* 渲染模板 */
    char *html = template_render(ctx, "error");
    template_context_destroy(ctx);

    if (!html) {
        return server_send_error(srv, conn, status, message);
    }

    /* 发送 HTML 响应 */
    HttpResponse resp;
    http_response_init(&resp, status);
    http_set_header(&resp, "Content-Type", "text/html; charset=utf-8");
    http_set_body(&resp, html, strlen(html));

    int ret = server_send_response(srv, conn, &resp);
    free(html);

    return ret;
}

/* === 发送 JSON 响应 === */

int server_send_json(Server *srv, Connection *conn, const char *json) {
    HttpResponse resp;
    http_response_init(&resp, HTTP_STATUS_OK);
    http_set_header(&resp, "Content-Type", "application/json");
    http_set_body(&resp, json, strlen(json));
    
    return server_send_response(srv, conn, &resp);
}

/* === 发送 HTML 响应 === */

int server_send_html(Server *srv, Connection *conn, const char *html) {
    HttpResponse resp;
    http_response_init(&resp, HTTP_STATUS_OK);
    http_set_header(&resp, "Content-Type", "text/html; charset=utf-8");
    http_set_body(&resp, html, strlen(html));
    
    return server_send_response(srv, conn, &resp);
}

/* === 发送重定向 === */

int server_send_redirect(Server *srv, Connection *conn, const char *location) {
    HttpResponse resp;
    http_response_init(&resp, 302);
    http_set_header(&resp, "Location", location);
    http_set_body(&resp, "", 0);
    
    return server_send_response(srv, conn, &resp);
}

/* === 停止服务器 === */

void server_stop(Server *srv) {
    srv->running = false;
}

/* === 清理服务器资源 === */

void server_cleanup(Server *srv) {
    /* 关闭所有连接 */
    for (int i = 0; i < srv->config.server.max_connections; i++) {
        if (srv->connections[i]) {
            server_close_connection(srv, srv->connections[i]);
        }
    }
    
    /* 关闭监听 socket */
    if (srv->listen_fd >= 0) {
        close(srv->listen_fd);
    }
    
    /* 关闭数据库 */
    if (srv->db) {
        database_close(srv->db);
    }
    
    LOG_INFO("服务器资源已清理");
}