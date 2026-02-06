#!/bin/bash
# run.sh - 运行脚本

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# 检查二进制文件
if [ ! -f "bin/cblog" ]; then
    echo "错误: 未找到可执行文件 bin/cblog"
    echo "请先运行 ./build.sh 构建项目"
    exit 1
fi

# 检查证书
if [ ! -f "data/cert.der" ] || [ ! -f "data/key.der" ]; then
    echo "警告: 未找到 TLS 证书"
    echo "请运行 ./build.sh 生成证书"
    exit 1
fi

# 检查数据库
if [ ! -f "data/blog.db" ]; then
    echo "警告: 未找到数据库"
    echo "正在初始化数据库..."
    ./bin/cblog --init-db
fi

# 创建日志目录
mkdir -p logs

# 启动服务器
echo "========================================"
echo "  RustBlog C语言版本 - 启动中..."
echo "========================================"
echo ""

./bin/cblog

# 如果需要前台运行，可以使用 trap 处理 Ctrl+C
# trap "echo '停止服务器...'; kill \$PID; exit 0" INT TERM
# ./bin/cblog &
# PID=$!
# wait $PID