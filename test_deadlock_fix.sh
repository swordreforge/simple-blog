#!/bin/bash

# 死锁修复测试脚本
# 用于验证修复后的系统稳定性

echo "=========================================="
echo "死锁修复验证测试"
echo "=========================================="
echo ""

# 检查是否有进程正在运行
if pgrep -f "rustblog" > /dev/null; then
    echo "⚠️  检测到正在运行的 rustblog 进程"
    echo "PID: $(pgrep -f rustblog)"
    read -p "是否停止现有进程? (y/n): " choice
    if [ "$choice" = "y" ]; then
        pkill -9 -f rustblog
        sleep 2
    else
        echo "❌ 测试已取消"
        exit 1
    fi
fi

echo "1. 启动应用..."
./target/x86_64-unknown-linux-musl/release/rustblog \
    --enable-cache \
    --cache-backend auto \
    --valkey-url redis://localhost:6379 \
    --log-level info \
    --workers 4 &
APP_PID=$!

echo "应用已启动 (PID: $APP_PID)"
echo ""

# 等待应用初始化
echo "2. 等待应用初始化..."
sleep 5

# 检查应用是否仍在运行
if ! ps -p $APP_PID > /dev/null; then
    echo "❌ 应用启动失败"
    echo "查看日志:"
    journalctl -n 50 --no-pager 2>/dev/null || echo "无法获取日志"
    exit 1
fi

echo "✅ 应用正在运行"
echo ""

# 检查进程状态
echo "3. 检查进程状态..."
PROCESS_STATE=$(ps -o state= -p $APP_PID | tr -d ' ')
echo "进程状态: $PROCESS_STATE"

if [ "$PROCESS_STATE" = "D" ]; then
    echo "❌ 警告: 进程处于不可中断睡眠状态 (D)，可能存在死锁"
else
    echo "✅ 进程状态正常"
fi
echo ""

# 检查线程状态
echo "4. 检查线程状态..."
THREAD_COUNT=$(ps -o nlwp= -p $APP_PID | tr -d ' ')
echo "线程数: $THREAD_COUNT"

D_STATE_COUNT=$(ps -L -o state= -p $APP_PID | grep -c "D" || echo "0")
echo "D 状态线程数: $D_STATE_COUNT"

if [ "$D_STATE_COUNT" -gt 5 ]; then
    echo "❌ 警告: 大量线程处于 D 状态，可能存在 I/O 死锁"
else
    echo "✅ 线程状态正常"
fi
echo ""

# 压力测试
echo "5. 执行压力测试..."
echo "=========================================="

# 测试 1: 并发访问首页
echo "测试 1: 并发访问首页 (100 并发, 1000 请求)"
if command -v ab &> /dev/null; then
    ab -n 1000 -c 100 http://localhost:8080/ 2>&1 | grep -E "Requests per second|Time per request|Failed requests"
elif command -v curl &> /dev/null; then
    for i in {1..100}; do
        curl -s http://localhost:8080/ > /dev/null &
    done
    wait
    echo "✅ 完成 100 个并发请求"
else
    echo "⚠️  未找到 ab 或 curl，跳过压力测试"
fi
echo ""

# 检查应用是否仍然运行
if ! ps -p $APP_PID > /dev/null; then
    echo "❌ 应用在压力测试中崩溃"
    exit 1
fi

echo "✅ 应用仍然运行"
echo ""

# 再次检查进程状态
echo "6. 压力测试后状态检查..."
PROCESS_STATE=$(ps -o state= -p $APP_PID | tr -d ' ')
D_STATE_COUNT=$(ps -L -o state= -p $APP_PID | grep -c "D" || echo "0")

echo "进程状态: $PROCESS_STATE"
echo "D 状态线程数: $D_STATE_COUNT"

if [ "$D_STATE_COUNT" -gt 5 ]; then
    echo "❌ 警告: 压力测试后仍有大量 D 状态线程"
else
    echo "✅ 应用状态稳定"
fi
echo ""

# 测试 2: 渲染大量 Markdown
echo "测试 2: 并发渲染 Markdown (50 并发)"
if command -v curl &> /dev/null; then
    # 尝试访问文章列表
    for i in {1..50}; do
        curl -s http://localhost:8080/collect > /dev/null &
    done
    wait
    echo "✅ 完成 50 个并发渲染请求"
else
    echo "⚠️  未找到 curl，跳过 Markdown 渲染测试"
fi
echo ""

# 最终状态检查
echo "7. 最终状态检查..."
if ! ps -p $APP_PID > /dev/null; then
    echo "❌ 应用在所有测试后崩溃"
    exit 1
fi

PROCESS_STATE=$(ps -o state= -p $APP_PID | tr -d ' ')
D_STATE_COUNT=$(ps -L -o state= -p $APP_PID | grep -c "D" || echo "0")

echo "进程状态: $PROCESS_STATE"
echo "D 状态线程数: $D_STATE_COUNT"
echo "运行时间: $(ps -o etime= -p $APP_PID | tr -d ' ')"
echo ""

# 判断测试结果
if [ "$PROCESS_STATE" = "D" ]; then
    echo "❌ 测试失败: 进程处于死锁状态"
    TEST_RESULT="FAILED"
elif [ "$D_STATE_COUNT" -gt 10 ]; then
    echo "⚠️  测试警告: 大量线程处于 D 状态"
    TEST_RESULT="WARNING"
else
    echo "✅ 测试通过: 应用运行稳定"
    TEST_RESULT="PASSED"
fi

echo ""
echo "=========================================="
echo "测试结果: $TEST_RESULT"
echo "=========================================="
echo ""

# 清理
echo "清理进程 (PID: $APP_PID)..."
kill $APP_PID 2>/dev/null
sleep 2

# 强制清理（如果需要）
if ps -p $APP_PID > /dev/null; then
    echo "强制终止进程..."
    kill -9 $APP_PID 2>/dev/null
fi

echo "✅ 清理完成"
echo ""
echo "=========================================="
echo "测试完成"
echo "=========================================="
echo ""

# 提供监控建议
echo "📊 监控建议:"
echo "1. 在生产环境中使用以下命令持续监控:"
echo "   watch -n 1 'ps aux | grep rustblog'"
echo ""
echo "2. 检查 D 状态线程:"
echo "   ps -L -o state,pid,comm -p <PID> | grep D"
echo ""
echo "3. 使用 strace 检测死锁:"
echo "   sudo strace -p <PID> -f -e trace=futex"
echo ""
echo "4. 使用 perf 分析性能:"
echo "   sudo perf top -p <PID>"
echo ""

if [ "$TEST_RESULT" = "FAILED" ]; then
    exit 1
elif [ "$TEST_RESULT" = "WARNING" ]; then
    exit 2
else
    exit 0
fi
