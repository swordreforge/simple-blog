/* main.c - RustBlog C语言重构版本 - 主入口 */
#include "include/server.h"
#include "include/router.h"
#include "include/database.h"
#include "include/common.h"
#include <signal.h>
#include <getopt.h>

static Server g_server;
static bool g_running = true;

/* === 信号处理 === */

void signal_handler(int sig) {
    if (sig == SIGINT || sig == SIGTERM) {
        LOG_INFO("收到停止信号，正在关闭服务器...");
        g_running = false;
        server_stop(&g_server);
    }
}

/* === 打印用法 === */

void print_usage(const char *prog_name) {
    printf("用法: %s [选项]\n", prog_name);
    printf("\n选项:\n");
    printf("  -c, --config FILE    配置文件路径 (默认: config.json)\n");
    printf("  -i, --init-db        初始化数据库并退出\n");
    printf("  -v, --version        显示版本信息\n");
    printf("  -h, --help           显示帮助信息\n");
    printf("\n示例:\n");
    printf("  %s -c config.json\n", prog_name);
    printf("  %s --init-db\n", prog_name);
}

/* === 打印版本 === */

void print_version(void) {
    printf("RustBlog C语言版本 v1.0.0\n");
    printf("使用 BearSSL + select 模型\n");
    printf("目标: 250-500KB 二进制大小\n");
}

/* === 主函数 === */

int main(int argc, char *argv[]) {
    const char *config_path = "config.json";
    bool init_db_only = false;
    
    /* 解析命令行参数 */
    static struct option long_options[] = {
        {"config",   required_argument, 0, 'c'},
        {"init-db",  no_argument,       0, 'i'},
        {"version",  no_argument,       0, 'v'},
        {"help",     no_argument,       0, 'h'},
        {0, 0, 0, 0}
    };
    
    int opt;
    while ((opt = getopt_long(argc, argv, "c:ivh", long_options, NULL)) != -1) {
        switch (opt) {
            case 'c':
                config_path = optarg;
                break;
            case 'i':
                init_db_only = true;
                break;
            case 'v':
                print_version();
                return 0;
            case 'h':
                print_usage(argv[0]);
                return 0;
            default:
                print_usage(argv[0]);
                return 1;
        }
    }
    
    /* 设置信号处理 */
    signal(SIGINT, signal_handler);
    signal(SIGTERM, signal_handler);
    signal(SIGPIPE, SIG_IGN);
    
    /* 初始化日志 */
    log_set_level(LOG_INFO);
    LOG_INFO("========================================");
    LOG_INFO("RustBlog C语言版本启动中...");
    LOG_INFO("========================================");
    
    /* 初始化服务器 */
    LOG_INFO("加载配置文件: %s", config_path);
    if (server_init(&g_server, config_path) < 0) {
        LOG_FATAL("服务器初始化失败");
        return 1;
    }
    
    /* 初始化路由 */
    LOG_INFO("初始化路由系统...");
    router_init();
    
    /* 如果只初始化数据库 */
    if (init_db_only) {
        LOG_INFO("初始化数据库...");
        if (database_create_tables(g_server.db) < 0) {
            LOG_ERROR("数据库初始化失败");
            return 1;
        }
        if (database_insert_default_data(g_server.db) < 0) {
            LOG_WARN("插入默认数据失败（可能已存在）");
        }
        LOG_INFO("数据库初始化完成");
        return 0;
    }
    
    /* 打印启动信息 */
    printf("\n");
    printf("╔════════════════════════════════════════╗\n");
    printf("║       RustBlog C语言版本 v1.0.0       ║\n");
    printf("╠════════════════════════════════════════╣\n");
    printf("║  服务器地址: https://%s:%d           ║\n", 
           g_server.config.server.host, g_server.config.server.port);
    printf("║  数据库路径: %-25s ║\n", g_server.config.database.path);
    printf("║  最大连接数: %-25d ║\n", g_server.config.server.max_connections);
    printf("║  TLS 启用: %-28s ║\n", 
           g_server.config.tls.enabled ? "是" : "否");
    printf("╚════════════════════════════════════════╝\n");
    printf("\n");
    
    /* 运行服务器 */
    LOG_INFO("服务器运行中...");
    if (server_run(&g_server) < 0) {
        LOG_ERROR("服务器运行出错");
        return 1;
    }
    
    /* 清理资源 */
    LOG_INFO("清理资源...");
    server_cleanup(&g_server);
    
    LOG_INFO("服务器已停止");
    return 0;
}
