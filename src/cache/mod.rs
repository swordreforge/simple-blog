mod backend;
mod concurrent;
mod keys;
mod local;
mod manager;
mod retry;
mod utils;
mod valkey;

pub use backend::{CacheConfig, CacheError};
pub use keys::PassageCacheKeys;
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
    pub async fn init_manager(
        &mut self,
        backend_type: &str,
        valkey_url: Option<&str>,
        config: CacheConfig,
    ) -> Result<(), CacheError> {
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

    /// 获取或加载缓存值（防止缓存击穿）
    ///
    /// # 参数
    /// - `key`: 缓存键
    /// - `loader`: 加载函数，当缓存未命中时调用
    /// - `ttl`: 缓存过期时间（秒）
    ///
    /// # 返回
    /// 返回缓存值或加载的值
    #[allow(dead_code)]
    pub async fn get_or_load<F, Fut>(
        &self,
        key: &str,
        loader: F,
        _ttl: u64,
    ) -> Result<Option<String>, CacheError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<Option<String>, CacheError>>,
    {
        if let Some(manager) = &self.manager {
            // 先尝试从缓存获取
            if let Some(value) = manager.get(key).await {
                return Ok(Some(value));
            }

            // 缓存未命中，执行加载函数
            let value = loader().await?;

            // 如果有值，设置到缓存
            if let Some(ref v) = value {
                let _ = manager.set(key, v).await;
            }

            Ok(value)
        } else {
            // 没有缓存管理器，直接调用加载函数
            loader().await
        }
    }

    /// 批量获取或加载缓存值（用于文章列表等场景）
    ///
    /// # 参数
    /// - `keys`: 缓存键列表
    /// - `loader`: 加载函数，接收未命中的键列表
    /// - `ttl`: 缓存过期时间（秒）
    ///
    /// # 返回
    /// 返回键值对映射
    #[allow(dead_code)]
    pub async fn get_or_load_many<F, Fut>(
        &self,
        keys: &[String],
        loader: F,
        _ttl: u64,
    ) -> Result<std::collections::HashMap<String, String>, CacheError>
    where
        F: FnOnce(Vec<String>) -> Fut,
        Fut: std::future::Future<
                Output = Result<std::collections::HashMap<String, String>, CacheError>,
            >,
    {
        if let Some(manager) = &self.manager {
            let mut results = std::collections::HashMap::new();
            let mut missed_keys = Vec::new();

            // 先尝试从缓存获取所有键
            for key in keys {
                if let Some(value) = manager.get(key).await {
                    results.insert(key.clone(), value);
                } else {
                    missed_keys.push(key.clone());
                }
            }

            // 如果有未命中的键，批量加载
            if !missed_keys.is_empty() {
                let loaded = loader(missed_keys.clone()).await?;

                // 将加载的值设置到缓存
                for (key, value) in &loaded {
                    let _ = manager.set(key, value).await;
                    results.insert(key.clone(), value.clone());
                }
            }

            Ok(results)
        } else {
            // 没有缓存管理器，直接调用加载函数
            loader(keys.to_vec()).await
        }
    }
}
