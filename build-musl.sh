#!/bin/bash

set -e

# 颜色输出
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}🔨 编译 RustBlog (musl 静态链接)${NC}"
echo "================================"

# 选择构建策略
STRATEGY=${1:-"auto"}

build_with_docker() {
    echo -e "${YELLOW}📦 使用 Docker 构建...${NC}"
    if ! command -v docker &> /dev/null; then
        echo -e "${RED}❌ Docker 未安装${NC}"
        return 1
    fi
    
    docker run --rm -v "$(pwd)":/src -w /src rustembedded/cross:x86_64-unknown-linux-musl \
        bash -c "rustup target add x86_64-unknown-linux-musl && cargo build --release --target x86_64-unknown-linux-musl"
}

build_with_podman() {
    echo -e "${YELLOW}📦 使用 Podman 构建...${NC}"
    if ! command -v podman &> /dev/null; then
        echo -e "${RED}❌ Podman 未安装${NC}"
        return 1
    fi
    
    podman run --rm -v "$(pwd)":/src -w /src rustembedded/cross:x86_64-unknown-linux-musl \
        bash -c "rustup target add x86_64-unknown-linux-musl && cargo build --release --target x86_64-unknown-linux-musl"
}

build_locally() {
    echo -e "${YELLOW}📦 本地构建...${NC}"
    
    # 检查 musl 目标
    if ! rustup target list | grep -q "x86_64-unknown-linux-musl"; then
        echo "📦 安装 musl 目标..."
        rustup target add x86_64-unknown-linux-musl
    fi
    
    # 检查 musl-gcc
    if ! which musl-gcc &> /dev/null; then
        echo -e "${YELLOW}⚠️  musl-gcc 未找到，尝试安装 musl-tools...${NC}"
        if command -v pacman &> /dev/null; then
            sudo pacman -S --noconfirm musl openssl 2>/dev/null || true
        elif command -v apt-get &> /dev/null; then
            sudo apt-get install -y musl-tools musl-dev libssl-dev 2>/dev/null || true
        elif command -v yum &> /dev/null; then
            sudo yum install -y musl-devel openssl-devel 2>/dev/null || true
        fi
    fi
    
    # 检查 musl-gcc 是否存在
    if ! which musl-gcc &> /dev/null; then
        echo -e "${RED}❌ musl-gcc 未找到${NC}"
        echo "请手动安装 musl-tools: sudo pacman -S musl-tools"
        return 1
    fi
    
    # 设置环境变量
    export PKG_CONFIG_ALLOW_CROSS=1
    export PKG_CONFIG_PATH=/usr/lib/x86_64-linux-gnu/pkgconfig
    
    # 尝试构建
    echo "🔨 开始编译..."
    echo "环境变量: PKG_CONFIG_ALLOW_CROSS=$PKG_CONFIG_ALLOW_CROSS"
    echo "PKG_CONFIG_PATH=$PKG_CONFIG_PATH"
    
    env PKG_CONFIG_ALLOW_CROSS=1 PKG_CONFIG_PATH=/usr/lib/x86_64-linux-gnu/pkgconfig \
        cargo build --release --target x86_64-unknown-linux-musl --no-default-features 2>&1
}

build_with_alpine() {
    echo -e "${YELLOW}🐳 使用 Alpine 容器构建...${NC}"
    if ! command -v docker &> /dev/null; then
        echo -e "${RED}❌ Docker 未安装${NC}"
        return 1
    fi
    
    docker run --rm -v "$(pwd)":/src -w /src alpine:latest \
        sh -c "apk add --no-cache rust cargo musl-dev openssl-dev && cargo build --release --target x86_64-unknown-linux-musl"
}

# 执行构建
case $STRATEGY in
    "docker")
        build_with_docker
        ;;
    "podman")
        build_with_podman
        ;;
    "alpine")
        build_with_alpine
        ;;
    "local")
        build_locally
        ;;
    "auto")
        echo "🔍 自动检测构建环境..."
        if command -v docker &> /dev/null; then
            echo "📦 Docker 可用，使用 Docker 构建"
            build_with_docker
        elif command -v podman &> /dev/null; then
            echo "📦 Podman 可用，使用 Podman 构建"
            build_with_podman
        else
            echo "📦 本地构建"
            build_locally
        fi
        ;;
    *)
        echo "❌ 未知的构建策略: $STRATEGY"
        echo "用法: $0 [docker|podman|alpine|local|auto]"
        exit 1
        ;;
esac

# 检查编译结果
if [ $? -eq 0 ]; then
    echo ""
    echo -e "${GREEN}✅ 编译成功!${NC}"
    echo ""
    echo "📦 生成的二进制文件位置:"
    ls -lh target/x86_64-unknown-linux-musl/release/rustblog
    echo ""
    echo "📊 文件大小:"
    ls -lh target/x86_64-unknown-linux-musl/release/rustblog | awk '{print $5}'
    echo ""
    echo "🔍 验证 musl 链接:"
    file target/x86_64-unknown-linux-musl/release/rustblog
    echo ""
    echo "🚀 运行命令:"
    echo "  ./target/x86_64-unknown-linux-musl/release/rustblog --help"
    echo ""
    echo "💡 如果需要在 musl 系统上运行，请确保目标系统有 musl 库"
else
    echo ""
    echo -e "${RED}❌ 编译失败!${NC}"
    echo ""
    echo "💡 查看 MUSL_BUILD_GUIDE.md 获取更多信息"
    exit 1
fi