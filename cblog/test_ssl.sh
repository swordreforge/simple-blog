#!/bin/bash

# 启动服务器
./bin/cblog 2>&1 &
CBLOG_PID=$!

echo "Server PID: $CBLOG_PID"
sleep 2

# 测试连接
echo "=== Testing HTTPS connection ==="
curl -k -v https://127.0.0.1:8080/ 2>&1 | head -50

# 杀死服务器
echo ""
echo "=== Killing server ==="
kill $CBLOG_PID 2>/dev/null
wait $CBLOG_PID 2>/dev/null