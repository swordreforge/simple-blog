//! 无锁数据结构性能测试
//!
//! 测试无锁队列和无锁对象池的性能提升效果，对比Mutex版本

use dynamic_route_actix::core::object_pool::{global_lockfree_object_pool, global_object_pool, LockFreeRouteObjectPool, RouteObjectPool};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

#[test]
fn test_lockfree_stack_single_thread_performance() {
    const OPERATIONS: usize = 1_000_000;
    
    let pool = LockFreeRouteObjectPool::new(1000, 500);
    
    // 测试推送性能
    let start = Instant::now();
    for i in 0..OPERATIONS {
        pool.push_string(format!("string-{}", i));
    }
    let push_time = start.elapsed();
    
    // 测试弹出性能
    let start = Instant::now();
    for _ in 0..OPERATIONS {
        let _ = pool.pull_string();
    }
    let pop_time = start.elapsed();
    
    println!("无锁栈单线程性能测试 ({}次操作):", OPERATIONS);
    println!("  推送时间: {:?}", push_time);
    println!("  弹出时间: {:?}", pop_time);
    println!("  总时间: {:?}", push_time + pop_time);
    println!("  平均每次操作: {:?}", (push_time + pop_time) / (OPERATIONS * 2) as u32);
    println!("  每秒操作次数: {:.0}", (OPERATIONS * 2) as f64 / (push_time + pop_time).as_secs_f64());
}

#[test]
fn test_mutex_pool_single_thread_performance() {
    const OPERATIONS: usize = 1_000_000;
    
    let pool = RouteObjectPool::new(1000, 500);
    
    // 测试推送性能
    let start = Instant::now();
    for i in 0..OPERATIONS {
        let mut s = pool.pull_string();
        s.push_str(&format!("string-{}", i));
        drop(s);
    }
    let push_time = start.elapsed();
    
    // 测试弹出性能
    let start = Instant::now();
    for _ in 0..OPERATIONS {
        let _ = pool.pull_string();
    }
    let pop_time = start.elapsed();
    
    println!("Mutex对象池单线程性能测试 ({}次操作):", OPERATIONS);
    println!("  推送时间: {:?}", push_time);
    println!("  弹出时间: {:?}", pop_time);
    println!("  总时间: {:?}", push_time + pop_time);
    println!("  平均每次操作: {:?}", (push_time + pop_time) / (OPERATIONS * 2) as u32);
    println!("  每秒操作次数: {:.0}", (OPERATIONS * 2) as f64 / (push_time + pop_time).as_secs_f64());
}

#[test]
fn test_lockfree_pool_concurrent_performance() {
    const THREADS: usize = 8;
    const OPERATIONS_PER_THREAD: usize = 100_000;
    const TOTAL_OPERATIONS: usize = THREADS * OPERATIONS_PER_THREAD;
    
    let pool = Arc::new(LockFreeRouteObjectPool::new(2000, 1000));
    
    // 创建生产者线程
    let mut producers = Vec::new();
    for thread_id in 0..THREADS {
        let pool_clone = Arc::clone(&pool);
        let producer = thread::spawn(move || {
            let start = Instant::now();
            for i in 0..OPERATIONS_PER_THREAD {
                pool_clone.push_string(format!("thread-{}-string-{}", thread_id, i));
            }
            start.elapsed()
        });
        producers.push(producer);
    }
    
    // 创建消费者线程
    let mut consumers = Vec::new();
    for _ in 0..THREADS {
        let pool_clone = Arc::clone(&pool);
        let consumer = thread::spawn(move || {
            let start = Instant::now();
            for _ in 0..OPERATIONS_PER_THREAD {
                let _ = pool_clone.pull_string();
            }
            start.elapsed()
        });
        consumers.push(consumer);
    }
    
    // 等待所有线程完成
    let mut total_producer_time = std::time::Duration::new(0, 0);
    for producer in producers {
        total_producer_time += producer.join().unwrap();
    }
    
    let mut total_consumer_time = std::time::Duration::new(0, 0);
    for consumer in consumers {
        total_consumer_time += consumer.join().unwrap();
    }
    
    let total_time = total_producer_time.max(total_consumer_time);
    
    println!("无锁对象池并发性能测试:");
    println!("  线程数: {}", THREADS);
    println!("  每线程操作数: {}", OPERATIONS_PER_THREAD);
    println!("  总操作数: {}", TOTAL_OPERATIONS);
    println!("  生产者总时间: {:?}", total_producer_time);
    println!("  消费者总时间: {:?}", total_consumer_time);
    println!("  实际总时间: {:?}", total_time);
    println!("  平均每次操作: {:?}", total_time / (TOTAL_OPERATIONS * 2) as u32);
    println!("  每秒操作次数: {:.0}", (TOTAL_OPERATIONS * 2) as f64 / total_time.as_secs_f64());
}

#[test]
fn test_mutex_pool_concurrent_performance() {
    const THREADS: usize = 8;
    const OPERATIONS_PER_THREAD: usize = 100_000;
    const TOTAL_OPERATIONS: usize = THREADS * OPERATIONS_PER_THREAD;
    
    let pool = Arc::new(RouteObjectPool::new(2000, 1000));
    
    // 创建生产者线程
    let mut producers = Vec::new();
    for thread_id in 0..THREADS {
        let pool_clone = Arc::clone(&pool);
        let producer = thread::spawn(move || {
            let start = Instant::now();
            for i in 0..OPERATIONS_PER_THREAD {
                let mut s = pool_clone.pull_string();
                s.push_str(&format!("thread-{}-string-{}", thread_id, i));
                drop(s);
            }
            start.elapsed()
        });
        producers.push(producer);
    }
    
    // 创建消费者线程
    let mut consumers = Vec::new();
    for _ in 0..THREADS {
        let pool_clone = Arc::clone(&pool);
        let consumer = thread::spawn(move || {
            let start = Instant::now();
            for _ in 0..OPERATIONS_PER_THREAD {
                let _ = pool_clone.pull_string();
            }
            start.elapsed()
        });
        consumers.push(consumer);
    }
    
    // 等待所有线程完成
    let mut total_producer_time = std::time::Duration::new(0, 0);
    for producer in producers {
        total_producer_time += producer.join().unwrap();
    }
    
    let mut total_consumer_time = std::time::Duration::new(0, 0);
    for consumer in consumers {
        total_consumer_time += consumer.join().unwrap();
    }
    
    let total_time = total_producer_time.max(total_consumer_time);
    
    println!("Mutex对象池并发性能测试:");
    println!("  线程数: {}", THREADS);
    println!("  每线程操作数: {}", OPERATIONS_PER_THREAD);
    println!("  总操作数: {}", TOTAL_OPERATIONS);
    println!("  生产者总时间: {:?}", total_producer_time);
    println!("  消费者总时间: {:?}", total_consumer_time);
    println!("  实际总时间: {:?}", total_time);
    println!("  平均每次操作: {:?}", total_time / (TOTAL_OPERATIONS * 2) as u32);
    println!("  每秒操作次数: {:.0}", (TOTAL_OPERATIONS * 2) as f64 / total_time.as_secs_f64());
}

#[test]
fn test_lockfree_vs_mutex_concurrent_comparison() {
    const THREADS: usize = 8;
    const OPERATIONS_PER_THREAD: usize = 100_000;
    const TOTAL_OPERATIONS: usize = THREADS * OPERATIONS_PER_THREAD;
    
    // 测试无锁版本
    let lockfree_pool = Arc::new(LockFreeRouteObjectPool::new(2000, 1000));
    
    let lockfree_start = Instant::now();
    
    let mut lockfree_threads = Vec::new();
    for thread_id in 0..THREADS {
        let pool_clone = Arc::clone(&lockfree_pool);
        let thread = thread::spawn(move || {
            for i in 0..OPERATIONS_PER_THREAD {
                let s = pool_clone.pull_string();
                pool_clone.push_string(format!("thread-{}-{}", thread_id, i));
            }
        });
        lockfree_threads.push(thread);
    }
    
    for thread in lockfree_threads {
        thread.join().unwrap();
    }
    
    let lockfree_time = lockfree_start.elapsed();
    
    // 测试Mutex版本
    let mutex_pool = Arc::new(RouteObjectPool::new(2000, 1000));
    
    let mutex_start = Instant::now();
    
    let mut mutex_threads = Vec::new();
    for thread_id in 0..THREADS {
        let pool_clone = Arc::clone(&mutex_pool);
        let thread = thread::spawn(move || {
            for i in 0..OPERATIONS_PER_THREAD {
                let mut s = pool_clone.pull_string();
                s.push_str(&format!("thread-{}-{}", thread_id, i));
                drop(s);
            }
        });
        mutex_threads.push(thread);
    }
    
    for thread in mutex_threads {
        thread.join().unwrap();
    }
    
    let mutex_time = mutex_start.elapsed();
    
    println!("无锁 vs Mutex 并发性能对比:");
    println!("  线程数: {}", THREADS);
    println!("  每线程操作数: {}", OPERATIONS_PER_THREAD);
    println!("  总操作数: {}", TOTAL_OPERATIONS);
    println!("  无锁版本总时间: {:?}", lockfree_time);
    println!("  Mutex版本总时间: {:?}", mutex_time);
    println!("  性能差异: {:.2}%", 
             (mutex_time.as_nanos() - lockfree_time.as_nanos()) as f64 / mutex_time.as_nanos() as f64 * 100.0);
    println!("  速度提升: {:.2}x", 
             mutex_time.as_secs_f64() / lockfree_time.as_secs_f64());
}

#[test]
fn test_global_lockfree_object_pool_performance() {
    const OPERATIONS: usize = 100_000;
    let pool = global_lockfree_object_pool();
    
    // 预热
    for i in 0..1000 {
        pool.push_string(format!("warmup-{}", i));
    }
    
    // 清空池
    for _ in 0..1000 {
        let _ = pool.pull_string();
    }
    
    // 性能测试
    let start = Instant::now();
    for i in 0..OPERATIONS {
        let mut s = pool.pull_string();
        s.push_str(&format!("test-{}", i));
        pool.push_string(s);
    }
    let elapsed = start.elapsed();
    
    println!("全局无锁对象池性能测试 ({}次操作):", OPERATIONS);
    println!("  总时间: {:?}", elapsed);
    println!("  平均每次操作: {:?}", elapsed / OPERATIONS as u32);
    println!("  每秒操作次数: {:.0}", OPERATIONS as f64 / elapsed.as_secs_f64());
    println!("  当前池大小: {}", pool.string_pool_size());
}

#[test]
fn test_lockfree_pool_stress_test() {
    const THREADS: usize = 16;
    const OPERATIONS_PER_THREAD: usize = 50_000;
    const TOTAL_OPERATIONS: usize = THREADS * OPERATIONS_PER_THREAD;
    
    let pool = Arc::new(LockFreeRouteObjectPool::new(5000, 2500));
    let start = Instant::now();
    
    let mut threads = Vec::new();
    for thread_id in 0..THREADS {
        let pool_clone = Arc::clone(&pool);
        let thread = thread::spawn(move || {
            for i in 0..OPERATIONS_PER_THREAD {
                // 随机选择推送或弹出
                if i % 2 == 0 {
                    pool_clone.push_string(format!("stress-{}-{}", thread_id, i));
                } else {
                    let _ = pool_clone.pull_string();
                }
            }
        });
        threads.push(thread);
    }
    
    for thread in threads {
        thread.join().unwrap();
    }
    
    let elapsed = start.elapsed();
    
    println!("无锁对象池压力测试:");
    println!("  线程数: {}", THREADS);
    println!("  每线程操作数: {}", OPERATIONS_PER_THREAD);
    println!("  总操作数: {}", TOTAL_OPERATIONS);
    println!("  总时间: {:?}", elapsed);
    println!("  平均每次操作: {:?}", elapsed / TOTAL_OPERATIONS as u32);
    println!("  每秒操作次数: {:.0}", TOTAL_OPERATIONS as f64 / elapsed.as_secs_f64());
    println!("  最终池大小: {}", pool.string_pool_size());
}