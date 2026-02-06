#ifndef ROUTER_H
#define ROUTER_H

#include "types.h"
#include "server.h"

/* === 路由处理函数类型 === */

typedef int (*RouteHandler)(Server *srv, Connection *conn, HttpRequest *req);

/* === 路由结构 === */

typedef struct {
    char pattern[256];
    HttpMethod method;
    RouteHandler handler;
    bool require_auth;  // 是否需要认证
    bool require_admin; // 是否需要管理员权限
} Route;

/* === 路由注册 === */

/**
 * 初始化路由表
 */
void router_init(void);

/**
 * 注册路由
 */
void router_register(const char *pattern, HttpMethod method, RouteHandler handler,
                     bool require_auth, bool require_admin);

/**
 * 匹配路由
 */
RouteHandler router_match(const char *path, HttpMethod method, bool *need_auth, bool *need_admin);

/**
 * 处理请求
 */
int router_handle_request(Server *srv, Connection *conn, HttpRequest *req);

/* === 预定义路由处理函数 === */

/* 页面路由 */
int handle_home(Server *srv, Connection *conn, HttpRequest *req);
int handle_passage_detail(Server *srv, Connection *conn, HttpRequest *req);
int handle_archive(Server *srv, Connection *conn, HttpRequest *req);
int handle_about(Server *srv, Connection *conn, HttpRequest *req);
int handle_friends(Server *srv, Connection *conn, HttpRequest *req);
int handle_login(Server *srv, Connection *conn, HttpRequest *req);
int handle_admin(Server *srv, Connection *conn, HttpRequest *req);

/* API 路由 */
int api_handle_passages_list(Server *srv, Connection *conn, HttpRequest *req);
int api_handle_passage_get(Server *srv, Connection *conn, HttpRequest *req);
int api_handle_passage_create(Server *srv, Connection *conn, HttpRequest *req);
int api_handle_passage_update(Server *srv, Connection *conn, HttpRequest *req);
int api_handle_passage_delete(Server *srv, Connection *conn, HttpRequest *req);

int api_handle_auth_login(Server *srv, Connection *conn, HttpRequest *req);
int api_handle_auth_logout(Server *srv, Connection *conn, HttpRequest *req);
int api_handle_auth_verify(Server *srv, Connection *conn, HttpRequest *req);

int api_handle_users_list(Server *srv, Connection *conn, HttpRequest *req);
int api_handle_user_create(Server *srv, Connection *conn, HttpRequest *req);
int api_handle_user_update(Server *srv, Connection *conn, HttpRequest *req);
int api_handle_user_delete(Server *srv, Connection *conn, HttpRequest *req);

int api_handle_comments_list(Server *srv, Connection *conn, HttpRequest *req);
int api_handle_comment_create(Server *srv, Connection *conn, HttpRequest *req);
int api_handle_comment_delete(Server *srv, Connection *conn, HttpRequest *req);

int api_handle_music_list(Server *srv, Connection *conn, HttpRequest *req);
int api_handle_music_get(Server *srv, Connection *conn, HttpRequest *req);
int api_handle_music_upload(Server *srv, Connection *conn, HttpRequest *req);
int api_handle_music_delete(Server *srv, Connection *conn, HttpRequest *req);

int api_handle_settings_get(Server *srv, Connection *conn, HttpRequest *req);
int api_handle_settings_update(Server *srv, Connection *conn, HttpRequest *req);

int api_handle_stats(Server *srv, Connection *conn, HttpRequest *req);

/* 静态文件处理 */
int handle_static_file(Server *srv, Connection *conn, HttpRequest *req);
int handle_favicon(Server *srv, Connection *conn, HttpRequest *req);

/* 错误处理 */
int handle_not_found(Server *srv, Connection *conn, HttpRequest *req);
int handle_method_not_allowed(Server *srv, Connection *conn, HttpRequest *req);
int handle_internal_error(Server *srv, Connection *conn, HttpRequest *req);

#endif /* ROUTER_H */