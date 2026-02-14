use super::backend::{CacheBackend, CacheError};

use async_trait::async_trait;

#[cfg(feature = "valkey")]
use std::time::Duration;

#[cfg(feature = "valkey")]
use redis::{
    aio::ConnectionManager,
    AsyncCommands, Client,
    RedisError,
};

/// Valkey 缓存后端
#[cfg(feature = "valkey")]
pub struct ValkeyCacheBackend {
    manager: ConnectionManager,
    key_prefix: String,
    operation_timeout: Duration,
    max_retries: usize,
    base_retry_delay: Duration,
}

#[cfg(feature = "valkey")]
impl ValkeyCacheBackend {
    /// 创建新的 Valkey 缓存后端
    pub async fn new(url: &str, key_prefix: Option<String>) -> Result<Self, CacheError> {
        Self::new_with_timeout(url, key_prefix, Duration::from_secs(8)).await
    }

    /// 创建新的 Valkey 缓存后端（带超时配置）
    pub async fn new_with_timeout(
        url: &str,
        key_prefix: Option<String>,
        operation_timeout: Duration,
    ) -> Result<Self, CacheError> {
        Self::new_with_retry_config(
            url,
            key_prefix,
            operation_timeout,
            5,                           // 最大重试次数（增加以提高稳定性）
            Duration::from_millis(200),  // 基础重试延迟（增加以避免过快重试）
        ).await
    }

    /// 创建新的 Valkey 缓存后端（带完整配置）
    pub async fn new_with_retry_config(
        url: &str,
        key_prefix: Option<String>,
        operation_timeout: Duration,
        max_retries: usize,
        base_retry_delay: Duration,
    ) -> Result<Self, CacheError> {
        println!("🔗 正在连接到 Valkey: {}", url);
        println!("⏱️  连接超时配置: 15秒, 操作超时: {:?}", operation_timeout);
        println!("🔄 重试配置: 最大{}次, 基础延迟{:?}", max_retries, base_retry_delay);

        // 使用 tokio::time::timeout 防止连接初始化时阻塞
        let client = Client::open(url).map_err(|e| {
            tracing::error!("创建 Redis 客户端失败: {}", e);
            CacheError::ConnectionError(format!("Failed to create Redis client: {}", e))
        })?;

        // 使用超时创建连接管理器（增加超时时间）
        let manager = tokio::time::timeout(
            Duration::from_secs(15),
            ConnectionManager::new(client)
        )
        .await
        .map_err(|_| {
            tracing::error!("❌ Valkey 连接超时（15秒）");
            CacheError::ConnectionError(
                "Connection to Valkey timed out after 15 seconds. Please check:".to_string()
            )
        })?
        .map_err(|e| {
            tracing::error!("创建连接管理器失败: {}", e);
            CacheError::ConnectionError(format!("Failed to create connection manager: {}", e))
        })?;

        println!("✅ Valkey 连接管理器创建成功");

        Ok(Self {
            manager,
            key_prefix: key_prefix.unwrap_or_else(|| "rustblog:".to_string()),
            operation_timeout,
            max_retries,
            base_retry_delay,
        })
    }

    /// 添加前缀到键
    fn prefixed_key(&self, key: &str) -> String {
        format!("{}{}", self.key_prefix, key)
    }

    /// 带超时的异步操作执行
        async fn execute_with_timeout<F, T>(&self, f: F) -> Result<T, CacheError>
        where
            F: std::future::Future<Output = Result<T, RedisError>>,
        {
            tokio::time::timeout(self.operation_timeout, f)
                .await
                .map_err(|_| CacheError::TimeoutError(
                    format!("Valkey operation timed out after {:?}", self.operation_timeout)
                ))?
                .map_err(|e| CacheError::ConnectionError(format!("Valkey operation failed: {}", e)))
        }
    
        /// 带重试的操作执行（指数退避）
        async fn execute_with_retry<F, Fut, T>(&self, f: F) -> Result<T, CacheError>
        where
            F: Fn() -> Fut,
            Fut: std::future::Future<Output = Result<T, RedisError>>,
        {
            let mut last_error = None;

            for attempt in 0..=self.max_retries {
                // 执行操作
                match self.execute_with_timeout(f()).await {
                    Ok(value) => {
                        // 成功时记录日志（仅在重试后成功时）
                        if attempt > 0 {
                            println!("✅ Valkey 操作在第 {} 次重试后成功", attempt);
                        }
                        return Ok(value);
                    }
                    Err(e) => {
                        // 记录错误类型
                        let error_type = match &e {
                            CacheError::TimeoutError(_) => "超时",
                            CacheError::ConnectionError(_) => "连接错误",
                            _ => "未知错误",
                        };

                        tracing::warn!("Valkey 操作失败 (尝试 {}/{}, 错误类型: {}): {}",
                                  attempt + 1, self.max_retries + 1, error_type, e);
                        last_error = Some(e);
                    }
                }

                // 如果还有重试机会，等待一段时间后重试
                if attempt < self.max_retries {
                    let delay = self.base_retry_delay * 2_u32.pow(attempt as u32);
                    tracing::info!("{:?} 后进行第 {} 次重试...",
                              delay, attempt + 2);
                    tokio::time::sleep(delay).await;
                }
            }

            let final_error = last_error.unwrap_or_else(||
                CacheError::ConnectionError("Valkey operation failed after all retries".to_string())
            );

            eprintln!("❌ Valkey 操作在 {} 次重试后仍然失败: {}", self.max_retries + 1, final_error);
            Err(final_error)
        }    /// 检查连接是否健康
    pub async fn health_check(&self) -> Result<(), CacheError> {
        let conn = self.manager.clone();

        // 使用较短的超时时间进行健康检查
        let health_check_timeout = std::cmp::min(self.operation_timeout, Duration::from_secs(3));

        tokio::time::timeout(health_check_timeout, async move {
            let mut conn = conn.clone();
            let _: String = redis::cmd("PING")
                .query_async(&mut conn)
                .await?;
            Ok::<(), redis::RedisError>(())
        })
        .await
        .map_err(|_| {
            tracing::error!("Valkey 健康检查超时（{:?}）", health_check_timeout);
            CacheError::TimeoutError(format!("Health check timed out after {:?}", health_check_timeout))
        })?
        .map_err(|e: redis::RedisError| {
            tracing::error!("Valkey 健康检查失败: {}", e);
            CacheError::ConnectionError(format!("Health check failed: {}", e))
        })
    }

    /// 批量删除键（带超时和错误处理）
    async fn batch_delete_keys(
        &self,
        conn: &ConnectionManager,
        keys: Vec<String>,
    ) -> Result<(), CacheError> {
        if keys.is_empty() {
            return Ok(());
        }

        tokio::time::timeout(
            self.operation_timeout,
            {
                let mut conn = conn.clone();
                async move {
                    conn.del::<_, ()>(keys).await
                }
            }
        )
        .await
        .map_err(|_| CacheError::ConnectionError(
            format!("DEL operation timed out after {:?}", self.operation_timeout)
        ))?
        .map_err(|e| CacheError::ConnectionError(format!("DEL failed: {}", e)))
    }
}

#[cfg(feature = "valkey")]
#[async_trait]
impl CacheBackend for ValkeyCacheBackend {
    async fn get(&self, key: &str) -> Option<String> {
        let prefixed_key = self.prefixed_key(key);
        let conn = self.manager.clone();

        self.execute_with_retry(|| {
            let mut conn = conn.clone();
            let prefixed_key = prefixed_key.clone();
            async move {
                conn.get(prefixed_key).await
            }
        }).await.ok().flatten()
    }

    async fn set(&self, key: &str, value: &str, ttl: Duration) -> Result<(), CacheError> {
        let prefixed_key = self.prefixed_key(key);
        let conn = self.manager.clone();
        let value = value.to_string();

        self.execute_with_retry(|| {
            let mut conn = conn.clone();
            let prefixed_key = prefixed_key.clone();
            let value = value.clone();
            async move {
                conn.set_ex(prefixed_key, value, ttl.as_secs()).await
            }
        }).await
    }

    async fn delete(&self, key: &str) -> Result<(), CacheError> {
        let prefixed_key = self.prefixed_key(key);
        let conn = self.manager.clone();

        self.execute_with_retry(|| {
            let mut conn = conn.clone();
            let prefixed_key = prefixed_key.clone();
            async move {
                conn.del(prefixed_key).await
            }
        }).await
    }

    async fn delete_many(&self, keys: &[String]) -> Result<(), CacheError> {
        if keys.is_empty() {
            return Ok(());
        }

        let prefixed_keys: Vec<String> = keys.iter()
            .map(|k| self.prefixed_key(k))
            .collect();
        let conn = self.manager.clone();

        self.execute_with_retry(|| {
            let mut conn = conn.clone();
            let prefixed_keys = prefixed_keys.clone();
            async move {
                conn.del(prefixed_keys).await
            }
        }).await
    }

    async fn delete_pattern(&self, pattern: &str) -> Result<(), CacheError> {
        let prefixed_pattern = self.prefixed_key(pattern);
        let conn = self.manager.clone();

        tracing::debug!("delete_pattern: 开始扫描匹配 '{}' 的键", pattern);
        let start_time = std::time::Instant::now();

        // 使用 SCAN 命令找到所有匹配的键，然后删除
        let mut keys_to_delete = Vec::new();
        let mut cursor: u64 = 0;
        let mut iteration_count = 0;
        let mut total_keys_scanned = 0;
        const MAX_ITERATIONS: usize = 1000;  // 防止无限循环
        const BATCH_SIZE: usize = 100;  // 每批处理的键数量
        const MAX_KEYS: usize = 10000;  // 最大处理的键数量

        loop {
            iteration_count += 1;
            if iteration_count > MAX_ITERATIONS {
                tracing::warn!("delete_pattern: 超过最大迭代次数 {}, 停止扫描", MAX_ITERATIONS);
                return Err(CacheError::ConnectionError(
                    format!("SCAN iteration exceeded maximum limit of {}", MAX_ITERATIONS)
                ));
            }

            // 为每次 SCAN 操作添加超时
            let (next_cursor, keys): (u64, Vec<String>) = tokio::time::timeout(
                self.operation_timeout,
                {
                    let mut conn = conn.clone();
                    let prefixed_pattern = prefixed_pattern.clone();
                    async move {
                        redis::cmd("SCAN")
                            .arg(cursor)
                            .arg("MATCH")
                            .arg(&prefixed_pattern)
                            .arg("COUNT")
                            .arg(BATCH_SIZE)
                            .query_async(&mut conn)
                            .await
                    }
                }
            )
            .await
            .map_err(|_| CacheError::ConnectionError(
                format!("SCAN operation timed out after {:?}", self.operation_timeout)
            ))?
            .map_err(|e| CacheError::ConnectionError(format!("SCAN failed: {}", e)))?;

            total_keys_scanned += keys.len();

            if !keys.is_empty() {
                tracing::debug!("delete_pattern: 迭代 {} 找到 {} 个键 (累计: {})",
                         iteration_count, keys.len(), total_keys_scanned);
            }

            keys_to_delete.extend(keys);

            // 如果积累了足够的键，先批量删除一次
            if keys_to_delete.len() >= BATCH_SIZE {
                eprintln!("🗑️  delete_pattern: 批量删除 {} 个键", keys_to_delete.len());
                self.batch_delete_keys(&conn, keys_to_delete.drain(..).collect()).await?;
            }

            // 防止处理过多键
            if keys_to_delete.len() >= MAX_KEYS {
                eprintln!("⚠️  delete_pattern: 达到最大键数量限制 {}, 停止扫描", MAX_KEYS);
                break;
            }

            cursor = next_cursor;
            if cursor == 0 {
                break;
            }
        }

        // 删除剩余的键
        if !keys_to_delete.is_empty() {
            tracing::debug!("delete_pattern: 删除剩余的 {} 个键", keys_to_delete.len());
            self.batch_delete_keys(&conn, keys_to_delete).await?;
        }

        let elapsed = start_time.elapsed();
        eprintln!("✅ delete_pattern: 完成，扫描了 {} 个键，耗时 {:?}",
                 total_keys_scanned, elapsed);

        Ok(())
    }
}

/// 禁用 Valkey 特性时的存根实现
#[cfg(not(feature = "valkey"))]
pub struct ValkeyCacheBackend;

#[cfg(not(feature = "valkey"))]
impl ValkeyCacheBackend {
    pub async fn new(_url: &str, _key_prefix: Option<String>) -> Result<Self, CacheError> {
        Err(CacheError::ConnectionError(
            "Valkey feature is not enabled. Please rebuild with --features valkey".to_string(),
        ))
    }

    /// 健康检查（存根实现）
    pub async fn health_check(&self) -> Result<(), CacheError> {
        Err(CacheError::ConnectionError(
            "Valkey feature is not enabled".to_string(),
        ))
    }
}

#[cfg(not(feature = "valkey"))]
#[async_trait]
impl CacheBackend for ValkeyCacheBackend {
    async fn get(&self, _key: &str) -> Option<String> {
        None
    }

    async fn set(&self, _key: &str, _value: &str, _ttl: std::time::Duration) -> Result<(), CacheError> {
        Err(CacheError::ConnectionError(
            "Valkey feature is not enabled".to_string(),
        ))
    }

    async fn delete(&self, _key: &str) -> Result<(), CacheError> {
        Err(CacheError::ConnectionError(
            "Valkey feature is not enabled".to_string(),
        ))
    }

    async fn delete_many(&self, _keys: &[String]) -> Result<(), CacheError> {
        Err(CacheError::ConnectionError(
            "Valkey feature is not enabled".to_string(),
        ))
    }

    async fn delete_pattern(&self, _pattern: &str) -> Result<(), CacheError> {
        Err(CacheError::ConnectionError(
            "Valkey feature is not enabled".to_string(),
        ))
    }
}