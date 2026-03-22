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
        matches!(
            self,
            CacheError::ConnectionError(_) | CacheError::TimeoutError(_)
        )
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();
        assert_eq!(config.default_ttl, 3600);
        assert!(config.enable_fallback);
    }

    #[test]
    fn test_cache_config_new() {
        let config = CacheConfig::new(7200, false);
        assert_eq!(config.default_ttl, 7200);
        assert!(!config.enable_fallback);
    }

    #[test]
    fn test_cache_config_clone() {
        let config = CacheConfig::new(1800, true);
        let cloned = config.clone();
        assert_eq!(config.default_ttl, cloned.default_ttl);
        assert_eq!(config.enable_fallback, cloned.enable_fallback);
    }

    #[test]
    fn test_cache_error_display() {
        let err = CacheError::ConnectionError("Connection refused".to_string());
        assert_eq!(err.to_string(), "连接错误: Connection refused");

        let err = CacheError::TimeoutError("Request timed out".to_string());
        assert_eq!(err.to_string(), "超时错误: Request timed out");

        let err = CacheError::Unknown("Unknown error".to_string());
        assert_eq!(err.to_string(), "未知错误: Unknown error");
    }

    #[test]
    fn test_cache_error_debug() {
        let err = CacheError::ConnectionError("Test error".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("ConnectionError"));
        assert!(debug_str.contains("Test error"));
    }

    #[test]
    fn test_cache_error_is_degradation_trigger() {
        let conn_err = CacheError::ConnectionError("Connection failed".to_string());
        assert!(conn_err.is_degradation_trigger());

        let timeout_err = CacheError::TimeoutError("Timeout".to_string());
        assert!(timeout_err.is_degradation_trigger());

        let unknown_err = CacheError::Unknown("Some error".to_string());
        assert!(!unknown_err.is_degradation_trigger());
    }

    #[test]
    fn test_cache_error_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        
        assert_send::<CacheError>();
        assert_sync::<CacheError>();
    }

    #[test]
    fn test_cache_config_with_different_ttl() {
        let config1 = CacheConfig::new(60, true);
        let config2 = CacheConfig::new(300, false);
        let config3 = CacheConfig::new(86400, true);

        assert_eq!(config1.default_ttl, 60);
        assert_eq!(config2.default_ttl, 300);
        assert_eq!(config3.default_ttl, 86400);
    }

    #[test]
    fn test_cache_config_debug() {
        let config = CacheConfig::new(1800, true);
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("default_ttl"));
        assert!(debug_str.contains("enable_fallback"));
    }
}
