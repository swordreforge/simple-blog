#!/bin/bash

# 检查服务器是否运行
if ! curl -s http://localhost:8080/ > /dev/null 2>&1; then
    echo "服务器未运行，请先启动服务器"
    exit 1
fi

# 测试 API 返回的路由数据
echo "=== 测试 API 返回的路由数据 ==="
curl -s -H "Cookie: $(cat cookies.txt 2>/dev/null || echo '')" \
    "http://localhost:8080/api/admin/dynamic-routes?page=1&limit=20" | \
    python3 -c "
import json
import sys
data = json.load(sys.stdin)
if data.get('success'):
    routes = data['data']['routes']
    print(f'总路由数: {len(routes)}')
    for route in routes:
        print(f\"ID: {route['id']}, 路径: {route['path']}, route_type: {route.get('route_type', 'N/A')}, route_type 类型: {type(route.get('route_type'))}\")
else:
    print('API 返回错误:', data.get('message'))
"