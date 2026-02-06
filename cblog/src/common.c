/* common.c - 公共工具函数 */
#include "include/common.h"
#include <stdio.h>
#include <stdarg.h>
#include <time.h>

static LogLevel g_log_level = LOG_INFO;

/* === 设置日志级别 === */

void log_set_level(LogLevel level) {
    g_log_level = level;
}

/* === 日志打印 === */

void log_printf(LogLevel level, const char *format, ...) {
    if (level < g_log_level) return;
    
    time_t now = time(NULL);
    struct tm *tm_info = localtime(&now);
    char time_buf[32];
    strftime(time_buf, sizeof(time_buf), "%Y-%m-%d %H:%M:%S", tm_info);
    
    const char *level_str = "???";
    switch (level) {
        case LOG_DEBUG: level_str = "DEBUG"; break;
        case LOG_INFO:  level_str = "INFO "; break;
        case LOG_WARN:  level_str = "WARN "; break;
        case LOG_ERROR: level_str = "ERROR"; break;
        case LOG_FATAL: level_str = "FATAL"; break;
    }
    
    fprintf(stderr, "[%s] [%s] ", time_buf, level_str);
    
    va_list args;
    va_start(args, format);
    vfprintf(stderr, format, args);
    va_end(args);
    
    fprintf(stderr, "\n");
    fflush(stderr);
}
