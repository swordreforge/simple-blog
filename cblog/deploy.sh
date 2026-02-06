#!/bin/bash
# deploy.sh - 部署脚本

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# 默认配置
DEPLOY_DIR="/opt/rustblog"
SERVICE_NAME="rustblog"
USER="www-data"

# 解析参数
while [[ $# -gt 0 ]]; do
    case $1 in
        --dir)
            DEPLOY_DIR="$2"
            shift 2
            ;;
        --user)
            USER="$2"
            shift 2
            ;;
        --service)
            SERVICE_NAME="$2"
            shift 2
            ;;
        --help)
            echo "用法: $0 [选项]"
            echo ""
            echo "选项:"
            echo "  --dir DIR      部署目录 (默认: /opt/rustblog)"
            echo "  --user USER    运行用户 (默认: www-data)"
            echo "  --service NAME 服务名称 (默认: rustblog)"
            echo "  --help         显示帮助"
            exit 0
            ;;
        *)
            echo -e "${RED}错误: 未知参数 $1${NC}"
            exit 1
            ;;
    esac
done

echo "========================================"
echo "  RustBlog C语言版本 - 部署脚本"
echo "========================================"
echo ""
echo "部署目录: $DEPLOY_DIR"
echo "运行用户: $USER"
echo "服务名称: $SERVICE_NAME"
echo ""

# 检查 root 权限
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}错误: 需要 root 权限${NC}"
    exit 1
fi

# 检查二进制文件
if [ ! -f "bin/cblog" ]; then
    echo -e "${RED}错误: 未找到可执行文件 bin/cblog${NC}"
    echo "请先运行 ./build.sh 构建项目"
    exit 1
fi

# 创建部署目录
echo "创建部署目录..."
mkdir -p "$DEPLOY_DIR"/{data,templates,static,uploads,markdown,logs}
echo -e "${GREEN}✓ 目录创建完成${NC}"

# 复制文件
echo "复制文件..."
cp bin/cblog "$DEPLOY_DIR/"
cp config.json "$DEPLOY_DIR/" 2>/dev/null || true

if [ -d "templates" ]; then
    cp -r templates/* "$DEPLOY_DIR/templates/" 2>/dev/null || true
fi

if [ -d "static" ]; then
    cp -r static/* "$DEPLOY_DIR/static/" 2>/dev/null || true
fi

if [ -f "data/cert.der" ]; then
    cp data/cert.der "$DEPLOY_DIR/data/"
fi

if [ -f "data/key.der" ]; then
    cp data/key.der "$DEPLOY_DIR/data/"
fi

echo -e "${GREEN}✓ 文件复制完成${NC}"

# 设置权限
echo "设置权限..."
chown -R $USER:$USER "$DEPLOY_DIR"
chmod 755 "$DEPLOY_DIR/cblog"
chmod 600 "$DEPLOY_DIR/data/key.der"
chmod 644 "$DEPLOY_DIR/data/cert.der"
echo -e "${GREEN}✓ 权限设置完成${NC}"

# 创建 systemd 服务
echo "创建 systemd 服务..."
cat > /etc/systemd/system/$SERVICE_NAME.service <<EOF
[Unit]
Description=RustBlog C Server
After=network.target

[Service]
Type=simple
User=$USER
WorkingDirectory=$DEPLOY_DIR
ExecStart=$DEPLOY_DIR/cblog
Restart=on-failure
RestartSec=5

# 安全加固
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=$DEPLOY_DIR

# 资源限制
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
echo -e "${GREEN}✓ 服务创建完成${NC}"

# 初始化数据库
if [ ! -f "$DEPLOY_DIR/data/blog.db" ]; then
    echo "初始化数据库..."
    su - $USER -s /bin/bash -c "cd $DEPLOY_DIR && ./cblog --init-db"
    echo -e "${GREEN}✓ 数据库初始化完成${NC}"
fi

# 启用并启动服务
echo "启动服务..."
systemctl enable $SERVICE_NAME
systemctl start $SERVICE_NAME

sleep 2

if systemctl is-active --quiet $SERVICE_NAME; then
    echo -e "${GREEN}✓ 服务启动成功${NC}"
else
    echo -e "${RED}✗ 服务启动失败${NC}"
    systemctl status $SERVICE_NAME
    exit 1
fi

# 显示结果
echo ""
echo "========================================"
echo "  部署完成！"
echo "========================================"
echo ""
echo "服务状态: systemctl status $SERVICE_NAME"
echo "查看日志: journalctl -u $SERVICE_NAME -f"
echo "停止服务: systemctl stop $SERVICE_NAME"
echo "重启服务: systemctl restart $SERVICE_NAME"
echo ""
echo "访问地址: https://localhost:443"
echo ""