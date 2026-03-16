#!/bin/bash
# 性能基准测试脚本

echo "🚀 开始性能基准测试..."
echo ""

echo "📊 基础匹配测试"
cargo test --release --test performance_tests -- --nocapture test_route_matching_performance
echo ""

echo "📊 并发查找测试"
cargo test --release --test performance_tests -- --nocapture test_concurrent_route_lookup
echo ""

echo "📊 并发插入测试"
cargo test --release --test performance_tests -- --nocapture test_concurrent_route_insert
echo ""

echo "📊 批量操作测试"
cargo test --release --test performance_tests -- --nocapture test_batch_operations_performance
echo ""

echo "📊 缓存性能测试"
cargo test --release --test performance_tests -- --nocapture test_cache_performance
echo ""

echo "📊 高负载场景测试"
cargo test --release --test performance_tests -- --nocapture test_high_load_scenario
echo ""

echo "✅ 基准测试完成！"