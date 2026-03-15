//! 锁监控模块
//! 用于检测和监控锁的使用情况，帮助发现死锁和性能问题

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
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
            self.total_wait_time_us.fetch_add(wait_time_us as usize, Ordering::Relaxed);

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
            eprintln!("⚠️  警告: 锁 '{}' 最大等待时间超过 1 秒，可能存在死锁或性能问题！", stats.name);
        }

        // 警告：如果当前有大量线程等待
        if stats.current_waiters > 10 {
            eprintln!("⚠️  警告: 锁 '{}' 当前有 {} 个线程在等待，可能存在严重竞争！", stats.name, stats.current_waiters);
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
            if stats.max_wait_time_us > 5_000_000 { // 5 秒
                eprintln!("❌ 锁 '{}' 出现严重问题：最大等待时间 {} 秒",
                          stats.name, stats.max_wait_time_us as f64 / 1_000_000.0);
                healthy = false;
            }

            if stats.current_waiters > 20 { // 20 个线程等待
                eprintln!("❌ 锁 '{}' 出现严重竞争：{} 个线程在等待",
                          stats.name, stats.current_waiters);
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