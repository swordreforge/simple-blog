#!/bin/bash
# 使用 Terser 压缩 templates/js 中的所有 JS 文件

OUTPUT_DIR="static/dist/js"
mkdir -p "$OUTPUT_DIR"

echo "开始使用 Terser 压缩 JS 文件..."
echo "输出目录: $OUTPUT_DIR"
echo ""

# 遍历 templates/js 目录中的所有 .js 文件
for file in templates/js/*.js; do
    if [ -f "$file" ]; then
        filename=$(basename "$file")
        
        # 跳过已经压缩过的文件
        if [[ "$filename" == *.min.js ]]; then
            echo "跳过已压缩文件: $filename"
            continue
        fi
        
        output_file="$OUTPUT_DIR/${filename%.js}.min.js"
        
        echo "压缩: $filename -> $output_file"
        
        # 使用 Terser 进行压缩和语法检查
        npx terser "$file" \
            --compress "ecma=2015,warnings=true,passes=2" \
            --mangle "toplevel=false,properties=false,reserved=['console','window','document','navigator','fetch','localStorage','sessionStorage']" \
            --output "$output_file" \
            --ecma 2015 \
            --source-map "includeSources=true,url='${filename%.js}.min.js.map'" \
            --format "comments=false,ascii_only=true,beautify=false" \
            --parse "ecma=2023"
        
        if [ $? -eq 0 ]; then
            original_size=$(wc -c < "$file")
            compressed_size=$(wc -c < "$output_file")
            map_size=$(wc -c < "$output_file.map" 2>/dev/null || echo "0")
            
            if [ "$original_size" -gt 0 ]; then
                reduction=$(echo "scale=1; (1 - $compressed_size / $original_size) * 100" | bc)
                echo "  ✓ 原始: $original_size 字节 -> 压缩: $compressed_size 字节 (减少 ${reduction}%)"
                echo "    Source Map: $map_size 字节"
            fi
        else
            echo "  ✗ 压缩失败: $filename"
            # 删除失败的输出文件
            rm -f "$output_file" "$output_file.map"
        fi
        
        echo ""
    fi
done

echo "=== 压缩报告 ==="
echo "输出目录: $OUTPUT_DIR"
echo ""
echo "压缩文件列表："
compressed_files=("$OUTPUT_DIR"/*.min.js)
if [ ${#compressed_files[@]} -gt 0 ] && [ -f "${compressed_files[0]}" ]; then
    ls -lh "$OUTPUT_DIR"/*.min.js | awk '{printf "  %-40s %8s\n", $9, $5}'
else
    echo "  没有生成压缩文件"
fi

echo ""
echo "=== 总计统计 ==="
total_original=0
total_compressed=0

for file in templates/js/*.js; do
    if [ -f "$file" ]; then
        filename=$(basename "$file")
        if [[ "$filename" != *.min.js ]]; then
            size=$(wc -c < "$file")
            total_original=$((total_original + size))
        fi
    fi
done

for file in "$OUTPUT_DIR"/*.min.js; do
    if [ -f "$file" ]; then
        size=$(wc -c < "$file")
        total_compressed=$((total_compressed + size))
    fi
done

if [ "$total_original" -gt 0 ]; then
    total_reduction=$(echo "scale=1; (1 - $total_compressed / $total_original) * 100" | bc)
    echo "  原始总计: $total_original 字节"
    echo "  压缩总计: $total_compressed 字节"
    echo "  总减少: ${total_reduction}%"
fi

echo ""
echo "Terser 压缩完成！"