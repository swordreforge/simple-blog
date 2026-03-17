//! Radix Tree性能测试
//!
//! 测试Radix Tree的性能表现，并与标准Trie树进行对比。

use dynamic_route_actix::core::route_radix_tree::RouteRadixTree;
use dynamic_route_actix::core::route_trie::RouteTrie;
use dynamic_route_actix::SimpleRoute;
use std::time::Instant;

/// Radix Tree基础匹配性能测试
#[test]
fn test_radix_basic_matching_performance() {
    let mut radix = RouteRadixTree::new();
    
    // 插入100个路由
    for i in 0..100 {
        let path = format!("/api/v1/resource/{}", i);
        radix.insert(
            &path,
            Box::new(SimpleRoute::new(format!("resource-{}", i), "application/json")),
        );
    }
    
    // 性能测试
    let iterations = 30000;
    let start = Instant::now();
    
    for i in 0..iterations {
        let path = format!("/api/v1/resource/{}", i % 100);
        let _result = radix.find(&path);
    }
    
    let duration = start.elapsed();
    let avg_time = duration.as_nanos() as f64 / iterations as f64;
    let ops_per_sec = iterations as f64 / duration.as_secs_f64();
    
    println!("📊 Radix基础匹配测试:");
    println!("  总匹配次数: {}", iterations);
    println!("  总耗时: {:?}", duration);
    println!("  平均每次匹配时间: {:.2} ns", avg_time);
    println!("  每秒匹配次数: {:.0}", ops_per_sec);
    
    // 性能应该保持在合理范围内
    assert!(avg_time < 2000.0, "平均匹配时间过长: {:.2} ns", avg_time);
}

/// Radix Tree vs Trie树性能对比测试
#[test]
fn test_radix_vs_trie_performance_comparison() {
    let test_routes = vec![
        "/users",
        "/users/{id}",
        "/users/{id}/posts",
        "/users/{id}/posts/{post_id}",
        "/posts",
        "/posts/{id}",
        "/posts/{id}/comments",
        "/posts/{id}/comments/{comment_id}",
        "/api/v1/users",
        "/api/v1/users/{id}",
        "/api/v1/posts",
        "/api/v1/posts/{id}",
        "/api/v2/users",
        "/api/v2/users/{id}",
        "/api/v2/posts",
        "/api/v2/posts/{id}",
        "/static/*",
        "/static/css/*",
        "/static/js/*",
        "/static/images/*",
    ];
    
    // 创建Radix Tree
    let mut radix = RouteRadixTree::new();
    for route in &test_routes {
        radix.insert(route, Box::new(SimpleRoute::new(route.to_string(), "application/json")));
    }
    
    // 创建Trie树
    let mut trie = RouteTrie::new();
    for route in &test_routes {
        trie.insert(route, Box::new(SimpleRoute::new(route.to_string(), "application/json")));
    }
    
    // 测试路径
    let test_paths = vec![
        "/users",
        "/users/123",
        "/users/123/posts",
        "/users/123/posts/456",
        "/posts",
        "/posts/789",
        "/posts/789/comments",
        "/posts/789/comments/999",
        "/api/v1/users",
        "/api/v1/users/123",
        "/api/v1/posts",
        "/api/v1/posts/456",
        "/api/v2/users",
        "/api/v2/users/789",
        "/api/v2/posts",
        "/api/v2/posts/999",
        "/static/css/style.css",
        "/static/js/app.js",
        "/static/images/logo.png",
    ];
    
    let iterations = 1000;
    
    // 测试Radix Tree性能
    let start = Instant::now();
    for _ in 0..iterations {
        for path in &test_paths {
            let _result = radix.find(path);
        }
    }
    let radix_duration = start.elapsed();
    
    // 测试Trie树性能
    let start = Instant::now();
    for _ in 0..iterations {
        for path in &test_paths {
            let _result = trie.find(path);
        }
    }
    let trie_duration = start.elapsed();
    
    let speedup = trie_duration.as_nanos() as f64 / radix_duration.as_nanos() as f64;
    
    println!("📊 Radix Tree vs Trie树性能对比:");
    println!("  Radix Tree匹配次数: {}, 耗时: {:?}", iterations * test_paths.len(), radix_duration);
    println!("  Trie树匹配次数: {}, 耗时: {:?}", iterations * test_paths.len(), trie_duration);
    println!("  性能提升: {:.2}x", speedup);
    
    // Radix Tree应该性能相当或更好
    assert!(radix_duration <= trie_duration * 2, "Radix Tree性能应该不低于Trie树的50%");
}

/// Radix Tree内存效率测试
#[test]
fn test_radix_memory_efficiency() {
    // 插入有共同前缀的路由
    let routes = vec![
        "/api/v1/users",
        "/api/v1/users/{id}",
        "/api/v1/posts",
        "/api/v1/posts/{id}",
        "/api/v1/comments",
        "/api/v1/comments/{id}",
        "/api/v1/admin/users",
        "/api/v1/admin/posts",
        "/api/v2/users",
        "/api/v2/users/{id}",
        "/api/v2/posts",
        "/api/v2/posts/{id}",
        "/api/v3/endpoint1",
        "/api/v3/endpoint2",
        "/api/v3/endpoint3",
    ];
    
    // Radix Tree
    let mut radix = RouteRadixTree::new();
    for route in &routes {
        radix.insert(route, Box::new(SimpleRoute::new(route.to_string(), "application/json")));
    }
    let radix_nodes = radix.node_count();
    
    // Trie树
    let mut trie = RouteTrie::new();
    for route in &routes {
        trie.insert(route, Box::new(SimpleRoute::new(route.to_string(), "application/json")));
    }
    let trie_nodes = trie.node_count();
    
    let node_reduction = ((trie_nodes - radix_nodes) as f64 / trie_nodes as f64) * 100.0;
    
    println!("📊 Radix Tree内存效率测试:");
    println!("  注册路由数量: {}", routes.len());
    println!("  Trie树节点数: {}", trie_nodes);
    println!("  Radix Tree节点数: {}", radix_nodes);
    println!("  节点减少: {:.1}%", node_reduction);
    println!("  Radix Tree优势: 压缩前缀节点，减少内存占用");
    
    // Radix Tree应该使用更少的节点
    assert!(radix_nodes <= trie_nodes, "Radix Tree应该使用不超过Trie树的节点数");
}

/// Radix Tree大规模性能测试
#[test]
fn test_radix_large_scale_performance() {
    let route_count = 2000;
    let test_count = 10000;
    
    let mut radix = RouteRadixTree::new();
    
    // 生成有共同前缀的路由
    for i in 0..route_count {
        let route = if i < 500 {
            format!("/api/v1/users/{}", i)
        } else if i < 1000 {
            format!("/api/v1/posts/{}", i)
        } else if i < 1500 {
            format!("/api/v2/comments/{}", i)
        } else {
            format!("/api/v3/resources/{}", i)
        };
        
        radix.insert(&route, Box::new(SimpleRoute::new(&route, "application/json")));
    }
    
    let start = Instant::now();
    
    for i in 0..test_count {
        let path = format!("/api/v1/users/{}", i % 500);
        let _result = radix.find(&path);
    }
    
    let duration = start.elapsed();
    let avg_time = duration.as_nanos() as f64 / test_count as f64;
    let ops_per_sec = test_count as f64 / duration.as_secs_f64();
    
    println!("📊 Radix Tree大规模性能测试:");
    println!("  路由数量: {}", route_count);
    println!("  测试次数: {}", test_count);
    println!("  总耗时: {:?}", duration);
    println!("  平均每次查找时间: {:.2} ns", avg_time);
    println!("  每秒查找次数: {:.0}", ops_per_sec);
    
    // 性能应该保持稳定
    assert!(avg_time < 5000.0, "大规模查找性能下降: {:.2} ns", avg_time);
    assert!(ops_per_sec > 100_000.0, "吞吐量过低: {:.0} 次/秒", ops_per_sec);
}

/// Radix Tree参数提取性能测试
#[test]
fn test_radix_parameter_extraction_performance() {
    let mut radix = RouteRadixTree::new();
    
    // 插入带参数的路由
    radix.insert(
        "/users/{id}",
        Box::new(SimpleRoute::new("user", "application/json")),
    );
    radix.insert(
        "/users/{id}/posts/{post_id}",
        Box::new(SimpleRoute::new("post", "application/json")),
    );
    radix.insert(
        "/users/{id}/posts/{post_id}/comments/{comment_id}",
        Box::new(SimpleRoute::new("comment", "application/json")),
    );
    
    let iterations = 30000;
    let start = Instant::now();
    
    for i in 0..iterations {
        let path = format!("/users/{}/posts/{}/comments/{}", i, i * 2, i * 3);
        let result = radix.find(&path);
        assert!(result.is_some(), "应该能找到路由");
        
        let (_route, params) = result.unwrap();
        assert_eq!(params.len(), 3, "应该提取3个参数");
    }
    
    let duration = start.elapsed();
    let avg_time = duration.as_nanos() as f64 / iterations as f64;
    let ops_per_sec = iterations as f64 / duration.as_secs_f64();
    
    println!("📊 Radix Tree参数提取性能测试:");
    println!("  参数提取次数: {}", iterations);
    println!("  总耗时: {:?}", duration);
    println!("  平均每次提取时间: {:.2} ns", avg_time);
    println!("  每秒提取次数: {:.0}", ops_per_sec);
    
    // 参数提取性能应该保持高效
    assert!(avg_time < 3000.0, "参数提取时间过长: {:.2} ns", avg_time);
}

/// Radix Tree通配符性能测试
#[test]
fn test_radix_wildcard_performance() {
    let mut radix = RouteRadixTree::new();
    
    // 插入通配符路由
    radix.insert(
        "/static/*",
        Box::new(SimpleRoute::new("static", "application/octet-stream")),
    );
    radix.insert(
        "/api/*",
        Box::new(SimpleRoute::new("api", "application/json")),
    );
    
    let test_paths = vec![
        "/static/css/style.css",
        "/static/js/app.js",
        "/static/images/logo.png",
        "/static/fonts/font.ttf",
        "/api/v1/users",
        "/api/v2/posts",
        "/api/v3/comments",
    ];
    
    let iterations = 30000;
    let start = Instant::now();
    
    for _ in 0..iterations {
        for path in &test_paths {
            let _result = radix.find(path);
        }
    }
    
    let duration = start.elapsed();
    let total_ops = iterations * test_paths.len();
    let avg_time = duration.as_nanos() as f64 / total_ops as f64;
    let ops_per_sec = total_ops as f64 / duration.as_secs_f64();
    
    println!("📊 Radix Tree通配符性能测试:");
    println!("  匹配次数: {}", total_ops);
    println!("  总耗时: {:?}", duration);
    println!("  平均每次匹配时间: {:.2} ns", avg_time);
    println!("  每秒匹配次数: {:.0}", ops_per_sec);
    
    // 通配符匹配应该非常快
    assert!(avg_time < 1000.0, "通配符匹配时间过长: {:.2} ns", avg_time);
}

/// Radix Tree并发性能测试
#[test]
fn test_radix_concurrent_performance() {
    use std::sync::Arc;
    use std::thread;
    
    let radix = Arc::new(std::sync::RwLock::new(RouteRadixTree::new()));
    
    // 插入路由
    {
        let mut radix = radix.write().unwrap();
        for i in 0..100 {
            let path = format!("/api/v1/resource/{}", i);
            radix.insert(
                &path,
                Box::new(SimpleRoute::new(format!("resource-{}", i), "application/json")),
            );
        }
    }
    
    let iterations = 100;
    let num_threads = 10;
    let mut handles = vec![];
    
    let start = Instant::now();
    
    for _ in 0..num_threads {
        let radix_clone = Arc::clone(&radix);
        let handle = thread::spawn(move || {
            for i in 0..iterations {
                let path = format!("/api/v1/resource/{}", i % 100);
                let radix = radix_clone.read().unwrap();
                let _result = radix.find(&path);
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    let duration = start.elapsed();
    let total_ops = iterations * num_threads;
    let ops_per_sec = total_ops as f64 / duration.as_secs_f64();
    
    println!("📊 Radix Tree并发性能测试:");
    println!("  总匹配次数: {}", total_ops);
    println!("  总耗时: {:?}", duration);
    println!("  每秒匹配次数: {:.0}", ops_per_sec);
    
    // 并发性能应该保持良好
    assert!(ops_per_sec > 100_000.0, "并发性能过低: {:.0} 次/秒", ops_per_sec);
}

/// Radix Tree RouteTable集成性能测试
#[test]
fn test_radix_route_table_performance() {
    use dynamic_route_actix::RouteTable;
    
    let table = RouteTable::new();
    let route_count = 1000;
    
    // 插入路由
    for i in 0..route_count {
        table.insert(
            format!("/api/v1/resource/{}", i),
            Box::new(SimpleRoute::new(format!("resource-{}", i), "application/json")),
        );
    }
    
    let test_count = 10000;
    let start = Instant::now();
    
    for i in 0..test_count {
        let path = format!("/api/v1/resource/{}", i % route_count);
        let _result = table.get_with(&path, |_route| "found");
    }
    
    let duration = start.elapsed();
    let avg_time = duration.as_nanos() as f64 / test_count as f64;
    let ops_per_sec = test_count as f64 / duration.as_secs_f64();
    
    println!("📊 Radix Tree RouteTable性能测试:");
    println!("  路由数量: {}", route_count);
    println!("  查找次数: {}", test_count);
    println!("  总时间: {:?}", duration);
    println!("  平均每次查找时间: {:.2} ns", avg_time);
    println!("  每秒查找次数: {:.0}", ops_per_sec);
    
    // RouteTable性能应该保持高效
    assert!(avg_time < 2000.0, "RouteTable查找时间过长: {:.2} ns", avg_time);
    assert!(ops_per_sec > 500_000.0, "吞吐量过低: {:.0} 次/秒", ops_per_sec);
}