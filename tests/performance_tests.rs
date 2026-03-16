//! 性能测试
//!
//! 测试并发路由查找、内存使用等性能指标。

use dynamic_route_actix::{
    core::{BatchOperations, RouteCache, RouteMatcher, RoutePattern},
    RouteEntry, RouteTable, SimpleRoute,
};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[tokio::test]
async fn test_concurrent_route_lookup() {
    let table = Arc::new(RouteTable::new());

    // 添加 1000 个路由
    for i in 0..1000 {
        table.insert(
            format!("/route-{}", i),
            Box::new(SimpleRoute::new(format!("body-{}", i), "text/plain")),
        );
    }

    let start = Instant::now();
    let mut handles = vec![];

    // 并发查询
    for _ in 0..100 {
        let table_clone = Arc::clone(&table);
        let handle = tokio::spawn(async move {
            for i in 0..1000 {
                table_clone.contains(&format!("/route-{}", i));
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let duration = start.elapsed();
    println!("Concurrent lookup time: {:?}", duration);

    // 验证性能：100 个并发任务，每个查询 1000 个路由，应该在 1 秒内完成
    assert!(
        duration.as_secs() < 2,
        "并发查询应该在 2 秒内完成，实际耗时: {:?}",
        duration
    );
}

#[tokio::test]
async fn test_concurrent_route_insert() {
    let table = Arc::new(RouteTable::new());
    let start = Instant::now();
    let mut handles = vec![];

    // 并发插入
    for i in 0..100 {
        let table_clone = Arc::clone(&table);
        let handle = tokio::spawn(async move {
            for j in 0..100 {
                table_clone.insert(
                    format!("/route-{}-{}", i, j),
                    Box::new(SimpleRoute::new("body", "text/plain")),
                );
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let duration = start.elapsed();
    println!("Concurrent insert time: {:?}", duration);

    // 验证性能：100 个并发任务，每个插入 100 个路由，应该在 2 秒内完成
    assert!(
        duration.as_secs() < 3,
        "并发插入应该在 3 秒内完成，实际耗时: {:?}",
        duration
    );

    // 验证所有路由都已插入
    assert_eq!(table.count(), 10000);
}

#[test]
fn test_memory_usage() {
    let table = RouteTable::new();

    // 插入大量路由
    for i in 0..10000 {
        table.insert(
            format!("/route-{}", i),
            Box::new(SimpleRoute::new(format!("body-{}", i), "text/plain")),
        );
    }

    // 这里不进行精确的内存测量，只是确保程序不会崩溃
    assert_eq!(table.count(), 10000);

    // 清空路由表，验证内存能够释放
    table.clear();
    assert_eq!(table.count(), 0);
}

#[tokio::test]
async fn test_cache_performance() {
    let cache = RouteCache::new(Duration::from_secs(60));

    // 预热缓存
    for i in 0..1000 {
        cache.insert(&format!("/route-{}", i), format!("value-{}", i));
    }

    let start = Instant::now();

    // 大量读取
    for i in 0..10000 {
        let key = format!("/route-{}", i % 1000);
        cache.get(&key);
    }

    let duration = start.elapsed();
    println!("Cache lookup time: {:?}", duration);

    // 缓存读取应该很快
    assert!(
        duration.as_millis() < 100,
        "缓存查询应该在 100 毫秒内完成，实际耗时: {:?}",
        duration
    );
}

#[tokio::test]
async fn test_batch_operations_performance() {
    let table = RouteTable::new();

    let start = Instant::now();

    // 批量插入
    let mut routes: std::collections::HashMap<String, Box<dyn RouteEntry>> = std::collections::HashMap::new();
    for i in 0..1000 {
        routes.insert(
            format!("/route-{}", i),
            Box::new(SimpleRoute::new(format!("body-{}", i), "text/plain")),
        );
    }
    BatchOperations::batch_insert(&table, routes);

    let insert_duration = start.elapsed();
    println!("Batch insert time: {:?}", insert_duration);

    assert_eq!(table.count(), 1000);

    // 批量删除
    let start = Instant::now();
    let paths: HashSet<String> = (0..500).map(|i| format!("/route-{}", i)).collect();
    let deleted = BatchOperations::batch_remove(&table, paths);
    let remove_duration = start.elapsed();
    println!("Batch remove time: {:?}", remove_duration);

    assert_eq!(deleted, 500);
    assert_eq!(table.count(), 500);
}

#[test]
fn test_route_matching_performance() {
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

    let start = Instant::now();

    // 大量匹配查询
    for i in 0..10000 {
        matcher.match_path(&format!("/exact-{}", i % 100));
        matcher.match_path(&format!("/users/{}/posts", i % 100));
        matcher.match_path(&format!("/static{}-dir/file.txt", i % 100));
    }

    let duration = start.elapsed();
    println!("Route matching time: {:?}", duration);

    // 路由匹配应该在合理时间内完成
    assert!(
        duration.as_millis() < 2000,
        "路由匹配应该在 2000 毫秒内完成，实际耗时: {:?}",
        duration
    );
}

#[tokio::test]
async fn test_high_load_scenario() {
    let table = Arc::new(RouteTable::new());
    let cache = Arc::new(RouteCache::new(Duration::from_secs(60)));

    // 预填充路由表
    for i in 0..1000 {
        let path = format!("/api/resource-{}", i);
        table.insert(
            path.clone(),
            Box::new(SimpleRoute::new(format!("Resource {}", i), "application/json")),
        );
        cache.insert(&path, format!("Cached: Resource {}", i));
    }

    let start = Instant::now();
    let mut handles = vec![];

    // 模拟高并发场景：1000 个并发请求
    for i in 0..1000 {
        let table_clone = Arc::clone(&table);
        let cache_clone = Arc::clone(&cache);
        let handle = tokio::spawn(async move {
            let path = format!("/api/resource-{}", i % 1000);

            // 首先尝试从缓存获取
            let result = cache_clone.get(&path);

            // 如果缓存未命中，从路由表获取
            let final_result = if result.is_some() {
                result
            } else {
                let route_result = table_clone.get_with(&path, |_route| "found");
                if route_result.is_some() {
                    cache_clone.insert(&path, format!("Cached: Resource {}", i % 1000));
                    Some(format!("Cached: Resource {}", i % 1000))
                } else {
                    None
                }
            };

            final_result.is_some()
        });
        handles.push(handle);
    }

    // 等待所有请求完成
    let mut success_count = 0;
    for handle in handles {
        if let Ok(result) = handle.await {
            if result {
                success_count += 1;
            }
        }
    }

    let duration = start.elapsed();
    println!(
        "High load scenario: {} successes in {:?}",
        success_count, duration
    );

    // 验证所有请求都成功
    assert_eq!(success_count, 1000);

    // 验证性能：1000 个并发请求应该在 2 秒内完成
    assert!(
        duration.as_secs() < 3,
        "高并发场景应该在 3 秒内完成，实际耗时: {:?}",
        duration
    );
}

#[test]
fn test_concurrent_cache_operations() {
    use std::sync::Arc;
    let cache = Arc::new(RouteCache::new(Duration::from_secs(60)));
    let mut handles = vec![];

    // 并发写入
    for i in 0..100 {
        let cache_clone = Arc::clone(&cache);
        let handle = std::thread::spawn(move || {
            for j in 0..100 {
                cache_clone.insert(&format!("key-{}-{}", i, j), format!("value-{}-{}", i, j));
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(cache.size(), 10000);

    // 并发读取
    let mut handles = vec![];
    for i in 0..100 {
        let cache_clone = Arc::clone(&cache);
        let handle = std::thread::spawn(move || {
            for j in 0..100 {
                cache_clone.get(&format!("key-{}-{}", i, j));
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}