#!/bin/bash
# 测试 Valkey 超时和连接稳定性

set -e

echo "🧪 测试 Valkey 超时和连接稳定性"
echo "================================"

# 检查 Valkey 是否运行
if ! redis-cli ping > /dev/null 2>&1; then
    echo "❌ Valkey 未运行，跳过测试"
    exit 1
fi

echo "✅ Valkey 正在运行"

# 清空测试数据
echo "🧹 清空测试数据..."
redis-cli KEYS "test:*" | xargs -r redis-cli DEL > /dev/null 2>&1

# 启动应用（使用 Valkey 后端）
echo "🚀 启动应用（使用 Valkey 后端）..."
cargo build --release --features valkey > /dev/null 2>&1

# 后台运行应用
./target/release/rustblog \
    --cache-backend valkey \
    --valkey-url redis://localhost:6379 \
    > /tmp/rustblog_test.log 2>&1 &
APP_PID=$!

echo "应用 PID: $APP_PID"

# 等待应用启动
sleep 3

# 检查应用是否还在运行
if ! kill -0 $APP_PID 2>/dev/null; then
    echo "❌ 应用启动失败"
    cat /tmp/rustblog_test.log
    exit 1
fi

echo "✅ 应用启动成功"

# 模拟 Valkey 连接问题
echo "🔌 模拟 Valkey 连接问题..."
redis-cli CONFIG SET timeout 2 > /dev/null 2>&1
redis-cli CONFIG SET tcp-keepalive 1 > /dev/null 2>&1

# 等待几秒让连接超时
sleep 5

# 检查应用是否还在运行
if ! kill -0 $APP_PID 2>/dev/null; then
    echo "❌ 应用在 Valkey 连接问题后崩溃"
    cat /tmp/rustblog_test.log
    exit 1
fi

echo "✅ 应用在 Valkey 连接问题后仍然运行"

# 恢复 Valkey 配置
redis-cli CONFIG SET timeout 0 > /dev/null 2>&1
redis-cli CONFIG SET tcp-keepalive 300 > /dev/null 2>&1

# 清理
echo "🧹 清理..."
kill $APP_PID 2>/dev/null || true
wait $APP_PID 2>/dev/null || true

echo ""
echo "✅ 所有测试通过！"
echo "📝 查看日志: cat /tmp/rustblog_test.log"