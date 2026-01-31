use actix_web::{dev::Payload, Error, FromRequest, HttpRequest, HttpResponse};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

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

/// 限流器
#[derive(Debug)]
struct RateLimiter {
    second_windows: HashMap<String, SlidingWindow>,
    minute_windows: HashMap<String, SlidingWindow>,
}

impl RateLimiter {
    fn new() -> Self {
        Self {
            second_windows: HashMap::new(),
            minute_windows: HashMap::new(),
        }
    }

    fn check(&mut self, key: &str, config: &RateLimitConfig) -> Result<(), RateLimitError> {
        let second_window = self
            .second_windows
            .entry(key.to_string())
            .or_insert_with(|| SlidingWindow::new(Duration::from_secs(1), config.per_second));

        let minute_window = self
            .minute_windows
            .entry(key.to_string())
            .or_insert_with(|| SlidingWindow::new(Duration::from_secs(60), config.per_minute));

        if !second_window.check_and_record() {
            return Err(RateLimitError::TooManyRequestsPerSecond);
        }

        if !minute_window.check_and_record() {
            if let Some(_) = second_window.timestamps.last() {
                second_window.timestamps.pop();
            }
            return Err(RateLimitError::TooManyRequestsPerMinute);
        }

        Ok(())
    }

    fn cleanup(&mut self) {
        let now = Instant::now();
        let second_cutoff = now - Duration::from_secs(2);
        let minute_cutoff = now - Duration::from_secs(120);

        self.second_windows.retain(|_, window| {
            window.timestamps.last().map_or(false, |&t| t > second_cutoff)
        });
        self.minute_windows.retain(|_, window| {
            window.timestamps.last().map_or(false, |&t| t > minute_cutoff)
        });
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

/// 全局限流器实例
use once_cell::sync::Lazy;

static RATE_LIMITER: Lazy<Arc<Mutex<RateLimiter>>> = Lazy::new(|| {
    Arc::new(Mutex::new(RateLimiter::new()))
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

        // 检查限流
        if let Ok(mut limiter) = RATE_LIMITER.lock() {
            limiter.cleanup();
            if let Err(e) = limiter.check(&key, &RATE_LIMIT_CONFIG) {
                return std::future::ready(Err(actix_web::error::ErrorBadRequest(e)));
            }
        }

        std::future::ready(Ok(RateLimitCheck))
    }
}