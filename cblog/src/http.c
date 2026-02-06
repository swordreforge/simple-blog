/* http.c - HTTP 协议处理 */
#include "include/http.h"
#include "include/common.h"
#include <string.h>
#include <stdlib.h>
#include <ctype.h>

/* === HTTP 方法字符串 === */

static const char *method_strings[] = {
    "GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS", "HEAD"
};

/* === MIME 类型映射 === */

static const struct {
    const char *ext;
    const char *mime;
} mime_types[] = {
    {".html", "text/html; charset=utf-8"},
    {".htm",  "text/html; charset=utf-8"},
    {".css",  "text/css; charset=utf-8"},
    {".js",   "application/javascript; charset=utf-8"},
    {".json", "application/json; charset=utf-8"},
    {".png",  "image/png"},
    {".jpg",  "image/jpeg"},
    {".jpeg", "image/jpeg"},
    {".gif",  "image/gif"},
    {".svg",  "image/svg+xml"},
    {".ico",  "image/x-icon"},
    {".webp", "image/webp"},
    {".woff", "font/woff"},
    {".woff2","font/woff2"},
    {".ttf",  "font/ttf"},
    {".eot",  "application/vnd.ms-fontobject"},
    {".mp3",  "audio/mpeg"},
    {".mp4",  "video/mp4"},
    {".webm", "video/webm"},
    {".pdf",  "application/pdf"},
    {".txt",  "text/plain; charset=utf-8"},
    {".md",   "text/markdown; charset=utf-8"},
    {NULL,    "application/octet-stream"}
};

/* === 解析 HTTP 方法 === */

HttpMethod http_parse_method(const char *method_str) {
    for (int i = 0; i < sizeof(method_strings) / sizeof(method_strings[0]); i++) {
        if (strcasecmp(method_str, method_strings[i]) == 0) {
            return (HttpMethod)i;
        }
    }
    return HTTP_GET;  /* 默认 */
}

/* === 跳过空白字符 === */

static const char *skip_whitespace(const char *p) {
    while (*p && isspace(*p)) p++;
    return p;
}

/* === 跳过非空白字符 === */

static const char *skip_until_whitespace(const char *p) {
    while (*p && !isspace(*p)) p++;
    return p;
}

/* === 解析 HTTP 请求 === */

int http_parse_request(const char *data, size_t len, HttpRequest *req) {
    if (!data || len == 0 || !req) return -1;
    
    memset(req, 0, sizeof(HttpRequest));
    
    const char *p = data;
    const char *end = data + len;
    
    /* 解析请求行: METHOD PATH VERSION */
    p = skip_whitespace(p);
    
    /* 方法 */
    const char *method_start = p;
    p = skip_until_whitespace(p);
    if (p >= end) return -1;
    
    char method_str[16];
    size_t method_len = p - method_start;
    if (method_len >= sizeof(method_str)) return -1;
    strncpy(method_str, method_start, method_len);
    method_str[method_len] = '\0';
    req->method = http_parse_method(method_str);
    
    /* 路径 */
    p = skip_whitespace(p);
    const char *path_start = p;
    while (*p && !isspace(*p) && *p != '?') p++;
    if (p >= end) return -1;
    
    size_t path_len = p - path_start;
    if (path_len >= sizeof(req->path)) return -1;
    strncpy(req->path, path_start, path_len);
    req->path[path_len] = '\0';
    
    /* 查询参数 */
    if (*p == '?') {
        p++;
        const char *query_start = p;
        while (*p && !isspace(*p)) p++;
        size_t query_len = p - query_start;
        if (query_len > 0 && query_len < sizeof(req->query)) {
            strncpy(req->query, query_start, query_len);
            req->query[query_len] = '\0';
            http_parse_params(req->query, req);
        }
    }
    
    /* 跳过版本号 */
    p = skip_whitespace(p);
    p = skip_until_whitespace(p);
    p = skip_whitespace(p);
    
    /* 解析头部 */
    p = skip_whitespace(p);
    while (*p && *p != '\r' && *p != '\n') {
        const char *key_start = p;
        while (*p && *p != ':' && *p != '\r' && *p != '\n') p++;
        if (*p != ':') break;
        
        size_t key_len = p - key_start;
        p++;  /* 跳过 ':' */
        p = skip_whitespace(p);
        
        const char *value_start = p;
        while (*p && *p != '\r' && *p != '\n') p++;
        size_t value_len = p - value_start;
        
        /* 去除尾部空白 */
        while (value_len > 0 && isspace(value_start[value_len - 1])) {
            value_len--;
        }
        
        /* 存储头部 */
        if (req->header_count < 32 && key_len < 128 && value_len < 512) {
            strncpy(req->headers[req->header_count].key, key_start, key_len);
            req->headers[req->header_count].key[key_len] = '\0';
            strncpy(req->headers[req->header_count].value, value_start, value_len);
            req->headers[req->header_count].value[value_len] = '\0';
            req->header_count++;
        }
        
        /* 跳过 CRLF */
        if (*p == '\r') p++;
        if (*p == '\n') p++;
    }
    
    /* 跳过空行 */
    if (*p == '\r') p++;
    if (*p == '\n') p++;
    
    /* 解析请求体 */
    if (p < end) {
        size_t body_len = end - p;
        if (body_len < sizeof(req->body)) {
            memcpy(req->body, p, body_len);
            req->body[body_len] = '\0';
            req->body_len = body_len;
            
            /* 尝试解析 JSON */
            http_parse_json_body(req);
        }
    }
    
    return 0;
}

/* === 解析 URL 参数 === */

int http_parse_params(const char *query, HttpRequest *req) {
    if (!query || !req) return -1;
    
    const char *p = query;
    char param_name[128];
    char param_value[512];
    
    while (*p) {
        /* 参数名 */
        const char *name_start = p;
        while (*p && *p != '=' && *p != '&') p++;
        size_t name_len = p - name_start;
        
        if (*p == '=') {
            p++;
            const char *value_start = p;
            while (*p && *p != '&') p++;
            size_t value_len = p - value_start;
            
            if (req->param_count < 32) {
                http_url_decode(name_start, param_name, sizeof(param_name));
                http_url_decode(value_start, param_value, sizeof(param_value));
                
                strncpy(req->params[req->param_count].name, param_name, 127);
                strncpy(req->params[req->param_count].value, param_value, 511);
                req->params[req->param_count].name[127] = '\0';
                req->params[req->param_count].value[511] = '\0';
                req->param_count++;
            }
        }
        
        if (*p == '&') p++;
    }
    
    return 0;
}

/* === 解析 JSON 请求体 === */

int http_parse_json_body(HttpRequest *req) {
    /* 简化版：只是存储原始 JSON */
    /* 实际项目中应该使用真正的 JSON 解析器 */
    if (req->body_len > 0 && req->body_len < sizeof(req->body)) {
        req->data = req->body;
        req->data_len = req->body_len;
    }
    return 0;
}

/* === 获取指定头部 === */

const char *http_get_header(const HttpRequest *req, const char *name) {
    if (!req || !name) return NULL;
    
    for (int i = 0; i < req->header_count; i++) {
        if (strcasecmp(req->headers[i].key, name) == 0) {
            return req->headers[i].value;
        }
    }
    return NULL;
}

/* === 获取指定参数 === */

const char *http_get_param(const HttpRequest *req, const char *name) {
    if (!req || !name) return NULL;
    
    for (int i = 0; i < req->param_count; i++) {
        if (strcmp(req->params[i].name, name) == 0) {
            return req->params[i].value;
        }
    }
    return NULL;
}

/* === URL 解码 === */

int http_url_decode(const char *src, char *dst, size_t dst_len) {
    if (!src || !dst) return -1;
    
    char *d = dst;
    const char *s = src;
    size_t remaining = dst_len - 1;
    
    while (*s && remaining > 0) {
        if (*s == '%') {
            if (s[1] && s[2]) {
                char hex[3] = {s[1], s[2], 0};
                *d++ = (char)strtol(hex, NULL, 16);
                s += 3;
                remaining--;
            } else {
                s++;
            }
        } else if (*s == '+') {
            *d++ = ' ';
            s++;
            remaining--;
        } else {
            *d++ = *s++;
            remaining--;
        }
    }
    *d = '\0';
    return 0;
}

/* === URL 编码 === */

int http_url_encode(const char *src, char *dst, size_t dst_len) {
    if (!src || !dst) return -1;
    
    char *d = dst;
    const char *s = src;
    size_t remaining = dst_len - 1;
    
    while (*s && remaining > 0) {
        if (isalnum(*s) || *s == '-' || *s == '_' || *s == '.' || *s == '~') {
            *d++ = *s++;
            remaining--;
        } else if (*s == ' ') {
            *d++ = '+';
            s++;
            remaining--;
        } else if (remaining >= 3) {
            snprintf(d, 4, "%%%02X", (unsigned char)*s);
            d += 3;
            s++;
            remaining -= 3;
        } else {
            break;
        }
    }
    *d = '\0';
    return 0;
}

/* === Base64 编码 === */

static const char base64_table[] = 
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

int http_base64_encode(const uint8_t *src, size_t src_len, char *dst, size_t dst_len) {
    if (!src || !dst) return -1;
    
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

int http_base64_decode(const char *src, uint8_t *dst, size_t *dst_len) {
    if (!src || !dst || !dst_len) return -1;
    
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

/* === 获取文件 MIME 类型 === */

const char *http_get_mime_type(const char *path) {
    if (!path) return "application/octet-stream";
    
    const char *ext = strrchr(path, '.');
    if (!ext) return "application/octet-stream";
    
    for (int i = 0; mime_types[i].ext; i++) {
        if (strcasecmp(ext, mime_types[i].ext) == 0) {
            return mime_types[i].mime;
        }
    }
    
    return "application/octet-stream";
}

/* === 判断是否为静态文件请求 === */

bool http_is_static_request(const char *path) {
    if (!path) return false;
    
    /* 检查是否以 /static/ 开头 */
    if (STR_STARTS_WITH(path, "/static/")) return true;
    
    /* 检查常见静态文件扩展名 */
    const char *ext = strrchr(path, '.');
    if (ext) {
        const char *static_exts[] = {".css", ".js", ".png", ".jpg", ".jpeg", 
                                     ".gif", ".svg", ".ico", ".woff", ".woff2",
                                     ".ttf", ".eot", ".webp", NULL};
        for (int i = 0; static_exts[i]; i++) {
            if (strcasecmp(ext, static_exts[i]) == 0) {
                return true;
            }
        }
    }
    
    return false;
}

/* === 创建 HTTP 响应 === */

void http_response_init(HttpResponse *resp, int status_code) {
    memset(resp, 0, sizeof(HttpResponse));
    resp->status_code = status_code;
    
    /* 设置默认状态文本 */
    switch (status_code) {
        case 200: STR_COPY(resp->status_text, "OK"); break;
        case 201: STR_COPY(resp->status_text, "Created"); break;
        case 400: STR_COPY(resp->status_text, "Bad Request"); break;
        case 401: STR_COPY(resp->status_text, "Unauthorized"); break;
        case 403: STR_COPY(resp->status_text, "Forbidden"); break;
        case 404: STR_COPY(resp->status_text, "Not Found"); break;
        case 405: STR_COPY(resp->status_text, "Method Not Allowed"); break;
        case 500: STR_COPY(resp->status_text, "Internal Server Error"); break;
        default:  snprintf(resp->status_text, sizeof(resp->status_text), "%d", status_code); break;
    }
    
    /* 设置默认 Content-Type */
    STR_COPY(resp->content_type, "text/html; charset=utf-8");
}

/* === 设置响应头部 === */

void http_set_header(HttpResponse *resp, const char *name, const char *value) {
    if (!resp || !name || !value) return;
    if (resp->header_count >= 32) return;
    
    strncpy(resp->headers[resp->header_count].key, name, 127);
    resp->headers[resp->header_count].key[127] = '\0';
    strncpy(resp->headers[resp->header_count].value, value, 511);
    resp->headers[resp->header_count].value[511] = '\0';
    resp->header_count++;
}

/* === 设置响应体 === */

void http_set_body(HttpResponse *resp, const char *body, int len) {
    if (!resp) return;
    resp->body = (char*)body;
    resp->body_len = len;
}

/* === 构建 HTTP 响应字符串 === */

int http_build_response(const HttpResponse *resp, char *buf, size_t buf_size) {
    if (!resp || !buf) return -1;
    
    char *p = buf;
    size_t remaining = buf_size;
    
    /* 状态行 */
    int len = snprintf(p, remaining, "HTTP/1.1 %d %s\r\n", 
                       resp->status_code, resp->status_text);
    if (len < 0 || (size_t)len >= remaining) return -1;
    p += len;
    remaining -= len;
    
    /* Content-Type */
    len = snprintf(p, remaining, "Content-Type: %s\r\n", resp->content_type);
    if (len < 0 || (size_t)len >= remaining) return -1;
    p += len;
    remaining -= len;
    
    /* Content-Length */
    if (resp->body) {
        len = snprintf(p, remaining, "Content-Length: %d\r\n", resp->body_len);
        if (len < 0 || (size_t)len >= remaining) return -1;
        p += len;
        remaining -= len;
    }
    
    /* 自定义头部 */
    for (int i = 0; i < resp->header_count; i++) {
        len = snprintf(p, remaining, "%s: %s\r\n", 
                       resp->headers[i].key, resp->headers[i].value);
        if (len < 0 || (size_t)len >= remaining) return -1;
        p += len;
        remaining -= len;
    }
    
    /* 空行 */
    len = snprintf(p, remaining, "\r\n");
    if (len < 0 || (size_t)len >= remaining) return -1;
    p += len;
    remaining -= len;
    
    /* 响应体 */
    if (resp->body && resp->body_len > 0) {
        if ((size_t)resp->body_len >= remaining) return -1;
        memcpy(p, resp->body, resp->body_len);
        p += resp->body_len;
    }
    
    return p - buf;
}

/* === 构建 JSON 响应 === */

char *http_build_json_response(bool success, const char *message, const char *data) {
    char buf[MAX_RESPONSE_SIZE];
    int len;
    
    if (data) {
        len = snprintf(buf, sizeof(buf), 
                      "{\"success\":%s,\"message\":\"%s\",\"data\":%s}",
                      success ? "true" : "false", message, data);
    } else {
        len = snprintf(buf, sizeof(buf), 
                      "{\"success\":%s,\"message\":\"%s\"}",
                      success ? "true" : "false", message);
    }
    
    if (len >= (int)sizeof(buf)) return NULL;
    
    char *result = SAFE_MALLOC(len + 1);
    strcpy(result, buf);
    return result;
}

/* === 构建错误响应 === */

char *http_build_error_response(int status, const char *message) {
    return http_build_json_response(false, message, NULL);
}