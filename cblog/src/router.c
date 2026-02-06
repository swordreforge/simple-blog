/* router.c - 路由系统 */
#include "include/router.h"
#include "include/common.h"
#include "include/server.h"
#include "include/http.h"
#include "include/template.h"
#include <string.h>
#include <stdlib.h>

/* === 路由表 === */

#define MAX_ROUTES 64

static Route routes[MAX_ROUTES];
static int route_count = 0;

/* === 初始化路由表 === */

void router_init(void) {
    route_count = 0;
    memset(routes, 0, sizeof(routes));
    
    /* 注册页面路由 */
    router_register("/", HTTP_GET, handle_home, false, false);
    router_register("/passage/*", HTTP_GET, handle_passage_detail, false, false);
    router_register("/archive", HTTP_GET, handle_archive, false, false);
    router_register("/about", HTTP_GET, handle_about, false, false);
    router_register("/friends", HTTP_GET, handle_friends, false, false);
    router_register("/login", HTTP_GET, handle_login, false, false);
    router_register("/admin", HTTP_GET, handle_admin, true, true);
    
    /* API 路由 - 文章 */
    router_register("/api/passages", HTTP_GET, api_handle_passages_list, false, false);
    router_register("/api/passage/*", HTTP_GET, api_handle_passage_get, false, false);
    router_register("/api/admin/passage", HTTP_POST, api_handle_passage_create, true, true);
    router_register("/api/admin/passage/*", HTTP_PUT, api_handle_passage_update, true, true);
    router_register("/api/admin/passage/*", HTTP_DELETE, api_handle_passage_delete, true, true);
    
    /* API 路由 - 认证 */
    router_register("/api/auth/login", HTTP_POST, api_handle_auth_login, false, false);
    router_register("/api/auth/logout", HTTP_POST, api_handle_auth_logout, true, false);
    router_register("/api/auth/verify", HTTP_GET, api_handle_auth_verify, true, false);
    
    /* API 路由 - 用户 */
    router_register("/api/admin/users", HTTP_GET, api_handle_users_list, true, true);
    router_register("/api/admin/user", HTTP_POST, api_handle_user_create, true, true);
    router_register("/api/admin/user/*", HTTP_PUT, api_handle_user_update, true, true);
    router_register("/api/admin/user/*", HTTP_DELETE, api_handle_user_delete, true, true);
    
    /* API 路由 - 评论 */
    router_register("/api/comments/*", HTTP_GET, api_handle_comments_list, false, false);
    router_register("/api/comment", HTTP_POST, api_handle_comment_create, false, false);
    router_register("/api/admin/comment/*", HTTP_DELETE, api_handle_comment_delete, true, true);
    
    /* API 路由 - 音乐 */
    router_register("/api/music", HTTP_GET, api_handle_music_list, false, false);
    router_register("/api/music/*", HTTP_GET, api_handle_music_get, false, false);
    router_register("/api/admin/music", HTTP_POST, api_handle_music_upload, true, true);
    router_register("/api/admin/music/*", HTTP_DELETE, api_handle_music_delete, true, true);
    
    /* API 路由 - 设置 */
    router_register("/api/admin/settings", HTTP_GET, api_handle_settings_get, true, true);
    router_register("/api/admin/settings", HTTP_PUT, api_handle_settings_update, true, true);
    
    /* API 路由 - 统计 */
    router_register("/api/stats", HTTP_GET, api_handle_stats, true, true);
    
    /* 静态文件 */
    router_register("/static/*", HTTP_GET, handle_static_file, false, false);
    router_register("/favicon.ico", HTTP_GET, handle_favicon, false, false);
    
    LOG_INFO("已注册 %d 个路由", route_count);
}

/* === 注册路由 === */

void router_register(const char *pattern, HttpMethod method, RouteHandler handler,
                     bool require_auth, bool require_admin) {
    if (route_count >= MAX_ROUTES) {
        LOG_WARN("路由表已满，无法注册: %s", pattern);
        return;
    }
    
    STR_COPY(routes[route_count].pattern, pattern);
    routes[route_count].method = method;
    routes[route_count].handler = handler;
    routes[route_count].require_auth = require_auth;
    routes[route_count].require_admin = require_admin;
    route_count++;
}

/* === 简单的路径匹配 === */

static bool path_match(const char *pattern, const char *path) {
    const char *p = pattern;
    const char *pp = path;
    
    while (*p && *pp) {
        if (*p == '*') {
            /* 通配符匹配 */
            p++;
            if (*p == '\0') return true;  /* 以 * 结尾，匹配剩余所有 */
            /* 跳过直到下一个 / */
            while (*pp && *pp != '/') pp++;
        } else if (*p == *pp) {
            p++;
            pp++;
        } else {
            return false;
        }
    }
    
    return (*p == '\0' && *pp == '\0');
}

/* === 匹配路由 === */

RouteHandler router_match(const char *path, HttpMethod method, bool *need_auth, bool *need_admin) {
    for (int i = 0; i < route_count; i++) {
        if (routes[i].method == method && path_match(routes[i].pattern, path)) {
            if (need_auth) *need_auth = routes[i].require_auth;
            if (need_admin) *need_admin = routes[i].require_admin;
            return routes[i].handler;
        }
    }
    return NULL;
}

/* === 处理请求 === */

int router_handle_request(Server *srv, Connection *conn, HttpRequest *req) {
    bool need_auth = false;
    bool need_admin = false;
    
    /* 匹配路由 */
    RouteHandler handler = router_match(req->path, req->method, &need_auth, &need_admin);
    
    if (!handler) {
        return handle_not_found(srv, conn, req);
    }
    
    /* 检查权限 */
    if (need_auth) {
        const char *auth_header = http_get_header(req, "Authorization");
        if (!auth_header) {
            return server_send_error_page(srv, conn, HTTP_STATUS_UNAUTHORIZED,
                                        "未登录",
                                        "您需要登录才能访问此页面，请先登录");
        }

        /* TODO: 验证 JWT token */
        /* jwt_verify(auth_header, &claims); */

        /* 如果需要管理员权限 */
        if (need_admin) {
            /* TODO: 检查是否为管理员 */
            /* if (!jwt_is_admin(auth_header)) */
            return server_send_error_page(srv, conn, HTTP_STATUS_FORBIDDEN,
                                        "权限不足",
                                        "您没有权限访问此页面，需要管理员权限");
        }
    }

    /* 执行处理器 */
    return handler(srv, conn, req);
}

/* === 页面处理器 === */

int handle_home(Server *srv, Connection *conn, HttpRequest *req) {
    /* 渲染首页 */
    TemplateContext *ctx = template_context_create();
    
    /* 设置模板变量 */
    template_set_var(ctx, "site_name", "RustBlog");
    template_set_var(ctx, "title", "首页 - RustBlog");
    template_set_var(ctx, "year", "2026");
    template_set_var(ctx, "passages", "<p class=\"passage\">暂无文章</p>");
    
    /* 渲染模板 */
    char *html = template_render(ctx, "index");
    
    /* 发送响应 */
    int ret = server_send_html(srv, conn, html);
    
    /* 清理资源 */
    template_context_destroy(ctx);
    free(html);
    
    return ret;
}

int handle_passage_detail(Server *srv, Connection *conn, HttpRequest *req) {
    /* TODO: 渲染文章详情页 */
    char html[1024];
    snprintf(html, sizeof(html), 
             "<!DOCTYPE html><html><head><title>文章详情</title></head>"
             "<body><h1>文章: %s</h1>"
             "<p>文章内容...</p>"
             "</body></html>", req->path + 9);  /* 跳过 /passage/ */
    return server_send_html(srv, conn, html);
}

int handle_archive(Server *srv, Connection *conn, HttpRequest *req) {
    return server_send_html(srv, conn, "<h1>归档页面</h1><p>开发中...</p>");
}

int handle_about(Server *srv, Connection *conn, HttpRequest *req) {
    return server_send_html(srv, conn, "<h1>关于页面</h1><p>开发中...</p>");
}

int handle_friends(Server *srv, Connection *conn, HttpRequest *req) {
    return server_send_html(srv, conn, "<h1>友链页面</h1><p>开发中...</p>");
}

int handle_login(Server *srv, Connection *conn, HttpRequest *req) {
    return server_send_html(srv, conn, "<h1>登录页面</h1><p>开发中...</p>");
}

int handle_admin(Server *srv, Connection *conn, HttpRequest *req) {
    return server_send_html(srv, conn, "<h1>管理后台</h1><p>开发中...</p>");
}

/* === API 处理器 === */

int api_handle_passages_list(Server *srv, Connection *conn, HttpRequest *req) {
    const char *limit_str = http_get_param(req, "limit");
    const char *page_str = http_get_param(req, "page");
    
    int page = page_str ? atoi(page_str) : 1;
    (void)limit_str;  /* 避免未使用警告 */
    (void)page;      /* 避免未使用警告 */
    
    /* TODO: 从数据库获取文章列表 */
    char *json = "{\"success\":true,\"data\":[],\"pagination\":{\"page\":1,\"limit\":10,\"total\":0}}";
    return server_send_json(srv, conn, json);
}

int api_handle_passage_get(Server *srv, Connection *conn, HttpRequest *req) {
    char *json = "{\"success\":false,\"message\":\"功能开发中\"}";
    return server_send_json(srv, conn, json);
}

int api_handle_passage_create(Server *srv, Connection *conn, HttpRequest *req) {
    char *json = "{\"success\":false,\"message\":\"功能开发中\"}";
    return server_send_json(srv, conn, json);
}

int api_handle_passage_update(Server *srv, Connection *conn, HttpRequest *req) {
    char *json = "{\"success\":false,\"message\":\"功能开发中\"}";
    return server_send_json(srv, conn, json);
}

int api_handle_passage_delete(Server *srv, Connection *conn, HttpRequest *req) {
    char *json = "{\"success\":false,\"message\":\"功能开发中\"}";
    return server_send_json(srv, conn, json);
}

int api_handle_auth_login(Server *srv, Connection *conn, HttpRequest *req) {
    char *json = "{\"success\":false,\"message\":\"功能开发中\"}";
    return server_send_json(srv, conn, json);
}

int api_handle_auth_logout(Server *srv, Connection *conn, HttpRequest *req) {
    char *json = "{\"success\":true,\"message\":\"登出成功\"}";
    return server_send_json(srv, conn, json);
}

int api_handle_auth_verify(Server *srv, Connection *conn, HttpRequest *req) {
    char *json = "{\"success\":true,\"message\":\"Token 有效\"}";
    return server_send_json(srv, conn, json);
}

int api_handle_users_list(Server *srv, Connection *conn, HttpRequest *req) {
    char *json = "{\"success\":false,\"message\":\"功能开发中\"}";
    return server_send_json(srv, conn, json);
}

int api_handle_user_create(Server *srv, Connection *conn, HttpRequest *req) {
    char *json = "{\"success\":false,\"message\":\"功能开发中\"}";
    return server_send_json(srv, conn, json);
}

int api_handle_user_update(Server *srv, Connection *conn, HttpRequest *req) {
    char *json = "{\"success\":false,\"message\":\"功能开发中\"}";
    return server_send_json(srv, conn, json);
}

int api_handle_user_delete(Server *srv, Connection *conn, HttpRequest *req) {
    char *json = "{\"success\":false,\"message\":\"功能开发中\"}";
    return server_send_json(srv, conn, json);
}

int api_handle_comments_list(Server *srv, Connection *conn, HttpRequest *req) {
    char *json = "{\"success\":false,\"message\":\"功能开发中\"}";
    return server_send_json(srv, conn, json);
}

int api_handle_comment_create(Server *srv, Connection *conn, HttpRequest *req) {
    char *json = "{\"success\":false,\"message\":\"功能开发中\"}";
    return server_send_json(srv, conn, json);
}

int api_handle_comment_delete(Server *srv, Connection *conn, HttpRequest *req) {
    char *json = "{\"success\":false,\"message\":\"功能开发中\"}";
    return server_send_json(srv, conn, json);
}

int api_handle_music_list(Server *srv, Connection *conn, HttpRequest *req) {
    char *json = "{\"success\":false,\"message\":\"功能开发中\"}";
    return server_send_json(srv, conn, json);
}

int api_handle_music_get(Server *srv, Connection *conn, HttpRequest *req) {
    char *json = "{\"success\":false,\"message\":\"功能开发中\"}";
    return server_send_json(srv, conn, json);
}

int api_handle_music_upload(Server *srv, Connection *conn, HttpRequest *req) {
    char *json = "{\"success\":false,\"message\":\"功能开发中\"}";
    return server_send_json(srv, conn, json);
}

int api_handle_music_delete(Server *srv, Connection *conn, HttpRequest *req) {
    char *json = "{\"success\":false,\"message\":\"功能开发中\"}";
    return server_send_json(srv, conn, json);
}

int api_handle_settings_get(Server *srv, Connection *conn, HttpRequest *req) {
    char *json = "{\"success\":false,\"message\":\"功能开发中\"}";
    return server_send_json(srv, conn, json);
}

int api_handle_settings_update(Server *srv, Connection *conn, HttpRequest *req) {
    char *json = "{\"success\":false,\"message\":\"功能开发中\"}";
    return server_send_json(srv, conn, json);
}

int api_handle_stats(Server *srv, Connection *conn, HttpRequest *req) {
    char *json = "{\"success\":false,\"message\":\"功能开发中\"}";
    return server_send_json(srv, conn, json);
}

/* === 静态文件处理 === */

int handle_static_file(Server *srv, Connection *conn, HttpRequest *req) {
    /* TODO: 实现静态文件服务 */
    char *json = "{\"success\":false,\"message\":\"静态文件服务开发中\"}";
    return server_send_json(srv, conn, json);
}

int handle_favicon(Server *srv, Connection *conn, HttpRequest *req) {
    /* TODO: 返回 favicon */
    return server_send_error(srv, conn, HTTP_STATUS_NOT_FOUND, "Not Found");
}

/* === 错误处理 === */

int handle_not_found(Server *srv, Connection *conn, HttpRequest *req) {
    const char *path = req->path[0] ? req->path : "/";
    char desc[512];
    snprintf(desc, sizeof(desc), "您访问的页面 <strong>%s</strong> 不存在或已被删除", path);
    return server_send_error_page(srv, conn, HTTP_STATUS_NOT_FOUND, "页面不存在", desc);
}

int handle_method_not_allowed(Server *srv, Connection *conn, HttpRequest *req) {
    const char *method_str = "UNKNOWN";
    switch (req->method) {
        case HTTP_GET: method_str = "GET"; break;
        case HTTP_POST: method_str = "POST"; break;
        case HTTP_PUT: method_str = "PUT"; break;
        case HTTP_DELETE: method_str = "DELETE"; break;
        case HTTP_PATCH: method_str = "PATCH"; break;
        case HTTP_OPTIONS: method_str = "OPTIONS"; break;
        case HTTP_HEAD: method_str = "HEAD"; break;
    }
    char desc[512];
    snprintf(desc, sizeof(desc), "当前页面不支持 <strong>%s</strong> 方法", method_str);
    return server_send_error_page(srv, conn, HTTP_STATUS_METHOD_NOT_ALLOWED, "方法不允许", desc);
}

int handle_internal_error(Server *srv, Connection *conn, HttpRequest *req) {
    return server_send_error_page(srv, conn, HTTP_STATUS_INTERNAL_SERVER_ERROR,
                                  "服务器内部错误",
                                  "服务器遇到了一些问题，请稍后重试或联系管理员");
}