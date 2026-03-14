#!/bin/bash
# 使用 Terser 安全地压缩 JS 文件，避免变量名冲突
# 适用于多个JS文件在同一页面使用的场景

OUTPUT_DIR="static/dist/js"
mkdir -p "$OUTPUT_DIR"

echo "开始使用 Terser 安全压缩 JS 文件..."
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
        
        # 使用 Terser 进行压缩
        # 关键配置：
        # - toplevel=false: 不混淆顶层变量，避免全局变量冲突
        # - properties=false: 不混淆属性名
        # - reserved: 保留常见全局对象
        npx terser "$file" \
            --compress "ecma=2015,passes=2,drop_console=false,keep_fargs=false,unsafe=false" \
            --mangle "toplevel=false,properties=false,reserved=['console','window','document','navigator','fetch','localStorage','sessionStorage','XMLHttpRequest','FormData','Event','HTMLElement','Node','Element','Array','Object','String','Number','Boolean','Date','Math','JSON','Promise','setTimeout','setInterval','clearTimeout','clearInterval','requestAnimationFrame','cancelAnimationFrame']" \
            --output "$output_file" \
            --ecma 2015 \
            --format "comments=false,ascii_only=false,beautify=false,quote_style=0" \
            --parse "ecma=2023"
        
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
echo "✅ Terser 安全压缩完成！"
echo ""
echo "提示:"
echo "  - 使用 Terser 安全压缩策略，避免变量名冲突"
echo "  - toplevel=false: 不混淆顶层变量，避免全局变量冲突"
echo "  - properties=false: 不混淆属性名"
echo "  - 保留常见全局对象和API"
echo "  - 每个文件独立处理，确保多文件共存"
echo "  - 压缩文件保存在: $OUTPUT_DIR"
