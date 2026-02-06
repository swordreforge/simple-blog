#ifndef HTTP_H
#define HTTP_H

#include "types.h"

/* === HTTP 请求解析 === */

/**
 * 解析 HTTP 请求
 */
int http_parse_request(const char *data, size_t len, HttpRequest *req);

/**
 * 解析 HTTP 方法
 */
HttpMethod http_parse_method(const char *method_str);

/**
 * 解析 HTTP 头部
 */
int http_parse_headers(const char *data, HttpRequest *req);

/**
 * 解析 URL 参数
 */
int http_parse_params(const char *query, HttpRequest *req);

/**
 * 解析 JSON 请求体
 */
int http_parse_json_body(HttpRequest *req);

/**
 * 获取指定头部
 */
const char *http_get_header(const HttpRequest *req, const char *name);

/**
 * 获取指定参数
 */
const char *http_get_param(const HttpRequest *req, const char *name);

/* === HTTP 响应构建 === */

/**
 * 创建 HTTP 响应
 */
void http_response_init(HttpResponse *resp, int status_code);

/**
 * 设置响应头部
 */
void http_set_header(HttpResponse *resp, const char *name, const char *value);

/**
 * 设置响应体
 */
void http_set_body(HttpResponse *resp, const char *body, int len);

/**
 * 构建 HTTP 响应字符串
 */
int http_build_response(const HttpResponse *resp, char *buf, size_t buf_size);

/**
 * 构建 JSON 响应
 */
char *http_build_json_response(bool success, const char *message, const char *data);

/**
 * 构建错误响应
 */
char *http_build_error_response(int status, const char *message);

/* === HTTP 工具函数 === */

/**
 * URL 解码
 */
int http_url_decode(const char *src, char *dst, size_t dst_len);

/**
 * URL 编码
 */
int http_url_encode(const char *src, char *dst, size_t dst_len);

/**
 * Base64 解码
 */
int http_base64_decode(const char *src, uint8_t *dst, size_t *dst_len);

/**
 * Base64 编码
 */
int http_base64_encode(const uint8_t *src, size_t src_len, char *dst, size_t dst_len);

/**
 * 解析 Content-Type
 */
const char *http_parse_content_type(const char *content_type);

/**
 * 判断是否为静态文件请求
 */
bool http_is_static_request(const char *path);

/**
 * 获取文件 MIME 类型
 */
const char *http_get_mime_type(const char *path);

#endif /* HTTP_H */