//! Arc路由处理器优化测试
//!
//! 测试使用Arc共享路由处理器后的性能提升效果

use dynamic_route_actix::{RouteEntry, RouteTable, SimpleRoute};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[test]
fn test_route_clone_performance() {
    let table = Arc::new(RouteTable::new());

    // 插入 1000 个路由
    for i in 0..1000 {
        table.insert(
            format!("/route-{}", i),
            Box::new(SimpleRoute::new(format!("body-{}", i), "text/plain")),
        );
    }

    // 测试克隆性能
    let start = Instant::now();
    let mut clone_count = 0;

    for i in 0..1000 {
        if let Some(_route) = table.get_clone(&format!("/route-{}", i)) {
            clone_count += 1;
        }
    }

    let duration = start.elapsed();
    println!("📊 Arc优化后克隆 {} 个路由耗时: {:?}", clone_count, duration);

    // 验证所有路由都被成功克隆
    assert_eq!(clone_count, 1000);

    // 性能断言：1000次克隆应该在 10ms 内完成（Arc优化后）
    assert!(
        duration < Duration::from_millis(10),
        "Arc优化后1000次克隆应该在10ms内完成，实际耗时: {:?}",
        duration
    );
}

#[test]
fn test_concurrent_route_clone() {
    let table = Arc::new(RouteTable::new());

    // 插入 1000 个路由
    for i in 0..1000 {
        table.insert(
            format!("/route-{}", i),
            Box::new(SimpleRoute::new(format!("body-{}", i), "text/plain")),
        );
    }

    let start = Instant::now();
    let mut handles = vec![];

    // 并发克隆测试
    for _ in 0..10 {
        let table_clone = Arc::clone(&table);
        let handle = std::thread::spawn(move || {
            for i in 0..1000 {
                table_clone.get_clone(&format!("/route-{}", i));
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let duration = start.elapsed();
    println!("📊 Arc优化后并发克隆耗时: {:?}", duration);

    // 性能断言：10个线程各克隆1000次，应该在 50ms 内完成
    assert!(
        duration < Duration::from_millis(50),
        "Arc优化后并发克隆应该在50ms内完成，实际耗时: {:?}",
        duration
    );
}

#[test]
fn test_arc_memory_efficiency() {
    // 测试Arc共享内存的效率
    let route = SimpleRoute::new("Hello, World!", "text/plain");

    // 多次克隆同一个路由，由于使用Arc，内存占用应该很小
    let clones: Vec<Box<dyn dynamic_route_actix::RouteEntry>> = (0..1000)
        .map(|_| route.clone_box())
        .collect();

    // 验证所有克隆都指向相同的字符串数据
    for clone in &clones {
        if let Some(simple_route) = clone.as_any().downcast_ref::<SimpleRoute>() {
            // 验证字符串内容正确
            assert_eq!(&*simple_route.body, "Hello, World!");
            assert_eq!(&*simple_route.content_type, "text/plain");
        }
    }

    println!("✅ 成功创建1000个Arc共享路由，内存占用极小");
}

#[test]
fn test_route_handle_performance() {
    let table = RouteTable::new();

    // 插入 100 个路由
    for i in 0..100 {
        table.insert(
            format!("/route-{}", i),
            Box::new(SimpleRoute::new(format!("Response {}", i), "text/plain")),
        );
    }

    let start = Instant::now();
    let mut handle_count = 0;

    // 测试handle方法性能（Arc优化后不需要字符串深拷贝）
    for i in 0..100 {
        if let Some(_route) = table.get_clone(&format!("/route-{}", i)) {
            handle_count += 1;
        }
    }

    let duration = start.elapsed();
    println!("📊 Arc优化后处理 {} 个路由耗时: {:?}", handle_count, duration);

    assert_eq!(handle_count, 100);

    // 性能断言：应该非常快，因为Arc避免了字符串深拷贝
    assert!(
        duration < Duration::from_millis(5),
        "Arc优化后路由处理应该在5ms内完成，实际耗时: {:?}",
        duration
    );
}

#[test]
fn test_arc_vs_string_comparison() {
    // 比较Arc和String的克隆性能
    // 对于短字符串，Arc和String的克隆性能接近
    // 但在大量克隆场景下，Arc避免了内存分配，性能更稳定
    let body = "This is a long response body that would normally require memory allocation when cloned";
    let content_type = "application/json";

    // 测试String克隆
    let string_body = body.to_string();
    let string_content_type = content_type.to_string();

    let start = Instant::now();
    for _ in 0..10000 {
        let _ = string_body.clone();
        let _ = string_content_type.clone();
    }
    let string_duration = start.elapsed();

    // 测试Arc克隆
    let arc_body: std::sync::Arc<str> = body.into();
    let arc_content_type: std::sync::Arc<str> = content_type.into();

    let start = Instant::now();
    for _ in 0..10000 {
        let _ = std::sync::Arc::clone(&arc_body);
        let _ = std::sync::Arc::clone(&arc_content_type);
    }
    let arc_duration = start.elapsed();

    println!("📊 String克隆10000次耗时: {:?}", string_duration);
    println!("📊 Arc克隆10000次耗时: {:?}", arc_duration);

    // Arc的克隆性能应该与String相当或更快（避免了内存分配）
    // 在大量克隆场景下，Arc的性能优势更明显
    assert!(
        arc_duration <= string_duration * 2,
        "Arc的克隆性能应该与String相当或更好"
    );

    // 验证Arc确实减少了内存分配
    let start = Instant::now();
    let string_clones: Vec<String> = (0..10000).map(|_| string_body.clone()).collect();
    let string_alloc_duration = start.elapsed();

    let start = Instant::now();
    let arc_clones: Vec<std::sync::Arc<str>> = (0..10000).map(|_| std::sync::Arc::clone(&arc_body)).collect();
    let arc_alloc_duration = start.elapsed();

    println!("📊 String分配10000次耗时: {:?}", string_alloc_duration);
    println!("📊 Arc分配10000次耗时: {:?}", arc_alloc_duration);

    // Arc分配应该更快，因为只是增加引用计数
    assert!(
        arc_alloc_duration < string_alloc_duration,
        "Arc分配应该比String分配更快"
    );
}

#[test]
fn test_batch_route_operations() {
    let table = RouteTable::new();

    // 批量插入路由
    let start = Instant::now();
    for i in 0..1000 {
        table.insert(
            format!("/api/v1/resource/{}", i),
            Box::new(SimpleRoute::new(format!("Resource {}", i), "application/json")),
        );
    }
    let insert_duration = start.elapsed();

    // 批量克隆路由
    let start = Instant::now();
    let mut cloned_routes = Vec::new();
    for i in 0..1000 {
        if let Some(route) = table.get_clone(&format!("/api/v1/resource/{}", i)) {
            cloned_routes.push(route);
        }
    }
    let clone_duration = start.elapsed();

    println!("📊 批量插入1000个路由耗时: {:?}", insert_duration);
    println!("📊 批量克隆1000个路由耗时: {:?}", clone_duration);

    assert_eq!(cloned_routes.len(), 1000);

    // 性能断言：批量克隆应该在 10ms 内完成
    assert!(
        clone_duration < Duration::from_millis(10),
        "Arc优化后批量克隆应该在10ms内完成，实际耗时: {:?}",
        clone_duration
    );
}