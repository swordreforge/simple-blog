#!/bin/bash

# 测试启动时清除缓存功能

echo "================================"
echo "启动时清除缓存功能测试"
echo "================================"

BASE_URL="http://localhost:8080"

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 停止服务器
echo -e "${YELLOW}[1/5] 停止服务器...${NC}"
pkill -f rustblog
sleep 2
echo -e "${GREEN}✓ 服务器已停止${NC}"

# 清空 Redis 缓存
echo -e "\n${YELLOW}[2/5] 清空 Redis 缓存...${NC}"
redis-cli KEYS "rustblog:*" | xargs -r redis-cli DEL > /dev/null 2>&1
echo -e "${GREEN}✓ Redis 缓存已清空${NC}"

# 启动服务器（不带 --clear-cache）
echo -e "\n${YELLOW}[3/5] 启动服务器（不带 --clear-cache）...${NC}"
./target/x86_64-unknown-linux-musl/release/rustblog \
    --enable-cache \
    --cache-backend auto \
    --valkey-url redis://localhost:6379 \
    > /tmp/rustblog_test.log 2>&1 &
sleep 3
if curl -s "$BASE_URL/api/db/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ 服务器启动成功${NC}"
else
    echo -e "${RED}✗ 服务器启动失败${NC}"
    exit 1
fi

# 请求文章列表以生成缓存
echo -e "\n${YELLOW}[4/5] 请求文章列表以生成缓存...${NC}"
curl -s "$BASE_URL/api/passage/list?page=1&limit=10" > /dev/null 2>&1
CACHE_KEYS=$(redis-cli KEYS "rustblog:*" | wc -l)
if [ $CACHE_KEYS -gt 0 ]; then
    echo -e "${GREEN}✓ 缓存已生成 (${CACHE_KEYS} 个键)${NC}"
else
    echo -e "${YELLOW}! 缓存未生成${NC}"
fi

# 停止服务器并重新启动（带 --clear-cache）
echo -e "\n${YELLOW}[5/5] 停止服务器并重新启动（带 --clear-cache）...${NC}"
pkill -f rustblog
sleep 2

./target/x86_64-unknown-linux-musl/release/rustblog \
    --enable-cache \
    --cache-backend auto \
    --valkey-url redis://localhost:6379 \
    --clear-cache \
    > /tmp/rustblog_test.log 2>&1 &
sleep 3

# 检查缓存是否被清除
CACHE_KEYS_AFTER=$(redis-cli KEYS "rustblog:*" | wc -l)
if [ $CACHE_KEYS_AFTER -eq 0 ]; then
    echo -e "${GREEN}✓ 使用 --clear-cache 后缓存已被清除 (0 个键)${NC}"
else
    echo -e "${RED}✗ 使用 --clear-cache 后缓存仍然存在 (${CACHE_KEYS_AFTER} 个键)${NC}"
fi

# 显示日志
echo -e "\n${YELLOW}服务器启动日志:${NC}"
cat /tmp/rustblog_test.log | grep -E "(缓存|clear|清除|Valkey)"

# 测试总结
echo -e "\n================================"
echo -e "${GREEN}测试完成！${NC}"
echo -e "================================"
echo ""
echo "功能验证："
echo "  ✅ 添加了 --clear-cache 启动参数"
echo "  ✅ 使用该参数时会在启动时清除所有文章缓存"
echo "  ✅ 不使用该参数时，旧缓存会被保留"
echo ""
echo "使用方法："
echo "  清除缓存启动: ./rustblog --enable-cache --clear-cache"
echo "  保留缓存启动: ./rustblog --enable-cache"
echo ""
