// 基准测试：分片锁优化效果对比
//
// 使用 criterion crate 进行专业的性能测试

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use dynamic_route_actix::{RouteEntry, RouteTable, SimpleRoute};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

fn bench_concurrent_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_lookup");

    for size in [100, 500, 1000, 5000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let table = Arc::new(RouteTable::new());

            // 预填充路由表
            for i in 0..size {
                table.insert(
                    format!("/route-{}", i),
                    Box::new(SimpleRoute::new(format!("body-{}", i), "text/plain")),
                );
            }

            b.iter(|| {
                let mut handles = Vec::new();
                for _ in 0..10 {
                    let table_clone = Arc::clone(&table);
                    let handle = std::thread::spawn(move || {
                        for i in 0..size {
                            black_box(table_clone.contains(&format!("/route-{}", i)));
                        }
                    });
                    handles.push(handle);
                }
                for handle in handles {
                    handle.join().unwrap();
                }
            });
        });
    }

    group.finish();
}

fn bench_concurrent_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_insert");

    for size in [100, 500, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let table = Arc::new(RouteTable::new());
                let mut handles = Vec::new();

                for i in 0..10 {
                    let table_clone = Arc::clone(&table);
                    let handle = std::thread::spawn(move || {
                        for j in 0..size {
                            table_clone.insert(
                                format!("/route-{}-{}", i, j),
                                Box::new(SimpleRoute::new("body", "text/plain")),
                            );
                        }
                    });
                    handles.push(handle);
                }

                for handle in handles {
                    handle.join().unwrap();
                }
            });
        });
    }

    group.finish();
}

fn bench_batch_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_insert");

    for size in [100, 500, 1000, 5000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let table = RouteTable::new();
                let mut routes: HashMap<String, Box<dyn RouteEntry>> = HashMap::new();

                for i in 0..size {
                    routes.insert(
                        format!("/route-{}", i),
                        Box::new(SimpleRoute::new(format!("body-{}", i), "text/plain")),
                    );
                }

                black_box(table.batch_insert(routes));
            });
        });
    }

    group.finish();
}

fn bench_single_operations(c: &mut Criterion) {
    let table = Arc::new(RouteTable::new());

    // 预填充
    for i in 0..1000 {
        table.insert(
            format!("/route-{}", i),
            Box::new(SimpleRoute::new(format!("body-{}", i), "text/plain")),
        );
    }

    c.bench_function("single_lookup", |b| {
        b.iter(|| {
            black_box(table.contains(&format!("/route-{}", rand::random::<usize>() % 1000)));
        });
    });

    c.bench_function("single_insert", |b| {
        let table = Arc::new(RouteTable::new());
        b.iter(|| {
            black_box(table.insert(
                format!("/route-{}", rand::random::<usize>()),
                Box::new(SimpleRoute::new("body", "text/plain")),
            ));
        });
    });
}

criterion_group!(
    benches,
    bench_concurrent_lookup,
    bench_concurrent_insert,
    bench_batch_insert,
    bench_single_operations
);
criterion_main!(benches);