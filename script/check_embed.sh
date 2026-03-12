#!/bin/bash
# 检查嵌入文件

echo "=== 检查当前目录的文件结构 ==="
echo "当前目录: $(pwd)"
echo ""
echo "templates 目录:"
ls -la templates/*.html 2>/dev/null | head -5
echo ""
echo "img 目录:"
ls -la img/*.webp 2>/dev/null | head -5
echo ""
echo "music 目录:"
ls -la music/*.mp3 2>/dev/null | head -5
echo ""
echo "=== 编译并测试 ==="
cargo build --release --target x86_64-unknown-linux-musl 2>&1 | grep -E "(Compiling|Finished|error)"
echo ""
echo "=== 检查二进制文件 ==="
ls -lh target/x86_64-unknown-linux-musl/release/rustblog
echo ""
echo "=== 测试运行 ==="
rm -rf /tmp/test-embed && mkdir -p /tmp/test-embed/data
timeout 5 ./target/x86_64-unknown-linux-musl/release/rustblog -p 9999 --db-path /tmp/test-embed/data/blog.db 2>&1 | grep -E "(调试|总共嵌入|成功加载)" || echo "测试完成或超时"