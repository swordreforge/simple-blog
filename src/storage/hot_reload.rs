//! 热重载模块
//!
//! 提供文件变化监听和自动重新加载路由的功能。

use crate::core::RouteEntry;
use crate::storage::RouteStorage;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use thiserror::Error;

/// 热重载错误类型
#[derive(Error, Debug)]
pub enum HotReloadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Notify error: {0}")]
    Notify(#[from] notify::Error),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Path does not exist: {0}")]
    PathNotFound(String),
}

/// 热重载配置
#[derive(Debug, Clone)]
pub struct HotReloadConfig {
    /// 文件变化检查间隔
    pub debounce_duration: Duration,
    /// 是否在启动时立即加载
    pub load_on_startup: bool,
}

impl Default for HotReloadConfig {
    fn default() -> Self {
        Self {
            debounce_duration: Duration::from_millis(500),
            load_on_startup: true,
        }
    }
}

/// 热重载管理器
///
/// 监听文件系统变化，并在检测到变化时自动重新加载路由。
///
/// # 示例
///
/// ```no_run
/// use dynamic_route_actix::storage::{FileStorage, RouteStorage};
/// use dynamic_route_actix::storage::hot_reload::HotReloadManager;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let storage = FileStorage::new("./data/routes.json");
///     let config = dynamic_route_actix::storage::hot_reload::HotReloadConfig::default();
///
///     let mut manager = HotReloadManager::new(storage, config)?;
///
///     // 启动热重载
///     manager.start().await?;
///
///     // 当文件变化时，路由会自动重新加载
///
///     Ok(())
/// }
/// ```
pub struct HotReloadManager<S>
where
    S: RouteStorage + Send + Sync + 'static,
{
    storage: Arc<S>,
    config: HotReloadConfig,
    watcher: Option<RecommendedWatcher>,
    routes: Arc<RwLock<HashMap<String, Box<dyn RouteEntry>>>>,
}

impl<S> HotReloadManager<S>
where
    S: RouteStorage + Send + Sync + 'static,
{
    /// 创建新的热重载管理器
    ///
    /// # 参数
    ///
    /// * `storage` - 路由存储实现
    /// * `config` - 热重载配置
    ///
    /// # 返回
    ///
    /// 返回热重载管理器实例
    pub fn new(storage: S, config: HotReloadConfig) -> Result<Self, HotReloadError> {
        Ok(Self {
            storage: Arc::new(storage),
            config,
            watcher: None,
            routes: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// 启动热重载
    ///
    /// # 参数
    ///
    /// * `watch_path` - 要监听的文件或目录路径
    pub async fn start(&mut self, watch_path: &Path) -> Result<(), HotReloadError> {
        // 验证路径存在
        if !watch_path.exists() {
            return Err(HotReloadError::PathNotFound(watch_path.display().to_string()));
        }

        // 如果配置要求在启动时加载
        if self.config.load_on_startup {
            self.reload_routes().await?;
        }

        // 创建文件系统监听器
        let routes = self.routes.clone();
        let storage = self.storage.clone();
        let debounce_duration = self.config.debounce_duration;

        let mut watcher = notify::recommended_watcher(move |res| {
            if let Ok(event) = res {
                if let Err(e) = handle_file_event(event, routes.clone(), storage.clone(), debounce_duration) {
                    eprintln!("Error handling file event: {}", e);
                }
            }
        })?;

        // 开始监听
        watcher.watch(watch_path, RecursiveMode::Recursive)?;
        self.watcher = Some(watcher);

        println!("Hot reload started, watching: {}", watch_path.display());

        Ok(())
    }

    /// 停止热重载
    pub fn stop(&mut self) {
        self.watcher = None;
    }

    /// 手动重新加载路由
    pub async fn reload_routes(&self) -> Result<(), HotReloadError> {
        println!("Reloading routes...");
        match self.storage.load().await {
            Ok(routes) => {
                let mut guard = self.routes.write().await;
                *guard = routes;
                println!("Routes reloaded successfully, {} routes loaded", guard.len());
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Failed to reload routes: {}", e);
                eprintln!("{}", error_msg);
                Err(HotReloadError::Storage(error_msg))
            }
        }
    }

    /// 获取当前路由
    pub async fn get_routes(&self) -> HashMap<String, Box<dyn RouteEntry>> {
        let guard = self.routes.read().await;
        guard.iter().map(|(k, v)| (k.clone(), v.clone_box())).collect()
    }

    /// 获取指定路径的路由
    pub async fn get_route(&self, path: &str) -> Option<Box<dyn RouteEntry>> {
        let guard = self.routes.read().await;
        guard.get(path).map(|route| route.clone_box())
    }
}

/// 处理文件系统事件
fn handle_file_event(
    event: notify::Event,
    routes: Arc<RwLock<HashMap<String, Box<dyn RouteEntry>>>>,
    storage: Arc<dyn RouteStorage>,
    debounce_duration: Duration,
) -> Result<(), HotReloadError> {
    // 只处理写入和创建事件
    if event.kind.is_modify() || event.kind.is_create() {
        println!("File change detected: {:?}", event.paths);

        // 使用 tokio 运行时处理异步操作
        let handle = std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                // 防抖延迟
                tokio::time::sleep(debounce_duration).await;

                match storage.load().await {
                    Ok(new_routes) => {
                        let mut guard = routes.write().await;
                        *guard = new_routes;
                        println!("Routes reloaded successfully, {} routes loaded", guard.len());
                    }
                    Err(e) => {
                        eprintln!("Failed to reload routes: {}", e);
                    }
                }
            });
        });

        handle.join().unwrap();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MemoryStorage;

    #[tokio::test]
    async fn test_hot_reload_config_default() {
        let config = HotReloadConfig::default();
        assert_eq!(config.debounce_duration, Duration::from_millis(500));
        assert!(config.load_on_startup);
    }

    #[tokio::test]
    async fn test_hot_reload_manager_creation() {
        let storage = MemoryStorage::new();
        let config = HotReloadConfig::default();

        let manager = HotReloadManager::new(storage, config);
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_hot_reload_manager_reload_routes() {
        let storage = MemoryStorage::new();
        let config = HotReloadConfig::default();

        let manager = HotReloadManager::new(storage, config).unwrap();

        // 手动重新加载（应该成功，即使没有路由）
        let result = manager.reload_routes().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_hot_reload_manager_get_routes() {
        let storage = MemoryStorage::new();
        let config = HotReloadConfig::default();

        let manager = HotReloadManager::new(storage, config).unwrap();

        // 获取路由（应该是空的）
        let routes = manager.get_routes().await;
        assert_eq!(routes.len(), 0);
    }

    #[tokio::test]
    async fn test_hot_reload_manager_start_invalid_path() {
        let storage = MemoryStorage::new();
        let config = HotReloadConfig::default();

        let mut manager = HotReloadManager::new(storage, config).unwrap();

        // 尝试监听不存在的路径
        let result = manager.start(Path::new("/nonexistent/path")).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), HotReloadError::PathNotFound(_)));
    }

    #[tokio::test]
    async fn test_hot_reload_manager_stop() {
        let storage = MemoryStorage::new();
        let config = HotReloadConfig::default();

        let mut manager = HotReloadManager::new(storage, config).unwrap();

        // 停止管理器
        manager.stop();
        assert!(manager.watcher.is_none());
    }
}