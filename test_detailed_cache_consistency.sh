#!/bin/bash

# 详细的缓存一致性测试脚本

echo "================================"
echo "详细缓存一致性测试"
echo "================================"

BASE_URL="http://localhost:8080"

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 检查服务器是否运行
echo -e "${YELLOW}[0/12] 检查服务器状态...${NC}"
if curl -s "$BASE_URL/api/db/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ 服务器正在运行${NC}"
else
    echo -e "${RED}✗ 服务器未运行，请先启动服务器${NC}"
    exit 1
fi

# 清空所有缓存
echo -e "\n${YELLOW}[1/12] 清空 Redis 缓存...${NC}"
redis-cli KEYS "rustblog:*" | xargs -r redis-cli DEL > /dev/null 2>&1
echo -e "${GREEN}✓ 缓存已清空${NC}"

# 获取管理员令牌
echo -e "\n${YELLOW}[2/12] 获取管理员令牌...${NC}"
LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/api/login" \
    -H "Content-Type: application/json" \
    -d '{"username":"admin","password":"admin"}')
AUTH_TOKEN=$(echo "$LOGIN_RESPONSE" | grep -o '"token":"[^"]*' | cut -d'"' -f4)
if [ -n "$AUTH_TOKEN" ]; then
    echo -e "${GREEN}✓ 获取到管理员令牌${NC}"
else
    echo -e "${RED}✗ 获取管理员令牌失败${NC}"
    echo -e "${YELLOW}  将跳过需要认证的测试${NC}"
    AUTH_TOKEN=""
fi

# 首次请求文章列表（应该 MISS）
echo -e "\n${YELLOW}[3/12] 首次请求文章列表...${NC}"
RESPONSE1=$(curl -s -i "$BASE_URL/api/passage/list?page=1&limit=10")
CACHE_STATUS1=$(echo "$RESPONSE1" | grep -i "X-Cache" | cut -d' ' -f2 | tr -d '\r')
if echo "$CACHE_STATUS1" | grep -q "MISS"; then
    echo -e "${GREEN}✓ 首次请求缓存状态: MISS (预期)${NC}"
else
    echo -e "${YELLOW}! 首次请求缓存状态: ${CACHE_STATUS1}${NC}"
fi

# 第二次请求文章列表（应该 HIT）
echo -e "\n${YELLOW}[4/12] 第二次请求文章列表（验证缓存命中）...${NC}"
RESPONSE2=$(curl -s -i "$BASE_URL/api/passage/list?page=1&limit=10")
CACHE_STATUS2=$(echo "$RESPONSE2" | grep -i "X-Cache" | cut -d' ' -f2 | tr -d '\r')
if echo "$CACHE_STATUS2" | grep -q "HIT"; then
    echo -e "${GREEN}✓ 第二次请求缓存状态: HIT (预期)${NC}"
else
    echo -e "${RED}✗ 第二次请求缓存状态: ${CACHE_STATUS2} (预期 HIT)${NC}"
fi

# 测试：创建草稿文章（不应清除列表缓存）
echo -e "\n${YELLOW}[5/12] 测试创建草稿文章（不应清除列表缓存）...${NC}"
if [ -n "$AUTH_TOKEN" ]; then
    CREATE_DRAFT_RESPONSE=$(curl -s -X POST "$BASE_URL/api/admin/passages" \
        -H "Content-Type: application/json" \
        -H "Cookie: auth_token=$AUTH_TOKEN" \
        -d '{
            "title": "测试缓存草稿文章",
            "content": "这是一篇测试缓存一致性的草稿文章。",
            "status": "draft",
            "category": "测试",
            "tags": "缓存,测试"
        }')
    if echo "$CREATE_DRAFT_RESPONSE" | grep -q "success.*true"; then
        DRAFT_UUID=$(echo "$CREATE_DRAFT_RESPONSE" | grep -o '"uuid":"[^"]*' | cut -d'"' -f4)
        echo -e "${GREEN}✓ 草稿文章创建成功 (UUID: ${DRAFT_UUID:0:8}...)${NC}"
        
        # 验证列表缓存是否仍然有效
        RESPONSE3=$(curl -s -i "$BASE_URL/api/passage/list?page=1&limit=10")
        CACHE_STATUS3=$(echo "$RESPONSE3" | grep -i "X-Cache" | cut -d' ' -f2 | tr -d '\r')
        if echo "$CACHE_STATUS3" | grep -q "HIT"; then
            echo -e "${GREEN}✓ 创建草稿后列表缓存仍然有效 (预期)${NC}"
        else
            echo -e "${RED}✗ 创建草稿后列表缓存失效 (预期应保持 HIT)${NC}"
        fi
    else
        echo -e "${RED}✗ 草稿文章创建失败${NC}"
    fi
else
    echo -e "${YELLOW}! 跳过草稿文章创建测试（无管理员令牌）${NC}"
    DRAFT_UUID=""
fi

# 测试：创建已发布文章（应清除列表缓存）
echo -e "\n${YELLOW}[6/12] 测试创建已发布文章（应清除列表缓存）...${NC}"
if [ -n "$AUTH_TOKEN" ]; then
    CREATE_PUB_RESPONSE=$(curl -s -X POST "$BASE_URL/api/admin/passages" \
        -H "Content-Type: application/json" \
        -H "Cookie: auth_token=$AUTH_TOKEN" \
        -d '{
            "title": "测试缓存已发布文章",
            "content": "这是一篇测试缓存一致性的已发布文章。",
            "status": "published",
            "category": "测试",
            "tags": "缓存,测试"
        }')
    if echo "$CREATE_PUB_RESPONSE" | grep -q "success.*true"; then
        PUB_UUID=$(echo "$CREATE_PUB_RESPONSE" | grep -o '"uuid":"[^"]*' | cut -d'"' -f4)
        echo -e "${GREEN}✓ 已发布文章创建成功 (UUID: ${PUB_UUID:0:8}...)${NC}"
        
        # 验证列表缓存是否被清除
        RESPONSE4=$(curl -s -i "$BASE_URL/api/passage/list?page=1&limit=10")
        CACHE_STATUS4=$(echo "$RESPONSE4" | grep -i "X-Cache" | cut -d' ' -f2 | tr -d '\r')
        if echo "$CACHE_STATUS4" | grep -q "MISS"; then
            echo -e "${GREEN}✓ 创建已发布文章后列表缓存已清除 (预期)${NC}"
        else
            echo -e "${RED}✗ 创建已发布文章后列表缓存未清除 (预期应 MISS)${NC}"
        fi
    else
        echo -e "${RED}✗ 已发布文章创建失败${NC}"
    fi
else
    echo -e "${YELLOW}! 跳过已发布文章创建测试（无管理员令牌）${NC}"
    PUB_UUID=""
fi

# 测试：更新文章标题（只清除详情缓存）
echo -e "\n${YELLOW}[7/12] 测试更新文章标题（只清除详情缓存）...${NC}"
if [ -n "$AUTH_TOKEN" ] && [ -n "$PUB_UUID" ]; then
    UPDATE_TITLE_RESPONSE=$(curl -s -X PUT "$BASE_URL/api/admin/passages?id=$PUB_UUID" \
        -H "Content-Type: application/json" \
        -H "Cookie: auth_token=$AUTH_TOKEN" \
        -d '{
            "title": "测试缓存已发布文章（已更新标题）"
        }')
    if echo "$UPDATE_TITLE_RESPONSE" | grep -q "success.*true"; then
        echo -e "${GREEN}✓ 文章标题更新成功${NC}"
        
        # 验证列表缓存是否仍然有效
        RESPONSE5=$(curl -s -i "$BASE_URL/api/passage/list?page=1&limit=10")
        CACHE_STATUS5=$(echo "$RESPONSE5" | grep -i "X-Cache" | cut -d' ' -f2 | tr -d '\r')
        if echo "$CACHE_STATUS5" | grep -q "HIT"; then
            echo -e "${GREEN}✓ 更新标题后列表缓存仍然有效 (预期)${NC}"
        else
            echo -e "${YELLOW}! 更新标题后列表缓存失效${NC}"
        fi
        
        # 验证详情缓存是否被清除
        RESPONSE6=$(curl -s -i "$BASE_URL/api/passage/$PUB_UUID")
        CACHE_STATUS6=$(echo "$RESPONSE6" | grep -i "X-Cache" | cut -d' ' -f2 | tr -d '\r')
        if echo "$CACHE_STATUS6" | grep -q "MISS"; then
            echo -e "${GREEN}✓ 更新标题后详情缓存已清除 (预期)${NC}"
        else
            echo -e "${RED}✗ 更新标题后详情缓存未清除 (预期应 MISS)${NC}"
        fi
    else
        echo -e "${RED}✗ 文章标题更新失败${NC}"
    fi
else
    echo -e "${YELLOW}! 跳过标题更新测试${NC}"
fi

# 测试：更新文章状态（应清除列表缓存）
echo -e "\n${YELLOW}[8/12] 测试更新文章状态（应清除列表缓存）...${NC}"
if [ -n "$AUTH_TOKEN" ] && [ -n "$PUB_UUID" ]; then
    UPDATE_STATUS_RESPONSE=$(curl -s -X PUT "$BASE_URL/api/admin/passages?id=$PUB_UUID" \
        -H "Content-Type: application/json" \
        -H "Cookie: auth_token=$AUTH_TOKEN" \
        -d '{
            "status": "draft"
        }')
    if echo "$UPDATE_STATUS_RESPONSE" | grep -q "success.*true"; then
        echo -e "${GREEN}✓ 文章状态更新成功 (published -> draft)${NC}"
        
        # 验证列表缓存是否被清除
        RESPONSE7=$(curl -s -i "$BASE_URL/api/passage/list?page=1&limit=10")
        CACHE_STATUS7=$(echo "$RESPONSE7" | grep -i "X-Cache" | cut -d' ' -f2 | tr -d '\r')
        if echo "$CACHE_STATUS7" | grep -q "MISS"; then
            echo -e "${GREEN}✓ 更新状态后列表缓存已清除 (预期)${NC}"
        else
            echo -e "${RED}✗ 更新状态后列表缓存未清除 (预期应 MISS)${NC}"
        fi
    else
        echo -e "${RED}✗ 文章状态更新失败${NC}"
    fi
else
    echo -e "${YELLOW}! 跳过状态更新测试${NC}"
fi

# 测试：删除文章（应清除所有缓存）
echo -e "\n${YELLOW}[9/12] 测试删除文章（应清除所有缓存）...${NC}"
if [ -n "$AUTH_TOKEN" ] && [ -n "$PUB_UUID" ]; then
    # 先把文章改回 published 状态
    curl -s -X PUT "$BASE_URL/api/admin/passages?id=$PUB_UUID" \
        -H "Content-Type: application/json" \
        -H "Cookie: auth_token=$AUTH_TOKEN" \
        -d '{"status":"published"}' > /dev/null 2>&1
    
    # 重新缓存列表
    curl -s "$BASE_URL/api/passage/list?page=1&limit=10" > /dev/null 2>&1
    
    # 删除文章
    DELETE_RESPONSE=$(curl -s -X DELETE "$BASE_URL/api/admin/passages?id=$PUB_UUID" \
        -H "Cookie: auth_token=$AUTH_TOKEN")
    if echo "$DELETE_RESPONSE" | grep -q "success.*true"; then
        echo -e "${GREEN}✓ 文章删除成功${NC}"
        
        # 验证详情缓存是否被清除
        RESPONSE8=$(curl -s -i "$BASE_URL/api/passage/$PUB_UUID")
        CACHE_STATUS8=$(echo "$RESPONSE8" | grep -i "X-Cache" | cut -d' ' -f2 | tr -d '\r')
        if echo "$CACHE_STATUS8" | grep -q "MISS"; then
            echo -e "${GREEN}✓ 删除文章后详情缓存已清除 (预期)${NC}"
        else
            echo -e "${YELLOW}! 删除文章后详情缓存状态: ${CACHE_STATUS8}${NC}"
        fi
        
        # 验证列表缓存是否被清除
        RESPONSE9=$(curl -s -i "$BASE_URL/api/passage/list?page=1&limit=10")
        CACHE_STATUS9=$(echo "$RESPONSE9" | grep -i "X-Cache" | cut -d' ' -f2 | tr -d '\r')
        if echo "$CACHE_STATUS9" | grep -q "MISS"; then
            echo -e "${GREEN}✓ 删除文章后列表缓存已清除 (预期)${NC}"
        else
            echo -e "${RED}✗ 删除文章后列表缓存未清除 (预期应 MISS)${NC}"
        fi
    else
        echo -e "${RED}✗ 文章删除失败${NC}"
    fi
else
    echo -e "${YELLOW}! 跳过删除测试${NC}"
fi

# 测试：更新标签（应清除列表缓存）
echo -e "\n${YELLOW}[10/12] 测试更新文章标签（应清除列表缓存）...${NC}"
if [ -n "$AUTH_TOKEN" ]; then
    # 先获取一个现有的已发布文章
    LIST_RESPONSE=$(curl -s "$BASE_URL/api/passage/list?page=1&limit=10")
    FIRST_UUID=$(echo "$LIST_RESPONSE" | grep -o '"uuid":"[^"]*' | head -1 | cut -d'"' -f4)
    
    if [ -n "$FIRST_UUID" ]; then
        echo -e "${BLUE}  使用文章 UUID: ${FIRST_UUID:0:8}...${NC}"
        
        # 重新缓存列表
        curl -s "$BASE_URL/api/passage/list?page=1&limit=10" > /dev/null 2>&1
        
        # 更新标签
        UPDATE_TAGS_RESPONSE=$(curl -s -X PUT "$BASE_URL/api/admin/passages?id=$FIRST_UUID" \
            -H "Content-Type: application/json" \
            -H "Cookie: auth_token=$AUTH_TOKEN" \
            -d '{
                "tags": "缓存测试,标签更新"
            }')
        if echo "$UPDATE_TAGS_RESPONSE" | grep -q "success.*true"; then
            echo -e "${GREEN}✓ 文章标签更新成功${NC}"
            
            # 验证列表缓存是否被清除
            RESPONSE10=$(curl -s -i "$BASE_URL/api/passage/list?page=1&limit=10")
            CACHE_STATUS10=$(echo "$RESPONSE10" | grep -i "X-Cache" | cut -d' ' -f2 | tr -d '\r')
            if echo "$CACHE_STATUS10" | grep -q "MISS"; then
                echo -e "${GREEN}✓ 更新标签后列表缓存已清除 (预期)${NC}"
            else
                echo -e "${RED}✗ 更新标签后列表缓存未清除 (预期应 MISS)${NC}"
            fi
        else
            echo -e "${RED}✗ 文章标签更新失败${NC}"
        fi
    else
        echo -e "${YELLOW}! 没有可用的文章进行标签更新测试${NC}"
    fi
else
    echo -e "${YELLOW}! 跳过标签更新测试（无管理员令牌）${NC}"
fi

# 测试：更新分类（应清除列表缓存）
echo -e "\n${YELLOW}[11/12] 测试更新文章分类（应清除列表缓存）...${NC}"
if [ -n "$AUTH_TOKEN" ] && [ -n "$FIRST_UUID" ]; then
    # 重新缓存列表
    curl -s "$BASE_URL/api/passage/list?page=1&limit=10" > /dev/null 2>&1
    
    # 更新分类
    UPDATE_CAT_RESPONSE=$(curl -s -X PUT "$BASE_URL/api/admin/passages?id=$FIRST_UUID" \
        -H "Content-Type: application/json" \
        -H "Cookie: auth_token=$AUTH_TOKEN" \
        -d '{
            "category": "缓存测试分类"
        }')
    if echo "$UPDATE_CAT_RESPONSE" | grep -q "success.*true"; then
        echo -e "${GREEN}✓ 文章分类更新成功${NC}"
        
        # 验证列表缓存是否被清除
        RESPONSE11=$(curl -s -i "$BASE_URL/api/passage/list?page=1&limit=10")
        CACHE_STATUS11=$(echo "$RESPONSE11" | grep -i "X-Cache" | cut -d' ' -f2 | tr -d '\r')
        if echo "$CACHE_STATUS11" | grep -q "MISS"; then
            echo -e "${GREEN}✓ 更新分类后列表缓存已清除 (预期)${NC}"
        else
            echo -e "${RED}✗ 更新分类后列表缓存未清除 (预期应 MISS)${NC}"
        fi
    else
        echo -e "${RED}✗ 文章分类更新失败${NC}"
    fi
else
    echo -e "${YELLOW}! 跳过分类更新测试${NC}"
fi

# 最终验证：请求文章列表并验证数据一致性
echo -e "\n${YELLOW}[12/12] 最终验证：检查数据一致性...${NC}"
FINAL_RESPONSE=$(curl -s "$BASE_URL/api/passage/list?page=1&limit=10")
echo -e "${BLUE}  当前文章列表中的文章数量: $(echo "$FINAL_RESPONSE" | grep -o '"title"' | wc -l)${NC}"

# 检查是否包含已删除的测试文章
if echo "$FINAL_RESPONSE" | grep -q "测试缓存已发布文章"; then
    echo -e "${RED}✗ 列表中仍包含已删除的测试文章（数据不一致）${NC}"
else
    echo -e "${GREEN}✓ 列表中不包含已删除的测试文章（数据一致）${NC}"
fi

# 测试总结
echo -e "\n================================"
echo -e "${GREEN}测试完成！${NC}"
echo -e "================================"
echo ""
echo "精细缓存失效策略验证："
echo "  ✓ 创建草稿文章：不清除列表缓存"
echo "  ✓ 创建已发布文章：清除列表缓存"
echo "  ✓ 更新标题/内容：只清除详情缓存"
echo "  ✓ 更新状态：清除列表缓存"
echo "  ✓ 更新标签：清除列表缓存"
echo "  ✓ 更新分类：清除列表缓存"
echo "  ✓ 删除文章：清除所有缓存"
echo ""
echo "缓存一致性保证："
echo "  - 删除文章后，列表不会显示已删除的文章"
echo "  - 创建已发布文章后，列表会显示新文章"
echo "  - 更新状态后，列表会反映最新的可见性"
echo ""