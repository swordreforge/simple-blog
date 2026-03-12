#!/bin/bash
# 使用 Google Closure Compiler SIMPLE 级别压缩 templates/js 中的所有 JS 文件

OUTPUT_DIR="static/dist/js"
mkdir -p "$OUTPUT_DIR"

echo "开始压缩 JS 文件..."

# 遍历 templates/js 目录中的所有 .js 文件
for file in templates/js/*.js; do
    if [ -f "$file" ]; then
        filename=$(basename "$file")
        output_file="$OUTPUT_DIR/${filename%.js}.min.js"
        
        echo "压缩: $filename -> $output_file"
        
        npx google-closure-compiler \
            --compilation_level SIMPLE_OPTIMIZATIONS \
            --language_out ECMASCRIPT_2015 \
            --js="$file" \
            --js_output_file="$output_file"
        
        if [ $? -eq 0 ]; then
            original_size=$(wc -c < "$file")
            compressed_size=$(wc -c < "$output_file")
            reduction=$(echo "scale=1; (1 - $compressed_size / $original_size) * 100" | bc)
            echo "  原始: $original_size 字节 -> 压缩: $compressed_size 字节 (减少 ${reduction}%)"
        else
            echo "  压缩失败: $filename"
        fi
    fi
done

echo "压缩完成！"