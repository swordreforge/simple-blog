//! 字符串优化演示
//!
//! 展示字符串优化（SSO和字符串池）的效果

use dynamic_route_actix::core::string_optimized::*;
use std::time::Instant;
use std::sync::Arc;

fn main() {
    println!("=== 字符串优化演示 ===\n");

    // 1. 小字符串优化演示
    println!("1. 小字符串优化（SSO）");
    let short = SmallString::new("hello");
    println!("   短字符串: '{}' (长度: {}, 是否为小字符串: {})", 
             short, short.len(), short.is_small());
    
    let long = SmallString::new("this is a very long string that will not fit in SSO");
    println!("   长字符串: '{}' (长度: {}, 是否为小字符串: {})", 
             long, long.len(), long.is_small());

    // 2. 字符串池演示
    println!("\n2. 字符串池（String Pool）");
    let mut pool = StringPool::new();
    pool.prefill(&["users", "posts", "comments", "api", "v1"]);
    
    let s1 = pool.get_or_insert("users");
    let s2 = pool.get_or_insert("users");
    println!("   缓存命中率: {:.2}%", pool.hit_rate() * 100.0);
    println!("   相同字符串是否共享内存: {}", Arc::ptr_eq(&s1, &s2));

    // 3. 路径字符串池演示
    println!("\n3. 路径字符串池（PathStringPool）");
    let path_pool = PathStringPool::new();
    let method = path_pool.get("GET").unwrap();
    println!("   预填充的HTTP方法: '{}'", method);

    // 4. 智能字符串演示
    println!("\n4. 智能字符串（SmartString）");
    let smart_short = SmartString::from_string("hello");
    let smart_long = SmartString::from_string("this is a very long string");
    println!("   短字符串类型: {:?}", std::mem::discriminant(&smart_short));
    println!("   长字符串类型: {:?}", std::mem::discriminant(&smart_long));

    // 5. 性能对比演示
    println!("\n5. 性能对比演示");
    
    // 路径分割性能对比
    let test_path = "/api/v1/users/123/posts/456/comments/789";
    let iterations = 100_000;
    
    // 普通分割
    let start = Instant::now();
    for _ in 0..iterations {
        let segments: Vec<String> = test_path
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        std::hint::black_box(segments);
    }
    let regular_time = start.elapsed();
    
    // 池化分割
    let start = Instant::now();
    for _ in 0..iterations {
        let segments = split_path_pooled(test_path);
        std::hint::black_box(segments);
    }
    let pooled_time = start.elapsed();
    
    // 智能分割
    let start = Instant::now();
    for _ in 0..iterations {
        let segments = split_path_smart(test_path);
        std::hint::black_box(segments);
    }
    let smart_time = start.elapsed();
    
    println!("   路径分割性能对比 ({} 次迭代):", iterations);
    println!("   - 普通分割: {:?} ({:.2} ns/op)", 
             regular_time, regular_time.as_nanos() as f64 / iterations as f64);
    println!("   - 池化分割: {:?} ({:.2} ns/op) - 提升: {:.1}%", 
             pooled_time, pooled_time.as_nanos() as f64 / iterations as f64,
             (1.0 - pooled_time.as_secs_f64() / regular_time.as_secs_f64()) * 100.0);
    println!("   - 智能分割: {:?} ({:.2} ns/op) - 提升: {:.1}%", 
             smart_time, smart_time.as_nanos() as f64 / iterations as f64,
             (1.0 - smart_time.as_secs_f64() / regular_time.as_secs_f64()) * 100.0);

    // 6. 内存使用对比演示
    println!("\n6. 内存使用对比演示");
    
    let mut regular_strings: Vec<String> = Vec::with_capacity(1000);
    let mut small_strings: Vec<SmallString> = Vec::with_capacity(1000);
    
    for i in 0..1000 {
        regular_strings.push(format!("route_{}", i));
        small_strings.push(SmallString::new(format!("route_{}", i)));
    }
    
    println!("   1000个短字符串的内存占用:");
    println!("   - 普通String: ~{} bytes", 
             std::mem::size_of_val::<Vec<String>>(&regular_strings) + 
             regular_strings.iter().map(|s| s.capacity()).sum::<usize>());
    println!("   - SmallString: ~{} bytes", 
             std::mem::size_of_val::<Vec<SmallString>>(&small_strings) + 
             small_strings.iter().map(|s| s.len()).sum::<usize>());

    // 7. 统计信息演示
    println!("\n7. 优化统计信息");
    let stats = global_stats();
    println!("   总操作数: {}", stats.total_operations.load(std::sync::atomic::Ordering::Relaxed));
    println!("   小字符串比例: {:.2}%", stats.small_string_ratio() * 100.0);
    println!("   池化字符串比例: {:.2}%", stats.pooled_string_ratio() * 100.0);
    println!("   借用字符串比例: {:.2}%", stats.borrowed_string_ratio() * 100.0);

    println!("\n=== 演示完成 ===");
    println!("\n字符串优化带来的好处:");
    println!("1. 小字符串优化（SSO）: 短字符串（≤23字节）在栈上存储，避免堆分配");
    println!("2. 字符串池: 复用常用字符串，减少内存分配和复制");
    println!("3. 智能选择: 自动选择最优的字符串表示方式");
    println!("4. 预期提升: 5-10%的性能提升和内存使用优化");
}