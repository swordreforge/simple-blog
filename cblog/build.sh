#!/bin/bash
# build.sh - 一键构建脚本

set -e

echo "========================================"
echo "  RustBlog C语言版本 - 构建脚本"
echo "========================================"
echo ""

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 检查依赖
check_dependencies() {
    echo "检查依赖..."
    
    if ! command -v gcc &> /dev/null; then
        echo -e "${RED}错误: 未找到 gcc${NC}"
        exit 1
    fi
    
    if ! command -v make &> /dev/null; then
        echo -e "${RED}错误: 未找到 make${NC}"
        exit 1
    fi
    
    if ! command -v openssl &> /dev/null; then
        echo -e "${YELLOW}警告: 未找到 openssl（仅用于生成证书）${NC}"
    fi
    
    echo -e "${GREEN}✓ 依赖检查通过${NC}"
    echo ""
}

# 克隆 BearSSL
setup_bearssl() {
    if [ ! -d "bearssl" ]; then
        echo "下载 BearSSL..."
        if command -v git &> /dev/null; then
            git clone --depth 1 https://www.bearssl.org/git/BearSSL bearssl
            echo -e "${GREEN}✓ BearSSL 下载完成${NC}"
        else
            echo -e "${RED}错误: 未找到 git，无法下载 BearSSL${NC}"
            exit 1
        fi
    else
        echo -e "${GREEN}✓ BearSSL 已存在${NC}"
    fi
    echo ""
}

# 生成证书
generate_certs() {
    mkdir -p data
    
    if [ ! -f "data/cert.der" ] || [ ! -f "data/key.der" ]; then
        echo "生成自签名证书..."
        
        if command -v openssl &> /dev/null; then
            openssl req -x509 -newkey rsa:2048 -keyout data/key.pem -out data/cert.pem \
                -days 365 -nodes -subj "/C=CN/ST=Beijing/L=Beijing/O=RustBlog/CN=localhost" 2>/dev/null
            
            if [ -f "data/cert.pem" ] && [ -f "data/key.pem" ]; then
                openssl x509 -in data/cert.pem -outform der -out data/cert.der
                openssl rsa -in data/key.pem -outform der -out data/key.der
                
                echo -e "${GREEN}✓ 证书生成完成${NC}"
            else
                echo -e "${YELLOW}警告: 证书生成失败${NC}"
            fi
        else
            echo -e "${YELLOW}警告: 未找到 openssl，跳过证书生成${NC}"
        fi
    else
        echo -e "${GREEN}✓ 证书已存在${NC}"
    fi
    echo ""
}

# 下载 SQLite
download_sqlite() {
    if [ ! -f "sqlite3.c" ]; then
        echo "下载 SQLite 合并源..."
        if command -v wget &> /dev/null; then
            wget -q https://www.sqlite.org/2024/sqlite-amalgamation-3470200.zip -O /tmp/sqlite.zip
            unzip -q /tmp/sqlite.zip -d /tmp
            cp /tmp/sqlite-amalgamation-3470200/sqlite3.c .
            cp /tmp/sqlite-amalgamation-3470200/sqlite3.h src/include/
            rm -rf /tmp/sqlite.zip /tmp/sqlite-amalgamation-3470200
            echo -e "${GREEN}✓ SQLite 下载完成${NC}"
        elif command -v curl &> /dev/null; then
            curl -sL https://www.sqlite.org/2024/sqlite-amalgamation-3470200.zip -o /tmp/sqlite.zip
            unzip -q /tmp/sqlite.zip -d /tmp
            cp /tmp/sqlite-amalgamation-3470200/sqlite3.c .
            cp /tmp/sqlite-amalgamation-3470200/sqlite3.h src/include/
            rm -rf /tmp/sqlite.zip /tmp/sqlite-amalgamation-3470200
            echo -e "${GREEN}✓ SQLite 下载完成${NC}"
        else
            echo -e "${RED}错误: 未找到 wget 或 curl，无法下载 SQLite${NC}"
            exit 1
        fi
    else
        echo -e "${GREEN}✓ SQLite 已存在${NC}"
    fi
    echo ""
}

# 创建必要的目录
create_directories() {
    echo "创建目录结构..."
    mkdir -p build/{handlers,utils,bearssl}
    mkdir -p bin
    mkdir -p data
    mkdir -p templates
    mkdir -p static
    mkdir -p uploads
    mkdir -p markdown
    mkdir -p logs
    echo -e "${GREEN}✓ 目录创建完成${NC}"
    echo ""
}

# 编译项目
build_project() {
    echo "编译项目..."
    make clean 2>/dev/null || true
    make
    
    if [ -f "bin/cblog" ]; then
        local size=$(stat -c%s bin/cblog 2>/dev/null || echo "0")
        local size_kb=$((size / 1024))
        echo -e "${GREEN}✓ 编译完成！${NC}"
        echo "  二进制大小: ${size} bytes (${size_kb} KB)"
        echo ""
    else
        echo -e "${RED}错误: 编译失败${NC}"
        exit 1
    fi
}

# 初始化数据库
init_database() {
    if [ -f "bin/cblog" ]; then
        echo "初始化数据库..."
        if [ ! -f "data/blog.db" ]; then
            ./bin/cblog --init-db
            echo -e "${GREEN}✓ 数据库初始化完成${NC}"
        else
            echo -e "${GREEN}✓ 数据库已存在${NC}"
        fi
        echo ""
    fi
}

# 显示结果
show_result() {
    echo "========================================"
    echo "  构建完成！"
    echo "========================================"
    echo ""
    echo "可执行文件: bin/cblog"
    echo ""
    echo "快速启动:"
    echo "  cd $(pwd)"
    echo "  ./bin/cblog"
    echo ""
    echo "访问地址: https://localhost:443"
    echo ""
    echo "其他命令:"
    echo "  make clean      - 清理构建文件"
    echo "  make debug      - 调试模式编译"
    echo "  make size       - 分析二进制大小"
    echo ""
}

# 主流程
main() {
    check_dependencies
    setup_bearssl
    generate_certs
    download_sqlite
    create_directories
    build_project
    init_database
    show_result
}

# 运行
main "$@"