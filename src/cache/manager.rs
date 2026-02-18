use super::backend::{CacheBackend, CacheConfig, CacheError};
use super::local::LocalCacheBackend;
use super::valkey::ValkeyCacheBackend;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use dashmap::DashMap;

/// 降级配置
#[derive(Debug, Clone)]
pub struct DegradationConfig {
    /// 连续失败多少次才触发降级
    pub consecutive_failures_threshold: usize,
    /// 超时/连接错误的失败阈值
    pub critical_error_threshold: usize,
    /// 是否启用滑动窗口降级
    pub enable_sliding_window: bool,
    /// 滑动窗口时间（秒）
    pub sliding_window_seconds: u64,
    /// 滑动窗口内失败率达到多少百分比才降级（0-100）
    pub sliding_window_failure_rate: f32,
    /// Valkey 初始化失败后，尝试重连的间隔（秒）
    pub reconnect_interval: u64,
}

impl Default for DegradationConfig {
    fn default() -> Self {
        Self {
            consecutive_failures_threshold: 5,  // 连续失败 5 次才降级（增加阈值，避免过早降级）
            critical_error_threshold: 3,       // 连续 3 次严重错误就降级（增加阈值）
            enable_sliding_window: true,       // 默认启用滑动窗口
            sliding_window_seconds: 120,       // 120 秒窗口（增加窗口时间）
            sliding_window_failure_rate: 60.0, // 60% 失败率（提高阈值）
            reconnect_interval: 60,            // 每 60 秒尝试重连一次
        }
    }
}

/// 缓存管理器 - 支持自动降级和重连
pub struct CacheManager {
    primary: Arc<dyn CacheBackend>,
    fallback: Option<Arc<dyn CacheBackend>>,
    fallback_enabled: Arc<AtomicBool>,
    config: CacheConfig,
    degradation_config: DegradationConfig,
    primary_healthy: Arc<AtomicBool>,
    consecutive_failures: Arc<AtomicUsize>,
    // 滑动窗口：使用 DashMap 记录操作时间戳和是否失败（无锁）
    operation_history: DashMap<u64, (Instant, bool)>,
    valkey_backend: Option<Arc<ValkeyCacheBackend>>,
    valkey_url: Option<String>,  // 保存 Valkey URL 用于重连
    health_check_task: Option<Arc<tokio::task::JoinHandle<()>>>,
    reconnect_task: Option<Arc<tokio::task::JoinHandle<()>>>,  // 重连任务
    // 上次检查滑动窗口的时间（避免频繁检查）
    last_sliding_window_check: Arc<AtomicUsize>,
}

impl Clone for CacheManager {
    fn clone(&self) -> Self {
        Self {
            primary: Arc::clone(&self.primary),
            fallback: self.fallback.clone(),
            fallback_enabled: Arc::clone(&self.fallback_enabled),
            config: self.config.clone(),
            degradation_config: self.degradation_config.clone(),
            primary_healthy: Arc::clone(&self.primary_healthy),
            consecutive_failures: Arc::clone(&self.consecutive_failures),
            operation_history: DashMap::clone(&self.operation_history),
            valkey_backend: self.valkey_backend.clone(),
            valkey_url: self.valkey_url.clone(),
            health_check_task: self.health_check_task.clone(),
            reconnect_task: self.reconnect_task.clone(),
            last_sliding_window_check: Arc::clone(&self.last_sliding_window_check),
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
        let valkey_url_owned = valkey_url.map(|s| s.to_string());
        let (primary, fallback, valkey_backend) = match backend_type {
            "valkey" | "redis" => {
                let url = valkey_url.ok_or_else(|| {
                    CacheError::ConnectionError("Valkey URL is required for valkey backend".to_string())
                })?;

                match ValkeyCacheBackend::new(url, Some("rustblog:".to_string())).await {
                    Ok(valkey) => {
                        println!("✅ Valkey 缓存后端初始化成功");

                        // 执行健康检查
                        if let Err(e) = valkey.health_check().await {
                            tracing::warn!("Valkey 健康检查失败: {}, 降级到本地缓存", e);
                            let local = Arc::new(LocalCacheBackend::new(Some(10000))) as Arc<dyn CacheBackend>;
                            (local.clone(), None, None)
                        } else {
                            let local = Arc::new(LocalCacheBackend::new(Some(10000))) as Arc<dyn CacheBackend>;
                            let valkey_arc = Arc::new(valkey);
                            (Arc::clone(&valkey_arc) as Arc<dyn CacheBackend>, Some(local), Some(valkey_arc))
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Valkey 连接失败: {}, 使用本地缓存降级", e);
                        let local = Arc::new(LocalCacheBackend::new(Some(10000))) as Arc<dyn CacheBackend>;
                        (local.clone(), None, None)
                    }
                }
            }
            "local" => {
                println!("✅ 使用本地内存缓存");
                let local = Arc::new(LocalCacheBackend::new(Some(10000))) as Arc<dyn CacheBackend>;
                (local, None, None)
            }
            "auto" => {
                // 自动模式：尝试使用 Valkey，失败则降级到本地
                if let Some(url) = valkey_url {
                    match ValkeyCacheBackend::new(url, Some("rustblog:".to_string())).await {
                        Ok(valkey) => {
                            // 执行健康检查
                            if let Err(e) = valkey.health_check().await {
                                println!("⚠️  Valkey 不可用（健康检查失败: {}），使用本地缓存", e);
                                let local = Arc::new(LocalCacheBackend::new(Some(10000))) as Arc<dyn CacheBackend>;
                                (local.clone(), None, None)
                            } else {
                                println!("✅ 自动检测到 Valkey，使用 Valkey 缓存");
                                let local = Arc::new(LocalCacheBackend::new(Some(10000))) as Arc<dyn CacheBackend>;
                                let valkey_arc = Arc::new(valkey);
                                (Arc::clone(&valkey_arc) as Arc<dyn CacheBackend>, Some(local), Some(valkey_arc))
                            }
                        }
                        Err(e) => {
                            println!("⚠️  Valkey 不可用（连接失败: {}），使用本地缓存", e);
                            let local = Arc::new(LocalCacheBackend::new(Some(10000))) as Arc<dyn CacheBackend>;
                            (local.clone(), None, None)
                        }
                    }
                } else {
                    println!("⚠️  未配置 Valkey URL，使用本地缓存");
                    let local = Arc::new(LocalCacheBackend::new(Some(10000))) as Arc<dyn CacheBackend>;
                    (local.clone(), None, None)
                }
            }
            _ => {
                return Err(CacheError::Unknown(format!("Unknown cache backend: {}", backend_type)));
            }
        };

        let fallback_enabled = Arc::new(AtomicBool::new(config.enable_fallback));
        let primary_healthy = Arc::new(AtomicBool::new(valkey_backend.is_some()));
        let consecutive_failures = Arc::new(AtomicUsize::new(0));
        let degradation_config = DegradationConfig::default();
        let operation_history = DashMap::new();
        let last_sliding_window_check = Arc::new(AtomicUsize::new(0));

        // 启动后台健康检查任务
        let health_check_task = if valkey_backend.is_some() {
            let valkey_backend_clone = valkey_backend.clone();
            let primary_healthy_clone = Arc::clone(&primary_healthy);
            let consecutive_failures_clone = Arc::clone(&consecutive_failures);

            Some(Arc::new(tokio::spawn(async move {
                // 安全地获取 valkey_backend，如果为 None 则不执行健康检查
                if let Some(backend) = valkey_backend_clone {
                    Self::health_check_loop(
                        backend,
                        primary_healthy_clone,
                        consecutive_failures_clone,
                    ).await;
                }
            })))
        } else {
            None
        };

        // 启动重连任务（如果 Valkey URL 存在但连接失败）
        let reconnect_task = if valkey_url_owned.is_some() && valkey_backend.is_none() {
            let url = valkey_url_owned.clone().unwrap();
            let primary_healthy_clone = Arc::clone(&primary_healthy);
            let consecutive_failures_clone = Arc::clone(&consecutive_failures);
            let fallback_enabled_clone = Arc::clone(&fallback_enabled);
            let degradation_config_clone = degradation_config.clone();

            Some(Arc::new(tokio::spawn(async move {
                Self::reconnect_loop(
                    &url,
                    primary_healthy_clone,
                    consecutive_failures_clone,
                    fallback_enabled_clone,
                    degradation_config_clone,
                ).await;
            })))
        } else {
            None
        };

        Ok(Self {
            primary,
            fallback,
            fallback_enabled,
            config,
            degradation_config,
            primary_healthy,
            consecutive_failures,
            operation_history,
            valkey_backend,
            valkey_url: valkey_url_owned,
            health_check_task,
            reconnect_task,
            last_sliding_window_check,
        })
    }

    /// 获取缓存值
    pub async fn get(&self, key: &str) -> Option<String> {
        // 如果主缓存当前不健康，尝试先检查是否恢复
        if self.valkey_backend.is_some() && !self.primary_healthy.load(Ordering::Relaxed) {
            eprintln!("🔍 Valkey 当前不健康，尝试检查恢复状态...");
            if self.check_health().await.is_err() {
                eprintln!("⚠️  Valkey 仍然不健康，使用备用缓存");
                // 仍然不健康，使用备用缓存
                if self.fallback_enabled.load(Ordering::Relaxed) {
                    if let Some(fallback) = &self.fallback {
                        return fallback.get(key).await;
                    }
                }
                return None;
            } else {
                println!("✅ Valkey 已恢复健康状态");
            }
        }

        // 首先尝试主缓存
        match self.primary.get(key).await {
            Some(value) => {
                // 成功获取，重置失败计数器
                let was_degraded = !self.primary_healthy.swap(true, Ordering::Relaxed);
                if was_degraded {
                    println!("✅ Valkey 已从降级状态恢复");
                }
                self.consecutive_failures.store(0, Ordering::Relaxed);
                self.record_operation(false);
                Some(value)
            }
            None => {
                // 记录失败操作
                self.record_operation(true);

                // 获取失败，增加失败计数
                let failures = self.consecutive_failures.fetch_add(1, Ordering::Relaxed) + 1;

                // 检查是否应该降级（连续失败或滑动窗口失败率）
                let should_degrade = if self.valkey_backend.is_some() {
                    let consecutive_threshold = self.degradation_config.consecutive_failures_threshold;

                    // 优化：限制滑动窗口检查频率，避免高并发时频繁遍历
                    let sliding_window_triggered = if self.should_check_sliding_window() {
                        self.check_sliding_window_failure_rate()
                    } else {
                        false
                    };

                    if failures >= consecutive_threshold {
                        eprintln!("⚠️  Valkey 连续失败 {} 次（阈值: {}），触发降级",
                                  failures, consecutive_threshold);
                        true
                    } else if sliding_window_triggered {
                        eprintln!("⚠️  Valkey 滑动窗口失败率超过阈值（{}%），触发降级",
                                  self.degradation_config.sliding_window_failure_rate);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };

                if should_degrade {
                    eprintln!("🔄 Valkey 已降级到备用缓存");
                    self.primary_healthy.store(false, Ordering::Relaxed);
                }

                // 如果启用了降级，尝试从备用缓存获取
                if self.fallback_enabled.load(Ordering::Relaxed) {
                    if let Some(fallback) = &self.fallback {
                        return fallback.get(key).await;
                    }
                }
                None
            }
        }
    }

    /// 设置缓存值
    pub async fn set(&self, key: &str, value: &str) -> Result<(), CacheError> {
        let ttl = Duration::from_secs(self.config.default_ttl);

        // 如果主缓存当前不健康，尝试先检查是否恢复
        if self.valkey_backend.is_some() && !self.primary_healthy.load(Ordering::Relaxed) {
            eprintln!("🔍 Valkey 当前不健康，尝试检查恢复状态...");
            if self.check_health().await.is_ok() {
                println!("✅ Valkey 已恢复健康状态");
                // 恢复了，继续尝试主缓存
            } else {
                eprintln!("⚠️  Valkey 仍然不健康，使用备用缓存");
                // 仍然不健康，直接使用备用缓存
                if self.fallback_enabled.load(Ordering::Relaxed) {
                    if let Some(fallback) = &self.fallback {
                        return fallback.set(key, value, ttl).await;
                    }
                }
                return Err(CacheError::ConnectionError("Primary cache unhealthy and no fallback available".to_string()));
            }
        }

        // 尝试设置到主缓存
        match self.primary.set(key, value, ttl).await {
            Ok(()) => {
                // 成功设置，重置失败计数器
                let was_degraded = !self.primary_healthy.swap(true, Ordering::Relaxed);
                if was_degraded {
                    println!("✅ Valkey 已从降级状态恢复");
                }
                self.consecutive_failures.store(0, Ordering::Relaxed);
                self.record_operation(false);

                // 如果启用了降级，同时设置到备用缓存
                if self.fallback_enabled.load(Ordering::Relaxed) {
                    if let Some(fallback) = &self.fallback {
                        let _ = fallback.set(key, value, ttl).await;
                    }
                }
                Ok(())
            }
            Err(e) => {
                // 记录失败操作
                self.record_operation(true);

                // 增加失败计数
                let failures = self.consecutive_failures.fetch_add(1, Ordering::Relaxed) + 1;
                let is_critical_error = e.is_degradation_trigger();

                // 检查是否应该降级
                let should_degrade = if self.valkey_backend.is_some() {
                    let threshold = if is_critical_error {
                        self.degradation_config.critical_error_threshold
                    } else {
                        self.degradation_config.consecutive_failures_threshold
                    };

                    // 优化：限制滑动窗口检查频率，避免高并发时频繁遍历
                    let sliding_window_triggered = if self.should_check_sliding_window() {
                        self.check_sliding_window_failure_rate()
                    } else {
                        false
                    };

                    if failures >= threshold {
                        eprintln!("⚠️  Valkey 主缓存失败 ({}, 连续 {}/{}, 阈值: {}): {}, 触发降级",
                                  if is_critical_error { "严重错误" } else { "普通错误" },
                                  failures, threshold,
                                  if is_critical_error {
                                      self.degradation_config.critical_error_threshold
                                  } else {
                                      self.degradation_config.consecutive_failures_threshold
                                  },
                                  e);
                        true
                    } else if sliding_window_triggered {
                        eprintln!("⚠️  Valkey 滑动窗口失败率超过阈值（{}%），触发降级",
                                  self.degradation_config.sliding_window_failure_rate);
                        true
                    } else {
                        eprintln!("⚠️  Valkey 主缓存失败 ({}, 连续 {}/{}, 阈值: {}): {}",
                                  if is_critical_error { "严重错误" } else { "普通错误" },
                                  failures, threshold,
                                  if is_critical_error {
                                      self.degradation_config.critical_error_threshold
                                  } else {
                                      self.degradation_config.consecutive_failures_threshold
                                  },
                                  e);
                        false
                    }
                } else {
                    false
                };

                if should_degrade {
                    eprintln!("🔄 Valkey 已降级到备用缓存");
                    self.primary_healthy.store(false, Ordering::Relaxed);
                }

                // 主缓存失败，尝试降级到备用缓存
                if self.fallback_enabled.load(Ordering::Relaxed) {
                    if let Some(fallback) = &self.fallback {
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
        match self.primary.delete_pattern(pattern).await {
            Ok(()) => {
                // 从备用缓存删除（如果启用）
                if self.fallback_enabled.load(Ordering::Relaxed) {
                    if let Some(fallback) = &self.fallback {
                        let _ = fallback.delete_pattern(pattern).await;
                    }
                }
                Ok(())
            }
            Err(e) => {
                // 主缓存删除失败，尝试仅删除备用缓存
                if self.fallback_enabled.load(Ordering::Relaxed) {
                    if let Some(fallback) = &self.fallback {
                        eprintln!("⚠️  主缓存 delete_pattern 失败: {}, 仅删除备用缓存", e);
                        return fallback.delete_pattern(pattern).await;
                    }
                }
                Err(e)
            }
        }
    }

    /// 获取缓存统计信息
    pub fn get_stats(&self) -> CacheStats {
        CacheStats {
            has_fallback: self.fallback.is_some(),
            fallback_enabled: self.fallback_enabled.load(Ordering::Relaxed),
            default_ttl: self.config.default_ttl,
        }
    }

    /// 检查是否应该检查滑动窗口（添加时间间隔限制）
    /// 优化：避免在高并发场景下频繁检查滑动窗口
    fn should_check_sliding_window(&self) -> bool {
        if !self.degradation_config.enable_sliding_window {
            return false;
        }

        const CHECK_INTERVAL_SECONDS: u64 = 5; // 每 5 秒最多检查一次
        let now = Instant::now();
        let last_check_secs = self.last_sliding_window_check.load(Ordering::Relaxed);

        // 将当前时间转换为秒数
        let now_secs = now.elapsed().as_secs();

        // 如果距离上次检查不足 5 秒，跳过检查
        if now_secs.saturating_sub(last_check_secs as u64) < CHECK_INTERVAL_SECONDS {
            return false;
        }

        // 更新最后检查时间
        self.last_sliding_window_check.store(now_secs as usize, Ordering::Relaxed);
        true
    }

    /// 检查滑动窗口内的失败率（使用 DashMap 无锁方式）
    /// 优化：添加最大遍历限制，防止在大量并发请求时卡死
    fn check_sliding_window_failure_rate(&self) -> bool {
        if !self.degradation_config.enable_sliding_window {
            return false;
        }

        const MAX_ITERATIONS: usize = 500; // 最大遍历记录数，防止卡死
        const MIN_SAMPLE_SIZE: usize = 10; // 最小样本数，太少不计算

        let window_duration = Duration::from_secs(self.degradation_config.sliding_window_seconds);
        let now = Instant::now();
        let threshold = self.degradation_config.sliding_window_failure_rate;

        // DashMap 支持并发迭代，无需加锁
        let mut total_operations = 0usize;
        let mut failure_count = 0usize;
        let mut keys_to_remove: Vec<u64> = Vec::new();

        // 限制最大遍历次数，防止在大量并发请求时卡死
        for (i, entry) in self.operation_history.iter().enumerate() {
            if i >= MAX_ITERATIONS {
                break; // 达到最大遍历限制，提前退出
            }

            let (key, (timestamp, failed)) = entry.pair();

            // 移除窗口外的记录
            if now.duration_since(*timestamp) > window_duration {
                keys_to_remove.push(*key);
                continue;
            }

            total_operations += 1;
            if *failed {
                failure_count += 1;
            }
        }

        // 批量删除过期记录
        for key in keys_to_remove {
            self.operation_history.remove(&key);
        }

        // 样本太少不计算失败率
        if total_operations < MIN_SAMPLE_SIZE {
            return false;
        }

        let failure_rate = (failure_count as f32 / total_operations as f32) * 100.0;

        failure_rate >= threshold
    }

    /// 记录操作结果到滑动窗口（使用 DashMap 无锁方式）
    fn record_operation(&self, failed: bool) {
        if !self.degradation_config.enable_sliding_window {
            return;
        }

        // 使用循环计数器作为 key，避免溢出
        // 当达到 MAX_HISTORY_SIZE 时，会循环使用旧 key
        const MAX_HISTORY_SIZE: usize = 1000;
        static OPERATION_COUNTER: AtomicUsize = AtomicUsize::new(0);
        let key = (OPERATION_COUNTER.fetch_add(1, Ordering::Relaxed) % MAX_HISTORY_SIZE) as u64;

        self.operation_history.insert(key, (Instant::now(), failed));

        // 不需要额外清理，因为 key 会循环重用
        // 滑动窗口清理在 check_sliding_window_failure_rate 中处理
    }

    /// 后台健康检查循环
    async fn health_check_loop(
        valkey_backend: Arc<ValkeyCacheBackend>,
        primary_healthy: Arc<AtomicBool>,
        consecutive_failures: Arc<AtomicUsize>,
    ) {
        const HEALTH_CHECK_INTERVAL: Duration = Duration::from_secs(15);  // 每 15 秒检查一次（提高频率）
        const QUICK_CHECK_INTERVAL: Duration = Duration::from_secs(5);    // 快速检查间隔（在不健康时）

        let mut interval = tokio::time::interval(HEALTH_CHECK_INTERVAL);
        let mut quick_check = false;

        loop {
            if quick_check {
                tokio::time::sleep(QUICK_CHECK_INTERVAL).await;
            } else {
                interval.tick().await;
            }

            // 执行健康检查
            match valkey_backend.health_check().await {
                Ok(()) => {
                    let was_unhealthy = !primary_healthy.swap(true, Ordering::Relaxed);
                    if was_unhealthy {
                        println!("✅ Valkey 连接已恢复");
                        // 重置失败计数器
                        consecutive_failures.store(0, Ordering::Relaxed);
                        // 恢复正常检查频率
                        quick_check = false;
                    }
                }
                Err(e) => {
                    let was_healthy = primary_healthy.swap(false, Ordering::Relaxed);
                    if was_healthy {
                        eprintln!("⚠️  Valkey 健康检查失败: {}", e);
                        // 切换到快速检查模式
                        quick_check = true;
                    }
                    // 健康检查失败也算一次失败
                    let failures = consecutive_failures.fetch_add(1, Ordering::Relaxed) + 1;
                    eprintln!("⚠️  Valkey 连续失败次数: {}", failures);
                }
            }
        }
    }

    /// Valkey 重连循环（用于初始化失败后的自动重连）
    async fn reconnect_loop(
        valkey_url: &str,
        primary_healthy: Arc<AtomicBool>,
        consecutive_failures: Arc<AtomicUsize>,
        _fallback_enabled: Arc<AtomicBool>,
        degradation_config: DegradationConfig,
    ) {
        let reconnect_interval = Duration::from_secs(degradation_config.reconnect_interval);
        let mut interval = tokio::time::interval(reconnect_interval);

        loop {
            interval.tick().await;

            println!("🔄 尝试重新连接 Valkey...");

            match ValkeyCacheBackend::new(valkey_url, Some("rustblog:".to_string())).await {
                Ok(valkey) => {
                    match valkey.health_check().await {
                        Ok(()) => {
                            println!("✅ Valkey 重连成功，已恢复");
                            // 标记为健康
                            primary_healthy.store(true, Ordering::Relaxed);
                            // 重置失败计数器
                            consecutive_failures.store(0, Ordering::Relaxed);
                            // 注意：由于无法动态替换 primary，这里只是标记状态
                            // 实际的恢复需要重启服务或者后续版本实现动态替换
                            break;
                        }
                        Err(e) => {
                            eprintln!("⚠️  Valkey 重连后健康检查失败: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("⚠️  Valkey 重连失败: {}", e);
                }
            }
        }
    }

    /// 检查 Valkey 连接健康状态
    /// 注意：此方法仅检查连接状态，不修改 primary_healthy 标志
    /// 实际的健康状态更新在 get/set 操作中进行
    pub async fn check_health(&self) -> Result<(), CacheError> {
        if let Some(valkey) = &self.valkey_backend {
            valkey.health_check().await
        } else {
            // 如果没有 Valkey，总是健康
            Ok(())
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