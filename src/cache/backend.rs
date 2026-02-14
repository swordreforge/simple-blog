use async_trait::async_trait;
use std::time::Duration;

/// 缓存后端 trait - 定义所有缓存后端必须实现的接口
#[async_trait]
pub trait CacheBackend: Send + Sync {
    /// 获取缓存值
    async fn get(&self, key: &str) -> Option<String>;

    /// 设置缓存值
    async fn set(&self, key: &str, value: &str, ttl: Duration) -> Result<(), CacheError>;

    /// 删除缓存值
    async fn delete(&self, key: &str) -> Result<(), CacheError>;

    /// 批量删除缓存值
    #[allow(dead_code)]
    async fn delete_many(&self, keys: &[String]) -> Result<(), CacheError>;

    /// 根据模式删除缓存值（支持通配符）
    async fn delete_pattern(&self, pattern: &str) -> Result<(), CacheError>;
}

/// 缓存错误类型
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("连接错误: {0}")]
    ConnectionError(String),

    #[error("超时错误: {0}")]
    #[allow(dead_code)]
    TimeoutError(String),

    #[error("未知错误: {0}")]
    Unknown(String),
}

impl CacheError {
    /// 判断是否为需要触发降级的严重错误（超时或连接错误）
    pub fn is_degradation_trigger(&self) -> bool {
        matches!(self, CacheError::ConnectionError(_) | CacheError::TimeoutError(_))
    }
}

/// 缓存配置
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// 默认 TTL（秒）
    pub default_ttl: u64,
    /// 是否启用降级
    pub enable_fallback: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            default_ttl: 3600,
            enable_fallback: true,
        }
    }
}

impl CacheConfig {
    pub fn new(default_ttl: u64, enable_fallback: bool) -> Self {
        Self {
            default_ttl,
            enable_fallback,
        }
    }
}