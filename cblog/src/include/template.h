#ifndef TEMPLATE_H
#define TEMPLATE_H

#include "types.h"
#include <stdbool.h>

/* === 模板变量 === */

typedef struct {
    char key[128];
    char value[4096];
} TemplateVar;

/* === 模板上下文 === */

typedef struct {
    TemplateVar vars[64];
    int var_count;
    bool enable_cache;
} TemplateContext;

/* === 模板 API === */

/**
 * 初始化模板引擎
 */
void template_init(void);

/**
 * 创建模板上下文
 */
TemplateContext *template_context_create(void);

/**
 * 销毁模板上下文
 */
void template_context_destroy(TemplateContext *ctx);

/**
 * 设置模板变量
 */
void template_set_var(TemplateContext *ctx, const char *key, const char *value);

/**
 * 获取模板变量
 */
const char *template_get_var(TemplateContext *ctx, const char *key);

/**
 * 渲染模板
 */
char *template_render(TemplateContext *ctx, const char *template_name);

/**
 * 渲染模板字符串
 */
char *template_render_string(TemplateContext *ctx, const char *template_str);

/**
 * 清除模板缓存
 */
void template_clear_cache(void);

/**
 * 添加模板路径
 */
void template_add_path(const char *path);

/* === 模板函数 === */

/**
 * 包含其他模板
 */
char *template_include(const char *template_name);

/**
 * 格式化时间
 */
char *template_format_time(int64_t timestamp, const char *format);

/**
 * 截断文本
 */
char *template_truncate(const char *text, int max_len, const char *ellipsis);

/**
 * HTML 转义
 */
char *template_escape_html(const char *text);

/**
 * 转换换行符为 <br>
 */
char *template_nl2br(const char *text);

/**
 * URL 编码
 */
char *template_url_encode(const char *text);

/**
 * URL 解码
 */
char *template_url_decode(const char *text);

/* === 模板助手函数 === */

/**
 * 生成 CSRF Token
 */
char *template_csrf_token(void);

/**
 * 生成随机数
 */
int template_rand(int min, int max);

/**
 * 判断是否相等
 */
bool template_eq(const char *a, const char *b);

/**
 * 判断是否包含
 */
bool template_contains(const char *haystack, const char *needle);

#endif /* TEMPLATE_H */