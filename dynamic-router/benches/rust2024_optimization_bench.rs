//! Rust 2024 Edition 性能优化基准测试
//!
//! 测试新添加的Rust 2024优化特性的性能提升：
//! - FastHashMap (hashbrown + AHash)
//! - PHF静态路由
//! - 异步路由处理

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dynamic_route_actix::{
    static_routes,
    core::{
        phf_static_routes::{StaticRouteRegistry, HybridRouteTable},
        fast_hashmap::FastHashMap,
        SimpleRoute,
    },
};
use std::collections::HashMap;
use std::time::Duration;

// 定义API路由（编译期完美哈希）
static_routes! {
    API_ROUTES = {
        "/api/users" => "users_handler",
        "/api/posts" => "posts_handler",
        "/api/comments" => "comments_handler",
        "/api/auth" => "auth_handler",
        "/api/admin" => "admin_handler",
        "/api/settings" => "settings_handler",
        "/api/notifications" => "notifications_handler",
        "/api/search" => "search_handler",
        "/api/upload" => "upload_handler",
        "/api/download" => "download_handler"
    }
}

/// 基准测试：FastHashMap vs 标准HashMap - 插入性能
fn bench_hashmap_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("hashmap_insert");
    
    // 小规模插入
    group.bench_function("std_hashmap_small", |b| {
        b.iter(|| {
            let mut map: HashMap<String, i32> = HashMap::new();
            for i in 0..100 {
                map.insert(format!("key_{}", i), i);
            }
            black_box(map)
        })
    });
    
    group.bench_function("fast_hashmap_small", |b| {
        b.iter(|| {
            let mut map: FastHashMap<String, i32> = FastHashMap::new();
            for i in 0..100 {
                map.insert(format!("key_{}", i), i);
            }
            black_box(map)
        })
    });
    
    // 大规模插入
    group.bench_function("std_hashmap_large", |b| {
        b.iter(|| {
            let mut map: HashMap<String, i32> = HashMap::new();
            for i in 0..10000 {
                map.insert(format!("key_{}", i), i);
            }
            black_box(map)
        })
    });
    
    group.bench_function("fast_hashmap_large", |b| {
        b.iter(|| {
            let mut map: FastHashMap<String, i32> = FastHashMap::new();
            for i in 0..10000 {
                map.insert(format!("key_{}", i), i);
            }
            black_box(map)
        })
    });
    
    group.finish();
}

/// 基准测试：FastHashMap vs 标准HashMap - 查找性能
fn bench_hashmap_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("hashmap_lookup");
    
    // 准备测试数据
    let mut std_map: HashMap<String, i32> = HashMap::new();
    let mut fast_map: FastHashMap<String, i32> = FastHashMap::new();
    for i in 0..1000 {
        let key = format!("key_{}", i);
        std_map.insert(key.clone(), i);
        fast_map.insert(key, i);
    }
    
    // 随机查找
    group.bench_function("std_hashmap_lookup_hit", |b| {
        b.iter(|| {
            let key = format!("key_{}", black_box(500));
            black_box(std_map.get(&key))
        })
    });
    
    group.bench_function("fast_hashmap_lookup_hit", |b| {
        b.iter(|| {
            let key = format!("key_{}", black_box(500));
            black_box(fast_map.get(&key))
        })
    });
    
    group.bench_function("std_hashmap_lookup_miss", |b| {
        b.iter(|| {
            let key = format!("key_{}", black_box(9999));
            black_box(std_map.get(&key))
        })
    });
    
    group.bench_function("fast_hashmap_lookup_miss", |b| {
        b.iter(|| {
            let key = format!("key_{}", black_box(9999));
            black_box(fast_map.get(&key))
        })
    });
    
    group.finish();
}

/// 基准测试：FastHashMap vs 标准HashMap - 删除性能
fn bench_hashmap_remove(c: &mut Criterion) {
    let mut group = c.benchmark_group("hashmap_remove");
    
    group.bench_function("std_hashmap_remove", |b| {
        b.iter(|| {
            let mut map: HashMap<String, i32> = HashMap::new();
            for i in 0..100 {
                map.insert(format!("key_{}", i), i);
            }
            for i in 0..50 {
                map.remove(&format!("key_{}", i));
            }
            black_box(map)
        })
    });
    
    group.bench_function("fast_hashmap_remove", |b| {
        b.iter(|| {
            let mut map: FastHashMap<String, i32> = FastHashMap::new();
            for i in 0..100 {
                map.insert(format!("key_{}", i), i);
            }
            for i in 0..50 {
                map.remove(&format!("key_{}", i));
            }
            black_box(map)
        })
    });
    
    group.finish();
}

/// 基准测试：PHF静态路由 vs HashMap - 查找性能
fn bench_static_route_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("static_route_lookup");
    
    // 准备HashMap版本
    let mut route_map: HashMap<&str, &str> = HashMap::new();
    route_map.insert("/api/users", "users_handler");
    route_map.insert("/api/posts", "posts_handler");
    route_map.insert("/api/comments", "comments_handler");
    route_map.insert("/api/auth", "auth_handler");
    route_map.insert("/api/admin", "admin_handler");
    route_map.insert("/api/settings", "settings_handler");
    route_map.insert("/api/notifications", "notifications_handler");
    route_map.insert("/api/search", "search_handler");
    route_map.insert("/api/upload", "upload_handler");
    route_map.insert("/api/download", "download_handler");
    
    // PHF查找
    group.bench_function("phf_route_lookup_hit", |b| {
        b.iter(|| {
            black_box(API_ROUTES.get("/api/users"))
        })
    });
    
    group.bench_function("phf_route_lookup_miss", |b| {
        b.iter(|| {
            black_box(API_ROUTES.get("/api/nonexistent"))
        })
    });
    
    // HashMap查找
    group.bench_function("hashmap_route_lookup_hit", |b| {
        b.iter(|| {
            black_box(route_map.get("/api/users"))
        })
    });
    
    group.bench_function("hashmap_route_lookup_miss", |b| {
        b.iter(|| {
            black_box(route_map.get("/api/nonexistent"))
        })
    });
    
    group.finish();
}

/// 基准测试：混合路由表性能
fn bench_hybrid_route_table(c: &mut Criterion) {
    let mut group = c.benchmark_group("hybrid_route_table");
    
    let static_routes = StaticRouteRegistry::new();
    let mut table = HybridRouteTable::new(static_routes);
    
    // 插入动态路由
    for i in 0..100 {
        table.insert(
            format!("/dynamic/route_{}", i),
            Box::new(SimpleRoute::new(format!("Route {}", i), "text/plain"))
        );
    }
    
    // 查找动态路由
    group.bench_function("hybrid_table_dynamic_lookup", |b| {
        b.iter(|| {
            black_box(table.find("/dynamic/route_50"))
        })
    });
    
    group.bench_function("hybrid_table_miss", |b| {
        b.iter(|| {
            black_box(table.find("/nonexistent"))
        })
    });
    
    group.finish();
}

/// 基准测试：字符串哈希性能
fn bench_string_hashing(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_hashing");
    
    let strings: Vec<String> = (0..1000)
        .map(|i| format!("/api/v1/users/{}/posts/{}", i, i * 2))
        .collect();
    
    // 标准HashMap哈希
    group.bench_function("std_hashmap_hash", |b| {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        b.iter(|| {
            let mut hasher = DefaultHasher::new();
            for s in &strings {
                s.hash(&mut hasher);
            }
            black_box(hasher.finish())
        })
    });
    
    // AHash哈希
    group.bench_function("ahash_hash", |b| {
        use ahash::AHasher;
        use std::hash::{Hash, Hasher};
        
        b.iter(|| {
            let mut hasher = AHasher::default();
            for s in &strings {
                s.hash(&mut hasher);
            }
            black_box(hasher.finish())
        })
    });
    
    group.finish();
}

/// 基准测试：批量操作性能
fn bench_batch_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_operations");
    
    // 批量插入
    group.bench_function("std_hashmap_batch_insert", |b| {
        b.iter(|| {
            let mut map: HashMap<String, i32> = HashMap::new();
            for i in 0..1000 {
                map.insert(format!("key_{}", i), i);
            }
            black_box(map)
        })
    });
    
    group.bench_function("fast_hashmap_batch_insert", |b| {
        b.iter(|| {
            let mut map: FastHashMap<String, i32> = FastHashMap::new();
            for i in 0..1000 {
                map.insert(format!("key_{}", i), i);
            }
            black_box(map)
        })
    });
    
    // 批量查找
    let keys: Vec<String> = (0..1000).map(|i| format!("key_{}", i)).collect();
    
    let mut std_map: HashMap<String, i32> = HashMap::new();
    let mut fast_map: FastHashMap<String, i32> = FastHashMap::new();
    for i in 0..1000 {
        std_map.insert(format!("key_{}", i), i);
        fast_map.insert(format!("key_{}", i), i);
    }
    
    group.bench_function("std_hashmap_batch_lookup", |b| {
        b.iter(|| {
            let mut results = Vec::new();
            for key in &keys {
                if let Some(&value) = std_map.get(key) {
                    results.push(value);
                }
            }
            black_box(results)
        })
    });
    
    group.bench_function("fast_hashmap_batch_lookup", |b| {
        b.iter(|| {
            let mut results = Vec::new();
            for key in &keys {
                if let Some(&value) = fast_map.get(key) {
                    results.push(value);
                }
            }
            black_box(results)
        })
    });
    
    group.finish();
}

/// 基准测试：内存效率
fn bench_memory_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_efficiency");
    
    // 测试大量重复键的场景
    group.bench_function("std_hashmap_duplicate_keys", |b| {
        b.iter(|| {
            let mut map: HashMap<String, i32> = HashMap::new();
            for _ in 0..10000 {
                map.insert("users".to_string(), 1);
                map.insert("posts".to_string(), 2);
                map.insert("comments".to_string(), 3);
            }
            black_box(map)
        })
    });
    
    group.bench_function("fast_hashmap_duplicate_keys", |b| {
        b.iter(|| {
            let mut map: FastHashMap<String, i32> = FastHashMap::new();
            for _ in 0..10000 {
                map.insert("users".to_string(), 1);
                map.insert("posts".to_string(), 2);
                map.insert("comments".to_string(), 3);
            }
            black_box(map)
        })
    });
    
    group.finish();
}

/// 综合基准测试：模拟真实路由场景
fn bench_realistic_routing_scenario(c: &mut Criterion) {
    let mut group = c.benchmark_group("realistic_routing");
    
    let mut route_map: HashMap<&str, &str> = HashMap::new();
    route_map.insert("/api/users", "users_handler");
    route_map.insert("/api/posts", "posts_handler");
    route_map.insert("/api/comments", "comments_handler");
    route_map.insert("/api/auth", "auth_handler");
    
    // 模拟请求路径
    let request_paths: Vec<&str> = vec![
        "/api/users", "/api/posts", "/api/comments", "/api/auth",
        "/api/users", "/api/posts", "/api/comments", "/api/auth",
        "/api/users", "/api/posts", "/api/comments", "/api/auth",
    ];
    
    group.bench_function("phf_routing", |b| {
        b.iter(|| {
            for path in &request_paths {
                black_box(API_ROUTES.get(path));
            }
        })
    });
    
    group.bench_function("hashmap_routing", |b| {
        b.iter(|| {
            for path in &request_paths {
                black_box(route_map.get(path));
            }
        })
    });
    
    group.finish();
}

criterion_group! {
    name = rust2024_optimization_benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(100);
    targets =
        bench_hashmap_insert,
        bench_hashmap_lookup,
        bench_hashmap_remove,
        bench_static_route_lookup,
        bench_hybrid_route_table,
        bench_string_hashing,
        bench_batch_operations,
        bench_memory_efficiency,
        bench_realistic_routing_scenario
}

criterion_main!(rust2024_optimization_benches);