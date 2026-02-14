# 死锁修复总结

## 问题根源分析

基于 `rustblog_strace.log` 的深度分析，发现服务器卡死的真正原因是 **严重的死锁（Deadlock）**，而不是 OOM。

### 关键证据

1. **大量 futex 死锁等待**
   - 112,793 次 `FUTEX_WAIT` 调用
   - 多个线程在同一个地址等待锁，但永远无法唤醒
   - `FUTEX_WAKE` 返回 0（没有唤醒任何线程）

2. **线程间相互依赖**
   ```
   线程 7176: 等待 0x1823348 的锁
   线程 7177: 持有 0x1823348，等待 0x7f7633569b2c
   线程 7327: 持有 0x7f7633569b2c，等待其他锁
   ```
   形成经典的循环等待死锁。

3. **111 次 futex 超时**
   - 线程在等待锁时超时
   - 超时后没有正确处理，继续尝试获取锁
   - 导致活锁（Livelock）：线程不断重试但永远无法前进

4. **信号屏蔽加剧死锁**
   - `rt_sigprocmask(SIG_BLOCK, ~[RTMIN RT_1 RT_2], [], 8)`
   - 线程在获取锁时屏蔽了所有信号
   - 死锁后无法通过信号中断恢复

## 修复措施

### 1. 修复 RENDER_CACHE 死锁问题

**文件**: `src/handlers/api_handlers/passage.rs:1007-1070`

**问题**:
- 使用 `parking_lot::Mutex` 导致长时间持有锁
- 在持有锁的情况下进行缓存清理操作（遍历、收集、删除）
- 所有渲染请求都竞争同一个全局锁

**修复**:
```rust
// 使用 RwLock 替代 Mutex（读写分离）
static RENDER_CACHE: Lazy<RwLock<HashMap<String, (String, Instant)>>> = ...

// 使用读锁允许并发读取
let cache = RENDER_CACHE.read().unwrap();

// 缓存清理使用更高效的策略（移除最老条目）
// 在锁外计算，锁内只执行必要的操作
```

**改进**:
- ✅ 使用 `RwLock` 实现读写分离
- ✅ 减少锁的持有时间
- ✅ 使用 LRU 策略替代简单清理
- ✅ 添加 TTL（1小时）自动过期

### 2. 修复 cache/manager.rs 中的嵌套锁问题

**文件**: `src/cache/manager.rs`

**问题**:
- `check_sliding_window_failure_rate()` 中长时间持有锁
- 在 `get()` 和 `set()` 方法中调用 `check_sliding_window_failure_rate()` 后又获取同一个锁
- 导致嵌套锁问题

**修复**:
```rust
// 优化 check_sliding_window_failure_rate
fn check_sliding_window_failure_rate(&self) -> bool {
    // 快速复制必要数据，然后释放锁
    let (total, failures) = {
        let history = self.operation_history.lock().unwrap();
        // ... 快速复制数据
    };

    // 在锁外计算
    let failure_rate = (failures as f32 / total as f32) * 100.0;
    failure_rate >= threshold
}

// 移除 get/set 方法中的嵌套锁
// check_sliding_window_failure_rate 已经处理了锁
```

**改进**:
- ✅ 消除嵌套锁
- ✅ 减少锁持有时间
- ✅ 避免死锁

### 3. 修复 middleware/ratelimit.rs 中的锁使用

**文件**: `src/middleware/ratelimit.rs:150-188`

**问题**:
- 使用 `Mutex` 限制了并发性
- 在锁内执行 `cleanup()` 耗时操作
- 使用 `.lock()` 可能永久阻塞

**修复**:
```rust
// 使用 RwLock 替代 Mutex
static RATE_LIMITER: Lazy<Arc<RwLock<RateLimiter>>> = ...

// 使用 try_write 避免阻塞
if let Ok(mut limiter) = RATE_LIMITER.try_write() {
    limiter.cleanup();
    if let Err(e) = limiter.check(&key, &RATE_LIMIT_CONFIG) {
        return Err(...);
    }
} else {
    // 锁被占用，跳过限流检查
    eprintln!("⚠️  限流锁被占用，跳过限流检查");
}
```

**改进**:
- ✅ 使用 `RwLock` 提高并发性
- ✅ 使用 `try_write` 避免阻塞
- ✅ 降级策略：锁不可用时跳过限流，保证可用性

### 4. 添加全局锁监控

**文件**: `src/lock_monitor.rs` (新增)

**功能**:
- 监控锁的获取次数、等待次数、等待时间
- 检测死锁和性能问题
- 提供健康检查和统计报告

**关键特性**:
```rust
// 锁等待守卫
let guard = monitor.start_wait();
// ... 等待锁
guard.done(); // 记录等待时间

// 全局监控
GLOBAL_LOCK_MONITOR.register("cache_lock");
GLOBAL_LOCK_MONITOR.print_all_stats();
GLOBAL_LOCK_MONITOR.check_health();
```

**监控指标**:
- `acquire_count`: 获取锁的总次数
- `wait_count`: 等待获取锁的总次数
- `avg_wait_time_us`: 平均等待时间（微秒）
- `max_wait_time_us`: 最大等待时间（微秒）
- `current_waiters`: 当前等待的线程数

**健康检查**:
- 最大等待时间超过 5 秒 → 报警
- 当前等待线程超过 20 个 → 报警

## 修复效果对比

### 修复前
- ❌ 112,793 次 `FUTEX_WAIT` 调用
- ❌ 111 次 futex 超时
- ❌ 服务器完全卡死，ping 不通
- ❌ 线程间循环等待，形成死锁

### 修复后（预期）
- ✅ 使用 `RwLock` 实现读写分离，减少锁竞争
- ✅ 使用 `try_lock` 避免永久阻塞
- ✅ 消除嵌套锁，避免死锁
- ✅ 减少锁持有时间，提高并发性
- ✅ 添加锁监控，及时发现问题
- ✅ 降级策略保证系统可用性

## 测试建议

### 1. 压力测试
```bash
# 使用 ab (Apache Bench)
ab -n 10000 -c 100 http://localhost:8080/

# 使用 wrk
wrk -t12 -c400 -d30s http://localhost:8080/
```

### 2. 死锁检测
```bash
# 启动应用
./target/release/rustblog

# 监控进程状态
watch -n 1 'ps aux | grep rustblog'

# 检查是否有大量 D (uninterruptible sleep) 状态
ps aux | grep rustblog | grep D
```

### 3. 锁监控
在代码中添加锁监控：
```rust
use crate::lock_monitor::GLOBAL_LOCK_MONITOR;

let monitor = GLOBAL_LOCK_MONITOR.register("my_lock");
let _guard = monitor.start_wait();
let data = my_lock.lock().unwrap();
guard.done();
// ... 使用数据
```

定期打印统计信息：
```rust
GLOBAL_LOCK_MONITOR.print_all_stats();
GLOBAL_LOCK_MONITOR.check_health();
```

## 最佳实践总结

### 1. 避免锁的嵌套
```rust
// ❌ 错误：嵌套锁
fn bad() {
    let guard1 = lock1.lock();
    let guard2 = lock2.lock(); // 可能导致死锁
}

// ✅ 正确：按固定顺序获取锁
fn good() {
    let guard1 = lock1.lock();
    drop(guard1); // 释放第一个锁
    let guard2 = lock2.lock();
}
```

### 2. 减少锁的持有时间
```rust
// ❌ 错误：长时间持有锁
fn bad() {
    let guard = mutex.lock();
    expensive_operation(); // 在锁内执行耗时操作
}

// ✅ 正确：最小化锁的范围
fn good() {
    let data = {
        let guard = mutex.lock();
        guard.clone() // 只复制需要的数据
    };
    expensive_operation(); // 不在锁保护下执行
}
```

### 3. 使用超时机制
```rust
// ❌ 错误：可能永久阻塞
let guard = mutex.lock();

// ✅ 正确：使用 try_lock 或超时
if let Some(guard) = mutex.try_lock() {
    // 获取锁成功
} else {
    // 锁被占用，稍后重试或处理其他任务
}
```

### 4. 使用读写锁
```rust
// ✅ 使用 RwLock 实现读写分离
static CACHE: Lazy<RwLock<HashMap<...>>> = Lazy::new(...);

// 读取时使用读锁（允许并发）
let cache = CACHE.read().unwrap();

// 写入时使用写锁（独占）
let mut cache = CACHE.write().unwrap();
```

### 5. 使用无锁数据结构
```rust
// ✅ 使用 channel 替代共享内存
let (sender, receiver) = channel::unbounded();

// ✅ 使用原子操作
use std::sync::atomic::{AtomicUsize, Ordering};
let counter = AtomicUsize::new(0);
counter.fetch_add(1, Ordering::Relaxed);
```

## 文件修改清单

1. `src/handlers/api_handlers/passage.rs` - 修复 RENDER_CACHE 死锁
2. `src/cache/manager.rs` - 修复嵌套锁问题
3. `src/middleware/ratelimit.rs` - 优化锁使用
4. `src/lock_monitor.rs` - 新增锁监控模块
5. `src/main.rs` - 添加 lock_monitor 模块

## 后续优化建议

1. **使用 parking_lot 的 DeadlockDetection**
   ```rust
   #[cfg(debug_assertions)]
   parking_lot::DeadlockDetection::new();
   ```

2. **添加锁竞争日志**
   - 记录每次锁等待时间
   - 设置阈值，超过阈值记录警告

3. **使用更高级的并发原语**
   - `crossbeam::channel` 替代 `std::sync::mpsc`
   - `dashmap` 替代 `HashMap + Mutex`

4. **重构为异步架构**
   - 使用 `tokio::sync::RwLock`
   - 减少阻塞操作

5. **添加性能监控**
   - 集成 Prometheus 指标
   - 设置告警规则

## 版本信息

- 修复版本: v1.1.4+
- 修复日期: 2026-02-14
- 影响范围: 所有使用锁的代码路径
- 严重程度: 严重（导致服务器完全卡死）

## 参考资料

- [Rust 并发编程指南](https://doc.rust-lang.org/book/ch16-00-concurrency.html)
- [parking_lot 文档](https://docs.rs/parking_lot/)
- [tokio 同步原语](https://docs.rs/tokio/1/tokio/sync/)
- [Linux futex 系统调用](https://man7.org/linux/man-pages/man2/futex.2.html)