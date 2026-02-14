use actix_web::{dev::Payload, Error, FromRequest, HttpRequest, HttpResponse};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use dashmap::DashMap;

/// 滑动窗口计数器
#[derive(Debug, Clone)]
struct SlidingWindow {
    timestamps: Vec<Instant>,
    window_size: Duration,
    max_requests: usize,
}

impl SlidingWindow {
    fn new(window_size: Duration, max_requests: usize) -> Self {
        Self {
            timestamps: Vec::with_capacity(max_requests),
            window_size,
            max_requests,
        }
    }

    fn check_and_record(&mut self) -> bool {
        let now = Instant::now();
        let cutoff = now - self.window_size;
        self.timestamps.retain(|&t| t > cutoff);

        if self.timestamps.len() >= self.max_requests {
            false
        } else {
            self.timestamps.push(now);
            true
        }
    }
}

/// 限流器配置
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub per_second: usize,
    pub per_minute: usize,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            per_second: 3,
            per_minute: 10,
        }
    }
}

/// 限流器（使用 DashMap 实现无锁并发）
#[derive(Debug)]
struct RateLimiter {
    second_windows: DashMap<String, SlidingWindow>,
    minute_windows: DashMap<String, SlidingWindow>,
    max_entries: usize,  // 最大条目数限制，防止内存无限增长
}

impl RateLimiter {
    fn new() -> Self {
        Self {
            second_windows: DashMap::new(),
            minute_windows: DashMap::new(),
            max_entries: 10000,  // 限制最多 10000 个不同的 IP 地址
        }
    }

    fn check(&self, key: &str, config: &RateLimitConfig) -> Result<(), RateLimitError> {
        // DashMap 的 entry API 内部使用细粒度锁，支持高并发
        let mut second_window = self.second_windows.entry(key.to_string())
            .or_insert_with(|| SlidingWindow::new(Duration::from_secs(1), config.per_second));

        let mut minute_window = self.minute_windows.entry(key.to_string())
            .or_insert_with(|| SlidingWindow::new(Duration::from_secs(60), config.per_minute));

        if !second_window.check_and_record() {
            return Err(RateLimitError::TooManyRequestsPerSecond);
        }

        if !minute_window.check_and_record() {
            // 不弹出秒级窗口的时间戳，保持计数准确
            // 秒级和分钟级是独立的限流，应该分别统计
            return Err(RateLimitError::TooManyRequestsPerMinute);
        }

        Ok(())
    }

    fn cleanup(&self) {
        let now = Instant::now();
        let second_cutoff = now - Duration::from_secs(2);
        let minute_cutoff = now - Duration::from_secs(120);

        // DashMap 支持并发迭代和删除
        self.second_windows.retain(|_, window| {
            window.timestamps.last().map_or(false, |&t| t > second_cutoff)
        });
        self.minute_windows.retain(|_, window| {
            window.timestamps.last().map_or(false, |&t| t > minute_cutoff)
        });

        // 检查条目数是否超过限制，如果超过则随机删除部分条目
        if self.second_windows.len() > self.max_entries {
            // 随机删除 10% 的条目以释放内存
            let remove_count = self.max_entries / 10;
            let keys_to_remove: Vec<String> = self.second_windows.iter()
                .take(remove_count)
                .map(|entry| entry.key().clone())
                .collect();
            for key in keys_to_remove {
                self.second_windows.remove(&key);
                self.minute_windows.remove(&key);
            }
            eprintln!("⚠️  限流器条目数超过限制（{}），已清理 {} 条", self.second_windows.len(), remove_count);
        }
    }
}

#[derive(Debug)]
pub enum RateLimitError {
    TooManyRequestsPerSecond,
    TooManyRequestsPerMinute,
}

impl std::fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RateLimitError::TooManyRequestsPerSecond => {
                write!(f, "Too many requests per second")
            }
            RateLimitError::TooManyRequestsPerMinute => {
                write!(f, "Too many requests per minute")
            }
        }
    }
}

impl std::error::Error for RateLimitError {}

impl actix_web::ResponseError for RateLimitError {
    fn error_response(&self) -> HttpResponse {
        match self {
            RateLimitError::TooManyRequestsPerSecond => {
                HttpResponse::TooManyRequests().json(serde_json::json!({
                    "success": false,
                    "message": "Too many requests. Maximum 3 requests per second allowed."
                }))
            }
            RateLimitError::TooManyRequestsPerMinute => {
                HttpResponse::TooManyRequests().json(serde_json::json!({
                    "success": false,
                    "message": "Too many requests. Maximum 10 requests per minute allowed."
                }))
            }
        }
    }
}

/// 全局限流器实例（使用 DashMap，无需锁）
use once_cell::sync::Lazy;

static RATE_LIMITER: Lazy<Arc<RateLimiter>> = Lazy::new(|| {
    Arc::new(RateLimiter::new())
});

static RATE_LIMIT_CONFIG: Lazy<RateLimitConfig> = Lazy::new(RateLimitConfig::default);

/// 限流检查提取器
/// 在需要限流的 handler 中添加这个参数即可
pub struct RateLimitCheck;

impl FromRequest for RateLimitCheck {
    type Error = Error;
    type Future = std::future::Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        // 获取客户端IP
        let ip = req
            .connection_info()
            .peer_addr()
            .unwrap_or_else(|| "unknown")
            .to_string();
        let key = format!("{}", ip);

        // 定期清理过期窗口（每 100 次请求清理一次）
        static CLEANUP_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let counter = CLEANUP_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if counter % 100 == 0 {
            RATE_LIMITER.cleanup();
        }

        // 检查限流（DashMap 内部使用细粒度锁，无需 try_write）
        if let Err(e) = RATE_LIMITER.check(&key, &RATE_LIMIT_CONFIG) {
            return std::future::ready(Err(actix_web::error::ErrorBadRequest(e)));
        }

        std::future::ready(Ok(RateLimitCheck))
    }
}