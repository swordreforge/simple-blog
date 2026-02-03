use snowflake_id_generator::multi_thread::sync_generator::SnowflakeGenerator;
use once_cell::sync::Lazy;

/// ID 生成器，使用 snowflake-id-generator 库生成 Snowflake 风格的唯一 ID
pub struct IdGenerator {
    generator: SnowflakeGenerator,
}

impl IdGenerator {
    /// 创建新的 ID 生成器
    /// 
    /// # 参数
    /// * `machine_id` - 机器 ID，用于分布式环境下的唯一性标识
    #[cfg(test)]
    pub fn new(machine_id: u64) -> Self {
        // 参数：epoch (通常为0表示从1970-01-01开始), worker_id
        // worker_id 是 u16 类型，但 Snowflake 中实际只使用 10 位 (0-1023)
        let worker_id = (machine_id % 1024) as u16;
        let generator = SnowflakeGenerator::new(0, worker_id)
            .expect("Failed to create SnowflakeGenerator");
        
        Self { generator }
    }
    
    /// 从字节数组创建 ID 生成器
    /// 
    /// # 参数
    /// * `machine_id_bytes` - 机器 ID 字节数组（如 MAC 地址）
    pub fn from_bytes(machine_id_bytes: [u8; 6]) -> Self {
        // 直接使用字节数组的前2个字节作为 worker_id
        // Snowflake 中 worker_id 使用 10 位 (0-1023)
        let raw_worker_id = u16::from_be_bytes([machine_id_bytes[0], machine_id_bytes[1]]);
        let worker_id = (raw_worker_id % 1024) as u16;
        
        let generator = SnowflakeGenerator::new(0, worker_id)
            .expect("Failed to create SnowflakeGenerator");
        
        Self { generator }
    }
    
    /// 生成下一个唯一 ID
    /// 
    /// # 返回
    /// 返回生成的 ID 的字符串表示
    pub fn generate_id(&mut self) -> String {
        self.generator.generate_id().to_string()
    }
}

/// 全局 ID 生成器实例
static ID_GENERATOR: Lazy<LazyIdGenerator> = Lazy::new(|| {
    LazyIdGenerator::new()
});

/// 线程安全的全局 ID 生成器包装器
struct LazyIdGenerator {
    generator: std::sync::Mutex<IdGenerator>,
}

impl LazyIdGenerator {
    fn new() -> Self {
        let machine_id_bytes = crate::db::repositories::get_machine_id();
        Self {
            generator: std::sync::Mutex::new(IdGenerator::from_bytes(machine_id_bytes)),
        }
    }
    
    fn generate_id(&self) -> String {
        self.generator.lock().unwrap().generate_id()
    }
}

/// 生成全局唯一 ID 的便捷函数
/// 
/// # 返回
/// 返回生成的唯一 ID 字符串
pub fn generate_unique_id() -> String {
    ID_GENERATOR.generate_id()
}

/// 使用指定机器 ID 生成唯一 ID
/// 
/// # 参数
/// * `machine_id` - 机器 ID
/// 
/// # 返回
/// 返回生成的唯一 ID 字符串
#[cfg(test)]
pub fn generate_unique_id_with_machine(machine_id: u64) -> String {
    let mut generator = IdGenerator::new(machine_id);
    generator.generate_id()
}

/// 从字节数组生成唯一 ID
/// 
/// # 参数
/// * `machine_id_bytes` - 机器 ID 字节数组
/// 
/// # 返回
/// 返回生成的唯一 ID 字符串
#[cfg(test)]
pub fn generate_unique_id_from_bytes(machine_id_bytes: [u8; 6]) -> String {
    let mut generator = IdGenerator::from_bytes(machine_id_bytes);
    generator.generate_id()
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