use std::sync::atomic::{AtomicU64, Ordering};
use once_cell::sync::Lazy;

/// 无锁 ID 生成器，使用原子操作和 Snowflake 算法
pub struct IdGenerator {
    worker_id: u16,
    sequence: AtomicU64,
    last_timestamp: AtomicU64,
}

impl IdGenerator {
    /// 创建新的 ID 生成器
    #[cfg(test)]
    pub fn new(machine_id: u64) -> Self {
        let worker_id = (machine_id % 1024) as u16;
        Self {
            worker_id,
            sequence: AtomicU64::new(0),
            last_timestamp: AtomicU64::new(0),
        }
    }

    /// 从字节数组创建 ID 生成器
    pub fn from_bytes(machine_id_bytes: [u8; 6]) -> Self {
        let raw_worker_id = u16::from_be_bytes([machine_id_bytes[0], machine_id_bytes[1]]);
        let worker_id = (raw_worker_id % 1024) as u16;
        Self {
            worker_id,
            sequence: AtomicU64::new(0),
            last_timestamp: AtomicU64::new(0),
        }
    }

    /// 生成下一个唯一 ID（无锁实现）
    pub fn generate_id(&self) -> String {
        loop {
            // 获取当前时间戳（毫秒）
            let current_timestamp = Self::get_timestamp();

            // 尝试获取上一次的时间戳
            let last_timestamp = self.last_timestamp.load(Ordering::Acquire);

            if current_timestamp == last_timestamp {
                // 同一毫秒内，序列号递增
                let seq = self.sequence.fetch_add(1, Ordering::AcqRel);
                if seq >= 4095 {
                    // 序列号溢出，等待下一毫秒
                    self.wait_for_next_millis(last_timestamp);
                    continue; // 重试
                }
                return self.compose_id(current_timestamp, seq);
            } else if current_timestamp < last_timestamp {
                // 时钟回拨，等待
                self.wait_for_next_millis(last_timestamp);
                continue; // 重试
            } else {
                // 新的毫秒，重置序列号
                self.sequence.store(0, Ordering::Release);
                self.last_timestamp.store(current_timestamp, Ordering::Release);
                return self.compose_id(current_timestamp, 0);
            }
        }
    }

    fn compose_id(&self, timestamp: u64, sequence: u64) -> String {
        // Snowflake ID 格式: 41位时间戳 + 10位worker_id + 12位序列号
        let id = (timestamp << 22) | ((self.worker_id as u64) << 12) | sequence;
        id.to_string()
    }

    fn get_timestamp() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| {
                // 如果时间倒流（极不可能的情况），返回一个安全的默认值
                // 在实际生产环境中，这种情况应该被记录日志
                std::time::Duration::from_secs(0)
            });
        duration.as_millis() as u64
    }

    fn wait_for_next_millis(&self, last_timestamp: u64) {
        let mut current = Self::get_timestamp();
        while current <= last_timestamp {
            std::hint::spin_loop();
            current = Self::get_timestamp();
        }
        self.sequence.store(0, Ordering::Release);
        self.last_timestamp.store(current, Ordering::Release);
    }
}

/// 全局 ID 生成器实例（无锁）
static ID_GENERATOR: Lazy<IdGenerator> = Lazy::new(|| {
    let machine_id_bytes = crate::db::repositories::get_machine_id();
    IdGenerator::from_bytes(machine_id_bytes)
});

/// 生成全局唯一 ID 的便捷函数
/// 
/// # 返回
/// 返回生成的唯一 ID 字符串
pub fn generate_unique_id() -> String {
    ID_GENERATOR.generate_id()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_id_generation() {
        let mut generator = IdGenerator::new(1);
        let id1 = generator.generate_id();
        let id2 = generator.generate_id();
        
        assert_ne!(id1, id2, "生成的 ID 应该是唯一的");
    }
    
    #[test]
    fn test_global_id_generation() {
        let id1 = generate_unique_id();
        let id2 = generate_unique_id();
        
        assert_ne!(id1, id2, "全局生成的 ID 应该是唯一的");
    }
}