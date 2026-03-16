//! 专门用于性能分析的路由匹配基准测试

use dynamic_route_actix::core::{RouteMatcher, RoutePattern};
use std::time::Instant;

fn main() {
    let mut matcher = RouteMatcher::new();

    // 添加各种类型的路由模式
    for i in 0..100 {
        matcher.add_pattern(RoutePattern::Exact(format!("/exact-{}", i)));
        matcher.add_pattern(RoutePattern::Parameterized {
            pattern: format!("/users/{}/posts", i),
            param_names: vec!["id".to_string()],
        });
        matcher.add_pattern(RoutePattern::Wildcard {
            prefix: format!("/static{}-dir/", i),
            capture_name: None,
        });
    }

    // 预热
    for i in 0..100 {
        matcher.match_path(&format!("/exact-{}", i));
        matcher.match_path(&format!("/users/{}/posts", i));
        matcher.match_path(&format!("/static{}-dir/file.txt", i));
    }

    println!("🚀 开始性能测试...\n");

    // 性能测试1：基础匹配
    let start = Instant::now();
    let iterations = 10000;

    for i in 0..iterations {
        matcher.match_path(&format!("/exact-{}", i % 100));
        matcher.match_path(&format!("/users/{}/posts", i % 100));
        matcher.match_path(&format!("/static{}-dir/file.txt", i % 100));
    }

    let duration = start.elapsed();
    let total_matches = iterations * 3;
    let avg_time_ns = duration.as_nanos() as f64 / total_matches as f64;

    println!("📊 基础匹配测试:");
    println!("  总匹配次数: {}", total_matches);
    println!("  总耗时: {:?}", duration);
    println!("  平均每次匹配时间: {:.2} ns", avg_time_ns);
    println!("  每秒匹配次数: {:.0}", total_matches as f64 / duration.as_secs_f64());

    // 性能测试2：最佳匹配查找
    let start = Instant::now();
    let mut successful_matches = 0;

    for i in 0..iterations {
        if matcher.find_best_match(&format!("/exact-{}", i % 100)).is_some() {
            successful_matches += 1;
        }
        if matcher.find_best_match(&format!("/users/{}/posts", i % 100)).is_some() {
            successful_matches += 1;
        }
        if matcher.find_best_match(&format!("/static{}-dir/file.txt", i % 100)).is_some() {
            successful_matches += 1;
        }
    }

    let duration = start.elapsed();
    let avg_time_ns = duration.as_nanos() as f64 / total_matches as f64;

    println!("\n📊 最佳匹配测试:");
    println!("  总查找次数: {}", total_matches);
    println!("  成功匹配: {}", successful_matches);
    println!("  总耗时: {:?}", duration);
    println!("  平均每次查找时间: {:.2} ns", avg_time_ns);
    println!("  每秒查找次数: {:.0}", total_matches as f64 / duration.as_secs_f64());

    // 性能测试3：并发匹配
    println!("\n📊 并发匹配测试:");
    println!("  注意：RouteMatcher当前不支持Clone，跳过并发测试");

    println!("\n✅ 性能测试完成!");
}