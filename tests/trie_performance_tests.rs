//! Trie树性能测试
//!
//! 测试Trie树路由索引的性能优化效果。

use dynamic_route_actix::{
    core::{route_matcher::{RouteMatcher, RoutePattern, TrieBasedMatcher}, RouteTable},
    SimpleRoute,
};
use std::time::Instant;

/// 基础Trie匹配性能测试
#[test]
fn test_trie_basic_matching_performance() {
    let mut matcher = TrieBasedMatcher::new();

    // 添加 10000 个路由模式
    for i in 0..10000 {
        matcher.add_pattern(RoutePattern::from(&format!("/users/{}/posts", i)));
    }

    let start = Instant::now();
    let mut match_count = 0;

    // 执行 30000 次匹配
    for i in 0..30000 {
        let idx = i % 10000;
        if matcher.match_path(&format!("/users/{}/posts", idx)).is_some() {
            match_count += 1;
        }
    }

    let duration = start.elapsed();
    println!("📊 Trie基础匹配测试:");
    println!("  总匹配次数: {}", match_count);
    println!("  总耗时: {:?}", duration);
    println!("  平均每次匹配时间: {:.2} ns", duration.as_nanos() as f64 / match_count as f64);
    println!("  每秒匹配次数: {:.0}", match_count as f64 / duration.as_secs_f64());

    // 验证性能：应该在合理时间内完成
    assert!(match_count == 30000, "应该匹配所有路由");
}

/// Trie树与线性搜索性能对比
#[test]
fn test_trie_vs_linear_search_performance() {
    // 测试数据集
    let routes: Vec<String> = (0..1000)
        .map(|i| format!("/api/v1/users/{}/posts/{}", i, i * 2))
        .collect();

    // 1. Trie树匹配器
    let mut trie_matcher = TrieBasedMatcher::new();
    for route in &routes {
        trie_matcher.add_pattern(RoutePattern::from(route));
    }

    let start = Instant::now();
    let mut trie_matches = 0;
    for i in 0..1000 {
        let test_path = format!("/api/v1/users/{}/posts/{}", i, i * 2);
        if trie_matcher.match_path(&test_path).is_some() {
            trie_matches += 1;
        }
    }
    let trie_duration = start.elapsed();

    // 2. 线性搜索匹配器
    let mut linear_matcher = RouteMatcher::new();
    for route in &routes {
        linear_matcher.add_pattern(RoutePattern::from(route));
    }

    let start = Instant::now();
    let mut linear_matches = 0;
    for i in 0..1000 {
        let test_path = format!("/api/v1/users/{}/posts/{}", i, i * 2);
        if !linear_matcher.match_path(&test_path).is_empty() {
            linear_matches += 1;
        }
    }
    let linear_duration = start.elapsed();

    println!("📊 Trie树 vs 线性搜索性能对比:");
    println!("  Trie树匹配次数: {}, 耗时: {:?}", trie_matches, trie_duration);
    println!("  线性搜索匹配次数: {}, 耗时: {:?}", linear_matches, linear_duration);
    println!("  性能提升: {:.2}x", linear_duration.as_nanos() as f64 / trie_duration.as_nanos() as f64);

    // Trie树应该更快
    assert!(trie_duration <= linear_duration, "Trie树应该比线性搜索更快");
}

/// RouteTable的Trie树优化性能测试
#[test]
fn test_route_table_trie_performance() {
    let table = RouteTable::new();

    // 添加 1000 个路由
    for i in 0..1000 {
        table.insert(
            format!("/api/v1/users/{}/profile", i),
            Box::new(SimpleRoute::new(format!("user-{}", i), "application/json")),
        );
    }

    let start = Instant::now();
    let mut found_count = 0;

    // 执行 10000 次查找
    for i in 0..10000 {
        let idx = i % 1000;
        if table.contains(&format!("/api/v1/users/{}/profile", idx)) {
            found_count += 1;
        }
    }

    let duration = start.elapsed();
    println!("📊 RouteTable Trie优化性能测试:");
    println!("  查找次数: {}", found_count);
    println!("  总耗时: {:?}", duration);
    println!("  平均每次查找时间: {:.2} ns", duration.as_nanos() as f64 / found_count as f64);
    println!("  每秒查找次数: {:.0}", found_count as f64 / duration.as_secs_f64());

    assert_eq!(found_count, 10000, "应该找到所有路由");
}

/// Trie树内存使用测试
#[test]
fn test_trie_memory_efficiency() {
    let mut matcher = TrieBasedMatcher::new();

    // 添加有共同前缀的路由
    for i in 0..1000 {
        matcher.add_pattern(RoutePattern::from(&format!("/api/v1/users/{}", i)));
    }

    for i in 0..1000 {
        matcher.add_pattern(RoutePattern::from(&format!("/api/v1/posts/{}", i)));
    }

    let route_count = matcher.count();
    println!("📊 Trie树内存效率测试:");
    println!("  注册路由数量: {}", route_count);
    println!("  共享前缀路径: /api/v1/users/* 和 /api/v1/posts/*");
    println!("  Trie树优势: 共享前缀节点，减少内存占用");

    assert_eq!(route_count, 2000, "应该注册所有路由");
}

/// 复杂路径模式性能测试
#[test]
fn test_complex_pattern_trie_performance() {
    let mut matcher = TrieBasedMatcher::new();

    // 添加复杂路径模式
    for i in 0..500 {
        matcher.add_pattern(RoutePattern::from(&format!(
            "/api/v1/organizations/{}/departments/{}/employees/{}",
            i, i * 2, i * 3
        )));
    }

    let start = Instant::now();
    let mut matches = 0;

    // 测试复杂路径匹配
    for i in 0..500 {
        let test_path = format!(
            "/api/v1/organizations/{}/departments/{}/employees/{}",
            i, i * 2, i * 3
        );
        if matcher.match_path(&test_path).is_some() {
            matches += 1;
        }
    }

    let duration = start.elapsed();
    println!("📊 复杂路径模式Trie性能测试:");
    println!("  匹配次数: {}", matches);
    println!("  总耗时: {:?}", duration);
    println!("  平均每次匹配时间: {:.2} ns", duration.as_nanos() as f64 / matches as f64);

    assert_eq!(matches, 500, "应该匹配所有复杂路径");
}

/// Trie树通配符性能测试
#[test]
fn test_trie_wildcard_performance() {
    let mut matcher = TrieBasedMatcher::new();

    // 添加通配符路由
    matcher.add_pattern(RoutePattern::from("/static/*"));
    matcher.add_pattern(RoutePattern::from("/api/v1/*"));
    matcher.add_pattern(RoutePattern::from("/uploads/*"));

    let start = Instant::now();
    let mut matches = 0;

    // 测试通配符匹配
    for i in 0..10000 {
        let paths = vec![
            format!("/static/css/style{}.css", i),
            format!("/api/v1/users/{}", i),
            format!("/uploads/file{}.jpg", i),
        ];

        for path in paths {
            if matcher.match_path(&path).is_some() {
                matches += 1;
            }
        }
    }

    let duration = start.elapsed();
    println!("📊 Trie树通配符性能测试:");
    println!("  匹配次数: {}", matches);
    println!("  总耗时: {:?}", duration);
    println!("  平均每次匹配时间: {:.2} ns", duration.as_nanos() as f64 / matches as f64);

    assert_eq!(matches, 30000, "应该匹配所有通配符路径");
}

/// 参数提取性能测试
#[test]
fn test_parameter_extraction_performance() {
    let mut matcher = TrieBasedMatcher::new();

    // 添加带参数的路由
    matcher.add_pattern(RoutePattern::from("/users/{id}"));
    matcher.add_pattern(RoutePattern::from("/users/{id}/posts/{post_id}"));
    matcher.add_pattern(RoutePattern::from("/users/{id}/posts/{post_id}/comments/{comment_id}"));

    let start = Instant::now();
    let mut extractions = 0;

    // 测试参数提取
    for i in 0..10000 {
        let paths = vec![
            format!("/users/{}", i),
            format!("/users/{}/posts/{}", i, i * 2),
            format!("/users/{}/posts/{}/comments/{}", i, i * 2, i * 3),
        ];

        for path in paths {
            if let Some(params) = matcher.match_path(&path) {
                if !params.is_empty() {
                    extractions += 1;
                }
            }
        }
    }

    let duration = start.elapsed();
    println!("📊 参数提取性能测试:");
    println!("  参数提取次数: {}", extractions);
    println!("  总耗时: {:?}", duration);
    println!("  平均每次提取时间: {:.2} ns", duration.as_nanos() as f64 / extractions as f64);

    assert_eq!(extractions, 30000, "应该提取所有参数");
}

/// Trie树并发性能测试
#[test]
fn test_trie_concurrent_performance() {
    use std::sync::Arc;
    use std::thread;

    let matcher = Arc::new(std::sync::Mutex::new(TrieBasedMatcher::new()));

    // 添加路由
    {
        let mut m = matcher.lock().unwrap();
        for i in 0..1000 {
            m.add_pattern(RoutePattern::from(&format!("/concurrent/route/{}", i)));
        }
    }

    let start = Instant::now();
    let mut handles = vec![];

    // 并发匹配
    for _ in 0..10 {
        let matcher_clone = Arc::clone(&matcher);
        let handle = thread::spawn(move || {
            let mut matches = 0;
            for i in 0..1000 {
                let m = matcher_clone.lock().unwrap();
                if m.match_path(&format!("/concurrent/route/{}", i)).is_some() {
                    matches += 1;
                }
            }
            matches
        });
        handles.push(handle);
    }

    let total_matches: usize = handles.into_iter().map(|h| h.join().unwrap()).sum();
    let duration = start.elapsed();

    println!("📊 Trie树并发性能测试:");
    println!("  总匹配次数: {}", total_matches);
    println!("  总耗时: {:?}", duration);
    println!("  每秒匹配次数: {:.0}", total_matches as f64 / duration.as_secs_f64());

    assert_eq!(total_matches, 10000, "应该匹配所有路由");
}

/// Trie树vs HashMap查找性能对比
#[test]
fn test_trie_vs_hashmap_lookup() {
    use std::collections::HashMap;

    // 创建路由数据
    let routes: Vec<String> = (0..10000)
        .map(|i| format!("/users/{}/profile", i))
        .collect();

    // 1. Trie树查找
    let mut trie_matcher = TrieBasedMatcher::new();
    for route in &routes {
        trie_matcher.add_pattern(RoutePattern::from(route));
    }

    let start = Instant::now();
    let mut trie_found = 0;
    for i in 0..10000 {
        let path = format!("/users/{}/profile", i);
        if trie_matcher.match_path(&path).is_some() {
            trie_found += 1;
        }
    }
    let trie_duration = start.elapsed();

    // 2. HashMap查找
    let mut hashmap: HashMap<String, String> = HashMap::new();
    for route in &routes {
        hashmap.insert(route.clone(), "value".to_string());
    }

    let start = Instant::now();
    let mut hashmap_found = 0;
    for i in 0..10000 {
        let path = format!("/users/{}/profile", i);
        if hashmap.contains_key(&path) {
            hashmap_found += 1;
        }
    }
    let hashmap_duration = start.elapsed();

    println!("📊 Trie树 vs HashMap查找性能对比:");
    println!("  Trie树查找: {} 次, 耗时: {:?}", trie_found, trie_duration);
    println!("  HashMap查找: {} 次, 耗时: {:?}", hashmap_found, hashmap_duration);
    println!("  性能比率: {:.2}", trie_duration.as_nanos() as f64 / hashmap_duration.as_nanos() as f64);

    assert_eq!(trie_found, hashmap_found, "两者应该找到相同数量的路由");
}