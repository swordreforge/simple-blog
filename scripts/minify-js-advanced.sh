#!/bin/bash
# 使用 Google Closure Compiler ADVANCED 级别压缩 templates/js 中的所有 JS 文件

OUTPUT_DIR="static/dist/js"
mkdir -p "$OUTPUT_DIR"

echo "开始使用 ADVANCED 级别压缩 JS 文件..."

# 创建外部变量声明文件
cat > /tmp/externs.js << 'EOF'
// 浏览器全局变量和 API 声明
var window;
var document;
var navigator;
var console;
var localStorage;
var sessionStorage;
var fetch;
var XMLHttpRequest;
var WebSocket;
var Event;
var HTMLElement;
var HTMLDocument;
var Element;
var Node;
var Array;
var Object;
var String;
var Number;
var Boolean;
var Function;
var Date;
var Math;
var JSON;
var Promise;
var Error;
var TypeError;
var SyntaxError;
var RangeError;
var ReferenceError;
var URIError;
var Set;
var Map;
var WeakMap;
var WeakSet;
var Proxy;
var Reflect;
var Symbol;
var atob;
var btoa;
var setTimeout;
var setInterval;
var clearTimeout;
var clearInterval;
var requestAnimationFrame;
var cancelAnimationFrame;
var Blob;
var FileReader;
var FormData;
var URL;
var URLSearchParams;
var History;
var Location;
var Performance;
var IntersectionObserver;
var MutationObserver;
var ResizeObserver;
EOF

# 遍历 templates/js 目录中的所有 .js 文件（排除已压缩的文件）
for file in templates/js/*.js; do
    if [ -f "$file" ]; then
        filename=$(basename "$file")
        
        # 跳过已经压缩过的文件
        if [[ "$filename" == *.min.js ]]; then
            echo "跳过已压缩文件: $filename"
            continue
        fi
        
        output_file="$OUTPUT_DIR/${filename%.js}.advanced.min.js"
        
        echo "压缩: $filename -> $output_file"
        
        npx google-closure-compiler \
            --compilation_level ADVANCED_OPTIMIZATIONS \
            --language_out ECMASCRIPT_2015 \
            --externs /tmp/externs.js \
            --js="$file" \
            --js_output_file="$output_file" \
            --warning_level QUIET \
            2>&1 | head -20
        
        if [ ${PIPESTATUS[0]} -eq 0 ]; then
            original_size=$(wc -c < "$file")
            compressed_size=$(wc -c < "$output_file")
            echo "  ✓ 原始: $original_size 字节 -> 压缩: $compressed_size 字节"
        else
            echo "  ✗ 压缩失败: $filename"
            # 删除失败的输出文件
            rm -f "$output_file"
        fi
    fi
done

echo ""
echo "压缩完成！"
echo "输出目录: $OUTPUT_DIR"