//! 日志配置模块
//!
//! 使用 tracing 进行结构化日志记录

use std::path::Path;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

/// 初始化日志系统
///
/// # 参数
/// - `log_dir`: 日志目录路径
/// - `log_level`: 日志级别 (error, warn, info, debug, trace)
pub fn init_logging(log_dir: Option<&Path>, log_level: &str) {
    // 解析日志级别
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));

    // 构建订阅者
    let registry = tracing_subscriber::registry().with(env_filter);

    // 如果有文件输出，添加文件层
    if let Some(dir) = log_dir {
        // 确保日志目录存在
        std::fs::create_dir_all(dir).ok();

        // 创建按日滚动的日志文件
        let file_appender = RollingFileAppender::new(Rotation::DAILY, dir, "rustblog.log");

        // 创建包含控制台和文件输出的订阅者
        let subscriber = registry
            .with(
                fmt::layer()
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_level(true)
                    .with_ansi(true),
            )
            .with(
                fmt::layer()
                    .with_writer(file_appender)
                    .with_ansi(false)
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_level(true),
            );

        // 初始化全局订阅者
        if let Err(e) = subscriber.try_init() {
            eprintln!("Failed to initialize logging: {}", e);
        }
    } else {
        // 只使用控制台输出
        let subscriber = registry.with(
            fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_level(true)
                .with_ansi(true),
        );

        // 初始化全局订阅者
        if let Err(e) = subscriber.try_init() {
            eprintln!("Failed to initialize logging: {}", e);
        }
    }
}

/// 获取默认日志级别
#[allow(dead_code)]
pub fn default_log_level() -> &'static str {
    if cfg!(debug_assertions) {
        "debug"
    } else {
        "info"
    }
}
