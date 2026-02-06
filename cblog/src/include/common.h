#ifndef COMMON_H
#define COMMON_H

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <errno.h>
#include <time.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <sys/select.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <fcntl.h>
#include <signal.h>

/* === 日志级别 === */

typedef enum {
    LOG_DEBUG = 0,
    LOG_INFO,
    LOG_WARN,
    LOG_ERROR,
    LOG_FATAL
} LogLevel;

/* === 日志函数 === */

void log_set_level(LogLevel level);
void log_printf(LogLevel level, const char *format, ...);

#define LOG_DEBUG(...) log_printf(LOG_DEBUG, __VA_ARGS__)
#define LOG_INFO(...)  log_printf(LOG_INFO, __VA_ARGS__)
#define LOG_WARN(...)  log_printf(LOG_WARN, __VA_ARGS__)
#define LOG_ERROR(...) log_printf(LOG_ERROR, __VA_ARGS__)
#define LOG_FATAL(...) log_printf(LOG_FATAL, __VA_ARGS__)

/* === 错误处理 === */

#define CHECK_ERROR(cond, msg) \
    do { \
        if (cond) { \
            LOG_ERROR("%s: %s (errno=%d)", msg, strerror(errno), errno); \
            return -1; \
        } \
    } while(0)

#define CHECK_NULL(ptr, msg) \
    do { \
        if ((ptr) == NULL) { \
            LOG_ERROR("%s", msg); \
            return -1; \
        } \
    } while(0)

/* === 内存管理 === */

#define SAFE_MALLOC(size) \
    ({ \
        void *ptr = malloc(size); \
        if (ptr == NULL) { \
            LOG_ERROR("内存分配失败: %zu bytes", size); \
            exit(1); \
        } \
        ptr; \
    })

#define SAFE_REALLOC(ptr, size) \
    ({ \
        void *new_ptr = realloc(ptr, size); \
        if (new_ptr == NULL) { \
            LOG_ERROR("内存重新分配失败: %zu bytes", size); \
            exit(1); \
        } \
        new_ptr; \
    })

#define SAFE_FREE(ptr) \
    do { \
        if (ptr != NULL) { \
            free(ptr); \
            (ptr) = NULL; \
        } \
    } while(0)

#define SAFE_CALLOC(nmemb, size) \
    ({ \
        void *ptr = calloc(nmemb, size); \
        if (ptr == NULL) { \
            LOG_ERROR("内存分配失败: %zu bytes", (size_t)(nmemb) * (size)); \
            exit(1); \
        } \
        ptr; \
    })

/* === 字符串操作 === */

#define STR_COPY(dst, src) \
    do { \
        strncpy(dst, src, sizeof(dst) - 1); \
        dst[sizeof(dst) - 1] = '\0'; \
    } while(0)

#define STR_EQUAL(a, b) (strcmp(a, b) == 0)
#define STR_EQUAL_NOCASE(a, b) (strcasecmp(a, b) == 0)
#define STR_STARTS_WITH(str, prefix) (strncmp(str, prefix, strlen(prefix)) == 0)

/* === 时间操作 === */

#define NOW() ((int64_t)time(NULL))
#define SECONDS_PER_DAY 86400
#define SECONDS_PER_HOUR 3600
#define SECONDS_PER_MINUTE 60

/* === 文件操作 === */

#define FILE_EXISTS(path) (access(path, F_OK) == 0)
#define FILE_READABLE(path) (access(path, R_OK) == 0)
#define FILE_WRITABLE(path) (access(path, W_OK) == 0)

/* === 网络操作 === */

#define INVALID_SOCKET -1

/* === 配置 === */

#define MAX_PATH 512
#define MAX_URL 1024
#define MAX_BODY_SIZE 8192
#define MAX_RESPONSE_SIZE 32768

#endif /* COMMON_H */