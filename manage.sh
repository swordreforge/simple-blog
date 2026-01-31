#!/bin/bash

# rustblog 服务器管理脚本
# 用于启动或终止 rustblog 服务器

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# 检查可用内存 (80MB = 81920KB)
check_memory() {
    # 获取更准确的可用内存（包括 buffers/cache）
    # 兼容中文和英文版本的 free 命令
    local mem_line=$(free -k | grep -E "^(内存:|Mem:)")
    local available_kb=$(echo "$mem_line" | awk '{print $NF}')
    local available_mb=$((available_kb / 1024))
    local threshold_mb=80

    # 同时显示总内存和已用内存供参考
    local total_kb=$(echo "$mem_line" | awk '{print $2}')
    local total_mb=$((total_kb / 1024))
    local used_kb=$(echo "$mem_line" | awk '{print $3}')
    local used_mb=$((used_kb / 1024))

    echo -e "${CYAN}📊 内存检查:${NC}"
    echo "   总内存: ${total_mb} MB"
    echo "   已用内存: ${used_mb} MB"
    echo "   可用内存: ${available_mb} MB"

    if [ "$available_mb" -lt "$threshold_mb" ]; then
        echo -e "${RED}   ⚠️  警告: 可用内存小于 ${threshold_mb}MB!${NC}"
        echo -e "${YELLOW}   建议关闭其他程序或增加系统内存。${NC}"
        return 1
    else
        echo -e "${GREEN}   ✅ 内存充足${NC}"
        return 0
    fi
}

# 交互式配置启动参数
interactive_config() {
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}   RustBlog 服务器启动配置${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo ""

    # 端口
    read -p "🔌 监听端口 [默认: 8080]: " PORT
    PORT=${PORT:-8080}
    echo ""

    # 主机地址
    read -p "🌐 绑定地址 [默认: 127.0.0.1]: " HOST
    HOST=${HOST:-127.0.0.1}
    echo ""

    # 日志级别
    echo "📝 日志级别:"
    echo "   1) debug   - 详细调试信息"
    echo "   2) info    - 一般信息 (默认)"
    echo "   3) warn    - 警告信息"
    echo "   4) error   - 仅错误信息"
    read -p "   请选择 [1-4, 默认: 2]: " LOG_CHOICE
    case $LOG_CHOICE in
        1) LOG_LEVEL="debug" ;;
        2) LOG_LEVEL="info" ;;
        3) LOG_LEVEL="warn" ;;
        4) LOG_LEVEL="error" ;;
        *) LOG_LEVEL="info" ;;
    esac
    echo ""

    # 数据库路径
    read -p "💾 数据库路径 [默认: ./data/blog.db]: " DB_PATH
    DB_PATH=${DB_PATH:-"./data/blog.db"}
    echo ""

    # 模板目录
    read -p "📁 模板目录 [默认: templates]: " TEMPLATES_DIR
    TEMPLATES_DIR=${TEMPLATES_DIR:-"templates"}
    echo ""

    # 静态文件目录
    read -p "📁 静态文件目录 [默认: static]: " STATIC_DIR
    STATIC_DIR=${STATIC_DIR:-"static"}
    echo ""

    # GeoIP 数据库
    read -p "🌍 GeoIP 数据库路径 [默认: ./data/GeoLite2-City.mmdb]: " GEOIP_PATH
    GEOIP_PATH=${GEOIP_PATH:-"./data/GeoLite2-City.mmdb"}
    echo ""

    # TLS 配置
    read -p "🔒 是否启用 TLS/HTTPS? [y/N]: " ENABLE_TLS
    if [[ "$ENABLE_TLS" =~ ^[Yy]$ ]]; then
        read -p "   TLS 证书文件路径: " TLS_CERT
        read -p "   TLS 私钥文件路径: " TLS_KEY
        TLS_ARGS="--enable-tls --tls-cert \"$TLS_CERT\" --tls-key \"$TLS_KEY\""
    else
        TLS_ARGS=""
    fi
    echo ""

    # 模板缓存
    read -p "💾 是否禁用模板缓存? [y/N]: " DISABLE_CACHE
    if [[ "$DISABLE_CACHE" =~ ^[Yy]$ ]]; then
        CACHE_ARGS="--disable-template-cache"
    else
        CACHE_ARGS=""
    fi
    echo ""

    # 配置文件
    read -p "📄 配置文件路径 (TOML) [留空则不使用]: " CONFIG_FILE
    if [ -n "$CONFIG_FILE" ]; then
        CONFIG_ARGS="--config \"$CONFIG_FILE\""
    else
        CONFIG_ARGS=""
    fi
    echo ""

    # 后台运行
    read -p "🔄 是否后台运行? [Y/n]: " RUN_BG
    if [[ ! "$RUN_BG" =~ ^[Nn]$ ]]; then
        BG_MODE="yes"
    else
        BG_MODE="no"
    fi
    echo ""

    # 构建启动命令
    CMD="./rustblog"
    CMD="$CMD --port $PORT"
    CMD="$CMD --host $HOST"
    CMD="$CMD --log-level $LOG_LEVEL"
    CMD="$CMD --db-path \"$DB_PATH\""
    CMD="$CMD --templates-dir \"$TEMPLATES_DIR\""
    CMD="$CMD --static-dir \"$STATIC_DIR\""
    CMD="$CMD --geoip-db-path \"$GEOIP_PATH\""
    CMD="$CMD $TLS_ARGS"
    CMD="$CMD $CACHE_ARGS"
    CMD="$CMD $CONFIG_ARGS"

    # 显示配置摘要
    echo -e "${CYAN}========================================${NC}"
    echo -e "${CYAN}   配置摘要${NC}"
    echo -e "${CYAN}========================================${NC}"
    echo -e "   🌐 访问地址: http://${HOST}:${PORT}"
    echo -e "   📝 日志级别: ${LOG_LEVEL}"
    echo -e "   💾 数据库: ${DB_PATH}"
    echo -e "   📁 模板目录: ${TEMPLATES_DIR}"
    echo -e "   📁 静态目录: ${STATIC_DIR}"
    echo -e "   🔒 TLS: $([ -n "$TLS_ARGS" ] && echo "启用" || echo "禁用")"
    echo -e "   💾 模板缓存: $([ -n "$CACHE_ARGS" ] && echo "禁用" || echo "启用")"
    echo -e "   🔄 后台运行: ${BG_MODE}"
    echo -e "${CYAN}========================================${NC}"
    echo ""

    # 确认启动
    read -p "🚀 确认启动服务器? [Y/n]: " CONFIRM
    if [[ "$CONFIRM" =~ ^[Nn]$ ]]; then
        echo -e "${YELLOW}取消启动${NC}"
        exit 0
    fi

    echo ""

    # 执行启动
    if [ "$BG_MODE" = "yes" ]; then
        echo -e "${GREEN}🚀 在后台启动服务器...${NC}"
        nohup eval $CMD > rustblog.log 2>&1 &
        local pid=$!
        echo $pid > rustblog.pid
        echo -e "${GREEN}✅ 服务器已启动，PID: ${pid}${NC}"
        echo -e "${CYAN}📝 日志文件: rustblog.log${NC}"
        echo -e "${CYAN}📄 PID 文件: rustblog.pid${NC}"
    else
        echo -e "${GREEN}🚀 启动服务器...${NC}"
        eval $CMD
    fi
}

# 查找 rustblog 进程
find_rustblog_processes() {
    echo -e "${CYAN}🔍 正在查找 rustblog 进程...${NC}"
    echo ""

    local found=0

    # 方法 1: 使用 pid 文件
    if [ -f "rustblog.pid" ]; then
        local pid=$(cat rustblog.pid)
        if ps -p $pid > /dev/null 2>&1; then
            echo -e "${GREEN}✅ 通过 PID 文件找到进程:${NC}"
            echo "   PID: $pid"
            ps -p $pid -o pid,ppid,cmd --no-headers | sed 's/^/   /'
            echo ""
            found=1
        else
            echo -e "${YELLOW}⚠️  PID 文件存在但进程未运行 (PID: $pid)${NC}"
            echo ""
        fi
    fi

    # 方法 2: 使用 pgrep
    local pids=$(pgrep -f "./rustblog" 2>/dev/null || true)
    if [ -n "$pids" ]; then
        echo -e "${GREEN}✅ 通过 pgrep 找到进程:${NC}"
        echo "$pids" | while read pid; do
            echo "   PID: $pid"
            ps -p $pid -o pid,ppid,cmd --no-headers 2>/dev/null | sed 's/^/   /'
        done
        echo ""
        found=1
    fi

    # 方法 3: 使用 ps 和 grep
    local processes=$(ps aux | grep -v grep | grep "[r]ustblog" || true)
    if [ -n "$processes" ]; then
        echo -e "${GREEN}✅ 通过 ps 找到进程:${NC}"
        echo "$processes" | awk '{print "   PID: " $2 " | CMD: " $11 " " $12 " " $13 " " $14 " " $15}'
        echo ""
        found=1
    fi

    # 方法 4: 使用 lsof (如果可用)
    if command -v lsof >/dev/null 2>&1; then
        local lsof_procs=$(lsof -i :8080 2>/dev/null | grep rustblog || true)
        if [ -n "$lsof_procs" ]; then
            echo -e "${GREEN}✅ 通过 lsof (端口 8080) 找到进程:${NC}"
            echo "$lsof_procs" | awk '{print "   PID: " $2 " | USER: " $3 " | CMD: " $1}'
            echo ""
            found=1
        fi
    fi

    if [ $found -eq 0 ]; then
        echo -e "${YELLOW}⚠️  未找到运行中的 rustblog 进程${NC}"
        echo ""
        return 1
    fi

    return 0
}

# 终止 rustblog 进程
stop_server() {
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}   终止 RustBlog 服务器${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo ""

    # 查找进程
    if ! find_rustblog_processes; then
        echo -e "${YELLOW}没有需要终止的进程${NC}"
        exit 0
    fi

    echo -e "${YELLOW}请选择终止方式:${NC}"
    echo "   1) 通过 PID 文件终止"
    echo "   2) 通过 pgrep 终止所有 rustblog 进程"
    echo "   3) 通过进程名终止"
    echo "   4) 终止占用 8080 端口的进程"
    echo "   5) 手动输入 PID"
    echo "   6) 取消"
    echo ""
    read -p "   请选择 [1-6]: " STOP_CHOICE

    case $STOP_CHOICE in
        1)
            if [ -f "rustblog.pid" ]; then
                local pid=$(cat rustblog.pid)
                echo -e "${YELLOW}正在终止 PID: $pid ...${NC}"
                kill $pid 2>/dev/null && echo -e "${GREEN}✅ 进程已终止${NC}" || echo -e "${RED}❌ 终止失败${NC}"
                rm -f rustblog.pid
            else
                echo -e "${RED}❌ PID 文件不存在${NC}"
            fi
            ;;
        2)
            local pids=$(pgrep -f "./rustblog" 2>/dev/null || true)
            if [ -n "$pids" ]; then
                echo -e "${YELLOW}正在终止进程: $pids ...${NC}"
                kill $pids 2>/dev/null && echo -e "${GREEN}✅ 进程已终止${NC}" || echo -e "${RED}❌ 终止失败${NC}"
            else
                echo -e "${RED}❌ 未找到进程${NC}"
            fi
            ;;
        3)
            echo -e "${YELLOW}正在终止所有 rustblog 进程...${NC}"
            pkill -f "./rustblog" 2>/dev/null && echo -e "${GREEN}✅ 进程已终止${NC}" || echo -e "${RED}❌ 终止失败${NC}"
            ;;
        4)
            if command -v lsof >/dev/null 2>&1; then
                local pid=$(lsof -ti :8080 2>/dev/null || true)
                if [ -n "$pid" ]; then
                    echo -e "${YELLOW}正在终止占用 8080 端口的进程 (PID: $pid) ...${NC}"
                    kill $pid 2>/dev/null && echo -e "${GREEN}✅ 进程已终止${NC}" || echo -e "${RED}❌ 终止失败${NC}"
                else
                    echo -e "${RED}❌ 8080 端口未被占用${NC}"
                fi
            else
                echo -e "${RED}❌ lsof 命令不可用${NC}"
            fi
            ;;
        5)
            read -p "   请输入要终止的 PID: " MANUAL_PID
            if [ -n "$MANUAL_PID" ]; then
                echo -e "${YELLOW}正在终止 PID: $MANUAL_PID ...${NC}"
                kill $MANUAL_PID 2>/dev/null && echo -e "${GREEN}✅ 进程已终止${NC}" || echo -e "${RED}❌ 终止失败${NC}"
            fi
            ;;
        6)
            echo -e "${YELLOW}取消操作${NC}"
            exit 0
            ;;
        *)
            echo -e "${RED}❌ 无效选择${NC}"
            exit 1
            ;;
    esac

    echo ""
    echo -e "${CYAN}再次检查进程状态...${NC}"
    sleep 1
    find_rustblog_processes || echo -e "${GREEN}✅ 所有 rustblog 进程已终止${NC}"
}

# 查看服务器状态
check_status() {
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}   RustBlog 服务器状态${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo ""

    find_rustblog_processes

    if [ -f "rustblog.log" ]; then
        echo -e "${CYAN}📝 最近日志 (最后 10 行):${NC}"
        tail -n 10 rustblog.log | sed 's/^/   /'
    fi
}

# 查看日志
view_logs() {
    if [ -f "rustblog.log" ]; then
        if command -v less >/dev/null 2>&1; then
            less rustblog.log
        else
            cat rustblog.log
        fi
    else
        echo -e "${YELLOW}⚠️  日志文件不存在${NC}"
    fi
}

# 显示帮助
show_help() {
    cat << EOF
RustBlog 服务器管理脚本

用法: ./manage.sh [选项]

选项:
    start       交互式启动服务器
    stop        终止服务器
    restart     重启服务器
    status      查看服务器状态
    logs        查看服务器日志
    help        显示此帮助信息

示例:
    ./manage.sh start      # 交互式启动
    ./manage.sh stop       # 终止服务器
    ./manage.sh status     # 查看状态

EOF
}

# 主函数
main() {
    case "${1:-help}" in
        start)
            check_memory
            if [ $? -eq 0 ]; then
                interactive_config
            else
                read -p "内存不足，是否仍要启动? [y/N]: " FORCE_START
                if [[ "$FORCE_START" =~ ^[Yy]$ ]]; then
                    interactive_config
                else
                    echo -e "${YELLOW}取消启动${NC}"
                    exit 1
                fi
            fi
            ;;
        stop)
            stop_server
            ;;
        restart)
            echo -e "${YELLOW}正在重启服务器...${NC}"
            stop_server
            sleep 2
            check_memory
            if [ $? -eq 0 ]; then
                interactive_config
            else
                read -p "内存不足，是否仍要启动? [y/N]: " FORCE_START
                if [[ "$FORCE_START" =~ ^[Yy]$ ]]; then
                    interactive_config
                else
                    echo -e "${YELLOW}取消重启${NC}"
                    exit 1
                fi
            fi
            ;;
        status)
            check_status
            ;;
        logs)
            view_logs
            ;;
        help|--help|-h)
            show_help
            ;;
        *)
            echo -e "${RED}❌ 未知选项: $1${NC}"
            echo ""
            show_help
            exit 1
            ;;
    esac
}

main "$@"