//! 动态分片和负载均衡演示
//!
//! 这个示例展示了如何使用动态分片和负载均衡功能来管理路由。

use dynamic_route_actix::{
    DynamicRouteTable, DynamicShardingConfig, LoadBalanceStrategy, SimpleRoute,
};

fn main() {
    println!("=== 动态分片和负载均衡演示 ===\n");

    // 创建动态路由表配置
    let config = dynamic_route_actix::core::DynamicRouteTableConfig {
        sharding: DynamicShardingConfig {
            initial_shards: 8,
            min_shards: 4,
            max_shards: 16,
            strategy: LoadBalanceStrategy::Comprehensive,
            imbalance_threshold: 0.3,
            auto_rebalance: true,
            balance_check_interval: std::time::Duration::from_secs(10),
        },
        use_hash_distribution: true,
    };

    // 创建动态路由表
    let table = DynamicRouteTable::new(config.clone());

    println!("1. 初始状态");
    println!("   分片数量: {}", table.shard_count());
    println!("   路由总数: {}\n", table.count());

    // 添加一些路由
    println!("2. 添加路由");
    for i in 0..100 {
        table.insert(
            format!("/api/user/{}", i),
            Box::new(SimpleRoute::new(format!("User {}", i), "application/json")),
        );
        table.insert(
            format!("/api/product/{}", i),
            Box::new(SimpleRoute::new(format!("Product {}", i), "application/json")),
        );
    }
    println!("   路由总数: {}", table.count());

    // 显示各分片的负载
    println!("\n3. 各分片负载分布:");
    let metrics = table.get_shard_metrics();
    for (i, m) in metrics.iter().enumerate() {
        println!(
            "   分片 {}: {} 路由, 访问次数: {}, 负载分数: {:.3}",
            i, m.route_count, m.total_access, m.load_score()
        );
    }

    // 执行一些读取操作来模拟访问
    println!("\n4. 模拟访问");
    for i in 0..1000 {
        let idx = i % 100;
        table.get_with(&format!("/api/user/{}", idx), |_route| ());
    }

    // 更新指标
    let metrics_after = table.get_shard_metrics();
    println!("   访问后的各分片负载:");
    for (i, m) in metrics_after.iter().enumerate() {
        println!(
            "   分片 {}: {} 路由, 访问次数: {}, 负载分数: {:.3}",
            i, m.route_count, m.total_access, m.load_score()
        );
    }

    // 计算不均衡程度
    let imbalance = table.get_imbalance();
    println!("\n5. 负载不均衡程度: {:.3}", imbalance);

    // 执行重平衡
    if imbalance > config.sharding.imbalance_threshold {
        println!("\n6. 执行负载重平衡...");
        match table.rebalance() {
            Ok(moved) => println!("   移动了 {} 个路由", moved),
            Err(e) => println!("   重平衡失败: {}", e),
        }

        // 显示重平衡后的状态
        let imbalance_after = table.get_imbalance();
        println!("   重平衡后的不均衡程度: {:.3}", imbalance_after);

        let metrics_rebalanced = table.get_shard_metrics();
        println!("   重平衡后的分片负载:");
        for (i, m) in metrics_rebalanced.iter().enumerate() {
            println!(
                "   分片 {}: {} 路由, 访问次数: {}, 负载分数: {:.3}",
                i, m.route_count, m.total_access, m.load_score()
            );
        }
    } else {
        println!("\n6. 负载已经均衡，无需重平衡");
    }

    // 演示动态调整分片数量
    println!("\n7. 动态调整分片数量");
    println!("   当前分片数量: {}", table.shard_count());

    match table.adjust_shard_count(true) {
        Ok(_) => println!("   成功增加分片"),
        Err(e) => println!("   增加分片失败: {}", e),
    }
    println!("   新的分片数量: {}", table.shard_count());

    match table.adjust_shard_count(false) {
        Ok(_) => println!("   成功减少分片"),
        Err(e) => println!("   减少分片失败: {}", e),
    }
    println!("   最终分片数量: {}", table.shard_count());

    // 演示不同的负载均衡策略
    println!("\n8. 演示不同的负载均衡策略");
    let strategies = vec![
        LoadBalanceStrategy::RouteCount,
        LoadBalanceStrategy::AccessFrequency,
        LoadBalanceStrategy::Comprehensive,
    ];

    for strategy in strategies {
        table.set_load_balance_strategy(strategy);
        println!("   当前策略: {:?}", strategy);
    }

    println!("\n=== 演示完成 ===");
}