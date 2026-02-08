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

    /// 检查键是否存在
    async fn exists(&self, key: &str) -> bool;

    /// 清空所有缓存
    async fn clear(&self) -> Result<(), CacheError>;

    /// 获取后端名称（用于日志和调试）
    fn backend_name(&self) -> &'static str;

    /// 检查后端是否健康
    async fn health_check(&self) -> bool;
}

/// 缓存错误类型
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("连接错误: {0}")]
    ConnectionError(String),

    #[error("序列化错误: {0}")]
    SerializationError(String),

    #[error("反序列化错误: {0}")]
    DeserializationError(String),

    #[error("超时错误")]
    TimeoutError,

    #[error("未知错误: {0}")]
    Unknown(String),
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