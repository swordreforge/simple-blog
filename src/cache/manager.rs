use super::backend::{CacheBackend, CacheConfig, CacheError};
use super::local::LocalCacheBackend;
use super::valkey::ValkeyCacheBackend;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// 缓存管理器 - 支持自动降级
pub struct CacheManager {
    primary: Arc<dyn CacheBackend>,
    fallback: Option<Arc<dyn CacheBackend>>,
    fallback_enabled: Arc<AtomicBool>,
    config: CacheConfig,
}

impl Clone for CacheManager {
    fn clone(&self) -> Self {
        Self {
            primary: Arc::clone(&self.primary),
            fallback: self.fallback.clone(),
            fallback_enabled: Arc::clone(&self.fallback_enabled),
            config: self.config.clone(),
        }
    }
}

impl CacheManager {
    /// 创建缓存管理器
    pub async fn new(
        backend_type: &str,
        valkey_url: Option<&str>,
        config: CacheConfig,
    ) -> Result<Self, CacheError> {
        let (primary, fallback) = match backend_type {
            "valkey" | "redis" => {
                let url = valkey_url.ok_or_else(|| {
                    CacheError::ConnectionError("Valkey URL is required for valkey backend".to_string())
                })?;

                match ValkeyCacheBackend::new(url, Some("rustblog:".to_string())).await {
                    Ok(valkey) => {
                        println!("✅ Valkey 缓存后端初始化成功");
                        let local = Arc::new(LocalCacheBackend::new(Some(10000))) as Arc<dyn CacheBackend>;
                        (Arc::new(valkey) as Arc<dyn CacheBackend>, Some(local))
                    }
                    Err(e) => {
                        eprintln!("⚠️  Valkey 连接失败: {}, 使用本地缓存降级", e);
                        let local = Arc::new(LocalCacheBackend::new(Some(10000))) as Arc<dyn CacheBackend>;
                        (local.clone(), None)
                    }
                }
            }
            "local" => {
                println!("✅ 使用本地内存缓存");
                let local = Arc::new(LocalCacheBackend::new(Some(10000))) as Arc<dyn CacheBackend>;
                (local, None)
            }
            "auto" => {
                // 自动模式：尝试使用 Valkey，失败则降级到本地
                if let Some(url) = valkey_url {
                    match ValkeyCacheBackend::new(url, Some("rustblog:".to_string())).await {
                        Ok(valkey) => {
                            println!("✅ 自动检测到 Valkey，使用 Valkey 缓存");
                            let local = Arc::new(LocalCacheBackend::new(Some(10000))) as Arc<dyn CacheBackend>;
                            (Arc::new(valkey) as Arc<dyn CacheBackend>, Some(local))
                        }
                        Err(_) => {
                            println!("⚠️  Valkey 不可用，使用本地缓存");
                            let local = Arc::new(LocalCacheBackend::new(Some(10000))) as Arc<dyn CacheBackend>;
                            (local.clone(), None)
                        }
                    }
                } else {
                    println!("⚠️  未配置 Valkey URL，使用本地缓存");
                    let local = Arc::new(LocalCacheBackend::new(Some(10000))) as Arc<dyn CacheBackend>;
                    (local.clone(), None)
                }
            }
            _ => {
                return Err(CacheError::Unknown(format!("Unknown cache backend: {}", backend_type)));
            }
        };

        let fallback_enabled = Arc::new(AtomicBool::new(config.enable_fallback));

        Ok(Self {
            primary,
            fallback,
            fallback_enabled,
            config,
        })
    }

    /// 获取缓存值
    pub async fn get(&self, key: &str) -> Option<String> {
        // 首先尝试主缓存
        if let Some(value) = self.primary.get(key).await {
            return Some(value);
        }

        // 如果启用了降级，尝试从备用缓存获取
        if self.fallback_enabled.load(Ordering::Relaxed) {
            if let Some(fallback) = &self.fallback {
                if let Some(value) = fallback.get(key).await {
                    return Some(value);
                }
            }
        }

        None
    }

    /// 设置缓存值
    pub async fn set(&self, key: &str, value: &str) -> Result<(), CacheError> {
        let ttl = Duration::from_secs(self.config.default_ttl);
        
        // 尝试设置到主缓存
        match self.primary.set(key, value, ttl).await {
            Ok(()) => {
                // 如果启用了降级，同时设置到备用缓存
                if self.fallback_enabled.load(Ordering::Relaxed) {
                    if let Some(fallback) = &self.fallback {
                        let _ = fallback.set(key, value, ttl).await;
                    }
                }
                Ok(())
            }
            Err(e) => {
                // 主缓存失败，尝试降级到备用缓存
                if self.fallback_enabled.load(Ordering::Relaxed) {
                    if let Some(fallback) = &self.fallback {
                        eprintln!("⚠️  主缓存写入失败: {}, 降级到备用缓存", e);
                        return fallback.set(key, value, ttl).await;
                    }
                }
                Err(e)
            }
        }
    }

    /// 删除缓存值
    pub async fn delete(&self, key: &str) -> Result<(), CacheError> {
        // 从主缓存删除
        let primary_result = self.primary.delete(key).await;

        // 从备用缓存删除
        if self.fallback_enabled.load(Ordering::Relaxed) {
            if let Some(fallback) = &self.fallback {
                let _ = fallback.delete(key).await;
            }
        }

        primary_result
    }

    /// 批量删除缓存值
    #[allow(dead_code)]
    pub async fn delete_many(&self, keys: &[String]) -> Result<(), CacheError> {
        if keys.is_empty() {
            return Ok(());
        }

        // 从主缓存删除
        let primary_result = self.primary.delete_many(keys).await;

        // 从备用缓存删除
        if self.fallback_enabled.load(Ordering::Relaxed) {
            if let Some(fallback) = &self.fallback {
                let _ = fallback.delete_many(keys).await;
            }
        }

        primary_result
    }

    /// 根据模式删除缓存值
    pub async fn delete_pattern(&self, pattern: &str) -> Result<(), CacheError> {
        // 从主缓存删除
        let primary_result = self.primary.delete_pattern(pattern).await;

        // 从备用缓存删除
        if self.fallback_enabled.load(Ordering::Relaxed) {
            if let Some(fallback) = &self.fallback {
                let _ = fallback.delete_pattern(pattern).await;
            }
        }

        primary_result
    }

    /// 获取缓存统计信息
    pub fn get_stats(&self) -> CacheStats {
        CacheStats {
            has_fallback: self.fallback.is_some(),
            fallback_enabled: self.fallback_enabled.load(Ordering::Relaxed),
            default_ttl: self.config.default_ttl,
        }
    }
}

/// 缓存统计信息
#[derive(Debug, Clone)]
pub(crate) struct CacheStats {
    pub(crate) has_fallback: bool,
    pub(crate) fallback_enabled: bool,
    pub(crate) default_ttl: u64,
}