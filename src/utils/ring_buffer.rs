//! 环形缓冲区实现
//!
//! 用于优化缓存管理器的滑动窗口，减少内存分配和提高性能

use crossbeam_queue::ArrayQueue;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// 环形缓冲区条目
#[derive(Debug, Clone)]
pub struct RingBufferEntry<T> {
    /// 数据
    pub data: T,
    /// 时间戳
    pub timestamp: Instant,
}

/// 环形缓冲区
///
/// 基于 crossbeam_queue::ArrayQueue 实现真正的 lock-free push 路径，
/// 适用于高频写入 + 低频读取的场景（例如缓存管理器的滑动窗口统计）。
///
/// 语义：容量满时 push 会弹出最老的条目（FIFO），再插入新条目。
pub struct RingBuffer<T> {
    /// Lock-free 无界 FIFO 队列（有界容量）
    data: ArrayQueue<RingBufferEntry<T>>,
    /// 单调递增写入计数，用于诊断 / 测试
    write_index: AtomicU64,
    /// 缓冲区容量
    capacity: usize,
}

impl<T: Clone + Send> RingBuffer<T> {
    /// 创建新的环形缓冲区
    ///
    /// # 参数
    /// - `capacity`: 缓冲区容量
    pub fn new(capacity: usize) -> Self {
        Self {
            data: ArrayQueue::new(capacity),
            write_index: AtomicU64::new(0),
            capacity,
        }
    }

    /// 写入数据
    ///
    /// # 参数
    /// - `data`: 要写入的数据
    ///
    /// # 返回
    /// 被驱逐的最老条目（容量满时）
    pub fn push(&self, data: T) -> Option<RingBufferEntry<T>> {
        self.write_index.fetch_add(1, Ordering::Relaxed);
        let mut entry = RingBufferEntry {
            data,
            timestamp: Instant::now(),
        };
        loop {
            // 尝试直接 push（lock-free）
            entry = match self.data.push(entry) {
                Ok(()) => return None,
                Err(e) => e, // 队列满，e 是被拒绝的条目
            };
            // 队列满：驱逐最老的条目（队头），然后重试
            let evicted = self.data.pop();
            match self.data.push(entry) {
                Ok(()) => return evicted,
                Err(e) => {
                    // 另一个线程抢先填了空位，继续重试
                    entry = e;
                }
            }
        }
    }

    /// 读取指定位置的数据（仅供测试使用，生产路径请用 iter_valid）
    ///
    /// 注意：此方法会暂时排空队列再重新入队，不适合高并发场景。
    #[allow(dead_code)]
    pub fn get(&self, index: usize) -> Option<T> {
        if index >= self.capacity {
            return None;
        }
        // 排空队列，取第 index 个，然后全部重新入队
        let all: Vec<RingBufferEntry<T>> = std::iter::from_fn(|| self.data.pop()).collect();
        let result = all.get(index).map(|e| e.data.clone());
        for entry in all {
            let _ = self.data.push(entry);
        }
        result
    }

    /// 获取滑动窗口内所有有效条目
    ///
    /// # 参数
    /// - `max_age`: 超过此时间的条目将被丢弃（清理过期数据）
    ///
    /// # 返回
    /// 有效条目列表
    pub fn iter_valid(&self, max_age: std::time::Duration) -> Vec<RingBufferEntry<T>> {
        // 排空队列：保留有效条目并重新入队，丢弃过期条目（顺便清理）
        let all: Vec<RingBufferEntry<T>> = std::iter::from_fn(|| self.data.pop()).collect();
        let mut valid = Vec::with_capacity(all.len());
        for entry in all {
            if entry.timestamp.elapsed() < max_age {
                valid.push(entry.clone());
                // 重新入队；若队列被并发 push 填满则丢弃（可接受的近似统计）
                let _ = self.data.push(entry);
            }
            // 过期条目不重新入队，起到 GC 作用
        }
        valid
    }

    /// 获取缓冲区容量
    #[allow(dead_code)]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// 获取当前写入计数（单调递增）
    #[allow(dead_code)]
    pub fn write_index(&self) -> u64 {
        self.write_index.load(Ordering::Relaxed)
    }

    /// 清空缓冲区
    #[allow(dead_code)]
    pub fn clear(&self) {
        while self.data.pop().is_some() {}
    }
}

/// 用于缓存管理器的环形缓冲区实现
///
/// 专门优化用于记录操作历史，支持快速统计失败率
pub struct OperationHistoryBuffer {
    /// 操作历史环形缓冲区
    history: RingBuffer<bool>,
    /// 滑动窗口时间（秒）
    window_seconds: u64,
}

impl OperationHistoryBuffer {
    /// 创建新的操作历史缓冲区
    ///
    /// # 参数
    /// - `capacity`: 缓冲区容量
    /// - `window_seconds`: 滑动窗口时间（秒）
    pub fn new(capacity: usize, window_seconds: u64) -> Self {
        Self {
            history: RingBuffer::new(capacity),
            window_seconds,
        }
    }

    /// 记录操作结果
    ///
    /// # 参数
    /// - `failed`: 是否失败
    pub fn record(&self, failed: bool) {
        self.history.push(failed);
    }

    /// 计算滑动窗口内的失败率
    ///
    /// # 返回
    /// (总操作数, 失败数, 失败率百分比)
    pub fn calculate_failure_rate(&self) -> (usize, usize, f32) {
        let window_duration = std::time::Duration::from_secs(self.window_seconds);
        let valid_entries = self.history.iter_valid(window_duration);

        let total_operations = valid_entries.len();
        let failure_count = valid_entries.iter().filter(|e| e.data).count();

        let failure_rate = if total_operations > 0 {
            (failure_count as f32 / total_operations as f32) * 100.0
        } else {
            0.0
        };

        (total_operations, failure_count, failure_rate)
    }

    /// 检查是否应该降级
    ///
    /// # 参数
    /// - `threshold`: 失败率阈值（0-100）
    /// - `min_sample_size`: 最小样本数
    ///
    /// # 返回
    /// 是否应该降级
    pub fn should_degrade(&self, threshold: f32, min_sample_size: usize) -> bool {
        let (total, _failures, rate) = self.calculate_failure_rate();

        if total < min_sample_size {
            return false;
        }

        rate >= threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_ring_buffer_basic() {
        let buffer = RingBuffer::new(5);

        // 写入数据
        buffer.push(1);
        buffer.push(2);
        buffer.push(3);

        // 读取数据
        assert_eq!(buffer.get(0), Some(1));
        assert_eq!(buffer.get(1), Some(2));
        assert_eq!(buffer.get(2), Some(3));
        assert_eq!(buffer.get(10), None);
    }

    #[test]
    fn test_ring_buffer_overflow() {
        let buffer = RingBuffer::new(3);

        // 写入超过容量的数据
        buffer.push(1);
        buffer.push(2);
        buffer.push(3);
        let old = buffer.push(4); // 应该覆盖第一个

        assert!(old.is_some());
        assert_eq!(old.unwrap().data, 1);
    }

    #[test]
    fn test_ring_buffer_concurrent() {
        let buffer = Arc::new(RingBuffer::new(1000));
        let mut handles = Vec::new();

        // 并发写入
        for i in 0..10 {
            let buffer_clone = Arc::clone(&buffer);
            let handle = thread::spawn(move || {
                for j in 0..100 {
                    buffer_clone.push(i * 100 + j);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // 检查写入位置
        assert_eq!(buffer.write_index(), 1000);
    }

    #[test]
    fn test_operation_history_buffer() {
        let buffer = OperationHistoryBuffer::new(10, 5);

        // 记录操作
        buffer.record(false);
        buffer.record(false);
        buffer.record(true);
        buffer.record(false);
        buffer.record(true);

        let (total, failures, rate) = buffer.calculate_failure_rate();
        assert_eq!(total, 5);
        assert_eq!(failures, 2);
        assert!((rate - 40.0).abs() < 0.01);
    }

    #[test]
    fn test_should_degrade() {
        let buffer = OperationHistoryBuffer::new(10, 5);

        // 记录操作，失败率超过50%
        for _ in 0..10 {
            buffer.record(true);
        }

        assert!(buffer.should_degrade(50.0, 5));

        // 验证失败率计算
        let (total, _failures, rate) = buffer.calculate_failure_rate();
        assert_eq!(total, 10);
        assert_eq!(rate, 100.0);
    }

    #[test]
    fn test_should_not_degrade_low_sample() {
        let buffer = OperationHistoryBuffer::new(10, 5);

        // 只记录少量操作
        buffer.record(true);
        buffer.record(true);

        // 样本太少，不应降级
        assert!(!buffer.should_degrade(50.0, 5));

        // 检查失败率计算
        let (total, _failures, rate) = buffer.calculate_failure_rate();
        assert_eq!(total, 2);
        assert_eq!(rate, 100.0);
    }

    #[test]
    fn test_window_expiration() {
        let buffer = OperationHistoryBuffer::new(10, 1); // 1秒窗口

        // 记录操作
        buffer.record(true);
        buffer.record(true);

        // 等待窗口过期
        thread::sleep(Duration::from_secs(2));

        // 记录新操作
        buffer.record(false);

        let (total, failures, rate) = buffer.calculate_failure_rate();
        // 应该只有最后一个操作
        assert_eq!(total, 1);
        assert_eq!(failures, 0);
        assert_eq!(rate, 0.0);
    }
}
