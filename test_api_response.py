#!/usr/bin/env python3
import sqlite3
import json
import sys

# 连接数据库
conn = sqlite3.connect('/home/swordreforge/projects/rustblog-new/rustblog/target/x86_64-unknown-linux-musl/release/data/blog.db')
cursor = conn.cursor()

# 查询路由数据
cursor.execute("""
    SELECT id, route_name, route_type, path, handler_type, enabled
    FROM dynamic_routes
    WHERE path = '/aasdwdwdwdw'
""")

row = cursor.fetchone()
if row:
    id, route_name, route_type, path, handler_type, enabled = row
    print("数据库中的数据:")
    print(f"  ID: {id}")
    print(f"  路由名称: {route_name}")
    print(f"  路由类型: {route_type} (类型: {type(route_type)})")
    print(f"  路径: {path}")
    print(f"  处理器类型: {handler_type}")
    print(f"  是否启用: {enabled}")

    # 构造一个模拟的 API 响应
    route = {
        "id": id,
        "route_name": route_name,
        "route_type": route_type,
        "path": path,
        "handler_type": handler_type,
        "enabled": enabled
    }

    print("\n模拟的 API 响应:")
    print(json.dumps(route, indent=2, ensure_ascii=False))
else:
    print("未找到路由数据")

conn.close()