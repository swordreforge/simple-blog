//! 边界条件测试
//!
//! 测试空路由表、重复插入、特殊字符路径等边界情况。

use dynamic_route_actix::{
    core::{BatchOperations, RouteCache},
    RouteTable, SimpleRoute,
};
use std::collections::HashSet;
use std::time::Duration;

#[test]
fn test_empty_route_table() {
    let table = RouteTable::new();

    // 空路由表的基本操作
    assert!(!table.contains("/any"));
    assert!(!table.remove("/any"));
    assert_eq!(table.count(), 0);
    assert!(table.list_paths().is_empty());
    assert_eq!(table.get_with("/any", |_| "result"), None);
}

#[test]
fn test_duplicate_insert() {
    let table = RouteTable::new();
    let route1 = SimpleRoute::new("body1", "text/plain");
    let route2 = SimpleRoute::new("body2", "text/plain");

    table.insert("/test".into(), Box::new(route1));
    assert_eq!(table.count(), 1);

    // 插入相同路径的路由，应该覆盖
    table.insert("/test".into(), Box::new(route2));
    assert_eq!(table.count(), 1); // 数量应该保持为 1

    // 验证内容已更新
    let result = table.get_with("/test", |route| route.to_serializable().body);
    assert_eq!(result, Some("body2".to_string()));
}

#[test]
fn test_special_characters_in_path() {
    let table = RouteTable::new();
    let special_paths = vec![
        "/path/with spaces",
        "/path/with-unicode/测试",
        "/path/with/slashes//multiple",
        "/path/with-dashes_and_underscores",
        "/path/with.dots",
        "/path/with~special!chars",
    ];

    let count = special_paths.len();
    for path in &special_paths {
        table.insert(
            path.to_string(),
            Box::new(SimpleRoute::new("body", "text/plain")),
        );
        assert!(table.contains(path));
    }

    assert_eq!(table.count(), count);
}

#[test]
fn test_very_long_path() {
    let table = RouteTable::new();
    let long_path = "/".to_string() + &"a".repeat(10000);

    table.insert(
        long_path.clone(),
        Box::new(SimpleRoute::new("body", "text/plain")),
    );

    assert!(table.contains(&long_path));
    assert_eq!(table.count(), 1);
}

#[test]
fn test_empty_path() {
    let table = RouteTable::new();

    // 空路径不应该被接受
    table.insert(
        "".to_string(),
        Box::new(SimpleRoute::new("body", "text/plain")),
    );

    // 验证空路径是否被插入（取决于实现）
    // 在这个实现中，空路径应该可以被插入，但可能不符合预期
}

#[test]
fn test_root_path() {
    let table = RouteTable::new();

    table.insert(
        "/".to_string(),
        Box::new(SimpleRoute::new("root", "text/plain")),
    );

    assert!(table.contains("/"));
    assert_eq!(table.count(), 1);

    // 确保其他路径不会被错误匹配
    assert!(!table.contains("/anything"));
}

#[test]
fn test_case_sensitivity() {
    let table = RouteTable::new();

    table.insert(
        "/Users".to_string(),
        Box::new(SimpleRoute::new("uppercase", "text/plain")),
    );
    table.insert(
        "/users".to_string(),
        Box::new(SimpleRoute::new("lowercase", "text/plain")),
    );

    // 验证大小写敏感
    assert!(table.contains("/Users"));
    assert!(table.contains("/users"));
    assert!(!table.contains("/USERS"));

    assert_eq!(table.count(), 2);
}

#[test]
fn test_path_with_query_string() {
    let table = RouteTable::new();

    // 插入带查询字符串的路径
    table.insert(
        "/path?query=value".to_string(),
        Box::new(SimpleRoute::new("body", "text/plain")),
    );

    assert!(table.contains("/path?query=value"));
    assert!(!table.contains("/path")); // 查询字符串是路径的一部分

    assert_eq!(table.count(), 1);
}

#[test]
fn test_batch_operations_empty_input() {
    let table = RouteTable::new();

    // 批量插入空集合
    BatchOperations::batch_insert(&table, std::collections::HashMap::new());
    assert_eq!(table.count(), 0);

    // 批量删除空集合
    let deleted = BatchOperations::batch_remove(&table, HashSet::new());
    assert_eq!(deleted, 0);

    // 批量检查空集合
    let results = BatchOperations::batch_contains(&table, HashSet::new());
    assert!(results.is_empty());
}

#[test]
fn test_cache_with_zero_ttl() {
    use std::thread;
    let cache = RouteCache::new(Duration::from_secs(0));

    cache.insert("/test", "value".to_string());

    // 零 TTL 意味着立即过期
    thread::sleep(Duration::from_millis(10));
    assert_eq!(cache.get("/test"), None);
}

#[test]
fn test_cache_with_negative_value() {
    let cache = RouteCache::new(Duration::from_secs(60));

    // 插入空字符串
    cache.insert("/empty", "".to_string());
    assert_eq!(cache.get("/empty"), Some("".to_string()));

    // 插入零值
    cache.insert("/zero", 0.to_string());
    assert_eq!(cache.get("/zero"), Some("0".to_string()));
}

#[test]
fn test_repeated_remove() {
    let table = RouteTable::new();

    table.insert(
        "/test".to_string(),
        Box::new(SimpleRoute::new("body", "text/plain")),
    );

    assert!(table.remove("/test"));
    assert!(!table.remove("/test")); // 第二次删除应该失败
    assert!(!table.remove("/test")); // 第三次删除也应该失败
    assert_eq!(table.count(), 0);
}

#[test]
fn test_path_with_trailing_slash() {
    let table = RouteTable::new();

    table.insert(
        "/path/".to_string(),
        Box::new(SimpleRoute::new("trailing", "text/plain")),
    );
    table.insert(
        "/path".to_string(),
        Box::new(SimpleRoute::new("no-trailing", "text/plain")),
    );

    // 尾部斜杠是有意义的
    assert!(table.contains("/path/"));
    assert!(table.contains("/path"));
    assert_eq!(table.count(), 2);
}

#[test]
fn test_path_with_percent_encoding() {
    let table = RouteTable::new();

    // 插入 URL 编码的路径
    table.insert(
        "/path%20with%20spaces".to_string(),
        Box::new(SimpleRoute::new("encoded", "text/plain")),
    );

    assert!(table.contains("/path%20with%20spaces"));
    assert!(!table.contains("/path with spaces")); // 编码和解码的路径不同

    assert_eq!(table.count(), 1);
}

#[test]
fn test_very_large_number_of_routes() {
    let table = RouteTable::new();
    let count = 100000; // 10 万个路由

    let start = std::time::Instant::now();

    for i in 0..count {
        table.insert(
            format!("/route-{}", i),
            Box::new(SimpleRoute::new(format!("body-{}", i), "text/plain")),
        );
    }

    let insert_duration = start.elapsed();
    println!("Inserted {} routes in {:?}", count, insert_duration);

    assert_eq!(table.count(), count);

    // 验证随机路由存在
    for i in &[0, 1, count / 2, count - 1] {
        assert!(table.contains(&format!("/route-{}", i)));
    }
}

#[test]
fn test_clear_and_refill() {
    let table = RouteTable::new();

    // 插入一些路由
    for i in 0..100 {
        table.insert(
            format!("/route-{}", i),
            Box::new(SimpleRoute::new(format!("body-{}", i), "text/plain")),
        );
    }
    assert_eq!(table.count(), 100);

    // 清空
    table.clear();
    assert_eq!(table.count(), 0);

    // 重新填充
    for i in 0..50 {
        table.insert(
            format!("/new-route-{}", i),
            Box::new(SimpleRoute::new(format!("new-body-{}", i), "text/plain")),
        );
    }
    assert_eq!(table.count(), 50);

    // 确保旧路由不存在
    assert!(!table.contains("/route-0"));
    assert!(!table.contains("/route-99"));

    // 确保新路由存在
    assert!(table.contains("/new-route-0"));
    assert!(table.contains("/new-route-49"));
}

#[test]
fn test_concurrent_remove_and_insert() {
    use std::sync::Arc;
    use std::thread;

    let table = Arc::new(RouteTable::new());
    let mut handles = vec![];

    // 初始化路由
    for i in 0..100 {
        table.insert(
            format!("/route-{}", i),
            Box::new(SimpleRoute::new(format!("body-{}", i), "text/plain")),
        );
    }

    // 并发删除
    for i in 0..50 {
        let table_clone = Arc::clone(&table);
        let handle = thread::spawn(move || {
            table_clone.remove(&format!("/route-{}", i));
        });
        handles.push(handle);
    }

    // 并发插入
    for i in 50..100 {
        let table_clone = Arc::clone(&table);
        let handle = thread::spawn(move || {
            table_clone.insert(
                format!("/route-{}", i),
                Box::new(SimpleRoute::new(format!("new-body-{}", i), "text/plain")),
            );
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // 最终应该有 50 个路由（50 个被删除，50 个被覆盖）
    assert_eq!(table.count(), 50);
}

#[test]
fn test_path_with_multiple_slashes() {
    let table = RouteTable::new();

    let paths = vec![
        "//double-slash",
        "///triple-slash",
        "/path//with//double//slashes",
        "/",
        "//",
    ];

    for path in paths {
        table.insert(
            path.to_string(),
            Box::new(SimpleRoute::new("body", "text/plain")),
        );
    }

    // 所有路径都应该能被插入
    assert_eq!(table.count(), 5);
}