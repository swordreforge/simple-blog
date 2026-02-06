/* template.c - 模板引擎占位 */
#include "include/template.h"
#include "include/crypto.h"
#include "include/common.h"
#include <ctype.h>

void template_init(void) {
    LOG_INFO("模板引擎初始化");
}

TemplateContext *template_context_create(void) {
    TemplateContext *ctx = SAFE_CALLOC(1, sizeof(TemplateContext));
    return ctx;
}

void template_context_destroy(TemplateContext *ctx) {
    if (ctx) {
        free(ctx);
    }
}

void template_set_var(TemplateContext *ctx, const char *key, const char *value) {
    if (!ctx || !key) return;
    
    for (int i = 0; i < ctx->var_count; i++) {
        if (strcmp(ctx->vars[i].key, key) == 0) {
            STR_COPY(ctx->vars[i].value, value);
            return;
        }
    }
    
    if (ctx->var_count < 64) {
        STR_COPY(ctx->vars[ctx->var_count].key, key);
        STR_COPY(ctx->vars[ctx->var_count].value, value);
        ctx->var_count++;
    }
}

const char *template_get_var(TemplateContext *ctx, const char *key) {
    if (!ctx || !key) return NULL;
    
    for (int i = 0; i < ctx->var_count; i++) {
        if (strcmp(ctx->vars[i].key, key) == 0) {
            return ctx->vars[i].value;
        }
    }
    return NULL;
}

char *template_render(TemplateContext *ctx, const char *template_name) {
    /* 构建模板文件路径 */
    char template_path[512];
    snprintf(template_path, sizeof(template_path), "templates/%s.html", template_name);
    
    /* 读取模板文件 */
    FILE *fp = fopen(template_path, "r");
    if (!fp) {
        LOG_ERROR("无法打开模板文件: %s", template_path);
        char *error = SAFE_MALLOC(256);
        snprintf(error, 256, "<h1>模板加载失败</h1><p>无法找到模板: %s</p>", template_name);
        return error;
    }
    
    /* 获取文件大小 */
    fseek(fp, 0, SEEK_END);
    long file_size = ftell(fp);
    fseek(fp, 0, SEEK_SET);
    
    /* 读取文件内容 */
    char *template_content = SAFE_MALLOC(file_size + 1);
    size_t read_size = fread(template_content, 1, file_size, fp);
    template_content[read_size] = '\0';
    fclose(fp);
    
    /* 替换变量 */
    char *result = template_render_string(ctx, template_content);
    
    free(template_content);
    return result;
}

char *template_render_string(TemplateContext *ctx, const char *template_str) {
    if (!template_str) {
        char *result = SAFE_MALLOC(1);
        *result = '\0';
        return result;
    }
    
    /* 计算最大可能的大小 */
    size_t max_size = strlen(template_str) * 2;
    char *result = SAFE_MALLOC(max_size);
    size_t result_len = 0;
    
    const char *p = template_str;
    while (*p) {
        if (*p == '{' && *(p + 1) == '{') {
            /* 找到变量占位符的开始 */
            p += 2;
            if (!*p) break;
            
            /* 跳过空白字符 */
            while (*p && isspace(*p)) p++;
            if (!*p) break;
            
            /* 提取变量名 */
            char var_name[128] = {0};
            size_t var_len = 0;
            while (*p && *p != '}' && !isspace(*p) && var_len < sizeof(var_name) - 1) {
                var_name[var_len++] = *p++;
            }
            var_name[var_len] = '\0';
            
            /* 跳过到 }} */
            while (*p && *p != '}') p++;
            if (*p == '}') p++;
            if (*p == '}') p++;
            if (!*p) break;
            
            /* 获取变量值 */
            const char *var_value = template_get_var(ctx, var_name);
            if (var_value) {
                size_t value_len = strlen(var_value);
                /* 确保有足够的空间 */
                if (result_len + value_len >= max_size) {
                    max_size *= 2;
                    result = SAFE_REALLOC(result, max_size);
                }
                strcpy(result + result_len, var_value);
                result_len += value_len;
            }
        } else {
            /* 复制普通字符 */
            if (result_len >= max_size) {
                max_size *= 2;
                result = SAFE_REALLOC(result, max_size);
            }
            result[result_len++] = *p++;
        }
    }
    
    result[result_len] = '\0';
    return result;
}

void template_clear_cache(void) {
    /* 清除模板缓存 */
}

void template_add_path(const char *path) {
    /* 添加模板路径 */
}

char *template_include(const char *template_name) {
    char *html = SAFE_MALLOC(256);
    snprintf(html, 256, "<!-- Include: %s -->", template_name);
    return html;
}

char *template_format_time(int64_t timestamp, const char *format) {
    char *result = SAFE_MALLOC(64);
    time_t t = (time_t)timestamp;
    struct tm *tm_info = localtime(&t);
    strftime(result, 64, format ? format : "%Y-%m-%d %H:%M:%S", tm_info);
    return result;
}

char *template_truncate(const char *text, int max_len, const char *ellipsis) {
    if (!text) {
        char *result = SAFE_MALLOC(1);
        *result = '\0';
        return result;
    }
    
    size_t len = strlen(text);
    if (len <= (size_t)max_len) {
        char *result = SAFE_MALLOC(len + 1);
        strcpy(result, text);
        return result;
    }
    
    char *result = SAFE_MALLOC(max_len + strlen(ellipsis) + 1);
    strncpy(result, text, max_len);
    result[max_len] = '\0';
    strcat(result, ellipsis ? ellipsis : "...");
    return result;
}

char *template_escape_html(const char *text) {
    if (!text) {
        char *result = SAFE_MALLOC(1);
        *result = '\0';
        return result;
    }
    
    /* 简化版：只处理基本转义 */
    char *result = SAFE_MALLOC(strlen(text) * 6 + 1);
    char *p = result;
    
    for (const char *s = text; *s; s++) {
        switch (*s) {
            case '<':
                strcpy(p, "&lt;"); p += 4; break;
            case '>':
                strcpy(p, "&gt;"); p += 4; break;
            case '&':
                strcpy(p, "&amp;"); p += 5; break;
            case '"':
                strcpy(p, "&quot;"); p += 6; break;
            case '\'':
                strcpy(p, "&#39;"); p += 5; break;
            default:
                *p++ = *s; break;
        }
    }
    *p = '\0';
    
    return result;
}

char *template_nl2br(const char *text) {
    if (!text) {
        char *result = SAFE_MALLOC(1);
        *result = '\0';
        return result;
    }
    
    /* 计算 <br> 数量 */
    int br_count = 0;
    for (const char *s = text; *s; s++) {
        if (*s == '\n') br_count++;
    }
    
    char *result = SAFE_MALLOC(strlen(text) + br_count * 3 + 1);
    char *p = result;
    
    for (const char *s = text; *s; s++) {
        if (*s == '\n') {
            strcpy(p, "<br>"); p += 4;
        } else {
            *p++ = *s;
        }
    }
    *p = '\0';
    
    return result;
}

char *template_url_encode(const char *text) {
    if (!text) {
        char *result = SAFE_MALLOC(1);
        *result = '\0';
        return result;
    }
    
    char *result = SAFE_MALLOC(strlen(text) * 3 + 1);
    char *p = result;
    
    for (const char *s = text; *s; s++) {
        if (isalnum(*s) || *s == '-' || *s == '_' || *s == '.' || *s == '~') {
            *p++ = *s;
        } else if (*s == ' ') {
            *p++ = '+';
        } else {
            sprintf(p, "%%%02X", (unsigned char)*s);
            p += 3;
        }
    }
    *p = '\0';
    
    return result;
}

char *template_url_decode(const char *text) {
    if (!text) {
        char *result = SAFE_MALLOC(1);
        *result = '\0';
        return result;
    }
    
    char *result = SAFE_MALLOC(strlen(text) + 1);
    char *p = result;
    
    for (const char *s = text; *s; s++) {
        if (*s == '%') {
            if (s[1] && s[2]) {
                char hex[3] = {s[1], s[2], 0};
                *p++ = (char)strtol(hex, NULL, 16);
                s += 2;
            }
        } else if (*s == '+') {
            *p++ = ' ';
        } else {
            *p++ = *s;
        }
    }
    *p = '\0';
    
    return result;
}

char *template_csrf_token(void) {
    char *token = SAFE_MALLOC(33);
    crypto_random_string(token, 33);
    return token;
}

int template_rand(int min, int max) {
    uint8_t r;
    crypto_random_bytes(&r, 1);
    return min + (r % (max - min + 1));
}

bool template_eq(const char *a, const char *b) {
    if (!a || !b) return false;
    return strcmp(a, b) == 0;
}

bool template_contains(const char *haystack, const char *needle) {
    if (!haystack || !needle) return false;
    return strstr(haystack, needle) != NULL;
}