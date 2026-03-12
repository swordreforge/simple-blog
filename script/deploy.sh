#!/bin/bash
# RustBlog 部署脚本
# 用于编译并部署到指定服务器

set -e

echo "🔨 开始编译 RustBlog..."
echo "⚠️  重要: 必须从项目根目录编译！"

# 检查当前目录
if [ ! -f "Cargo.toml" ]; then
    echo "❌ 错误: 当前目录不是项目根目录 (未找到 Cargo.toml)"
    echo "请先执行: cd /home/swordreforge/project/rustblog-new/rustblog"
    exit 1
fi

echo "✅ 当前目录正确，开始编译..."

# 清理并编译
cargo clean
cargo build --release --target x86_64-unknown-linux-musl

echo "✅ 编译完成！"
echo ""
echo "📦 编译产物: target/x86_64-unknown-linux-musl/release/rustblog"
echo ""
echo "🚀 部署到服务器示例:"
echo "  1. 复制二进制文件: scp target/x86_64-unknown-linux-musl/release/rustblog user@server:/var/www/html/rust-test/"
echo "  2. 创建目录: ssh user@server 'mkdir -p /var/www/html/rust-test/{data,attachments,markdown}'"
echo "  3. 启动服务: ssh user@server 'cd /var/www/html/rust-test && ./rustblog -p 8080'"