//! 缓存优化专项测试
//!
//! 测试LRU缓存、缓存预热和智能缓存失效策略的性能和正确性。

use dynamic_route_actix::core::RouteCache;
use std::collections::HashMap;
use std::thread;
use std::time::{Duration, Instant};

#[test]
fn test_lru_cache_basic() {
    println!("\n=== LRU缓存基础测试 ===");
    let cache = RouteCache::new(3, Duration::from_secs(60));

    cache.insert("key1", "value1");
    cache.insert("key2", "value2");
    cache.insert("key3", "value3");

    assert_eq!(cache.size(), 3);
    assert_eq!(cache.get("key1"), Some("value1"));
    assert_eq!(cache.get("key2"), Some("value2"));
    assert_eq!(cache.get("key3"), Some("value3"));

    // 插入第4个键，应该驱逐LRU（key1）
    cache.insert("key4", "value4");
    assert_eq!(cache.size(), 3);

    assert_eq!(cache.get("key1"), None); // 被驱逐
    assert_eq!(cache.get("key2"), Some("value2"));
    assert_eq!(cache.get("key3"), Some("value3"));
    assert_eq!(cache.get("key4"), Some("value4"));

    println!("✓ LRU驱逐机制工作正常");
}

#[test]
fn test_lru_cache_with_access() {
    println!("\n=== LRU缓存访问顺序测试 ===");
    let cache = RouteCache::new(3, Duration::from_secs(60));

    cache.insert("key1", "value1");
    cache.insert("key2", "value2");
    cache.insert("key3", "value3");

    // 访问key1，使其变为最近使用
    cache.get("key1");

    // 插入第4个键，应该驱逐key2（LRU）
    cache.insert("key4", "value4");
    assert_eq!(cache.size(), 3);

    assert_eq!(cache.get("key1"), Some("value1")); // 被访问过，保留
    assert_eq!(cache.get("key2"), None); // 被驱逐
    assert_eq!(cache.get("key3"), Some("value3"));
    assert_eq!(cache.get("key4"), Some("value4"));

    println!("✓ LRU访问顺序更新正确");
}

#[test]
fn test_cache_warmup_performance() {
    println!("\n=== 缓存预热性能测试 ===");
    let cache = RouteCache::new(10000, Duration::from_secs(60));

    // 准备预热数据
    let mut warmup_data = HashMap::new();
    for i in 0..1000 {
        warmup_data.insert(format!("route-{}", i), format!("value-{}", i));
    }

    let start = Instant::now();
    cache.warmup(warmup_data);
    let warmup_duration = start.elapsed();

    println!("  预热1000个条目耗时: {:?}", warmup_duration);
    assert_eq!(cache.size(), 1000);

    // 验证预热的数据都可以访问
    let start = Instant::now();
    for i in 0..1000 {
        assert_eq!(
            cache.get(&format!("route-{}", i)),
            Some(format!("value-{}", i))
        );
    }
    let access_duration = start.elapsed();

    println!("  访问1000个预热条目耗时: {:?}", access_duration);
    assert!(warmup_duration.as_millis() < 100, "预热应该很快");
    assert!(access_duration.as_millis() < 50, "访问预热数据应该很快");

    println!("✓ 缓存预热性能良好");
}

#[test]
fn test_cache_stats_accuracy() {
    println!("\n=== 缓存统计信息测试 ===");
    let cache = RouteCache::new(1000, Duration::from_secs(60));

    // 插入一些数据
    cache.insert("key1", "value1");
    cache.insert("key2", "value2");
    cache.insert("key3", "value3");

    // 命中
    cache.get("key1");
    cache.get("key1");
    cache.get("key2");

    // 未命中
    cache.get("key4");
    cache.get("key5");

    let stats = cache.stats();
    println!("  命中次数: {}", stats.hits);
    println!("  未命中次数: {}", stats.misses);
    println!("  总访问次数: {}", stats.total_accesses);
    println!("  命中率: {:.2}%", stats.hit_rate() * 100.0);

    assert_eq!(stats.hits, 3);
    assert_eq!(stats.misses, 2);
    assert_eq!(stats.total_accesses, 5);
    assert_eq!(stats.hit_rate(), 0.6);

    println!("✓ 缓存统计信息准确");
}

#[test]
fn test_smart_eviction_based_on_frequency() {
    println!("\n=== 智能缓存失效策略测试 ===");
    let cache = RouteCache::new(1000, Duration::from_secs(60));

    // 插入多个条目
    for i in 0..100 {
        cache.insert(&format!("key-{}", i), format!("value-{}", i));
    }

    assert_eq!(cache.size(), 100);

    // 频繁访问某些条目（提高它们的优先级）
    for _ in 0..100 {
        cache.get("key-0");   // 最高频
        cache.get("key-1");   // 高频
        cache.get("key-2");   // 中频
    }

    // 中频访问一些条目
    for _ in 0..10 {
        cache.get("key-10");
        cache.get("key-11");
    }

    // 低频访问一些条目
    for _ in 0..2 {
        cache.get("key-20");
        cache.get("key-21");
    }

    let stats_before = cache.stats();
    println!("  智能驱逐前缓存大小: {}", cache.size());

    // 执行智能驱逐到目标大小50
    cache.smart_evict(50);

    let stats_after = cache.stats();
    println!("  智能驱逐后缓存大小: {}", cache.size());
    println!("  驱逐次数: {}", stats_after.evictions - stats_before.evictions);

    assert_eq!(cache.size(), 50);

    // 高频访问的条目应该保留
    assert_eq!(cache.get("key-0"), Some("value-0".to_string()));
    assert_eq!(cache.get("key-1"), Some("value-1".to_string()));
    assert_eq!(cache.get("key-2"), Some("value-2".to_string()));

    println!("✓ 智能缓存失效策略工作正常");
}

#[test]
fn test_cache_cleanup_expired() {
    println!("\n=== 过期缓存清理测试 ===");
    let cache = RouteCache::new(1000, Duration::from_millis(100));

    // 插入一些条目
    cache.insert("key1", "value1");
    cache.insert("key2", "value2");
    cache.insert("key3", "value3");

    assert_eq!(cache.size(), 3);

    // 等待过期
    thread::sleep(Duration::from_millis(150));

    // 清理过期条目
    let start = Instant::now();
    cache.cleanup_expired();
    let cleanup_duration = start.elapsed();

    println!("  清理过期条目耗时: {:?}", cleanup_duration);
    println!("  清理后缓存大小: {}", cache.size());

    assert_eq!(cache.size(), 0);

    // 插入新条目
    cache.insert("key4", "value4");
    assert_eq!(cache.get("key4"), Some("value4"));

    println!("✓ 过期缓存清理功能正常");
}

#[test]
fn test_cache_hit_rate_under_load() {
    println!("\n=== 高负载下的缓存命中率测试 ===");
    let cache = RouteCache::new(1000, Duration::from_secs(60));

    // 预热缓存（模拟热门路由）
    let mut warmup_data = HashMap::new();
    for i in 0..100 {
        warmup_data.insert(format!("hot-route-{}", i), format!("value-{}", i));
    }
    cache.warmup(warmup_data);

    let start = Instant::now();

    // 模拟真实负载：80%的请求访问热门路由，20%访问冷门路由
    let mut hit_count = 0;
    let mut miss_count = 0;

    for i in 0..10000 {
        if i % 5 == 0 {
            // 20% 冷门请求
            if cache.get(&format!("cold-route-{}", i)).is_some() {
                hit_count += 1;
            } else {
                miss_count += 1;
            }
        } else {
            // 80% 热门请求
            let hot_key = format!("hot-route-{}", i % 100);
            if cache.get(&hot_key).is_some() {
                hit_count += 1;
            } else {
                miss_count += 1;
            }
        }
    }

    let duration = start.elapsed();
    let hit_rate = hit_count as f64 / (hit_count + miss_count) as f64;

    println!("  总请求数: {}", hit_count + miss_count);
    println!("  命中次数: {}", hit_count);
    println!("  未命中次数: {}", miss_count);
    println!("  实际命中率: {:.2}%", hit_rate * 100.0);
    println!("  耗时: {:?}", duration);

    let cache_stats = cache.stats();
    println!("  缓存统计命中率: {:.2}%", cache_stats.hit_rate() * 100.0);

    // 在预热100个热门路由的情况下，命中率应该大于70%
    assert!(
        hit_rate > 0.7,
        "命中率应该大于70%，实际: {:.2}%",
        hit_rate * 100.0
    );

    println!("✓ 高负载下缓存命中率良好");
}

#[test]
fn test_cache_capacity_limits() {
    println!("\n=== 缓存容量限制测试 ===");
    let cache = RouteCache::new(100, Duration::from_secs(60));

    // 插入超过容量的条目
    for i in 0..200 {
        cache.insert(&format!("key-{}", i), format!("value-{}", i));
    }

    println!("  插入200个条目后，缓存大小: {}", cache.size());
    assert_eq!(cache.size(), 100);

    let stats = cache.stats();
    println!("  驱逐次数: {}", stats.evictions);

    // 应该有100次驱逐
    assert!(stats.evictions >= 90 && stats.evictions <= 110);

    println!("✓ 缓存容量限制工作正常");
}

#[test]
fn test_concurrent_cache_access() {
    println!("\n=== 并发缓存访问测试 ===");
    use std::sync::Arc;

    let cache = Arc::new(RouteCache::new(1000, Duration::from_secs(60)));
    let mut handles = vec![];

    // 预热缓存
    for i in 0..100 {
        cache.insert(&format!("key-{}", i), format!("value-{}", i));
    }

    // 并发读取
    for i in 0..10 {
        let cache_clone = Arc::clone(&cache);
        let handle = thread::spawn(move || {
            for j in 0..1000 {
                let key = format!("key-{}", j % 100);
                cache_clone.get(&key);
            }
        });
        handles.push(handle);
    }

    // 并发写入
    for i in 0..10 {
        let cache_clone = Arc::clone(&cache);
        let handle = thread::spawn(move || {
            for j in 0..50 {
                let key = format!("write-{}-{}", i, j);
                cache_clone.insert(&key, format!("value-{}-{}", i, j));
            }
        });
        handles.push(handle);
    }

    let start = Instant::now();
    for handle in handles {
        handle.join().unwrap();
    }
    let duration = start.elapsed();

    println!("  并发操作耗时: {:?}", duration);
    println!("  最终缓存大小: {}", cache.size());

    let stats = cache.stats();
    println!("  总访问次数: {}", stats.total_accesses);
    println!("  命中率: {:.2}%", stats.hit_rate() * 100.0);

    println!("✓ 并发缓存访问稳定");
}

#[test]
fn test_cache_reset_stats() {
    println!("\n=== 缓存统计重置测试 ===");
    let cache = RouteCache::new(1000, Duration::from_secs(60));

    cache.insert("key1", "value1");
    cache.get("key1");
    cache.get("key2");

    let stats_before = cache.stats();
    println!("  重置前 - 命中: {}, 未命中: {}", stats_before.hits, stats_before.misses);

    cache.reset_stats();

    let stats_after = cache.stats();
    println!("  重置后 - 命中: {}, 未命中: {}", stats_after.hits, stats_after.misses);

    assert_eq!(stats_after.hits, 0);
    assert_eq!(stats_after.misses, 0);
    assert_eq!(stats_after.total_accesses, 0);

    println!("✓ 缓存统计重置功能正常");
}