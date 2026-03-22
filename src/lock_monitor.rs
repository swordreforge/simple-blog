//! 锁监控模块
//! 用于检测和监控锁的使用情况，帮助发现死锁和性能问题

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

/// 锁监控统计
#[derive(Debug, Clone)]
pub struct LockStats {
    /// 锁名称
    pub name: String,
    /// 获取锁的总次数
    pub acquire_count: usize,
    /// 等待获取锁的总次数
    pub wait_count: usize,
    /// 平均等待时间（微秒）
    pub avg_wait_time_us: u64,
    /// 最大等待时间（微秒）
    pub max_wait_time_us: u64,
    /// 当前等待的线程数
    pub current_waiters: usize,
}

/// 锁监控器
#[allow(dead_code)]
pub struct LockMonitor {
    name: String,
    acquire_count: Arc<AtomicUsize>,
    wait_count: Arc<AtomicUsize>,
    total_wait_time_us: Arc<AtomicUsize>,
    max_wait_time_us: Arc<AtomicUsize>,
    current_waiters: Arc<AtomicUsize>,
}

#[allow(dead_code)]
impl LockMonitor {
    /// 创建新的锁监控器
    pub fn new(name: String) -> Self {
        Self {
            name,
            acquire_count: Arc::new(AtomicUsize::new(0)),
            wait_count: Arc::new(AtomicUsize::new(0)),
            total_wait_time_us: Arc::new(AtomicUsize::new(0)),
            max_wait_time_us: Arc::new(AtomicUsize::new(0)),
            current_waiters: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// 开始等待锁
    pub fn start_wait(&self) -> LockWaitGuard {
        self.current_waiters.fetch_add(1, Ordering::Relaxed);
        LockWaitGuard {
            monitor: self.clone(),
            start_time: Instant::now(),
        }
    }

    /// 获取锁（记录获取）
    pub fn record_acquire(&self, wait_time_us: u64) {
        self.acquire_count.fetch_add(1, Ordering::Relaxed);
        if wait_time_us > 0 {
            self.wait_count.fetch_add(1, Ordering::Relaxed);
            self.total_wait_time_us
                .fetch_add(wait_time_us as usize, Ordering::Relaxed);

            // 更新最大等待时间
            let mut current_max = self.max_wait_time_us.load(Ordering::Relaxed);
            let wait_time_us_usize = wait_time_us as usize;
            while wait_time_us_usize > current_max {
                match self.max_wait_time_us.compare_exchange_weak(
                    current_max,
                    wait_time_us_usize,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break,
                    Err(new_max) => current_max = new_max,
                }
            }
        }
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> LockStats {
        let acquire_count = self.acquire_count.load(Ordering::Relaxed);
        let wait_count = self.wait_count.load(Ordering::Relaxed);
        let total_wait_time_us = self.total_wait_time_us.load(Ordering::Relaxed);
        let avg_wait_time_us = total_wait_time_us.checked_div(wait_count).unwrap_or(0);
        let max_wait_time_us = self.max_wait_time_us.load(Ordering::Relaxed);
        let current_waiters = self.current_waiters.load(Ordering::Relaxed);

        LockStats {
            name: self.name.clone(),
            acquire_count,
            wait_count,
            avg_wait_time_us: avg_wait_time_us as u64,
            max_wait_time_us: max_wait_time_us as u64,
            current_waiters,
        }
    }

    /// 重置统计信息
    pub fn reset(&self) {
        self.acquire_count.store(0, Ordering::Relaxed);
        self.wait_count.store(0, Ordering::Relaxed);
        self.total_wait_time_us.store(0, Ordering::Relaxed);
        self.max_wait_time_us.store(0, Ordering::Relaxed);
    }

    /// 打印统计信息
    pub fn print_stats(&self) {
        let stats = self.get_stats();
        println!("📊 锁监控: {}", stats.name);
        println!("   - 获取次数: {}", stats.acquire_count);
        println!("   - 等待次数: {}", stats.wait_count);
        println!("   - 平均等待时间: {} μs", stats.avg_wait_time_us);
        println!("   - 最大等待时间: {} μs", stats.max_wait_time_us);
        println!("   - 当前等待线程: {}", stats.current_waiters);

        // 警告：如果最大等待时间超过 1 秒
        if stats.max_wait_time_us > 1_000_000 {
            eprintln!(
                "⚠️  警告: 锁 '{}' 最大等待时间超过 1 秒，可能存在死锁或性能问题！",
                stats.name
            );
        }

        // 警告：如果当前有大量线程等待
        if stats.current_waiters > 10 {
            eprintln!(
                "⚠️  警告: 锁 '{}' 当前有 {} 个线程在等待，可能存在严重竞争！",
                stats.name, stats.current_waiters
            );
        }
    }
}

impl Clone for LockMonitor {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            acquire_count: Arc::clone(&self.acquire_count),
            wait_count: Arc::clone(&self.wait_count),
            total_wait_time_us: Arc::clone(&self.total_wait_time_us),
            max_wait_time_us: Arc::clone(&self.max_wait_time_us),
            current_waiters: Arc::clone(&self.current_waiters),
        }
    }
}

/// 锁等待守卫
#[allow(dead_code)]
pub struct LockWaitGuard {
    monitor: LockMonitor,
    start_time: Instant,
}

#[allow(dead_code)]
impl LockWaitGuard {
    /// 完成等待，获取到锁
    pub fn done(self) {
        let wait_time_us = self.start_time.elapsed().as_micros() as u64;
        self.monitor.current_waiters.fetch_sub(1, Ordering::Relaxed);
        self.monitor.record_acquire(wait_time_us);
    }
}

impl Drop for LockWaitGuard {
    fn drop(&mut self) {
        // 如果守卫被销毁但没有调用 done()，仍然减少等待计数器
        self.monitor.current_waiters.fetch_sub(1, Ordering::Relaxed);
    }
}

/// 全局锁监控器管理
#[allow(dead_code)]
pub struct GlobalLockMonitor {
    monitors: parking_lot::RwLock<Vec<LockMonitor>>,
}

#[allow(dead_code)]
impl GlobalLockMonitor {
    /// 创建新的全局监控器
    pub fn new() -> Self {
        Self {
            monitors: parking_lot::RwLock::new(Vec::new()),
        }
    }

    /// 注册新的锁监控器
    pub fn register(&self, name: String) -> LockMonitor {
        let monitor = LockMonitor::new(name);
        // parking_lot::RwLock 不会锁中毒，直接获取写锁
        let mut monitors = self.monitors.write();
        monitors.push(monitor.clone());
        monitor
    }

    /// 打印所有锁的统计信息
    pub fn print_all_stats(&self) {
        // parking_lot::RwLock 不会锁中毒，直接获取读锁
        let monitors = self.monitors.read();
        println!("\n==========================================");
        println!("📊 全局锁监控报告");
        println!("==========================================\n");
        for monitor in monitors.iter() {
            monitor.print_stats();
            println!();
        }
        println!("==========================================\n");
    }

    /// 检查是否有锁出现严重问题
    pub fn check_health(&self) -> bool {
        // parking_lot::RwLock 不会锁中毒，直接获取读锁
        let monitors = self.monitors.read();
        let mut healthy = true;

        for monitor in monitors.iter() {
            let stats = monitor.get_stats();

            // 检查是否有严重的等待
            if stats.max_wait_time_us > 5_000_000 {
                // 5 秒
                eprintln!(
                    "❌ 锁 '{}' 出现严重问题：最大等待时间 {} 秒",
                    stats.name,
                    stats.max_wait_time_us as f64 / 1_000_000.0
                );
                healthy = false;
            }

            if stats.current_waiters > 20 {
                // 20 个线程等待
                eprintln!(
                    "❌ 锁 '{}' 出现严重竞争：{} 个线程在等待",
                    stats.name, stats.current_waiters
                );
                healthy = false;
            }
        }

        healthy
    }
}

impl Default for GlobalLockMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局锁监控器实例
#[allow(dead_code)]
pub static GLOBAL_LOCK_MONITOR: Lazy<GlobalLockMonitor> = Lazy::new(GlobalLockMonitor::new);

use once_cell::sync::Lazy;

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_lock_stats_debug() {
        let stats = LockStats {
            name: "test_lock".to_string(),
            acquire_count: 100,
            wait_count: 50,
            avg_wait_time_us: 1000,
            max_wait_time_us: 5000,
            current_waiters: 2,
        };
        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("test_lock"));
        assert!(debug_str.contains("100"));
    }

    #[test]
    fn test_lock_stats_clone() {
        let stats = LockStats {
            name: "test_lock".to_string(),
            acquire_count: 100,
            wait_count: 50,
            avg_wait_time_us: 1000,
            max_wait_time_us: 5000,
            current_waiters: 2,
        };
        let cloned = stats.clone();
        assert_eq!(stats.name, cloned.name);
        assert_eq!(stats.acquire_count, cloned.acquire_count);
    }

    #[test]
    fn test_lock_monitor_new() {
        let monitor = LockMonitor::new("test_lock".to_string());
        assert_eq!(monitor.name, "test_lock");
    }

    #[test]
    fn test_lock_monitor_clone() {
        let monitor = LockMonitor::new("test_lock".to_string());
        let cloned = monitor.clone();
        assert_eq!(monitor.name, cloned.name);
    }

    #[test]
    fn test_lock_monitor_start_wait() {
        let monitor = LockMonitor::new("test_lock".to_string());
        let guard = monitor.start_wait();
        let stats = monitor.get_stats();
        assert_eq!(stats.current_waiters, 1);
        drop(guard);
        
        let stats = monitor.get_stats();
        assert_eq!(stats.current_waiters, 0);
    }

    #[test]
    fn test_lock_monitor_record_acquire_no_wait() {
        let monitor = LockMonitor::new("test_lock".to_string());
        monitor.record_acquire(0);
        
        let stats = monitor.get_stats();
        assert_eq!(stats.acquire_count, 1);
        assert_eq!(stats.wait_count, 0);
        assert_eq!(stats.avg_wait_time_us, 0);
    }

    #[test]
    fn test_lock_monitor_record_acquire_with_wait() {
        let monitor = LockMonitor::new("test_lock".to_string());
        monitor.record_acquire(1000); // 1000 微秒
        
        let stats = monitor.get_stats();
        assert_eq!(stats.acquire_count, 1);
        assert_eq!(stats.wait_count, 1);
        assert_eq!(stats.avg_wait_time_us, 1000);
        assert_eq!(stats.max_wait_time_us, 1000);
    }

    #[test]
    fn test_lock_monitor_record_acquire_multiple() {
        let monitor = LockMonitor::new("test_lock".to_string());
        monitor.record_acquire(1000);
        monitor.record_acquire(2000);
        monitor.record_acquire(3000);
        
        let stats = monitor.get_stats();
        assert_eq!(stats.acquire_count, 3);
        assert_eq!(stats.wait_count, 3);
        assert_eq!(stats.avg_wait_time_us, 2000); // (1000 + 2000 + 3000) / 3
        assert_eq!(stats.max_wait_time_us, 3000);
    }

    #[test]
    fn test_lock_monitor_get_stats() {
        let monitor = LockMonitor::new("test_lock".to_string());
        monitor.record_acquire(1000);
        
        let stats = monitor.get_stats();
        assert_eq!(stats.name, "test_lock");
        assert_eq!(stats.acquire_count, 1);
        assert_eq!(stats.wait_count, 1);
    }

    #[test]
    fn test_lock_monitor_reset() {
        let monitor = LockMonitor::new("test_lock".to_string());
        monitor.record_acquire(1000);
        monitor.record_acquire(2000);
        
        monitor.reset();
        
        let stats = monitor.get_stats();
        assert_eq!(stats.acquire_count, 0);
        assert_eq!(stats.wait_count, 0);
        assert_eq!(stats.avg_wait_time_us, 0);
        assert_eq!(stats.max_wait_time_us, 0);
    }

    #[test]
    fn test_lock_monitor_max_wait_time_update() {
        let monitor = LockMonitor::new("test_lock".to_string());
        monitor.record_acquire(1000);
        monitor.record_acquire(500);
        monitor.record_acquire(2000);
        
        let stats = monitor.get_stats();
        assert_eq!(stats.max_wait_time_us, 2000);
    }

    #[test]
    fn test_lock_wait_guard_done() {
        let monitor = LockMonitor::new("test_lock".to_string());
        let guard = monitor.start_wait();
        thread::sleep(Duration::from_millis(10));
        guard.done();
        
        let stats = monitor.get_stats();
        assert_eq!(stats.acquire_count, 1);
        assert_eq!(stats.wait_count, 1);
        assert!(stats.avg_wait_time_us > 0);
    }

    #[test]
    fn test_lock_wait_guard_drop_without_done() {
        let monitor = LockMonitor::new("test_lock".to_string());
        {
            let _guard = monitor.start_wait();
            thread::sleep(Duration::from_millis(10));
            // 守卫在这里被drop，但没有调用done()
        }
        
        let stats = monitor.get_stats();
        assert_eq!(stats.current_waiters, 0);
        // acquire_count不应该增加，因为没有调用done()
        assert_eq!(stats.acquire_count, 0);
    }

    #[test]
    fn test_global_lock_monitor_new() {
        let global_monitor = GlobalLockMonitor::new();
        let stats = global_monitor.check_health();
        assert!(stats); // 没有监控器时应该是健康的
    }

    #[test]
    fn test_global_lock_monitor_register() {
        let global_monitor = GlobalLockMonitor::new();
        let monitor = global_monitor.register("test_lock".to_string());
        
        assert_eq!(monitor.name, "test_lock");
    }

    #[test]
    fn test_global_lock_monitor_check_health() {
        let global_monitor = GlobalLockMonitor::new();
        global_monitor.register("test_lock".to_string());
        
        // 没有严重问题时应该是健康的
        let healthy = global_monitor.check_health();
        assert!(healthy);
    }

    #[test]
    fn test_global_lock_monitor_default() {
        let global_monitor = GlobalLockMonitor::default();
        let healthy = global_monitor.check_health();
        assert!(healthy);
    }

    #[test]
    fn test_lock_monitor_concurrent_updates() {
        let monitor = Arc::new(LockMonitor::new("concurrent_lock".to_string()));
        let mut handles = vec![];

        for _i in 0..10 {
            let monitor_clone = Arc::clone(&monitor);
            let handle = thread::spawn(move || {
                let guard = monitor_clone.start_wait();
                thread::sleep(Duration::from_millis(1));
                guard.done();
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let stats = monitor.get_stats();
        assert_eq!(stats.acquire_count, 10);
        assert_eq!(stats.wait_count, 10);
    }

    #[test]
    fn test_lock_monitor_average_wait_time_zero_division() {
        let monitor = LockMonitor::new("test_lock".to_string());
        monitor.record_acquire(0); // 没有等待时间
        
        let stats = monitor.get_stats();
        assert_eq!(stats.avg_wait_time_us, 0);
    }

    #[test]
    fn test_lock_monitor_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        
        assert_send::<LockMonitor>();
        assert_sync::<LockMonitor>();
        
        assert_send::<LockStats>();
        assert_sync::<LockStats>();
        
        assert_send::<LockWaitGuard>();
        // LockWaitGuard 不需要 Sync
        
        assert_send::<GlobalLockMonitor>();
        assert_sync::<GlobalLockMonitor>();
    }

    #[test]
    fn test_lock_monitor_with_zero_wait_time() {
        let monitor = LockMonitor::new("test_lock".to_string());
        monitor.record_acquire(0);
        monitor.record_acquire(0);
        
        let stats = monitor.get_stats();
        assert_eq!(stats.acquire_count, 2);
        assert_eq!(stats.wait_count, 0);
        assert_eq!(stats.avg_wait_time_us, 0);
    }

    #[test]
    fn test_lock_monitor_large_wait_times() {
        let monitor = LockMonitor::new("test_lock".to_string());
        monitor.record_acquire(1_000_000); // 1 秒
        monitor.record_acquire(2_000_000); // 2 秒
        
        let stats = monitor.get_stats();
        assert_eq!(stats.max_wait_time_us, 2_000_000);
        assert_eq!(stats.avg_wait_time_us, 1_500_000);
    }

    #[test]
    fn test_lock_monitor_concurrent_acquires() {
        let monitor = Arc::new(LockMonitor::new("concurrent_acquires".to_string()));
        let mut handles = vec![];

        for _ in 0..5 {
            let monitor_clone = Arc::clone(&monitor);
            let handle = thread::spawn(move || {
                let guard = monitor_clone.start_wait();
                thread::sleep(Duration::from_millis(1));
                guard.done();
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // 等待一小段时间确保所有操作完成
        thread::sleep(Duration::from_millis(10));
        
        let stats = monitor.get_stats();
        assert_eq!(stats.acquire_count, 5);
        assert_eq!(stats.wait_count, 5);
    }
}
