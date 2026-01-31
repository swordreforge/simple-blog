// Cache module - simplified version for future use

/// 缓存配置
#[derive(Debug, Clone)]
pub struct CacheConfig;

impl Default for CacheConfig {
    fn default() -> Self {
        Self
    }
}

/// 应用缓存
pub struct AppCache;

impl AppCache {
    pub fn new(_config: CacheConfig) -> Self {
        Self
    }
}