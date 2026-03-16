//! 内存池性能测试
//!
//! 测试对象池优化的性能提升效果

use dynamic_route_actix::core::object_pool::{extract_params_optimized, normalize_path_optimized, split_path_optimized};
use dynamic_route_actix::{RouteTable, SimpleRoute};
use std::time::Instant;

#[test]
fn test_path_splitting_performance() {
    const ITERATIONS: usize = 100_000;
    let test_path = "/api/v1/users/123/posts/456/comments/789";
    
    // 测试传统路径分割
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let segments: Vec<String> = test_path
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        std::hint::black_box(&segments);
    }
    let traditional = start.elapsed();
    
    // 测试优化后的路径分割
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let segments = split_path_optimized(test_path);
        std::hint::black_box(&segments);
    }
    let optimized = start.elapsed();
    
    println!("路径分割性能测试 ({}次迭代):", ITERATIONS);
    println!("  传统方法: {:?}", traditional);
    println!("  优化方法: {:?}", optimized);
    if traditional > optimized {
        println!("  性能提升: {:.2}%", 
                 (traditional.as_nanos() - optimized.as_nanos()) as f64 / traditional.as_nanos() as f64 * 100.0);
    } else {
        println!("  性能差异: {:.2}%", 
                 (optimized.as_nanos() - traditional.as_nanos()) as f64 / traditional.as_nanos() as f64 * 100.0);
    }
}

#[test]
fn test_path_normalization_performance() {
    const ITERATIONS: usize = 100_000;
    let test_paths = vec![
        "/users/",
        "/users//123/",
        "  /api/v1/users/123  ",
        "/",
        "/api/v1/posts/",
    ];
    
    // 测试传统路径标准化
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let path = test_paths[i % test_paths.len()];
        let mut normalized = path.trim().to_string();
        if normalized.len() > 1 && normalized.ends_with('/') {
            normalized.pop();
        }
        while normalized.contains("//") {
            normalized = normalized.replace("//", "/");
        }
        std::hint::black_box(&normalized);
    }
    let traditional = start.elapsed();
    
    // 测试优化后的路径标准化
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let path = test_paths[i % test_paths.len()];
        let normalized = normalize_path_optimized(path);
        std::hint::black_box(&normalized);
    }
    let optimized = start.elapsed();
    
    println!("路径标准化性能测试 ({}次迭代):", ITERATIONS);
    println!("  传统方法: {:?}", traditional);
    println!("  优化方法: {:?}", optimized);
    if traditional > optimized {
        println!("  性能提升: {:.2}%", 
                 (traditional.as_nanos() - optimized.as_nanos()) as f64 / traditional.as_nanos() as f64 * 100.0);
    } else {
        println!("  性能差异: {:.2}%", 
                 (optimized.as_nanos() - traditional.as_nanos()) as f64 / traditional.as_nanos() as f64 * 100.0);
    }
}

#[test]
fn test_route_table_with_object_pool() {
    const ROUTE_COUNT: usize = 1000;
    
    // 创建路由表并插入大量路由
    let table = RouteTable::new();
    for i in 0..ROUTE_COUNT {
        table.insert(
            format!("/route-{}", i),
            Box::new(SimpleRoute::new(format!("body-{}", i), "text/plain")),
        );
    }
    
    // 测试查找性能
    const LOOKUP_ITERATIONS: usize = 100_000;
    let start = Instant::now();
    for i in 0..LOOKUP_ITERATIONS {
        let route_idx = i % ROUTE_COUNT;
        table.get_with(&format!("/route-{}", route_idx), |_route| {
            // 使用路由
            true
        });
    }
    let lookup_time = start.elapsed();
    
    println!("路由表查找性能测试:");
    println!("  路由数量: {}", ROUTE_COUNT);
    println!("  查找次数: {}", LOOKUP_ITERATIONS);
    println!("  总时间: {:?}", lookup_time);
    println!("  平均每次查找: {:?}", lookup_time / LOOKUP_ITERATIONS as u32);
    println!("  每秒查找次数: {:.0}", LOOKUP_ITERATIONS as f64 / lookup_time.as_secs_f64());
}







#[test]
fn test_extract_params_optimized_performance() {
    const ITERATIONS: usize = 100_000;
    let params = vec![
        ("id".to_string(), "123".to_string()),
        ("name".to_string(), "test".to_string()),
        ("page".to_string(), "1".to_string()),
    ];
    
    // 传统方法
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let result: std::collections::HashMap<String, String> = params.iter().cloned().collect();
        std::hint::black_box(&result);
    }
    let traditional = start.elapsed();
    
    // 优化方法
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let result = extract_params_optimized(&params);
        std::hint::black_box(&result);
    }
    let optimized = start.elapsed();
    
    println!("参数提取性能测试 ({}次迭代):", ITERATIONS);
    println!("  传统方法: {:?}", traditional);
    println!("  优化方法: {:?}", optimized);
}