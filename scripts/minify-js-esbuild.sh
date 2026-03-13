#!/bin/bash
# 使用 ESBuild 安全地压缩 JS 文件
# ESBuild 不会过度混淆变量名，避免全局冲突

OUTPUT_DIR="static/dist/js"
mkdir -p "$OUTPUT_DIR"

echo "开始使用 ESBuild 压缩 JS 文件..."
echo "输出目录: $OUTPUT_DIR"
echo ""

# 原始文件目录
INPUT_DIR="templates/js"

# 统计信息
total_original=0
total_compressed=0
compressed_count=0

# 遍历所有 JS 文件（排除已压缩的文件和第三方库）
for file in "$INPUT_DIR"/*.js; do
    if [ -f "$file" ]; then
        filename=$(basename "$file")
        
        # 跳过已经压缩过的文件
        if [[ "$filename" == *.min.js ]]; then
            echo "跳过已压缩文件: $filename"
            continue
        fi
        
        # 跳过第三方库文件
        if [[ "$filename" == npm/* ]]; then
            echo "跳过第三方库: $filename"
            continue
        fi
        
        output_file="$OUTPUT_DIR/${filename%.js}.min.js"
        
        echo "压缩: $filename -> $output_file"
        
        # 使用 ESBuild 压缩（使用 cjs 格式避免模块问题）
        npx esbuild "$file" \
            --minify \
            --target=es2015 \
            --format=cjs \
            --outfile="$output_file" \
            --allow-overwrite \
            --log-level=error \
            --banner:js="/* ESBuild compressed */"
        
        if [ $? -eq 0 ]; then
            original_size=$(wc -c < "$file")
            compressed_size=$(wc -c < "$output_file")
            
            if [ "$original_size" -gt 0 ]; then
                reduction=$(echo "scale=1; (1 - $compressed_size / $original_size) * 100" | bc)
                echo "  ✓ 原始: $original_size 字节 -> 压缩: $compressed_size 字节 (减少 ${reduction}%)"
                
                total_original=$((total_original + original_size))
                total_compressed=$((total_compressed + compressed_size))
                compressed_count=$((compressed_count + 1))
            fi
        else
            echo "  ✗ 压缩失败: $filename"
            # 删除失败的输出文件
            rm -f "$output_file"
        fi
        
        echo ""
    fi
done

echo "=== 压缩报告 ==="
echo "成功压缩: $compressed_count 个文件"
echo "原始大小: $total_original 字节"
echo "压缩后: $total_compressed 字节"

if [ $total_original -gt 0 ]; then
    total_reduction=$(echo "scale=1; (1 - $total_compressed / $total_original) * 100" | bc)
    total_saved=$((total_original - total_compressed))
    echo "总压缩率: ${total_reduction}%"
    echo "节省: $total_saved 字节"
fi

echo ""
echo "✅ ESBuild 压缩完成！"
echo ""
echo "提示:"
echo "  - ESBuild 使用安全的压缩策略，不会过度混淆变量名"
echo "  - 每个文件独立处理，避免全局变量冲突"
echo "  - 保持代码可读性和可调试性"
echo "  - 压缩文件保存在: $OUTPUT_DIR"