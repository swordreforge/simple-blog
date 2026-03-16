//! Cow<str> 和 Bytes 优化性能测试
//!
//! 测试第十三轮优化的性能提升效果

use std::borrow::Cow;
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use dynamic_route_actix::core::{
    cow_optimized::*,
    bytes_optimized::*,
    string_optimized::*,
    route_matcher::*,
};
use std::time::Duration;

/// 测试 Cow<str> 路由匹配性能
fn bench_cow_route_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("cow_route_matching");
    group.measurement_time(Duration::from_secs(10));

    // 测试不同路径长度
    for path_len in [3, 5, 7, 10].iter() {
        // 创建测试路径
        let path = (0..*path_len)
            .map(|i| format!("segment{}", i))
            .collect::<Vec<_>>()
            .join("/");

        // Cow 优化版本
        let cow_pattern = CowRoutePattern::from_str(path.as_str());

        group.bench_with_input(
            BenchmarkId::new("cow_match", path_len),
            path_len,
            |b, _| {
                b.iter(|| {
                    black_box(cow_pattern.match_path(black_box(path.as_str())));
                });
            },
        );

        // 原始版本
        let original_pattern = RoutePattern::from(path.as_str());

        group.bench_with_input(
            BenchmarkId::new("original_match", path_len),
            path_len,
            |b, _| {
                b.iter(|| {
                    black_box(original_pattern.match_path(black_box(path.as_str())));
                });
            },
        );
    }

    group.finish();
}

/// 测试 Cow<str> 路径连接性能
fn bench_cow_path_joining(c: &mut Criterion) {
    let mut group = c.benchmark_group("cow_path_joining");

    // 测试不同数量的路径段
    for segment_count in [2, 4, 8, 16].iter() {
        let segments: Vec<String> = (0..*segment_count)
            .map(|i| format!("segment{}", i))
            .collect();

        let cow_segments: Vec<Cow<'_, str>> = segments.iter()
            .map(|s| Cow::Borrowed(s.as_str()))
            .collect();

        let borrowed_segments: Vec<&str> = segments.iter()
            .map(|s| s.as_str())
            .collect();

        group.bench_with_input(
            BenchmarkId::new("cow_join", segment_count),
            segment_count,
            |b, _| {
                b.iter(|| {
                    black_box(join_cow(black_box(&cow_segments), "/"));
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("string_join", segment_count),
            segment_count,
            |b, _| {
                b.iter(|| {
                    black_box(borrowed_segments.join("/"));
                });
            },
        );
    }

    group.finish();
}

/// 测试 Cow<str> 路径规范化性能
fn bench_cow_path_normalization(c: &mut Criterion) {
    let mut group = c.benchmark_group("cow_path_normalization");

    let paths = ["/api/v1/users",
        "/api//v1//users",
        "/api/./v1/users",
        "/api/v1/../users",
        "/api/v1/users/../../.."];

    for (i, path) in paths.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("cow_normalize", i),
            path,
            |b, path| {
                b.iter(|| {
                    black_box(normalize_path(black_box(path)));
                });
            },
        );
    }

    group.finish();
}

/// 测试字符串池性能
fn bench_string_pool(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_pool");

    // 测试字符串池的命中率
    let mut pool = PathStringPool::new();
    let common_strings = vec!["api", "v1", "users", "posts", "comments"];

    // 预填充
    for s in &common_strings {
        pool.get_or_insert(s);
    }

    // 测试从池获取
    group.bench_function("pool_get", |b| {
        b.iter(|| {
            for s in &common_strings {
                black_box(pool.get(black_box(s)));
            }
        });
    });

    // 测试从池插入
    group.bench_function("pool_insert", |b| {
        b.iter(|| {
            let mut pool = PathStringPool::new();
            for s in &common_strings {
                black_box(pool.get_or_insert(black_box(s)));
            }
        });
    });

    group.finish();
}

/// 测试 Bytes 创建性能
fn bench_bytes_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("bytes_creation");

    let data = vec![0u8; 1024];

    group.bench_function("optimized_bytes_from_slice", |b| {
        b.iter(|| {
            black_box(OptimizedBytes::from_slice(black_box(&data)));
        });
    });

    group.bench_function("optimized_bytes_from_vec", |b| {
        b.iter(|| {
            black_box(OptimizedBytes::from_vec(black_box(data.clone())));
        });
    });

    group.bench_function("optimized_bytes_from_static", |b| {
        static STATIC_DATA: &[u8] = &[0u8; 1024];
        b.iter(|| {
            black_box(OptimizedBytes::from_static(STATIC_DATA));
        });
    });

    group.finish();
}

/// 测试 Bytes 操作性能
fn bench_bytes_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("bytes_operations");

    let data = vec![0u8; 1024];
    let bytes = OptimizedBytes::from_slice(&data);
    let pattern = vec![42u8; 10];

    group.bench_function("bytes_contains", |b| {
        b.iter(|| {
            black_box(bytes.contains(black_box(&pattern)));
        });
    });

    group.bench_function("bytes_find", |b| {
        b.iter(|| {
            black_box(bytes.find(black_box(&pattern)));
        });
    });

    group.bench_function("bytes_split_at", |b| {
        b.iter(|| {
            black_box(bytes.split_at(black_box(512)));
        });
    });

    group.bench_function("bytes_slice", |b| {
        b.iter(|| {
            black_box(bytes.slice(0..512));
        });
    });

    group.finish();
}

/// 测试 Bytes 构建器性能
fn bench_bytes_builder(c: &mut Criterion) {
    let mut group = c.benchmark_group("bytes_builder");

    let segments: Vec<Vec<u8>> = (0..10)
        .map(|i| format!("segment{}", i).into_bytes())
        .collect();

    group.bench_function("builder_extend", |b| {
        b.iter(|| {
            let mut builder = BytesBuilder::new();
            for segment in &segments {
                builder.extend(black_box(segment));
            }
            black_box(builder.build());
        });
    });

    group.bench_function("builder_with_capacity", |b| {
        b.iter(|| {
            let total_size: usize = segments.iter().map(|s| s.len()).sum();
            let mut builder = BytesBuilder::with_capacity(total_size);
            for segment in &segments {
                builder.extend(black_box(segment));
            }
            black_box(builder.build());
        });
    });

    group.finish();
}

/// 测试 Bytes 池性能
fn bench_bytes_pool(c: &mut Criterion) {
    let mut group = c.benchmark_group("bytes_pool");

    let mut pool = BytesPool::new(4096, 16);
    let data = vec![0u8; 1024];

    group.bench_function("pool_get_put", |b| {
        b.iter(|| {
            let mut buf = pool.get();
            buf.extend_from_slice(black_box(&data));
            pool.put(black_box(buf));
        });
    });

    group.bench_function("pool_get_with_capacity", |b| {
        b.iter(|| {
            let mut buf = pool.get_with_capacity(black_box(1024));
            buf.extend_from_slice(black_box(&data));
            pool.put(black_box(buf));
        });
    });

    group.finish();
}

/// 测试 Bytes 分割性能
fn bench_bytes_splitting(c: &mut Criterion) {
    let mut group = c.benchmark_group("bytes_splitting");

    let data = b"a,b,c,d,e,f,g,h,i,j";
    let delimiter = b",";

    group.bench_function("splitter_next", |b| {
        b.iter(|| {
            let mut splitter = BytesSplitter::new(black_box(data), black_box(delimiter));
            while splitter.next().is_some() {
                black_box(());
            }
        });
    });

    group.bench_function("splitter_collect_all", |b| {
        b.iter(|| {
            let splitter = BytesSplitter::new(black_box(data), black_box(delimiter));
            black_box(splitter.collect_all());
        });
    });

    group.finish();
}

/// 测试综合性能：路由匹配 + 字符串处理
fn bench_comprehensive_route_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("comprehensive_route_matching");

    // 创建多个路由模式
    let patterns = vec![
        CowRoutePattern::from_str("/api/v1/users"),
        CowRoutePattern::from_str("/api/v1/users/{id}"),
        CowRoutePattern::from_str("/api/v1/users/{id}/posts"),
        CowRoutePattern::from_str("/api/v1/users/{id}/posts/{post_id}"),
        CowRoutePattern::from_str("/api/v1/static/*"),
    ];

    // 测试路径
    let test_paths = vec![
        "/api/v1/users",
        "/api/v1/users/123",
        "/api/v1/users/123/posts",
        "/api/v1/users/123/posts/456",
        "/api/v1/static/css/style.css",
    ];

    group.bench_function("cow_comprehensive_match", |b| {
        b.iter(|| {
            for pattern in &patterns {
                for path in &test_paths {
                    black_box(pattern.match_path(black_box(path)));
                }
            }
        });
    });

    // 原始版本对比
    let original_patterns: Vec<RoutePattern> = vec![
        RoutePattern::from("/api/v1/users"),
        RoutePattern::from("/api/v1/users/{id}"),
        RoutePattern::from("/api/v1/users/{id}/posts"),
        RoutePattern::from("/api/v1/users/{id}/posts/{post_id}"),
        RoutePattern::from("/api/v1/static/*"),
    ];

    group.bench_function("original_comprehensive_match", |b| {
        b.iter(|| {
            for pattern in &original_patterns {
                for path in &test_paths {
                    black_box(pattern.match_path(black_box(path)));
                }
            }
        });
    });

    group.finish();
}

/// 测试内存分配减少
fn bench_memory_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation");

    let path = "/api/v1/users/123/posts/456/comments/789";

    // 使用 Cow 的参数提取
    group.bench_function("cow_param_extraction", |b| {
        b.iter(|| {
            let extractor = ParamExtractor::new(black_box(path));
            black_box(extractor.extract_segments());
        });
    });

    // 原始的字符串分割
    group.bench_function("string_split", |b| {
        b.iter(|| {
            black_box(path.split('/').collect::<Vec<_>>());
        });
    });

    group.finish();
}

/// 测试 Bytes 转换性能
fn bench_bytes_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("bytes_conversion");

    let data = b"hello world, this is a test";

    group.bench_function("to_hex", |b| {
        b.iter(|| {
            black_box(BytesConverter::to_hex(black_box(data)));
        });
    });

    group.bench_function("to_base64", |b| {
        b.iter(|| {
            black_box(BytesConverter::to_base64(black_box(data)));
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_cow_route_matching,
    bench_cow_path_joining,
    bench_cow_path_normalization,
    bench_string_pool,
    bench_bytes_creation,
    bench_bytes_operations,
    bench_bytes_builder,
    bench_bytes_pool,
    bench_bytes_splitting,
    bench_comprehensive_route_matching,
    bench_memory_allocation,
    bench_bytes_conversion,
);

criterion_main!(benches);