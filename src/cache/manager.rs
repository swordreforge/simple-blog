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

    /// 设置缓存值（自定义 TTL）
    pub async fn set_with_ttl(&self, key: &str, value: &str, ttl: Duration) -> Result<(), CacheError> {
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

    /// 检查键是否存在
    pub async fn exists(&self, key: &str) -> bool {
        // 检查主缓存
        if self.primary.exists(key).await {
            return true;
        }

        // 检查备用缓存
        if self.fallback_enabled.load(Ordering::Relaxed) {
            if let Some(fallback) = &self.fallback {
                return fallback.exists(key).await;
            }
        }

        false
    }

    /// 清空所有缓存
    pub async fn clear(&self) -> Result<(), CacheError> {
        // 清空主缓存
        let primary_result = self.primary.clear().await;

        // 清空备用缓存
        if self.fallback_enabled.load(Ordering::Relaxed) {
            if let Some(fallback) = &self.fallback {
                let _ = fallback.clear().await;
            }
        }

        primary_result
    }

    /// 获取当前使用的后端名称
    pub fn backend_name(&self) -> &'static str {
        self.primary.backend_name()
    }

    /// 检查健康状态
    pub async fn health_check(&self) -> bool {
        let primary_healthy = self.primary.health_check().await;
        
        if !primary_healthy {
            eprintln!("⚠️  主缓存后端不健康");
            
            if self.fallback_enabled.load(Ordering::Relaxed) {
                if let Some(fallback) = &self.fallback {
                    let fallback_healthy = fallback.health_check().await;
                    if fallback_healthy {
                        println!("✅ 备用缓存后端健康");
                        return true;
                    }
                }
            }
            
            return false;
        }
        
        true
    }

    /// 获取缓存统计信息
    pub fn get_stats(&self) -> CacheStats {
        CacheStats {
            backend: self.primary.backend_name().to_string(),
            has_fallback: self.fallback.is_some(),
            fallback_enabled: self.fallback_enabled.load(Ordering::Relaxed),
            default_ttl: self.config.default_ttl,
        }
    }
}

/// 缓存统计信息
#[derive(Debug, Clone)]
pub(crate) struct CacheStats {
    pub(crate) backend: String,
    pub(crate) has_fallback: bool,
    pub(crate) fallback_enabled: bool,
    pub(crate) default_ttl: u64,
}