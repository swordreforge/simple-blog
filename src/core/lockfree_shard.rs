//! 无锁动态分片实现
//!
//! 使用原子操作和无锁数据结构减少锁竞争，提升并发性能。

use super::route_entry::RouteEntry;
use super::route_radix_tree::RouteRadixTree;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// 无锁分片负载指标
///
/// 使用原子操作实现无锁计数器，避免锁竞争
#[derive(Debug)]
pub struct LockfreeShardMetrics {
    /// 分片中的路由数量
    pub route_count: AtomicUsize,
    /// 总访问次数
    pub total_access: AtomicUsize,
    /// 读取次数
    pub read_count: AtomicUsize,
    /// 写入次数
    pub write_count: AtomicUsize,
    /// 平均访问延迟（纳秒）
    pub avg_latency_ns: AtomicUsize,
    /// 当前活跃连接数
    pub active_connections: AtomicUsize,
}

impl Default for LockfreeShardMetrics {
    fn default() -> Self {
        Self {
            route_count: AtomicUsize::new(0),
            total_access: AtomicUsize::new(0),
            read_count: AtomicUsize::new(0),
            write_count: AtomicUsize::new(0),
            avg_latency_ns: AtomicUsize::new(0),
            active_connections: AtomicUsize::new(0),
        }
    }
}

impl LockfreeShardMetrics {
    /// 增加路由计数
    pub fn increment_route_count(&self) {
        self.route_count.fetch_add(1, Ordering::Relaxed);
    }

    /// 减少路由计数
    pub fn decrement_route_count(&self) {
        self.route_count.fetch_sub(1, Ordering::Relaxed);
    }

    /// 获取路由计数
    pub fn get_route_count(&self) -> usize {
        self.route_count.load(Ordering::Relaxed)
    }

    /// 记录读取操作
    pub fn record_read(&self, latency_ns: u64) {
        self.read_count.fetch_add(1, Ordering::Relaxed);
        self.total_access.fetch_add(1, Ordering::Relaxed);

        // 更新平均延迟（使用原子操作）
        let new_latency = latency_ns as usize;
        let total = self.total_access.load(Ordering::Relaxed);
        let current_avg = self.avg_latency_ns.load(Ordering::Relaxed);
        let new_avg = (current_avg * total.saturating_sub(1) + new_latency).checked_div(total);
        if let Some(avg) = new_avg {
            self.avg_latency_ns.store(avg, Ordering::Relaxed);
        }
    }

    /// 记录写入操作
    pub fn record_write(&self) {
        self.write_count.fetch_add(1, Ordering::Relaxed);
        self.total_access.fetch_add(1, Ordering::Relaxed);
    }

    /// 增加活跃连接数
    pub fn increment_active_connections(&self) {
        self.active_connections.fetch_add(1, Ordering::Relaxed);
    }

    /// 减少活跃连接数
    pub fn decrement_active_connections(&self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }

    /// 计算负载分数（0.0 - 1.0，越高负载越大）
    pub fn load_score(&self) -> f64 {
        let route_count = self.route_count.load(Ordering::Relaxed);
        let total_access = self.total_access.load(Ordering::Relaxed);
        let avg_latency = self.avg_latency_ns.load(Ordering::Relaxed);
        let active_connections = self.active_connections.load(Ordering::Relaxed);

        let count_score = (route_count as f64 / 1000.0).min(1.0);
        let access_score = (total_access as f64 / 10000.0).min(1.0);
        let latency_score = (avg_latency as f64 / 1_000_000.0).min(1.0);
        let connection_score = (active_connections as f64 / 100.0).min(1.0);

        // 加权计算负载分数
        count_score * 0.3 + access_score * 0.3 + latency_score * 0.2 + connection_score * 0.2
    }

    /// 创建快照（用于调试和监控）
    pub fn snapshot(&self) -> ShardMetricsSnapshot {
        ShardMetricsSnapshot {
            route_count: self.route_count.load(Ordering::Relaxed),
            total_access: self.total_access.load(Ordering::Relaxed),
            read_count: self.read_count.load(Ordering::Relaxed),
            write_count: self.write_count.load(Ordering::Relaxed),
            avg_latency_ns: self.avg_latency_ns.load(Ordering::Relaxed),
            active_connections: self.active_connections.load(Ordering::Relaxed),
        }
    }
}

/// 分片指标快照
#[derive(Debug, Clone)]
pub struct ShardMetricsSnapshot {
    pub route_count: usize,
    pub total_access: usize,
    pub read_count: usize,
    pub write_count: usize,
    pub avg_latency_ns: usize,
    pub active_connections: usize,
}

/// 无锁动态分片
///
/// 使用读写分离策略：
/// - 读操作：直接访问RouteRadixTree（无锁）
/// - 写操作：使用RwLock保护RouteRadixTree
/// - 指标更新：使用原子操作（无锁）
pub struct LockfreeDynamicShard {
    inner: RouteRadixTree,
    metrics: LockfreeShardMetrics,
    id: usize,
}

impl LockfreeDynamicShard {
    pub fn new(id: usize) -> Self {
        Self {
            inner: RouteRadixTree::new(),
            metrics: LockfreeShardMetrics::default(),
            id,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn route_count(&self) -> usize {
        self.metrics.get_route_count()
    }

    pub fn metrics(&self) -> &LockfreeShardMetrics {
        &self.metrics
    }

    /// 插入路由（需要锁保护）
    pub fn insert(&mut self, path: &str, route: Box<dyn RouteEntry>) {
        let existed = self.inner.contains(path);
        self.inner.insert(path, route);
        if !existed {
            self.metrics.increment_route_count();
        }
        self.metrics.record_write();
    }

    /// 插入路由（直接使用Arc，零拷贝）
    pub fn insert_arc(&mut self, path: &str, route: Arc<dyn RouteEntry>) {
        let existed = self.inner.contains(path);
        self.inner.insert_arc(path, route);
        if !existed {
            self.metrics.increment_route_count();
        }
        self.metrics.record_write();
    }

    /// 移除路由（需要锁保护）
    pub fn remove(&mut self, path: &str) -> Option<Arc<dyn RouteEntry>> {
        let result = self.inner.remove(path);
        if result.is_some() {
            self.metrics.decrement_route_count();
        }
        self.metrics.record_write();
        result
    }

    /// 查找路由（无锁读操作）
    pub fn find(&self, path: &str) -> Option<(&Arc<dyn RouteEntry>, Vec<(String, String)>)> {
        let start = std::time::Instant::now();
        let result = self.inner.find(path);
        let duration = start.elapsed();

        if result.is_some() {
            self.metrics.record_read(duration.as_nanos() as u64);
        }

        result
    }

    /// 检查路径是否存在（无锁读操作）
    pub fn contains(&self, path: &str) -> bool {
        self.inner.contains(path)
    }

    /// 列出所有路径（无锁读操作）
    pub fn list_paths(&self) -> Vec<String> {
        self.inner.list_paths()
    }

    /// 列出所有路径（带预分配容量优化）
    pub fn list_paths_with_capacity(&self) -> Vec<String> {
        let count = self.route_count();
        let mut paths = Vec::with_capacity(count);
        paths.extend(self.inner.list_paths());
        paths
    }

    /// 清空分片（需要锁保护）
    pub fn clear(&mut self) {
        self.inner.clear();
        // 重置指标
        self.metrics.route_count.store(0, Ordering::Relaxed);
        self.metrics.total_access.store(0, Ordering::Relaxed);
        self.metrics.read_count.store(0, Ordering::Relaxed);
        self.metrics.write_count.store(0, Ordering::Relaxed);
        self.metrics.avg_latency_ns.store(0, Ordering::Relaxed);
        self.metrics.active_connections.store(0, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::super::SimpleRoute;
    use super::*;

    #[test]
    fn test_lockfree_metrics_atomic_operations() {
        let metrics = LockfreeShardMetrics::default();

        // 测试路由计数
        metrics.increment_route_count();
        assert_eq!(metrics.get_route_count(), 1);

        metrics.increment_route_count();
        assert_eq!(metrics.get_route_count(), 2);

        metrics.decrement_route_count();
        assert_eq!(metrics.get_route_count(), 1);

        // 测试写入记录
        metrics.record_write();
        assert_eq!(metrics.write_count.load(Ordering::Relaxed), 1);

        // 测试读取记录
        metrics.record_read(1000);
        assert_eq!(metrics.read_count.load(Ordering::Relaxed), 1);
        // 因为之前有一次写入，total_access=2，所以平均延迟 = (0*1 + 1000)/2 = 500
        assert_eq!(metrics.avg_latency_ns.load(Ordering::Relaxed), 500);
    }

    #[test]
    fn test_lockfree_metrics_load_score() {
        let metrics = LockfreeShardMetrics::default();

        // 初始负载分数应该为0
        assert_eq!(metrics.load_score(), 0.0);

        // 增加一些负载
        for _ in 0..100 {
            metrics.increment_route_count();
        }

        for _ in 0..1000 {
            metrics.record_write();
        }

        for _ in 0..10000 {
            metrics.record_read(1000);
        }

        let score = metrics.load_score();
        assert!(score > 0.0);
        assert!(score <= 1.0);
    }

    #[test]
    fn test_lockfree_shard_basic_operations() {
        let mut shard = LockfreeDynamicShard::new(0);
        let route = SimpleRoute::new("test", "text/plain");

        // 测试插入
        shard.insert("/test", Box::new(route.clone()));
        assert_eq!(shard.route_count(), 1);
        assert!(shard.contains("/test"));

        // 测试查找
        let found = shard.find("/test");
        assert!(found.is_some());

        // 测试移除
        shard.remove("/test");
        assert_eq!(shard.route_count(), 0);
        assert!(!shard.contains("/test"));
    }

    #[test]
    fn test_lockfree_shard_concurrent_reads() {
        let shard = Arc::new(std::sync::RwLock::new(LockfreeDynamicShard::new(0)));

        // 预填充一些路由
        {
            let mut guard = shard.write().unwrap();
            for i in 0..100 {
                guard.insert(
                    &format!("/route-{}", i),
                    Box::new(SimpleRoute::new("content", "text/plain")),
                );
            }
        }

        // 并发读取测试
        let mut handles = vec![];
        for thread_id in 0..10 {
            let shard_clone = Arc::clone(&shard);
            let handle = std::thread::spawn(move || {
                for i in 0..1000 {
                    let idx = thread_id * 10 + (i % 10);
                    let guard = shard_clone.read().unwrap();
                    guard.find(&format!("/route-{}", idx));
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // 验证读取指标
        let guard = shard.read().unwrap();
        let metrics = guard.metrics();
        assert!(metrics.read_count.load(Ordering::Relaxed) > 0);
    }
}