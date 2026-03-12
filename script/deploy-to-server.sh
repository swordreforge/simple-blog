#!/bin/bash
# 部署脚本 - 将编译好的二进制文件上传到服务器

set -e

# 配置
SERVER="root@iZbp1fdgr9nq6je870axqjZ"
SERVER_PATH="/var/www/html/rust-test"
LOCAL_BINARY="./target/x86_64-unknown-linux-musl/release/rustblog"

echo "🚀 开始部署到服务器..."
echo "📦 服务器: $SERVER"
echo "📁 路径: $SERVER_PATH"
echo ""

# 检查二进制文件是否存在
if [ ! -f "$LOCAL_BINARY" ]; then
    echo "❌ 错误: 二进制文件不存在: $LOCAL_BINARY"
    echo "请先运行: ./deploy.sh 编译项目"
    exit 1
fi

# 显示文件大小和修改时间
echo "📋 二进制文件信息:"
ls -lh "$LOCAL_BINARY"
echo ""

# 上传二进制文件
echo "📤 上传二进制文件到服务器..."
scp "$LOCAL_BINARY" "$SERVER:$SERVER_PATH/rustblog.new"

echo "📋 备份旧版本..."
ssh "$SERVER" "cd $SERVER_PATH && [ -f rustblog ] && cp rustblog rustblog.backup || true"

echo "🔄 替换为新版本..."
ssh "$SERVER" "cd $SERVER_PATH && mv rustblog.new rustblog && chmod +x rustblog"

echo "✅ 部署完成！"
echo ""
echo "🔧 在服务器上重启服务:"
echo "  ssh $SERVER 'cd $SERVER_PATH && ./rustblog'"