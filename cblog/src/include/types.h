#ifndef TYPES_H
#define TYPES_H

#include <stdint.h>
#include <stdbool.h>
#include <time.h>

/* === 基础类型定义 === */

typedef struct {
    int id;
    char uuid[37];  // UUID 格式: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
    char title[256];
    char content[16384];  // Markdown 内容
    char html_content[32768];  // 渲染后的 HTML
    char summary[512];
    char author[64];
    char tags[512];  // JSON 数组字符串
    char category[128];
    char status[32];  // draft, published, archived
    char file_path[512];
    char visibility[32];  // public, private, password
    bool is_scheduled;
    int64_t published_at;  // Unix 时间戳
    char cover_image[512];
    int64_t created_at;
    int64_t updated_at;
} Passage;

typedef struct {
    int id;
    char username[64];
    char password_hash[256];
    char email[128];
    char role[32];  // admin, user
    char status[32];  // active, inactive
    int64_t created_at;
    int64_t updated_at;
} User;

typedef struct {
    int id;
    char username[64];
    char content[4096];
    char passage_uuid[37];
    int64_t created_at;
} Comment;

typedef struct {
    int id;
    char title[128];
    char description[512];
    char icon[64];
    int sort_order;
    bool is_enabled;
    int64_t created_at;
    int64_t updated_at;
} Category;

typedef struct {
    int id;
    char name[64];
    char description[256];
    char color[16];
    int category_id;
    int sort_order;
    bool is_enabled;
    int64_t created_at;
    int64_t updated_at;
} Tag;

typedef struct {
    int id;
    char nickname[128];
    char link_url[512];
    char avatar_url[512];
    char motto[256];
    int sort_order;
    bool is_enabled;
    int64_t created_at;
    int64_t updated_at;
} FriendLink;

typedef struct {
    int id;
    char title[128];
    char artist[128];
    char file_path[512];
    char file_name[256];
    char duration[16];
    char cover_image[512];
    int64_t created_at;
} MusicTrack;

/* === HTTP 相关类型 === */

typedef enum {
    HTTP_GET,
    HTTP_POST,
    HTTP_PUT,
    HTTP_DELETE,
    HTTP_PATCH,
    HTTP_OPTIONS,
    HTTP_HEAD
} HttpMethod;

typedef struct {
    char key[128];
    char value[512];
} HttpHeader;

typedef struct {
    char name[128];
    char value[1024];
} HttpParam;

typedef struct {
    HttpMethod method;
    char path[512];
    char query[512];
    HttpHeader headers[32];
    int header_count;
    HttpParam params[32];
    int param_count;
    char body[8192];
    int body_len;
    char *data;  // 解析后的 JSON 数据
    int data_len;
} HttpRequest;

typedef struct {
    int status_code;
    char status_text[64];
    char content_type[128];
    char *body;
    int body_len;
    HttpHeader headers[32];
    int header_count;
} HttpResponse;

/* === 服务器配置 === */

typedef struct {
    char host[64];
    int port;
    int max_connections;
    int timeout;
} ServerConfig;

typedef struct {
    char path[512];
    int cache_size;
    int page_size;
} DatabaseConfig;

typedef struct {
    char cert_path[512];
    char key_path[512];
    bool enabled;
} TlsConfig;

typedef struct {
    ServerConfig server;
    DatabaseConfig database;
    TlsConfig tls;
} Config;

/* === 连接状态 === */

typedef enum {
    CONN_STATE_ACCEPTING,
    CONN_STATE_HANDSHAKE,
    CONN_STATE_READING,
    CONN_STATE_WRITING,
    CONN_STATE_CLOSING,
    CONN_STATE_CLOSED
} ConnectionState;

/* === 响应宏 === */

#define HTTP_STATUS_OK 200
#define HTTP_STATUS_CREATED 201
#define HTTP_STATUS_BAD_REQUEST 400
#define HTTP_STATUS_UNAUTHORIZED 401
#define HTTP_STATUS_FORBIDDEN 403
#define HTTP_STATUS_NOT_FOUND 404
#define HTTP_STATUS_METHOD_NOT_ALLOWED 405
#define HTTP_STATUS_INTERNAL_SERVER_ERROR 500

/* === 工具宏 === */

#define ARRAY_SIZE(arr) (sizeof(arr) / sizeof((arr)[0]))
#define MIN(a, b) ((a) < (b) ? (a) : (b))
#define MAX(a, b) ((a) > (b) ? (a) : (b))

/* === 时间相关 === */

static inline int64_t current_timestamp(void) {
    return (int64_t)time(NULL);
}

static inline void timestamp_to_string(int64_t ts, char *buf, size_t len) {
    time_t t = (time_t)ts;
    struct tm *tm_info = localtime(&t);
    strftime(buf, len, "%Y-%m-%d %H:%M:%S", tm_info);
}

#endif /* TYPES_H */