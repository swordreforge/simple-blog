#!/bin/bash

# Valkey 连接测试脚本
# 用于测试优化后的 Valkey 连接稳定性

echo "=========================================="
echo "Valkey 连接稳定性测试"
echo "=========================================="
echo ""

# 检查 Valkey 服务是否运行
echo "1. 检查 Valkey 服务状态..."
if command -v redis-cli &> /dev/null; then
    if redis-cli ping &> /dev/null; then
        echo "✅ Valkey 服务正在运行"
        redis-cli ping
    else
        echo "❌ Valkey 服务未运行或无法连接"
        echo "提示: 请先启动 Valkey 服务（如: redis-server 或 valkey-server）"
        exit 1
    fi
else
    echo "⚠️  未找到 redis-cli 命令，无法检查 Valkey 服务"
fi

echo ""
echo "2. 启动应用（带缓存功能）..."
echo "=========================================="

# 启动应用（使用自动模式，优先 Valkey，失败时降级到本地缓存）
./target/release/rustblog \
    --enable-cache \
    --cache-backend auto \
    --valkey-url redis://localhost:6379 \
    --cache-ttl 3600 \
    --cache-fallback true \
    --log-level debug &
APP_PID=$!

echo "应用已启动 (PID: $APP_PID)"
echo ""
echo "3. 等待应用初始化..."
sleep 5

# 检查应用是否仍在运行
if ! ps -p $APP_PID > /dev/null; then
    echo "❌ 应用启动失败"
    exit 1
fi

echo "✅ 应用正在运行"
echo ""
echo "4. 测试缓存操作..."

# 使用 curl 测试应用
echo "正在测试首页..."
curl -s http://localhost:8080/ > /dev/null
if [ $? -eq 0 ]; then
    echo "✅ 首页访问成功"
else
    echo "❌ 首页访问失败"
fi

echo ""
echo "5. 查看应用日志（Ctrl+C 停止）..."
echo "=========================================="
echo "日志将显示 Valkey 连接状态、重试情况和降级信息"
echo "=========================================="
echo ""

# 显示应用日志
tail -f /proc/$APP_PID/fd/1 2>/dev/null || echo "无法读取应用日志"

# 清理
# kill $APP_PID 2>/dev/null