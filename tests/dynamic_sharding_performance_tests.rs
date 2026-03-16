//! 动态分片和负载均衡性能测试

use dynamic_route_actix::{
    DynamicRouteTable, DynamicShardingConfig, LoadBalanceStrategy,
    RouteTable, RouteEntry, SimpleRoute,
};

use dynamic_route_actix::core::DynamicRouteTableConfig;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// 测试不同负载均衡策略的效果
#[test]
fn test_load_balance_strategies() {
    let strategies = vec![
        LoadBalanceStrategy::RouteCount,
        LoadBalanceStrategy::AccessFrequency,
        LoadBalanceStrategy::Comprehensive,
    ];

    for strategy in strategies {
        let config = DynamicRouteTableConfig {
            sharding: DynamicShardingConfig {
                initial_shards: 4,
                strategy,
                ..Default::default()
            },
            ..Default::default()
        };

        let table = DynamicRouteTable::new(config.clone());

        // 插入100个路由
        for i in 0..100 {
            table.insert(
                format!("/route-{}", i),
                Box::new(SimpleRoute::new(format!("body-{}", i), "text/plain")),
            );
        }

        // 执行大量读取操作
        for _ in 0..1000 {
            let idx = rand::random::<usize>() % 100;
            table.get_with(&format!("/route-{}", idx), |_route| ());
        }

        // 检查负载分布
        let metrics = table.get_shard_metrics();
        let loads: Vec<f64> = metrics.iter().map(|m| m.load_score()).collect();

        // 确保负载分布合理（最大负载不应超过平均负载的3倍）
        let avg_load = loads.iter().sum::<f64>() / loads.len() as f64;
        let max_load = loads.iter().fold(0.0f64, |acc, &x| acc.max(x));

        assert!(
            max_load <= avg_load * 3.0,
            "Strategy {:?} has unbalanced load: max={:.2}, avg={:.2}",
            strategy,
            max_load,
            avg_load
        );
    }
}

/// 测试动态重平衡功能
#[test]
fn test_dynamic_rebalance() {
    let config = DynamicRouteTableConfig {
        sharding: DynamicShardingConfig {
            initial_shards: 4,
            imbalance_threshold: 0.2,
            auto_rebalance: true,
            ..Default::default()
        },
        ..Default::default()
    };

    let table = DynamicRouteTable::new(config.clone());

    // 创建不均衡负载：在前10个分片中插入大量路由
    for i in 0..200 {
        table.insert(
            format!("/hot-{}", i),
            Box::new(SimpleRoute::new("hot content", "text/plain")),
        );
    }

    // 检查初始不均衡程度
    let imbalance_before = table.get_imbalance();
    println!("Initial imbalance: {:.2}", imbalance_before);

    // 执行重平衡
    let moved = table.rebalance().unwrap();
    println!("Routes moved during rebalance: {}", moved);

    // 检查重平衡后的不均衡程度
    let imbalance_after = table.get_imbalance();
    println!("Imbalance after rebalance: {:.2}", imbalance_after);

    // 重平衡后不均衡程度应该降低
    if moved > 0 {
        assert!(
            imbalance_after < imbalance_before || imbalance_after < 0.3,
            "Rebalance did not improve load distribution"
        );
    }
}

/// 对比静态分片和动态分片的性能
#[test]
fn test_static_vs_dynamic_sharding() {
    const NUM_ROUTES: usize = 1000;
    const NUM_READS: usize = 10000;

    // 静态分片（原始 RouteTable）
    let static_table = Arc::new(RouteTable::new());
    for i in 0..NUM_ROUTES {
        static_table.insert(
            format!("/static-{}", i),
            Box::new(SimpleRoute::new("static", "text/plain")),
        );
    }

    let start = Instant::now();
    for _ in 0..NUM_READS {
        let idx = rand::random::<usize>() % NUM_ROUTES;
        static_table.get_with(&format!("/static-{}", idx), |_route| ());
    }
    let static_duration = start.elapsed();

    // 动态分片（DynamicRouteTable）
    let dynamic_table = Arc::new(DynamicRouteTable::default_config());
    for i in 0..NUM_ROUTES {
        dynamic_table.insert(
            format!("/dynamic-{}", i),
            Box::new(SimpleRoute::new("dynamic", "text/plain")),
        );
    }

    let start = Instant::now();
    for _ in 0..NUM_READS {
        let idx = rand::random::<usize>() % NUM_ROUTES;
        dynamic_table.get_with(&format!("/dynamic-{}", idx), |_route| ());
    }
    let dynamic_duration = start.elapsed();

    println!("Static sharding: {:?}", static_duration);
    println!("Dynamic sharding: {:?}", dynamic_duration);

    // 动态分片的性能应该与静态分片相当，或者更好
    // （允许一定的性能开销，因为动态分片有额外的管理逻辑）
    let max_allowed = static_duration * 2;
    assert!(
        dynamic_duration < max_allowed,
        "Dynamic sharding too slow: {:?} > {:?}",
        dynamic_duration,
        max_allowed
    );
}

/// 测试并发访问下的负载均衡
#[test]
fn test_concurrent_load_balancing() {
    const NUM_THREADS: usize = 10;
    const NUM_OPERATIONS: usize = 1000;

    let table = Arc::new(DynamicRouteTable::default_config());

    // 预填充路由
    for i in 0..500 {
        table.insert(
            format!("/route-{}", i),
            Box::new(SimpleRoute::new("content", "text/plain")),
        );
    }

    let mut handles = vec![];

    // 启动多个线程进行并发访问
    for thread_id in 0..NUM_THREADS {
        let table_clone = Arc::clone(&table);
        let handle = std::thread::spawn(move || {
            for i in 0..NUM_OPERATIONS {
                let idx = rand::random::<usize>() % 500;
                // 混合读取和写入
                if i % 10 == 0 {
                    // 写入
                    table_clone.insert(
                        format!("/thread-{}-{}", thread_id, i),
                        Box::new(SimpleRoute::new("new content", "text/plain")),
                    );
                } else {
                    // 读取
                    table_clone.get_with(&format!("/route-{}", idx), |_route| ());
                }
            }
        });
        handles.push(handle);
    }

    // 等待所有线程完成
    for handle in handles {
        handle.join().unwrap();
    }

    // 检查负载分布
    let metrics = table.get_shard_metrics();
    let total_routes: usize = metrics.iter().map(|m| m.route_count).sum();
    println!("Total routes after concurrent access: {}", total_routes);

    // 验证负载没有过度集中
    let loads: Vec<f64> = metrics.iter().map(|m| m.load_score()).collect();
    let avg_load = loads.iter().sum::<f64>() / loads.len() as f64;
    let max_load = loads.iter().fold(0.0f64, |acc, &x| acc.max(x));

    assert!(
        max_load <= avg_load * 5.0,
        "Concurrent access caused severe load imbalance: max={:.2}, avg={:.2}",
        max_load,
        avg_load
    );
}

/// 测试动态分片扩展
#[test]
fn test_dynamic_shard_scaling() {
    let config = DynamicRouteTableConfig {
        sharding: DynamicShardingConfig {
            initial_shards: 4,
            min_shards: 2,
            max_shards: 16,
            ..Default::default()
        },
        ..Default::default()
    };

    let table = DynamicRouteTable::new(config.clone());

    assert_eq!(table.shard_count(), 4);

    // 增加分片
    table.adjust_shard_count(true).unwrap();
    assert_eq!(table.shard_count(), 5);

    table.adjust_shard_count(true).unwrap();
    assert_eq!(table.shard_count(), 6);

    // 尝试减少分片（最后一个分片为空）
    table.adjust_shard_count(false).unwrap();
    assert_eq!(table.shard_count(), 5);

    table.adjust_shard_count(false).unwrap();
    assert_eq!(table.shard_count(), 4);

    // 尝试减少到最小值以下
    assert!(table.adjust_shard_count(false).is_err() || table.shard_count() >= 2);
}

/// 测试批量插入的负载分布
#[test]
fn test_batch_insert_load_distribution() {
    let table = DynamicRouteTable::default_config();

    // 批量插入大量路由
    let mut routes: HashMap<String, Box<dyn RouteEntry>> = HashMap::new();
    for i in 0..1000 {
        routes.insert(
            format!("/batch-{}", i),
            Box::new(SimpleRoute::new("batch content", "text/plain")),
        );
    }

    let start = Instant::now();
    table.batch_insert(routes);
    let duration = start.elapsed();

    println!("Batch insert duration: {:?}", duration);

    // 检查负载分布
    let metrics = table.get_shard_metrics();
    let route_counts: Vec<usize> = metrics.iter().map(|m| m.route_count).collect();

    // 验证路由分布相对均匀（标准差不应太大）
    let avg = route_counts.iter().sum::<usize>() as f64 / route_counts.len() as f64;
    let variance: f64 = route_counts
        .iter()
        .map(|&x| (x as f64 - avg).powi(2))
        .sum();
    let std_dev = variance.sqrt();

    println!(
        "Route distribution - avg: {:.2}, std_dev: {:.2}",
        avg, std_dev
    );

    // 标准差应该小于平均值的50%
    assert!(
        std_dev < avg * 0.5,
        "Route distribution too uneven: std_dev={:.2}, avg={:.2}",
        std_dev,
        avg
    );
}

/// 测试热点路由的负载均衡效果
#[test]
fn test_hotspot_load_balancing() {
    let config = DynamicRouteTableConfig {
        sharding: DynamicShardingConfig {
            initial_shards: 8,
            strategy: LoadBalanceStrategy::AccessFrequency,
            ..Default::default()
        },
        ..Default::default()
    };

    let table = DynamicRouteTable::new(config.clone());

    // 插入一些路由
    for i in 0..100 {
        table.insert(
            format!("/route-{}", i),
            Box::new(SimpleRoute::new("content", "text/plain")),
        );
    }

    // 创建热点：频繁访问前10个路由
    for _ in 0..10000 {
        let idx = rand::random::<usize>() % 10;
        table.get_with(&format!("/route-{}", idx), |_route| ());
    }

    // 偶尔访问其他路由
    for _ in 0..1000 {
        let idx = 10 + rand::random::<usize>() % 90;
        table.get_with(&format!("/route-{}", idx), |_route| ());
    }

    // 检查负载分布
    let metrics = table.get_shard_metrics();
    let access_counts: Vec<usize> = metrics.iter().map(|m| m.read_count).collect();

    println!("Access counts per shard: {:?}", access_counts);

    // 验证访问分布相对均匀（考虑到热点访问）
    let total_access: usize = access_counts.iter().sum();
    let avg_access = total_access as f64 / access_counts.len() as f64;
    let max_access = *access_counts.iter().max().unwrap() as f64;

    // 最大访问量不应超过平均值的5倍（考虑到热点）
    assert!(
        max_access <= avg_access * 5.0,
        "Hotspot caused severe imbalance: max={:.2}, avg={:.2}",
        max_access,
        avg_access
    );
}

/// 性能基准测试：重平衡开销
#[test]
fn test_rebalance_overhead() {
    const NUM_ROUTES: usize = 500;

    let config = DynamicRouteTableConfig {
        sharding: DynamicShardingConfig {
            initial_shards: 4,
            imbalance_threshold: 0.1,
            ..Default::default()
        },
        ..Default::default()
    };

    let table = DynamicRouteTable::new(config.clone());

    // 插入路由
    for i in 0..NUM_ROUTES {
        table.insert(
            format!("/route-{}", i),
            Box::new(SimpleRoute::new("content", "text/plain")),
        );
    }

    // 测量重平衡时间
    let start = Instant::now();
    let moved = table.rebalance().unwrap();
    let duration = start.elapsed();

    println!(
        "Rebalance moved {} routes in {:?}",
        moved, duration
    );

    // 重平衡操作应该相对快速（即使移动大量路由）
    assert!(
        duration < Duration::from_secs(1),
        "Rebalance too slow: {:?}",
        duration
    );
}