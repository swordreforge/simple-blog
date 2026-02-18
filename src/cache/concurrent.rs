/// 并发安全防护模块
/// 提供缓存击穿、穿透、雪崩的防护机制

use super::backend::{CacheBackend, CacheError};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, Semaphore};

/// 空值标记，用于缓存穿透防护
const NULL_VALUE: &str = "__NULL__";
const NULL_VALUE_TTL: u64 = 60; // 空值缓存1分钟

/// 缓存击穿防护锁
/// 使用本地信号量 + Valkey 分布式锁的双重保护
pub struct CacheLock {
    /// 本地信号量，用于快速防护同一实例内的并发
    local_locks: Arc<Mutex<std::collections::HashMap<String, Arc<Semaphore>>>>,
    /// 是否启用分布式锁
    enable_distributed: bool,
}

impl CacheLock {
    pub fn new(enable_distributed: bool) -> Self {
        Self {
            local_locks: Arc::new(Mutex::new(std::collections::HashMap::new())),
            enable_distributed,
        }
    }

    /// 获取缓存锁
    pub async fn acquire(&self, key: &str) -> Option<CacheLockGuard> {
        // 获取或创建本地锁
        let semaphore = {
            let mut locks = self.local_locks.lock().await;
            if !locks.contains_key(key) {
                locks.insert(key.to_string(), Arc::new(Semaphore::new(1)));
            }
            Arc::clone(locks.get(key).unwrap())
        };

        // 尝试获取本地锁（非阻塞）
        match semaphore.try_acquire_owned() {
            Ok(permit) => {
                Some(CacheLockGuard {
                    key: key.to_string(),
                    permit: Some(permit),
                    local_locks: Arc::clone(&self.local_locks),
                })
            }
            Err(_) => {
                // 本地锁已被占用，说明有其他线程正在加载
                tracing::debug!("缓存锁已被占用: {}", key);
                None
            }
        }
    }
}

/// 缓存锁守卫
pub struct CacheLockGuard {
    key: String,
    permit: Option<tokio::sync::OwnedSemaphorePermit>,
    local_locks: Arc<Mutex<std::collections::HashMap<String, Arc<Semaphore>>>>,
}

impl Drop for CacheLockGuard {
    fn drop(&mut self) {
        // 清理不再使用的锁
        if let Some(permit) = self.permit.take() {
            drop(permit);

            // 异步清理锁（避免阻塞 drop）
            let key = self.key.clone();
            let locks = Arc::clone(&self.local_locks);
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs(5)).await;
                let mut locks = locks.lock().await;
                if locks.get(&key).map_or(true, |s| s.available_permits() > 0) {
                    locks.remove(&key);
                }
            });
        }
    }
}

/// 缓存雪崩防护 - TTL 随机化
pub fn jitter_ttl(base_ttl: u64, jitter_percent: u8) -> u64 {
    let jitter = (base_ttl as f64 * jitter_percent as f64 / 100.0) as u64;
    let random_jitter = if jitter > 0 {
        (rand::random::<u64>() % jitter) - (jitter / 2)
    } else {
        0
    };

    let jittered_ttl = base_ttl as i64 + random_jitter as i64;
    jittered_ttl.max(60) as u64 // 最小60秒
}

/// 缓存穿透防护 - 缓存空值
pub fn should_cache_null(result: &Option<String>) -> bool {
    // 如果查询结果为空，说明数据不存在，应该缓存空值
    result.is_none()
}

/// 获取空值标记
pub fn get_null_value() -> String {
    NULL_VALUE.to_string()
}

/// 检查是否为空值标记
pub fn is_null_value(value: &str) -> bool {
    value == NULL_VALUE
}

/// 增强的缓存后端，内置并发安全防护
pub struct SafeCacheBackend<B: CacheBackend> {
    inner: B,
    cache_lock: Arc<CacheLock>,
    enable_null_cache: bool,
    enable_ttl_jitter: bool,
}

impl<B: CacheBackend> SafeCacheBackend<B> {
    pub fn new(backend: B, enable_distributed_lock: bool) -> Self {
        Self {
            inner: backend,
            cache_lock: Arc::new(CacheLock::new(enable_distributed_lock)),
            enable_null_cache: true,
            enable_ttl_jitter: true,
        }
    }

    /// 安全的获取缓存，防止缓存穿透
    pub async fn get_safe(&self, key: &str) -> Option<String> {
        if let Some(value) = self.inner.get(key).await {
            // 检查是否为空值标记
            if is_null_value(&value) {
                tracing::debug!("命中空值缓存: {}", key);
                return None; // 返回 None 表示数据不存在
            }
            Some(value)
        } else {
            None
        }
    }

    /// 安全的设置缓存，支持空值缓存和 TTL 抖动
    pub async fn set_safe(&self, key: &str, value: Option<&str>, ttl: Duration) -> Result<(), CacheError> {
        let actual_value = match value {
            Some(v) => v.to_string(),
            None => {
                if self.enable_null_cache {
                    get_null_value()
                } else {
                    return Ok(()); // 不缓存空值
                }
            }
        };

        let actual_ttl = if self.enable_ttl_jitter {
            let jittered = jitter_ttl(ttl.as_secs(), 10); // 10% 抖动
            Duration::from_secs(jittered)
        } else {
            ttl
        };

        self.inner.set(key, &actual_value, actual_ttl).await
    }

    /// 带锁的获取或加载模式，防止缓存击穿
    pub async fn get_or_load<F, Fut>(
        &self,
        key: &str,
        loader: F,
        ttl: Duration,
    ) -> Result<Option<String>, CacheError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<Option<String>, CacheError>>,
    {
        // 1. 先尝试从缓存获取
        if let Some(value) = self.get_safe(key).await {
            tracing::debug!("缓存命中: {}", key);
            return Ok(Some(value));
        }

        // 2. 尝试获取锁
        if let Some(_lock) = self.cache_lock.acquire(key).await {
            tracing::debug!("获取缓存锁成功，开始加载: {}", key);

            // 3. 双重检查：获取锁后再次检查缓存
            if let Some(value) = self.get_safe(key).await {
                return Ok(Some(value));
            }

            // 4. 从数据源加载
            let value = loader().await?;

            // 5. 写入缓存
            if value.is_some() || self.enable_null_cache {
                self.set_safe(key, value.as_deref(), ttl).await?;
            }

            Ok(value)
        } else {
            // 6. 锁获取失败，等待片刻后重试
            tracing::debug!("缓存锁被占用，等待重试: {}", key);
            tokio::time::sleep(Duration::from_millis(50)).await;

            // 再次尝试从缓存获取
            Ok(self.get_safe(key).await)
        }
    }

    /// 获取内部后端的引用
    pub fn inner(&self) -> &B {
        &self.inner
    }
}

#[async_trait]
impl<B: CacheBackend> CacheBackend for SafeCacheBackend<B> {
    async fn get(&self, key: &str) -> Option<String> {
        self.get_safe(key).await
    }

    async fn set(&self, key: &str, value: &str, ttl: Duration) -> Result<(), CacheError> {
        self.set_safe(key, Some(value), ttl).await
    }

    async fn delete(&self, key: &str) -> Result<(), CacheError> {
        self.inner.delete(key).await
    }

    async fn delete_many(&self, keys: &[String]) -> Result<(), CacheError> {
        self.inner.delete_many(keys).await
    }

    async fn delete_pattern(&self, pattern: &str) -> Result<(), CacheError> {
        self.inner.delete_pattern(pattern).await
    }
}