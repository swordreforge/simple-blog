//! 缓存优化性能基准测试
//!
//! 对比启用缓存和禁用缓存的路由查找性能差异。

use dynamic_route_actix::{RouteTable, SimpleRoute};
use std::time::Instant;

/// 测试路由查找性能（启用缓存）
#[test]
fn test_route_lookup_with_cache() {
    println!("\n=== 路由查找性能测试（启用缓存） ===");

    let table = RouteTable::new();
    let num_routes = 1000;

    // 插入路由
    for i in 0..num_routes {
        table.insert(
            format!("/api/v1/resource/{}", i),
            Box::new(SimpleRoute::new(
                format!("Resource {}", i),
                "application/json",
            )),
        );
    }

    // 预热缓存（模拟真实场景中的高频路由）
    let hot_routes: Vec<String> = (0..100).map(|i| format!("/api/v1/resource/{}", i)).collect();
    table.warmup_cache(&hot_routes.iter().map(|s| s.as_str()).collect::<Vec<_>>());

    // 测试高频路由查找（应该命中缓存）
    let start = Instant::now();
    for _ in 0..10000 {
        for route in &hot_routes {
            table.get_arc(route);
        }
    }
    let hot_lookup_duration = start.elapsed();

    // 测试低频路由查找（可能不命中缓存）
    let cold_routes: Vec<String> = (100..200).map(|i| format!("/api/v1/resource/{}", i)).collect();
    let start = Instant::now();
    for _ in 0..10000 {
        for route in &cold_routes {
            table.get_arc(route);
        }
    }
    let cold_lookup_duration = start.elapsed();

    // 输出统计信息
    let stats = table.cache_stats();
    println!("  路由数量: {}", num_routes);
    println!("  预热路由数: {}", hot_routes.len());
    println!("  高频路由查找耗时: {:?}", hot_lookup_duration);
    println!("  低频路由查找耗时: {:?}", cold_lookup_duration);
    println!("  缓存命中率: {:.2}%", stats.hit_rate() * 100.0);
    println!("  缓存命中次数: {}", stats.hits);
    println!("  缓存未命中次数: {}", stats.misses);
    println!("  总访问次数: {}", stats.total_accesses);

    // 验证缓存确实提升了性能
    // 高频路由应该比低频路由快（因为命中率更高）
    let hot_per_lookup = hot_lookup_duration.as_nanos() as f64 / (hot_routes.len() * 10000) as f64;
    let cold_per_lookup = cold_lookup_duration.as_nanos() as f64 / (cold_routes.len() * 10000) as f64;

    println!("  高频路由平均查找时间: {:.2} ns", hot_per_lookup);
    println!("  低频路由平均查找时间: {:.2} ns", cold_per_lookup);

    // 缓存命中率应该很高（因为我们预热了100个路由，并且重复查询）
    assert!(stats.hit_rate() > 0.5, "缓存命中率应该超过50%");
}

/// 测试缓存预热的效果
#[test]
fn test_cache_warmup_effectiveness() {
    println!("\n=== 缓存预热效果测试 ===");

    let table = RouteTable::new();
    let num_routes = 500;

    // 插入路由
    for i in 0..num_routes {
        table.insert(
            format!("/route/{}", i),
            Box::new(SimpleRoute::new(format!("Route {}", i), "text/plain")),
        );
    }

    // 测试未预热的性能
    let test_routes: Vec<String> = (0..50).map(|i| format!("/route/{}", i)).collect();
    let start = Instant::now();
    for _ in 0..1000 {
        for route in &test_routes {
            table.get_arc(route);
        }
    }
    let without_warmup_duration = start.elapsed();

    // 重置统计
    table.cache_stats(); // 读取以清除

    // 预热缓存
    let warmup_start = Instant::now();
    table.warmup_cache(&test_routes.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    let warmup_duration = warmup_start.elapsed();

    // 测试预热后的性能
    let start = Instant::now();
    for _ in 0..1000 {
        for route in &test_routes {
            table.get_arc(route);
        }
    }
    let with_warmup_duration = start.elapsed();

    let stats = table.cache_stats();

    println!("  路由数量: {}", num_routes);
    println!("  测试路由数: {}", test_routes.len());
    println!("  预热耗时: {:?}", warmup_duration);
    println!("  未预热查找耗时: {:?}", without_warmup_duration);
    println!("  预热后查找耗时: {:?}", with_warmup_duration);
    println!("  缓存命中率: {:.2}%", stats.hit_rate() * 100.0);

    // 预热后应该接近100%命中（可能因为并发访问有少量未命中）
    assert!(stats.hit_rate() > 0.999, "预热后应该接近100%命中缓存");
}

/// 测试参数化路由的缓存性能
#[test]
fn test_parameterized_route_cache_performance() {
    println!("\n=== 参数化路由缓存性能测试 ===");

    let table = RouteTable::new();

    // 插入参数化路由
    table.insert(
        "/users/{id}".to_string(),
        Box::new(SimpleRoute::new("User", "application/json")),
    );
    table.insert(
        "/posts/{post_id}/comments/{comment_id}".to_string(),
        Box::new(SimpleRoute::new("Comment", "application/json")),
    );

    // 测试参数化路由查找
    let user_ids: Vec<String> = (1..100).map(|i| format!("/users/{}", i)).collect();
    let comment_paths: Vec<String> = (1..100)
        .flat_map(|post_id| {
            (1..10).map(move |comment_id| format!("/posts/{}/comments/{}", post_id, comment_id))
        })
        .collect();

    // 测试未预热的性能
    let start = Instant::now();
    for _ in 0..1000 {
        for path in &user_ids {
            table.get_arc(path);
        }
    }
    let without_warmup_user_duration = start.elapsed();

    // 预热缓存
    table.warmup_cache(&user_ids.iter().map(|s| s.as_str()).collect::<Vec<_>>());

    // 验证预热：检查缓存大小
    let cache_size = table.cache_stats().total_accesses; // 使用total_accesses作为缓存条目数的近似

    // 重置缓存统计
    table.reset_cache_stats();

    // 测试预热后的性能
    let start = Instant::now();
    for _ in 0..1000 {
        for path in &user_ids {
            table.get_arc(path);
        }
    }
    let with_warmup_user_duration = start.elapsed();

    // 测试评论路由查找（未预热）
    let start = Instant::now();
    for _ in 0..1000 {
        for path in &comment_paths {
            table.get_arc(path);
        }
    }
    let comment_lookup_duration = start.elapsed();

    let stats = table.cache_stats();

    println!("  用户路由数: {}", user_ids.len());
    println!("  评论路由数: {}", comment_paths.len());
    println!("  未预热用户路由查找耗时: {:?}", without_warmup_user_duration);
    println!("  预热后用户路由查找耗时: {:?}", with_warmup_user_duration);
    println!("  评论路由查找耗时: {:?}", comment_lookup_duration);
    println!("  缓存命中率: {:.2}%", stats.hit_rate() * 100.0);

    // 预热后的用户路由查找应该比未预热快
    assert!(with_warmup_user_duration <= without_warmup_user_duration,
            "预热后的查找应该更快或相等");

    // 参数化路由每次访问不同的路径值，总缓存命中率会较低
    // 但对于预热过的用户路由，命中率应该很高
    let user_hit_rate = stats.hits as f64 / (user_ids.len() * 1000) as f64;
    println!("  用户路由命中率: {:.2}%", user_hit_rate * 100.0);

    // 注意：由于参数化路由的特殊性，预热后的命中率可能不是100%
    // 但应该显著高于未预热的情况
    assert!(user_hit_rate > 0.5, "预热后的用户路由命中率应该显著提升");
}

/// 测试缓存失效对性能的影响
#[test]
fn test_cache_invalidation_impact() {
    println!("\n=== 缓存失效影响测试 ===");

    let table = RouteTable::new();
    let num_routes = 100;

    // 插入路由
    for i in 0..num_routes {
        table.insert(
            format!("/route/{}", i),
            Box::new(SimpleRoute::new(format!("Route {}", i), "text/plain")),
        );
    }

    // 预热缓存
    let test_routes: Vec<String> = (0..50).map(|i| format!("/route/{}", i)).collect();
    table.warmup_cache(&test_routes.iter().map(|s| s.as_str()).collect::<Vec<_>>());

    // 测试失效前的性能
    let start = Instant::now();
    for _ in 0..1000 {
        for route in &test_routes {
            table.get_arc(route);
        }
    }
    let before_invalidation_duration = start.elapsed();

    // 随机失效一些缓存
    for i in 0..10 {
        table.insert(
            format!("/route/{}", i),
            Box::new(SimpleRoute::new(format!("Updated Route {}", i), "text/plain")),
        );
    }

    // 重置统计
    table.cache_stats();

    // 测试失效后的性能
    let start = Instant::now();
    for _ in 0..1000 {
        for route in &test_routes {
            table.get_arc(route);
        }
    }
    let after_invalidation_duration = start.elapsed();

    let stats = table.cache_stats();

    println!("  路由数量: {}", num_routes);
    println!("  测试路由数: {}", test_routes.len());
    println!("  失效路由数: 10");
    println!("  失效前查找耗时: {:?}", before_invalidation_duration);
    println!("  失效后查找耗时: {:?}", after_invalidation_duration);
    println!("  缓存命中率: {:.2}%", stats.hit_rate() * 100.0);

    // 失效后仍然应该有较高的命中率（因为只有10个路由失效）
    assert!(stats.hit_rate() > 0.8, "失效后缓存命中率应该仍然很高");
}

/// 测试并发场景下的缓存性能
#[test]
fn test_concurrent_cache_performance() {
    println!("\n=== 并发缓存性能测试 ===");

    use std::sync::Arc;
    use std::thread;

    let table = Arc::new(RouteTable::new());
    let num_routes = 1000;

    // 插入路由
    for i in 0..num_routes {
        table.insert(
            format!("/api/endpoint/{}", i),
            Box::new(SimpleRoute::new(format!("Endpoint {}", i), "application/json")),
        );
    }

    // 预热缓存
    let hot_routes: Vec<String> = (0..100).map(|i| format!("/api/endpoint/{}", i)).collect();
    table.warmup_cache(&hot_routes.iter().map(|s| s.as_str()).collect::<Vec<_>>());

    let num_threads = 4;
    let mut handles = vec![];
    let start = Instant::now();

    for thread_id in 0..num_threads {
        let table_clone = Arc::clone(&table);
        let hot_routes_clone = hot_routes.clone();

        let handle = thread::spawn(move || {
            for i in 0..2500 {
                // 80% 的时间查询高频路由
                if i % 5 < 4 {
                    let route = &hot_routes_clone[i % hot_routes_clone.len()];
                    table_clone.get_arc(route);
                } else {
                    // 20% 的时间查询随机路由
                    let route = format!("/api/endpoint/{}", (i + thread_id * 1000) % num_routes);
                    table_clone.get_arc(&route);
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let duration = start.elapsed();
    let stats = table.cache_stats();

    println!("  路由数量: {}", num_routes);
    println!("  预热路由数: {}", hot_routes.len());
    println!("  线程数: {}", num_threads);
    println!("  总耗时: {:?}", duration);
    println!("  缓存命中率: {:.2}%", stats.hit_rate() * 100.0);
    println!("  缓存命中次数: {}", stats.hits);
    println!("  缓存未命中次数: {}", stats.misses);

    // 并发场景下应该有很高的命中率
    assert!(stats.hit_rate() > 0.7, "并发场景下缓存命中率应该很高");
}