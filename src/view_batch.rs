/// 阅读记录批量处理模块
/// 
/// 使用 tokio::sync::mpsc 通道批量写入阅读记录
/// 预期效果：数据库写入减少 80-90%

use tokio::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

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
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,      // 每100条记录批量写入
            batch_timeout: 5,     // 5秒超时自动写入
        }
    }
}

/// 批量处理器
pub struct ViewBatchProcessor {
    tx: mpsc::UnboundedSender<ViewRecord>,
    _handle: tokio::task::JoinHandle<()>,
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
    pub fn record_view(&self, record: ViewRecord) -> Result<(), mpsc::error::SendError<ViewRecord>> {
        self.tx.send(record)
    }

    /// 批量处理器主循环
    async fn batch_processor(
        pool: Arc<Pool<SqliteConnectionManager>>,
        mut rx: mpsc::UnboundedReceiver<ViewRecord>,
        config: BatchConfig,
    ) {
        let mut batch = Vec::with_capacity(config.batch_size);
        let mut interval = tokio::time::interval(Duration::from_secs(config.batch_timeout));
        
        loop {
            tokio::select! {
                // 接收新记录
                result = rx.recv() => {
                    match result {
                        Some(record) => {
                            batch.push(record);
                            
                            // 达到批量大小，立即写入
                            if batch.len() >= config.batch_size {
                                if let Err(e) = Self::flush_batch(&pool, &mut batch).await {
                                    eprintln!("批量写入阅读记录失败: {}", e);
                                }
                            }
                        }
                        None => {
                            // 通道关闭，写入剩余记录并退出
                            if !batch.is_empty() {
                                if let Err(e) = Self::flush_batch(&pool, &mut batch).await {
                                    eprintln!("批量写入阅读记录失败: {}", e);
                                }
                            }
                            break;
                        }
                    }
                }
                // 定时器触发
                _ = interval.tick() => {
                    // 超时，写入当前批次
                    if !batch.is_empty() {
                        if let Err(e) = Self::flush_batch(&pool, &mut batch).await {
                            eprintln!("批量写入阅读记录失败: {}", e);
                        }
                    }
                }
            }
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
    ip == "127.0.0.1" || 
    ip == "::1" || 
    ip == "localhost" || 
    ip == "0.0.0.0" || 
    ip.is_empty() ||
    ip.starts_with("127.") ||
    ip.starts_with("192.168.") ||
    ip.starts_with("10.") ||
    ip.starts_with("172.16.") || ip.starts_with("172.17.") || ip.starts_with("172.18.") ||
    ip.starts_with("172.19.") || ip.starts_with("172.20.") || ip.starts_with("172.21.") ||
    ip.starts_with("172.22.") || ip.starts_with("172.23.") || ip.starts_with("172.24.") ||
    ip.starts_with("172.25.") || ip.starts_with("172.26.") || ip.starts_with("172.27.") ||
    ip.starts_with("172.28.") || ip.starts_with("172.29.") || ip.starts_with("172.30.") ||
    ip.starts_with("172.31.")
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
}