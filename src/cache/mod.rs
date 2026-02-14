mod backend;
mod manager;
mod local;
mod valkey;
mod utils;

pub use backend::{CacheConfig, CacheError};
pub use manager::CacheManager;
pub use utils::*;

/// 应用缓存（兼容旧接口）
pub struct AppCache {
    manager: Option<CacheManager>,
}

impl AppCache {
    pub fn new(_config: CacheConfig) -> Self {
        Self { manager: None }
    }

    /// 初始化缓存管理器
    pub async fn init_manager(&mut self, backend_type: &str, valkey_url: Option<&str>, config: CacheConfig) -> Result<(), CacheError> {
        let manager = CacheManager::new(backend_type, valkey_url, config).await?;
        self.manager = Some(manager);
        Ok(())
    }

    /// 清除所有缓存
    pub async fn clear_all(&self) {
        if let Some(manager) = &self.manager {
            let _ = manager.delete_pattern("passage:*").await;
            println!("🧹 已清除所有文章缓存");
        }
    }

    /// 获取缓存管理器
    pub fn manager(&self) -> Option<&CacheManager> {
        self.manager.as_ref()
    }
}