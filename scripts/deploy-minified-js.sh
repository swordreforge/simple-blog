#!/bin/bash
# 将压缩后的 JS 文件部署到 templates/js 目录

echo "开始部署压缩后的 JS 文件..."

SOURCE_DIR="static/dist/js"
TARGET_DIR="templates/js"

# 确保目标目录存在
mkdir -p "$TARGET_DIR"

# 复制压缩后的文件（替换原始文件）
for file in "$SOURCE_DIR"/*.min.js; do
    if [ -f "$file" ]; then
        # 获取文件名（去掉 .min 后缀）
        basename=$(basename "$file")
        target_name="${basename%.min.js}.js"
        target_file="$TARGET_DIR/$target_name"
        
        echo "复制: $basename -> $target_file"
        cp "$file" "$target_file"
    fi
done

echo ""
echo "部署完成！"
echo "压缩后的文件已复制到 templates/js/ 目录"
echo ""
echo "下一步：重新编译 Rust 项目"
echo "  cargo build --release"