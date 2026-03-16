//! 字符串优化性能基准测试
//!
//! 测试字符串优化（SSO和字符串池）的性能提升

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dynamic_route_actix::core::string_optimized::*;
use std::sync::Arc;
use std::time::Duration;

/// 基准测试：普通字符串创建
fn bench_string_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_creation");
    
    // 短字符串
    group.bench_function("short_string_regular", |b| {
        b.iter(|| {
            let s: String = "hello".to_string();
            black_box(s)
        })
    });
    
    group.bench_function("short_string_small_opt", |b| {
        b.iter(|| {
            let s = SmallString::new("hello");
            black_box(s)
        })
    });
    
    group.bench_function("short_string_smart", |b| {
        b.iter(|| {
            let s = SmartString::from_string("hello");
            black_box(s)
        })
    });
    
    // 长字符串
    group.bench_function("long_string_regular", |b| {
        b.iter(|| {
            let s: String = "this is a very long string that will not fit in SSO".to_string();
            black_box(s)
        })
    });
    
    group.bench_function("long_string_small_opt", |b| {
        b.iter(|| {
            let s = SmallString::new("this is a very long string that will not fit in SSO");
            black_box(s)
        })
    });
    
    group.bench_function("long_string_smart", |b| {
        b.iter(|| {
            let s = SmartString::from_string("this is a very long string that will not fit in SSO");
            black_box(s)
        })
    });
    
    group.finish();
}

/// 基准测试：路径分割
fn bench_path_split(c: &mut Criterion) {
    let mut group = c.benchmark_group("path_split");
    
    let path1 = "/api/v1/users/123/posts/456";
    
    // 普通分割
    group.bench_function("regular_split", |b| {
        b.iter(|| {
            let segments: Vec<String> = black_box(path1)
                .split('/')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect();
            black_box(segments)
        })
    });
    
    // 小字符串优化
    group.bench_function("small_string_split", |b| {
        b.iter(|| {
            let segments = split_path_small(black_box(path1));
            black_box(segments)
        })
    });
    
    // 池化分割
    group.bench_function("pooled_split", |b| {
        b.iter(|| {
            let segments = split_path_pooled(black_box(path1));
            black_box(segments)
        })
    });
    
    // 智能分割
    group.bench_function("smart_split", |b| {
        b.iter(|| {
            let segments = split_path_smart(black_box(path1));
            black_box(segments)
        })
    });
    
    group.finish();
}

/// 基准测试：字符串池性能
fn bench_string_pool(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_pool");
    
    // 测试缓存命中率
    group.bench_function("pool_hit", |b| {
        let mut pool = StringPool::new();
        pool.prefill(&["users", "posts", "comments", "api", "v1"]);
        
        b.iter(|| {
            let s = pool.get_or_insert(black_box("users"));
            black_box(s)
        })
    });
    
    group.bench_function("pool_miss", |b| {
        let mut pool = StringPool::new();
        
        b.iter(|| {
            let s = pool.get_or_insert(black_box("unique_string_12345"));
            black_box(s)
        })
    });
    
    // 测试路径字符串池
    group.bench_function("path_pool_hit", |b| {
        let mut pool = PathStringPool::new();
        
        b.iter(|| {
            let s = pool.get_or_insert(black_box("GET"));
            black_box(s)
        })
    });
    
    group.finish();
}

/// 基准测试：路径连接
fn bench_path_join(c: &mut Criterion) {
    let mut group = c.benchmark_group("path_join");
    
    let segments = vec!["api", "v1", "users", "123", "posts"];
    
    // 普通连接
    group.bench_function("regular_join", |b| {
        b.iter(|| {
            let result = segments.join("/");
            black_box(result)
        })
    });
    
    // 优化连接
    group.bench_function("optimized_join", |b| {
        b.iter(|| {
            let result = join_paths_optimized(black_box(&segments));
            black_box(result)
        })
    });
    
    group.finish();
}

/// 基准测试：字符串克隆
fn bench_string_clone(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_clone");
    
    let s1: String = "hello".to_string();
    let s2: String = "this is a very long string for cloning test".to_string();
    
    group.bench_function("clone_short_string", |b| {
        b.iter(|| {
            let s = black_box(&s1).clone();
            black_box(s)
        })
    });
    
    group.bench_function("clone_long_string", |b| {
        b.iter(|| {
            let s = black_box(&s2).clone();
            black_box(s)
        })
    });
    
    // Arc字符串克隆（零拷贝）
    group.bench_function("clone_arc_string", |b| {
        let arc_s: Arc<str> = Arc::from("this is a very long string for cloning test");
        b.iter(|| {
            let s = black_box(&arc_s).clone();
            black_box(s)
        })
    });
    
    group.finish();
}

/// 基准测试：重复字符串操作
fn bench_repeated_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("repeated_operations");
    
    // 模拟路由匹配场景：重复解析路径
    let paths = vec![
        "/api/v1/users",
        "/api/v1/posts",
        "/api/v1/comments",
        "/admin/dashboard",
        "/auth/login",
    ];
    
    group.bench_function("repeated_parse_regular", |b| {
        b.iter(|| {
            for path in &paths {
                let segments: Vec<String> = path
                    .split('/')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect();
                black_box(segments);
            }
        })
    });
    
    group.bench_function("repeated_parse_pooled", |b| {
        b.iter(|| {
            for path in &paths {
                let segments = split_path_pooled(path);
                black_box(segments);
            }
        })
    });
    
    group.bench_function("repeated_parse_smart", |b| {
        b.iter(|| {
            for path in &paths {
                let segments = split_path_smart(path);
                black_box(segments);
            }
        })
    });
    
    group.finish();
}

/// 基准测试：内存使用
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    
    // 测试大量短字符串的内存占用
    group.bench_function("many_short_strings_regular", |b| {
        b.iter(|| {
            let mut strings: Vec<String> = Vec::with_capacity(1000);
            for i in 0..1000 {
                strings.push(format!("route_{}", i));
            }
            black_box(strings)
        })
    });
    
    group.bench_function("many_short_strings_small", |b| {
        b.iter(|| {
            let mut strings: Vec<SmallString> = Vec::with_capacity(1000);
            for i in 0..1000 {
                strings.push(SmallString::new(format!("route_{}", i)));
            }
            black_box(strings)
        })
    });
    
    // 测试大量重复字符串的内存占用
    group.bench_function("many_duplicate_strings_regular", |b| {
        b.iter(|| {
            let mut strings: Vec<String> = Vec::with_capacity(1000);
            for _ in 0..1000 {
                strings.push("users".to_string());
            }
            black_box(strings)
        })
    });
    
    group.bench_function("many_duplicate_strings_pooled", |b| {
        let mut pool = StringPool::new();
        pool.prefill(&["users"]);
        
        b.iter(|| {
            let mut strings: Vec<Arc<str>> = Vec::with_capacity(1000);
            for _ in 0..1000 {
                strings.push(pool.get_or_insert("users"));
            }
            black_box(strings)
        })
    });
    
    group.finish();
}

/// 混合基准测试：模拟真实路由场景
fn bench_real_world_scenario(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_world_scenario");
    
    // 模拟真实的路由处理流程
    let paths: Vec<String> = (0..1000)
        .map(|i| format!("/api/v1/resource/{}/action/{}", i % 10, i % 5))
        .collect();
    
    group.bench_function("route_processing_regular", |b| {
        b.iter(|| {
            for path in &paths {
                let segments: Vec<String> = path
                    .split('/')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect();
                
                if segments.len() >= 4 {
                    let resource = &segments[2];
                    let action = &segments[4];
                    black_box((resource, action));
                }
            }
        })
    });
    
    group.bench_function("route_processing_smart", |b| {
        b.iter(|| {
            for path in &paths {
                let segments = split_path_smart(path);
                
                if segments.len() >= 4 {
                    let resource = &segments[2];
                    let action = &segments[4];
                    black_box((resource, action));
                }
            }
        })
    });
    
    group.bench_function("route_processing_pooled", |b| {
        b.iter(|| {
            for path in &paths {
                let segments = split_path_pooled(path);
                
                if segments.len() >= 4 {
                    let resource = &segments[2];
                    let action = &segments[4];
                    black_box((resource, action));
                }
            }
        })
    });
    
    group.finish();
}

criterion_group! {
    name = string_optimization_benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(100);
    targets =
        bench_string_creation,
        bench_path_split,
        bench_string_pool,
        bench_path_join,
        bench_string_clone,
        bench_repeated_operations,
        bench_memory_usage,
        bench_real_world_scenario
}

criterion_main!(string_optimization_benches);