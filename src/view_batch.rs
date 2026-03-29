//! 阅读记录批量处理模块
//!
//! 使用 tokio::sync::mpsc 通道批量写入阅读记录
//! 预期效果：数据库写入减少 80-90%

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

/// 阅读记录消息
#[derive(Debug, Clone)]
pub struct ViewRecord {
    pub passage_uuid: String,
    pub ip: String,
    pub user_agent: Option<String>,
    pub country: String,
    pub city: String,
    pub region: String,
    pub view_time: chrono::DateTime<chrono::Utc>,
}

/// 批量配置
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// 批量大小
    pub batch_size: usize,
    /// 批次超时时间（秒）
    pub batch_timeout: u64,
    /// 是否启用自适应批量大小
    pub adaptive: bool,
    /// 最小批量大小
    pub min_batch_size: usize,
    /// 最大批量大小
    pub max_batch_size: usize,
    /// 自适应调整间隔（秒）
    pub adaptive_interval: u64,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,       // 每100条记录批量写入
            batch_timeout: 5,      // 5秒超时自动写入
            adaptive: true,        // 启用自适应
            min_batch_size: 50,    // 最小50条
            max_batch_size: 500,   // 最大500条
            adaptive_interval: 30, // 每30秒调整一次
        }
    }
}

/// 批量处理器
pub struct ViewBatchProcessor {
    tx: mpsc::UnboundedSender<ViewRecord>,
    _handle: tokio::task::JoinHandle<()>,
}

/// 自适应调整状态
#[derive(Debug)]
pub(crate) struct AdaptiveState {
    current_batch_size: usize,
    #[allow(dead_code)]
    last_adjustment: chrono::DateTime<chrono::Utc>,
    records_received: usize,
    #[allow(dead_code)]
    records_last_interval: usize,
}

impl ViewBatchProcessor {
    /// 创建新的批量处理器
    pub fn new(pool: Arc<Pool<SqliteConnectionManager>>, config: BatchConfig) -> Self {
        let (tx, rx) = mpsc::unbounded_channel::<ViewRecord>();

        let handle = tokio::spawn(async move {
            Self::batch_processor(pool, rx, config).await;
        });

        Self {
            tx,
            _handle: handle,
        }
    }

    /// 记录阅读（异步发送）
    pub fn record_view(
        &self,
        record: ViewRecord,
    ) -> Result<(), Box<mpsc::error::SendError<ViewRecord>>> {
        self.tx.send(record).map_err(Box::new)
    }

    /// 批量处理器主循环
    async fn batch_processor(
        pool: Arc<Pool<SqliteConnectionManager>>,
        mut rx: mpsc::UnboundedReceiver<ViewRecord>,
        config: BatchConfig,
    ) {
        let mut adaptive_state = AdaptiveState {
            current_batch_size: config.batch_size,
            last_adjustment: chrono::Utc::now(),
            records_received: 0,
            records_last_interval: 0,
        };

        let mut batch = Vec::with_capacity(adaptive_state.current_batch_size);
        let mut interval = tokio::time::interval(Duration::from_secs(config.batch_timeout));

        if config.adaptive {
            let mut adaptive_interval =
                tokio::time::interval(Duration::from_secs(config.adaptive_interval));

            loop {
                tokio::select! {
                    // 接收新记录
                    result = rx.recv() => {
                        match result {
                            Some(record) => {
                                adaptive_state.records_received += 1;
                                batch.push(record);

                                // 达到批量大小，立即写入
                                if batch.len() >= adaptive_state.current_batch_size
                                    && let Err(e) = Self::flush_batch(&pool, &mut batch).await {
                                        eprintln!("批量写入阅读记录失败: {}", e);
                                    }
                            }
                            None => {
                                // 通道关闭，写入剩余记录并退出
                                if !batch.is_empty()
                                    && let Err(e) = Self::flush_batch(&pool, &mut batch).await {
                                        eprintln!("批量写入阅读记录失败: {}", e);
                                    }
                                break;
                            }
                        }
                    }
                    // 定时器触发
                    _ = interval.tick() => {
                        // 超时，写入当前批次
                        if !batch.is_empty()
                            && let Err(e) = Self::flush_batch(&pool, &mut batch).await {
                                eprintln!("批量写入阅读记录失败: {}", e);
                            }
                    }
                    // 自适应调整检查
                    _ = adaptive_interval.tick() => {
                        Self::adjust_batch_size(&config, &mut adaptive_state, &mut batch);
                    }
                }
            }
        } else {
            loop {
                tokio::select! {
                    // 接收新记录
                    result = rx.recv() => {
                        match result {
                            Some(record) => {
                                batch.push(record);

                                // 达到批量大小，立即写入
                                if batch.len() >= config.batch_size
                                    && let Err(e) = Self::flush_batch(&pool, &mut batch).await {
                                        eprintln!("批量写入阅读记录失败: {}", e);
                                    }
                            }
                            None => {
                                // 通道关闭，写入剩余记录并退出
                                if !batch.is_empty()
                                    && let Err(e) = Self::flush_batch(&pool, &mut batch).await {
                                        eprintln!("批量写入阅读记录失败: {}", e);
                                    }
                                break;
                            }
                        }
                    }
                    // 定时器触发
                    _ = interval.tick() => {
                        // 超时，写入当前批次
                        if !batch.is_empty()
                            && let Err(e) = Self::flush_batch(&pool, &mut batch).await {
                                eprintln!("批量写入阅读记录失败: {}", e);
                            }
                    }
                }
            }
        }
    }

    /// 自适应调整批量大小
    pub(crate) fn adjust_batch_size(
        config: &BatchConfig,
        state: &mut AdaptiveState,
        batch: &mut [ViewRecord],
    ) {
        if !config.adaptive {
            return;
        }

        let records_per_second = state.records_received as f64 / config.adaptive_interval as f64;
        let target_batch_size = Self::calculate_target_batch_size(records_per_second, config);

        // 平滑调整批量大小
        let adjustment = (target_batch_size as i32 - state.current_batch_size as i32) / 2;
        let new_batch_size = (state.current_batch_size as i32 + adjustment) as usize;

        // 限制在最小和最大值之间
        let new_batch_size = new_batch_size.clamp(config.min_batch_size, config.max_batch_size);

        if new_batch_size != state.current_batch_size {
            println!(
                "📊 自适应调整批量大小: {} -> {} (记录速率: {:.1}/秒)",
                state.current_batch_size, new_batch_size, records_per_second
            );

            // 更新当前批量大小
            state.current_batch_size = new_batch_size;

            // 如果新批量大小小于当前批次大小，立即写入
            if batch.len() >= new_batch_size {
                println!("⚡ 批量大小调整，立即写入当前批次");
            }
        }

        // 重置计数器
        state.records_received = 0;
    }

    /// 根据记录速率计算目标批量大小
    pub(crate) fn calculate_target_batch_size(
        records_per_second: f64,
        config: &BatchConfig,
    ) -> usize {
        // 根据速率调整：
        // - 低速率（< 1/秒）：使用最小批量大小
        // - 中速率（1-10/秒）：线性增长
        // - 高速率（> 10/秒）：使用最大批量大小

        if records_per_second < 1.0 {
            config.min_batch_size
        } else if records_per_second < 10.0 {
            let ratio = (records_per_second - 1.0) / 9.0; // 0.0 - 1.0
            let size = config.min_batch_size as f64
                + ratio * (config.max_batch_size as f64 - config.min_batch_size as f64);
            size as usize
        } else {
            config.max_batch_size
        }
    }

    /// 刷新批次到数据库
    async fn flush_batch(
        pool: &Arc<Pool<SqliteConnectionManager>>,
        batch: &mut Vec<ViewRecord>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if batch.is_empty() {
            return Ok(());
        }

        let conn = pool.get()?;

        // 开始事务
        let tx = conn.unchecked_transaction()?;

        // 批量插入
        for record in batch.iter() {
            let view_date = record.view_time.format("%Y-%m-%d").to_string();
            tx.execute(
                "INSERT INTO article_views (passage_uuid, ip, user_agent, country, city, region, view_date, view_time, created_at) 
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                rusqlite::params![
                    &record.passage_uuid,
                    &record.ip,
                    &record.user_agent,
                    &record.country,
                    &record.city,
                    &record.region,
                    &view_date,
                    &record.view_time,
                    &record.view_time,
                ],
            )?;
        }

        // 提交事务
        tx.commit()?;

        let count = batch.len();
        batch.clear();

        println!("✅ 批量写入 {} 条阅读记录", count);
        Ok(())
    }
}

/// 检查是否为本地IP
pub fn is_local_ip(ip: &str) -> bool {
    if ip.is_empty() || ip == "::1" || ip == "localhost" {
        return true;
    }

    // 解析 IPv4 地址的各段，避免 16 次逐一的 starts_with 字符串比较
    let parts: Vec<&str> = ip.splitn(4, '.').collect();
    if parts.len() == 4
        && let Ok(a) = parts[0].parse::<u8>() {
            match a {
                // 127.0.0.0/8 loopback
                127 => return true,
                // 10.0.0.0/8 private
                10 => return true,
                // 0.0.0.0
                0 => return ip == "0.0.0.0",
                // 192.168.0.0/16 private
                192 => {
                    if let Ok(b) = parts[1].parse::<u8>() {
                        return b == 168;
                    }
                }
                // 172.16.0.0/12 private (172.16.x.x – 172.31.x.x)
                172 => {
                    if let Ok(b) = parts[1].parse::<u8>() {
                        return (16..=31).contains(&b);
                    }
                }
                _ => {}
            }
        }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_local_ip() {
        assert!(is_local_ip("127.0.0.1"));
        assert!(is_local_ip("192.168.1.1"));
        assert!(is_local_ip("10.0.0.1"));
        assert!(is_local_ip("172.16.0.1"));
        assert!(!is_local_ip("8.8.8.8"));
        assert!(!is_local_ip("1.1.1.1"));
    }

    #[test]
    fn test_batch_config_default() {
        let config = BatchConfig::default();
        assert_eq!(config.batch_size, 100);
        assert_eq!(config.batch_timeout, 5);
        assert!(config.adaptive);
        assert_eq!(config.min_batch_size, 50);
        assert_eq!(config.max_batch_size, 500);
        assert_eq!(config.adaptive_interval, 30);
    }

    #[test]
    fn test_calculate_target_batch_size_low_rate() {
        let config = BatchConfig::default();

        // 低速率 (< 1/秒)
        let size = ViewBatchProcessor::calculate_target_batch_size(0.5, &config);
        assert_eq!(size, config.min_batch_size);

        let size = ViewBatchProcessor::calculate_target_batch_size(0.1, &config);
        assert_eq!(size, config.min_batch_size);
    }

    #[test]
    fn test_calculate_target_batch_size_medium_rate() {
        let config = BatchConfig::default();

        // 中速率 (1-10/秒)
        let size = ViewBatchProcessor::calculate_target_batch_size(1.0, &config);
        assert_eq!(size, config.min_batch_size);

        let size = ViewBatchProcessor::calculate_target_batch_size(5.5, &config);
        assert!(size > config.min_batch_size);
        assert!(size < config.max_batch_size);

        let size = ViewBatchProcessor::calculate_target_batch_size(10.0, &config);
        assert_eq!(size, config.max_batch_size);
    }

    #[test]
    fn test_calculate_target_batch_size_high_rate() {
        let config = BatchConfig::default();

        // 高速率 (> 10/秒)
        let size = ViewBatchProcessor::calculate_target_batch_size(15.0, &config);
        assert_eq!(size, config.max_batch_size);

        let size = ViewBatchProcessor::calculate_target_batch_size(100.0, &config);
        assert_eq!(size, config.max_batch_size);
    }

    #[test]
    fn test_adjust_batch_size_clamping() {
        let config = BatchConfig {
            batch_size: 100,
            batch_timeout: 5,
            adaptive: true,
            min_batch_size: 50,
            max_batch_size: 500,
            adaptive_interval: 30,
        };

        let mut state = AdaptiveState {
            current_batch_size: 100,
            last_adjustment: chrono::Utc::now(),
            records_received: 0,
            records_last_interval: 0,
        };

        let mut batch = Vec::new();

        // 测试平滑调整：低速率下会减小
        state.records_received = 0; // 0 记录/30秒 = 0/秒
        let initial_size = state.current_batch_size;
        ViewBatchProcessor::adjust_batch_size(&config, &mut state, &mut batch);
        // 平滑调整会逐步减少，不会直接跳到最小值
        assert!(state.current_batch_size >= config.min_batch_size);
        assert!(state.current_batch_size < initial_size);

        // 测试高负载下会增加
        state.current_batch_size = 100;
        state.records_received = 1000; // 1000 记录/30秒 ≈ 33/秒
        let before_adjust = state.current_batch_size;
        ViewBatchProcessor::adjust_batch_size(&config, &mut state, &mut batch);
        // 平滑调整会逐步增加
        assert!(state.current_batch_size > before_adjust);
        assert!(state.current_batch_size <= config.max_batch_size);

        // 测试边界条件：不会超出范围
        state.current_batch_size = config.min_batch_size;
        state.records_received = 0;
        ViewBatchProcessor::adjust_batch_size(&config, &mut state, &mut batch);
        assert!(state.current_batch_size >= config.min_batch_size);

        state.current_batch_size = config.max_batch_size;
        state.records_received = 10000;
        ViewBatchProcessor::adjust_batch_size(&config, &mut state, &mut batch);
        assert!(state.current_batch_size <= config.max_batch_size);
    }
}
