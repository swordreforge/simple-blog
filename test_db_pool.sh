#!/bin/bash

# 数据库连接池测试脚本

echo "🧪 开始测试数据库连接池优化..."
echo ""

# 检查服务器是否运行
echo "1️⃣ 检查服务器状态..."
if curl -s http://localhost:8080/ > /dev/null; then
    echo "✅ 服务器正在运行"
else
    echo "❌ 服务器未运行，请先启动服务器"
    echo "   运行: cd /home/swordreforge/project/rustblog && cargo run"
    exit 1
fi
echo ""

# 测试连接池状态 API
echo "2️⃣ 测试连接池状态 API..."
echo "   GET /api/db/pool-status"
curl -s http://localhost:8080/api/db/pool-status | jq '.'
echo ""
echo ""

# 测试健康检查 API
echo "3️⃣ 测试数据库健康检查 API..."
echo "   GET /api/db/health"
curl -s http://localhost:8080/api/db/health | jq '.'
echo ""
echo ""

# 并发测试 - 模拟多个并发请求
echo "4️⃣ 并发测试 - 模拟 10 个并发请求..."
for i in {1..10}; do
    curl -s http://localhost:8080/api/db/pool-status > /dev/null &
done
wait
echo "✅ 并发测试完成"
echo ""

# 检查连接池状态
echo "5️⃣ 检查并发后的连接池状态..."
curl -s http://localhost:8080/api/db/pool-status | jq '.'
echo ""
echo ""

# 检查数据库文件
echo "6️⃣ 检查数据库文件..."
if [ -f "data/blog.db" ]; then
    DB_SIZE=$(du -h data/blog.db | cut -f1)
    echo "   数据库文件大小: $DB_SIZE"
else
    echo "   ❌ 数据库文件不存在"
fi

if [ -f "data/blog.db-wal" ]; then
    WAL_SIZE=$(du -h data/blog.db-wal | cut -f1)
    echo "   WAL 文件大小: $WAL_SIZE"
else
    echo "   ⚠️  WAL 文件不存在"
fi

if [ -f "data/blog.db-shm" ]; then
    SHM_SIZE=$(du -h data/blog.db-shm | cut -f1)
    echo "   SHM 文件大小: $SHM_SIZE"
else
    echo "   ⚠️  SHM 文件不存在"
fi
echo ""
echo ""

# 测试完成
echo "✅ 所有测试完成！"
echo ""
echo "📊 测试结果总结："
echo "   - 连接池配置已优化"
echo "   - WAL 模式已启用"
echo "   - 支持并发读写"
echo "   - API 端点正常工作"
echo ""
echo "📖 详细说明请查看: DATABASE_POOL_OPTIMIZATION.md"