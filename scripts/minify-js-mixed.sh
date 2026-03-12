#!/bin/bash
# 混合压缩方案：独立文件用 ADVANCED，有依赖的文件用 SIMPLE

OUTPUT_DIR="static/dist/js"
mkdir -p "$OUTPUT_DIR"

echo "开始使用混合方案压缩 JS 文件..."

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

# 独立文件（没有跨文件依赖），使用 ADVANCED
ADVANCED_FILES=(
    "about-focus-mode.js"
    "about-inline-2.js"
    "collect-focus-mode.js"
    "ecc-encrypt.js"
    "floating-text.js"
    "login.js"
    "markdown-preview-modal.js"
    "modal-animations.js"
    "music-player.js"
    "passage-focus-mode.js"
    "passage-shortcuts.js"
    "quick-actions.js"
    "virtual-scroll.js"
)

# 有依赖的文件，使用 SIMPLE
SIMPLE_FILES=(
    "about-inline-1.js"
    "admin-4730.js"
    "admin-inline-1.js"
    "admin-inline-2.js"
    "admin-inline-4.js"
    "admin-inline.js"
    "chart.js"
    "filemanager.js"
    "keyboard-shortcuts.js"
)

# 压缩 ADVANCED 级别的文件
echo ""
echo "=== ADVANCED 级别压缩（独立文件）==="
for filename in "${ADVANCED_FILES[@]}"; do
    file="templates/js/$filename"
    if [ -f "$file" ]; then
        output_file="$OUTPUT_DIR/${filename%.js}.min.js"
        
        echo "压缩: $filename -> $output_file"
        
        npx google-closure-compiler \
            --compilation_level ADVANCED_OPTIMIZATIONS \
            --language_out ECMASCRIPT_2015 \
            --externs /tmp/externs.js \
            --js="$file" \
            --js_output_file="$output_file" \
            --warning_level QUIET \
            2>&1 | head -5
        
        if [ ${PIPESTATUS[0]} -eq 0 ]; then
            original_size=$(wc -c < "$file")
            compressed_size=$(wc -c < "$output_file")
            echo "  ✓ 原始: $original_size 字节 -> 压缩: $compressed_size 字节"
        else
            echo "  ✗ 压缩失败: $filename，尝试 SIMPLE..."
            rm -f "$output_file"
            npx google-closure-compiler \
                --compilation_level SIMPLE_OPTIMIZATIONS \
                --language_out ECMASCRIPT_2015 \
                --js="$file" \
                --js_output_file="$output_file" \
                --warning_level QUIET 2>&1 | head -3
            if [ $? -eq 0 ]; then
                echo "  ✓ SIMPLE 压缩成功"
            fi
        fi
    fi
done

# 压缩 SIMPLE 级别的文件
echo ""
echo "=== SIMPLE 级别压缩（有依赖的文件）==="
for filename in "${SIMPLE_FILES[@]}"; do
    file="templates/js/$filename"
    if [ -f "$file" ]; then
        output_file="$OUTPUT_DIR/${filename%.js}.min.js"
        
        echo "压缩: $filename -> $output_file"
        
        npx google-closure-compiler \
            --compilation_level SIMPLE_OPTIMIZATIONS \
            --language_out ECMASCRIPT_2015 \
            --js="$file" \
            --js_output_file="$output_file" \
            --warning_level QUIET 2>&1 | head -3
        
        if [ $? -eq 0 ]; then
            original_size=$(wc -c < "$file")
            compressed_size=$(wc -c < "$output_file")
            echo "  ✓ 原始: $original_size 字节 -> 压缩: $compressed_size 字节"
        else
            echo "  ✗ 压缩失败: $filename"
        fi
    fi
done

# 清理旧的 advanced.min.js 文件
echo ""
echo "清理旧的 advanced.min.js 文件..."
rm -f static/dist/js/*.advanced.min.js 2>/dev/null

# 生成压缩报告
echo ""
echo "=== 压缩报告 ==="
echo "输出目录: $OUTPUT_DIR"
echo ""
echo "文件列表："
ls -lh "$OUTPUT_DIR"/*.min.js 2>/dev/null | awk '{printf "  %-40s %8s\n", $9, $5}'

echo ""
echo "混合压缩完成！"