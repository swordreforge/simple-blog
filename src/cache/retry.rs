//! 缓存删除失败重试机制
//! 用于确保缓存删除操作的最终一致性

use super::backend::{CacheBackend, CacheError};
use async_trait::async_trait;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// 重试任务
#[derive(Debug, Clone)]
enum RetryTask {
    Delete(String),
    DeleteMany(Vec<String>),
    DeletePattern(String),
}

/// 重试队列配置
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// 最大重试次数
    pub max_retries: usize,
    /// 重试间隔（秒）
    pub retry_interval: u64,
    /// 队列最大长度
    pub max_queue_size: usize,
    /// 指数退避倍数
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_interval: 1,
            max_queue_size: 1000,
            backoff_multiplier: 2.0,
        }
    }
}

/// 重试队列管理器
#[allow(dead_code)]
#[derive(Clone)]
pub struct RetryQueue<B: CacheBackend> {
    backend: Arc<B>,
    config: RetryConfig,
    queue: Arc<Mutex<VecDeque<(RetryTask, usize, tokio::time::Instant)>>>,
    is_running: Arc<std::sync::atomic::AtomicBool>,
}

#[allow(dead_code)]
impl<B: CacheBackend + Send + Sync + 'static> RetryQueue<B> {
    /// 创建新的重试队列
    pub fn new(backend: B, config: RetryConfig) -> Self {
        let backend = Arc::new(backend);
        let queue = Arc::new(Mutex::new(VecDeque::new()));
        let is_running = Arc::new(std::sync::atomic::AtomicBool::new(false));

        Self {
            backend,
            config,
            queue,
            is_running,
        }
    }

    /// 启动后台重试任务
    pub fn start(&self) {
        if self.is_running.swap(true, std::sync::atomic::Ordering::SeqCst) {
            tracing::warn!("重试队列已经在运行中");
            return;
        }

        let queue = Arc::clone(&self.queue);
        let backend = Arc::clone(&self.backend);
        let config = self.config.clone();
        let is_running_clone = Arc::clone(&self.is_running);

        tokio::spawn(async move {
            tracing::info!("缓存删除重试队列已启动");

            while is_running_clone.load(std::sync::atomic::Ordering::SeqCst) {
                // 获取下一个待重试任务
                let task_info = {
                    let mut queue_guard = queue.lock().await;
                    queue_guard.pop_front()
                };

                if let Some((task, retry_count, retry_time)) = task_info {
                    // 检查是否到达重试时间
                    let now = tokio::time::Instant::now();

                    if now >= retry_time {
                        // 执行重试
                        let result = match &task {
                            RetryTask::Delete(key) => {
                                tracing::debug!("重试删除缓存: {} (第{}次)", key, retry_count);
                                backend.delete(key).await
                            }
                            RetryTask::DeleteMany(keys) => {
                                tracing::debug!("重试批量删除缓存: {:?} (第{}次)", keys, retry_count);
                                backend.delete_many(keys).await
                            }
                            RetryTask::DeletePattern(pattern) => {
                                tracing::debug!("重试模式删除缓存: {} (第{}次)", pattern, retry_count);
                                backend.delete_pattern(pattern).await
                            }
                        };

                        match result {
                            Ok(_) => {
                                tracing::info!("缓存删除重试成功 (第{}次)", retry_count);
                            }
                            Err(e) => {
                                tracing::warn!("缓存删除重试失败 (第{}次): {}", retry_count, e);

                                // 检查是否需要继续重试
                                if retry_count < config.max_retries {
                                    // 计算下次重试时间（指数退避）
                                    let backoff = Duration::from_secs_f64(
                                        config.retry_interval as f64 * config.backoff_multiplier.powi(retry_count as i32 - 1)
                                    );

                                    let next_retry_time = now + backoff;

                                    // 重新加入队列
                                    let mut queue_guard = queue.lock().await;
                                    if queue_guard.len() < config.max_queue_size {
                                        queue_guard.push_back((task, retry_count + 1, next_retry_time));
                                    } else {
                                        tracing::error!("重试队列已满，丢弃重试任务");
                                    }
                                } else {
                                    tracing::error!("缓存删除重试已达最大次数，放弃重试");
                                }
                            }
                        }
                    } else {
                        // 还未到达重试时间，重新放回队列头部
                        let mut queue_guard = queue.lock().await;
                        queue_guard.push_front((task, retry_count, retry_time));
                    }
                } else {
                    // 队列为空，等待一段时间
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }

            tracing::info!("缓存删除重试队列已停止");
        });
    }

    /// 停止后台重试任务
    pub fn stop(&self) {
        self.is_running.store(false, std::sync::atomic::Ordering::SeqCst);
    }

    /// 添加删除重试任务
    pub async fn retry_delete(&self, key: String) {
        self.add_task(RetryTask::Delete(key)).await;
    }

    /// 添加批量删除重试任务
    pub async fn retry_delete_many(&self, keys: Vec<String>) {
        if keys.is_empty() {
            return;
        }
        self.add_task(RetryTask::DeleteMany(keys)).await;
    }

    /// 添加模式删除重试任务
    pub async fn retry_delete_pattern(&self, pattern: String) {
        self.add_task(RetryTask::DeletePattern(pattern)).await;
    }

    /// 添加重试任务到队列
    async fn add_task(&self, task: RetryTask) {
        let mut queue_guard = self.queue.lock().await;

        if queue_guard.len() >= self.config.max_queue_size {
            tracing::error!("重试队列已满，丢弃重试任务");
            return;
        }

        // 立即重试（延迟1秒）
        let retry_time = tokio::time::Instant::now() + Duration::from_secs(1);
        queue_guard.push_back((task, 1, retry_time));

        tracing::debug!("添加重试任务到队列，当前队列长度: {}", queue_guard.len());
    }

    /// 获取队列状态
    pub async fn get_queue_size(&self) -> usize {
        let queue_guard = self.queue.lock().await;
        queue_guard.len()
    }
}

/// 带重试机制的缓存后端
#[allow(dead_code)]
pub struct RetryCacheBackend<B: CacheBackend + Clone + Send + Sync + 'static> {
    inner: B,
    retry_queue: Option<RetryQueue<B>>,
}

#[allow(dead_code)]
impl<B: CacheBackend + Clone + Send + Sync + 'static> RetryCacheBackend<B> {
    /// 创建新的带重试机制的缓存后端
    pub fn new(backend: B, enable_retry: bool) -> Self {
        let retry_queue = if enable_retry {
            let retry_queue = RetryQueue::new(backend.clone(), RetryConfig::default());
            retry_queue.start();
            Some(retry_queue)
        } else {
            None
        };

        Self {
            inner: backend,
            retry_queue,
        }
    }

    /// 停止重试队列
    pub fn stop_retry(&self) {
        if let Some(queue) = &self.retry_queue {
            queue.stop();
        }
    }

    /// 获取队列状态
    pub async fn get_queue_size(&self) -> usize {
        if let Some(queue) = &self.retry_queue {
            queue.get_queue_size().await
        } else {
            0
        }
    }
}

impl<B: CacheBackend + Clone + 'static> Clone for RetryCacheBackend<B> {
    fn clone(&self) -> Self {
        // 注意：重试队列不应该被克隆，克隆后的实例共享同一个队列
        Self {
            inner: self.inner.clone(),
            retry_queue: self.retry_queue.clone(),
        }
    }
}

#[allow(dead_code)]
#[async_trait]
impl<B: CacheBackend + Clone + Send + Sync + 'static> CacheBackend for RetryCacheBackend<B> {
    async fn get(&self, key: &str) -> Option<String> {
        self.inner.get(key).await
    }

    async fn set(&self, key: &str, value: &str, ttl: Duration) -> Result<(), CacheError> {
        self.inner.set(key, value, ttl).await
    }

    async fn delete(&self, key: &str) -> Result<(), CacheError> {
        match self.inner.delete(key).await {
            Ok(_) => Ok(()),
            Err(e) => {
                // 删除失败，加入重试队列
                if let Some(queue) = &self.retry_queue {
                    tracing::warn!("缓存删除失败，加入重试队列: {}", key);
                    queue.retry_delete(key.to_string()).await;
                }
                Err(e)
            }
        }
    }

    async fn delete_many(&self, keys: &[String]) -> Result<(), CacheError> {
        match self.inner.delete_many(keys).await {
            Ok(_) => Ok(()),
            Err(e) => {
                // 删除失败，加入重试队列
                if let Some(queue) = &self.retry_queue {
                    tracing::warn!("批量缓存删除失败，加入重试队列: {:?}", keys);
                    queue.retry_delete_many(keys.to_vec()).await;
                }
                Err(e)
            }
        }
    }

    async fn delete_pattern(&self, pattern: &str) -> Result<(), CacheError> {
        match self.inner.delete_pattern(pattern).await {
            Ok(_) => Ok(()),
            Err(e) => {
                // 删除失败，加入重试队列
                if let Some(queue) = &self.retry_queue {
                    tracing::warn!("模式缓存删除失败，加入重试队列: {}", pattern);
                    queue.retry_delete_pattern(pattern.to_string()).await;
                }
                Err(e)
            }
        }
    }
}