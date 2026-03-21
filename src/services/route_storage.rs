//! 路由存储抽象层
//!
//! 定义统一的路由存储接口，支持多种存储后端：
//! - DatabaseRouteStorage: SQLite数据库存储
//! - MemoryRouteStorage: 内存存储（运行时）
//! - FileRouteStorage: 文件系统存储（JSON格式）

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::RwLock;
use tokio::time::{Duration, interval};

use crate::db::models::DynamicRoute;

/// 路由存储错误类型
#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum StorageError {
    #[error("存储未找到: {0}")]
    NotFound(String),

    #[error("存储已存在: {0}")]
    AlreadyExists(String),

    #[error("存储容量超出限制: {0}")]
    CapacityExceeded(String),

    #[error("无效数据: {0}")]
    InvalidData(String),

    #[error("无效路径: {0}")]
    InvalidPath(String),

    #[error("文件操作错误: {0}")]
    FileError(String),

    #[error("序列化错误: {0}")]
    SerializationError(String),

    #[error("数据库错误: {0}")]
    DatabaseError(String),

    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON错误: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("其他错误: {0}")]
    Other(String),
}

/// 路由存储 trait
///
/// 定义统一的存储接口，所有存储实现都需要实现此 trait
#[async_trait::async_trait]
#[allow(dead_code)]
pub trait RouteStorage: Send + Sync {
    /// 保存路由
    async fn save_route(&self, route: &DynamicRoute) -> Result<i64, StorageError>;

    /// 根据ID加载路由
    async fn load_route(&self, id: i64) -> Result<Option<DynamicRoute>, StorageError>;

    /// 根据路径加载路由
    async fn load_route_by_path(&self, path: &str) -> Result<Option<DynamicRoute>, StorageError>;

    /// 删除路由
    async fn delete_route(&self, id: i64) -> Result<(), StorageError>;

    /// 列出所有路由
    async fn list_routes(&self) -> Result<Vec<DynamicRoute>, StorageError>;

    /// 获取路由总数
    async fn count_routes(&self) -> Result<usize, StorageError>;

    /// 检查路由是否存在
    async fn route_exists(&self, id: i64) -> Result<bool, StorageError>;

    /// 清空所有路由
    async fn clear_all(&self) -> Result<(), StorageError>;
}

/// 内存路由存储实现
///
/// 使用 HashMap 在内存中存储路由配置，适用于：
/// - 快速原型开发
/// - 测试环境
/// - 需要高性能但不需要持久化的场景
#[derive(Clone)]
pub struct MemoryRouteStorage {
    routes: Arc<RwLock<HashMap<i64, DynamicRoute>>>,
    path_index: Arc<RwLock<HashMap<String, i64>>>, // 路径到ID的映射索引
    #[allow(dead_code)]
    max_routes: usize,
    #[allow(dead_code)]
    cleanup_interval: Duration,
    next_id: Arc<RwLock<i64>>,
}

impl MemoryRouteStorage {
    /// 创建新的内存存储
    ///
    /// # 参数
    /// - `max_routes`: 最大路由数量限制
    /// - `cleanup_interval_secs`: 清理任务间隔（秒）
    pub fn new(max_routes: usize, cleanup_interval_secs: u64) -> Self {
        let storage = Self {
            routes: Arc::new(RwLock::new(HashMap::new())),
            path_index: Arc::new(RwLock::new(HashMap::new())),
            max_routes,
            cleanup_interval: Duration::from_secs(cleanup_interval_secs),
            next_id: Arc::new(RwLock::new(1)),
        };

        // 启动后台清理任务
        storage.start_cleanup_task();

        storage
    }

    /// 启动后台清理任务
    fn start_cleanup_task(&self) {
        let routes = Arc::clone(&self.routes);
        let path_index = Arc::clone(&self.path_index);
        let max_routes = self.max_routes;
        let cleanup_interval = self.cleanup_interval;

        tokio::spawn(async move {
            let mut interval_timer = interval(cleanup_interval);
            loop {
                interval_timer.tick().await;

                // 定期清理已禁用的路由（可选功能）
                if let Ok(mut routes) = routes.write()
                    && let Ok(mut path_index) = path_index.write() {
                        // 清理已禁用的路由
                        let disabled_routes: Vec<i64> = routes
                            .iter()
                            .filter(|(_, route)| !route.enabled)
                            .map(|(id, _)| *id)
                            .collect();

                        for id in disabled_routes {
                            if let Some(route) = routes.remove(&id) {
                                path_index.remove(&route.path);
                            }
                        }

                        // 检查路由数量限制
                        if routes.len() > max_routes {
                            // 按创建时间排序，删除最旧的
                            let mut route_list: Vec<_> = routes.iter().collect();
                            route_list.sort_by_key(|a| a.1.created_at);

                            let to_remove = route_list.len() - max_routes;
                            let ids_to_remove: Vec<i64> = route_list
                                .iter()
                                .take(to_remove)
                                .map(|(id, _)| **id)
                                .collect();
                            for id in ids_to_remove {
                                if let Some(route) = routes.remove(&id) {
                                    path_index.remove(&route.path);
                                }
                            }
                        }
                    }
            }
        });
    }

    /// 生成下一个ID
    #[allow(dead_code)]
    fn next_id(&self) -> i64 {
        let mut id = self.next_id.write().unwrap();
        let current = *id;
        *id += 1;
        current
    }

    /// 获取存储统计信息
    #[allow(dead_code)]
    pub fn get_stats(&self) -> RouteStorageStats {
        let routes = self.routes.read().unwrap();
        RouteStorageStats {
            total_routes: routes.len(),
            enabled_routes: routes.values().filter(|r| r.enabled).count(),
            disabled_routes: routes.values().filter(|r| !r.enabled).count(),
            memory_usage_bytes: std::mem::size_of_val(&*routes),
        }
    }
}

#[async_trait::async_trait]
impl RouteStorage for MemoryRouteStorage {
    async fn save_route(&self, route: &DynamicRoute) -> Result<i64, StorageError> {
        let mut routes = self.routes.write().unwrap();
        let mut path_index = self.path_index.write().unwrap();

        // 检查路由数量限制
        if routes.len() >= self.max_routes && !routes.contains_key(&route.id.unwrap_or(0)) {
            return Err(StorageError::CapacityExceeded(format!(
                "Maximum routes limit {} reached",
                self.max_routes
            )));
        }

        let id = route.id.unwrap_or_else(|| self.next_id());
        let mut route = route.clone();
        route.id = Some(id);

        // 检查路径冲突
        if let Some(&existing_id) = path_index.get(&route.path)
            && existing_id != id {
                return Err(StorageError::AlreadyExists(format!(
                    "Path '{}' already exists with ID {}",
                    route.path, existing_id
                )));
            }

        // 更新路径索引
        if let Some(old_route) = routes.get(&id) {
            path_index.remove(&old_route.path);
        }
        path_index.insert(route.path.clone(), id);

        // 插入路由
        routes.insert(id, route);

        Ok(id)
    }

    async fn load_route(&self, id: i64) -> Result<Option<DynamicRoute>, StorageError> {
        let routes = self.routes.read().unwrap();
        Ok(routes.get(&id).cloned())
    }

    async fn load_route_by_path(&self, path: &str) -> Result<Option<DynamicRoute>, StorageError> {
        let path_index = self.path_index.read().unwrap();
        if let Some(&id) = path_index.get(path) {
            let routes = self.routes.read().unwrap();
            Ok(routes.get(&id).cloned())
        } else {
            Ok(None)
        }
    }

    async fn delete_route(&self, id: i64) -> Result<(), StorageError> {
        let mut routes = self.routes.write().unwrap();
        let mut path_index = self.path_index.write().unwrap();

        if let Some(route) = routes.remove(&id) {
            path_index.remove(&route.path);
        }

        Ok(())
    }

    async fn list_routes(&self) -> Result<Vec<DynamicRoute>, StorageError> {
        let routes = self.routes.read().unwrap();
        Ok(routes.values().cloned().collect())
    }

    async fn count_routes(&self) -> Result<usize, StorageError> {
        let routes = self.routes.read().unwrap();
        Ok(routes.len())
    }

    async fn route_exists(&self, id: i64) -> Result<bool, StorageError> {
        let routes = self.routes.read().unwrap();
        Ok(routes.contains_key(&id))
    }

    async fn clear_all(&self) -> Result<(), StorageError> {
        let mut routes = self.routes.write().unwrap();
        let mut path_index = self.path_index.write().unwrap();
        routes.clear();
        path_index.clear();
        Ok(())
    }
}

/// 文件路由存储实现
///
/// 使用 JSON 文件存储路由配置，适用于：
/// - 配置文件管理
/// - 版本控制集成
/// - 需要可读性和可编辑性的场景
pub struct FileRouteStorage {
    base_dir: PathBuf,
    routes_dir: PathBuf,
    backups_dir: PathBuf,
    #[allow(dead_code)]
    max_file_size: usize, // 字节
    #[allow(dead_code)]
    backup_enabled: bool,
    #[allow(dead_code)]
    backup_count: usize,
}

impl FileRouteStorage {
    /// 创建新的文件存储
    ///
    /// # 参数
    /// - `base_dir`: 基础目录路径
    /// - `max_file_size`: 最大文件大小（字节）
    /// - `backup_enabled`: 是否启用备份
    /// - `backup_count`: 保留的备份数量
    pub fn new(
        base_dir: impl AsRef<Path>,
        max_file_size: usize,
        backup_enabled: bool,
        backup_count: usize,
    ) -> Result<Self, StorageError> {
        let base_path = PathBuf::from(base_dir.as_ref());

        // 创建必要的目录
        let routes_dir = base_path.join("routes");
        let backups_dir = base_path.join("backups");

        fs::create_dir_all(&routes_dir).map_err(|e| {
            StorageError::FileError(format!("Failed to create routes directory: {}", e))
        })?;
        fs::create_dir_all(&backups_dir).map_err(|e| {
            StorageError::FileError(format!("Failed to create backups directory: {}", e))
        })?;

        // 创建 .gitignore 文件，避免提交路由配置到版本控制
        let gitignore_path = base_path.join(".gitignore");
        if !gitignore_path.exists() {
            fs::write(&gitignore_path, "# 路由配置文件\n*.json\n!templates/\n").map_err(|e| {
                StorageError::FileError(format!("Failed to create .gitignore: {}", e))
            })?;
        }

        Ok(Self {
            base_dir: base_path,
            routes_dir,
            backups_dir,
            max_file_size,
            backup_enabled,
            backup_count,
        })
    }

    /// 获取路由文件路径
    #[allow(dead_code)]
    fn get_route_file_path(&self, id: i64) -> PathBuf {
        self.routes_dir.join(format!("route_{}.json", id))
    }

    /// 验证文件路径安全性
    #[allow(dead_code)]
    fn validate_path(&self, path: &Path) -> Result<(), StorageError> {
        // 防止路径遍历攻击
        if path.has_root() || path.starts_with("..") {
            return Err(StorageError::InvalidPath("Invalid path".to_string()));
        }

        // 确保路径在 base_dir 内
        let full_path = self.base_dir.join(path);
        if !full_path.starts_with(&self.base_dir) {
            return Err(StorageError::InvalidPath(
                "Path outside base directory".to_string(),
            ));
        }

        Ok(())
    }

    /// 备份路由文件
    #[allow(dead_code)]
    fn backup_route_file(&self, id: i64) -> Result<PathBuf, StorageError> {
        if !self.backup_enabled {
            return Ok(PathBuf::new());
        }

        let source_path = self.get_route_file_path(id);
        if !source_path.exists() {
            return Ok(PathBuf::new());
        }

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_path = self
            .backups_dir
            .join(format!("route_{}_backup_{}.json", id, timestamp));

        fs::copy(&source_path, &backup_path)
            .map_err(|e| StorageError::FileError(format!("Failed to backup route: {}", e)))?;

        // 清理过期备份
        self.cleanup_old_backups(id)?;

        Ok(backup_path)
    }

    /// 清理过期备份文件
    #[allow(dead_code)]
    fn cleanup_old_backups(&self, route_id: i64) -> Result<(), StorageError> {
        if !self.backup_enabled {
            return Ok(());
        }

        let mut backup_files = Vec::new();

        // 收集该路由的所有备份文件
        for entry in fs::read_dir(&self.backups_dir).map_err(|e| {
            StorageError::FileError(format!("Failed to read backups directory: {}", e))
        })? {
            let entry = entry.map_err(|e| {
                StorageError::FileError(format!("Failed to read directory entry: {}", e))
            })?;
            let path = entry.path();

            if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                // 解析文件名：route_123_backup_20260317_100000.json
                let pattern = format!("route_{}_backup_", route_id);
                if file_name.starts_with(&pattern) && file_name.ends_with(".json")
                    && let Ok(metadata) = entry.metadata()
                        && let Ok(modified) = metadata.modified() {
                            backup_files.push((path, modified));
                        }
            }
        }

        // 按修改时间排序（最新的在前）
        backup_files.sort_by_key(|b| std::cmp::Reverse(b.1));

        // 删除多余的备份
        for (path, _) in backup_files.iter().skip(self.backup_count) {
            fs::remove_file(path).map_err(|e| {
                StorageError::FileError(format!("Failed to remove old backup: {}", e))
            })?;
        }

        Ok(())
    }

    /// 生成下一个可用ID
    #[allow(dead_code)]
    fn next_id(&self) -> Result<i64, StorageError> {
        let mut max_id = 0i64;

        for entry in fs::read_dir(&self.routes_dir).map_err(|e| {
            StorageError::FileError(format!("Failed to read routes directory: {}", e))
        })? {
            let entry = entry.map_err(|e| {
                StorageError::FileError(format!("Failed to read directory entry: {}", e))
            })?;
            let path = entry.path();

            if let Some(file_name) = path.file_stem().and_then(|s| s.to_str())
                && let Some(id_str) = file_name.strip_prefix("route_")
                    && let Ok(id) = id_str.parse::<i64>() {
                        max_id = max_id.max(id);
                    }
        }

        Ok(max_id + 1)
    }

    /// 获取存储统计信息
    #[allow(dead_code)]
    pub fn get_stats(&self) -> Result<RouteStorageStats, StorageError> {
        let mut total = 0;
        let mut enabled = 0;
        let mut disabled = 0;
        let mut total_size = 0;

        for entry in fs::read_dir(&self.routes_dir).map_err(|e| {
            StorageError::FileError(format!("Failed to read routes directory: {}", e))
        })? {
            let entry = entry.map_err(|e| {
                StorageError::FileError(format!("Failed to read directory entry: {}", e))
            })?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len() as usize;
                }

                // 读取文件以统计启用状态
                if let Ok(content) = fs::read_to_string(&path)
                    && let Ok(route) = serde_json::from_str::<DynamicRoute>(&content) {
                        total += 1;
                        if route.enabled {
                            enabled += 1;
                        } else {
                            disabled += 1;
                        }
                    }
            }
        }

        Ok(RouteStorageStats {
            total_routes: total,
            enabled_routes: enabled,
            disabled_routes: disabled,
            memory_usage_bytes: total_size,
        })
    }
}

#[async_trait::async_trait]
impl RouteStorage for FileRouteStorage {
    async fn save_route(&self, route: &DynamicRoute) -> Result<i64, StorageError> {
        let id = if let Some(id) = route.id {
            id
        } else {
            self.next_id()?
        };
        let mut route = route.clone();
        route.id = Some(id);

        let file_path = self.get_route_file_path(id);

        // 验证文件路径
        self.validate_path(file_path.strip_prefix(&self.base_dir).unwrap())?;

        // 如果文件已存在，先备份
        if file_path.exists() && self.backup_enabled {
            self.backup_route_file(id)?;
        }

        // 序列化为 JSON
        let content = serde_json::to_string_pretty(&route)?;

        // 检查文件大小限制
        if content.len() > self.max_file_size {
            return Err(StorageError::CapacityExceeded(format!(
                "File size {} exceeds limit {}",
                content.len(),
                self.max_file_size
            )));
        }

        // 写入文件
        fs::write(&file_path, content)
            .map_err(|e| StorageError::FileError(format!("Failed to write route file: {}", e)))?;

        Ok(id)
    }

    async fn load_route(&self, id: i64) -> Result<Option<DynamicRoute>, StorageError> {
        let file_path = self.get_route_file_path(id);

        if !file_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&file_path)
            .map_err(|e| StorageError::FileError(format!("Failed to read route file: {}", e)))?;

        let route: DynamicRoute = serde_json::from_str(&content)?;

        Ok(Some(route))
    }

    async fn load_route_by_path(&self, path: &str) -> Result<Option<DynamicRoute>, StorageError> {
        // 遍历所有路由文件，查找匹配路径的路由
        for entry in fs::read_dir(&self.routes_dir).map_err(|e| {
            StorageError::FileError(format!("Failed to read routes directory: {}", e))
        })? {
            let entry = entry.map_err(|e| {
                StorageError::FileError(format!("Failed to read directory entry: {}", e))
            })?;
            let file_path = entry.path();

            if file_path.extension().and_then(|s| s.to_str()) == Some("json")
                && let Ok(content) = fs::read_to_string(&file_path)
                    && let Ok(route) = serde_json::from_str::<DynamicRoute>(&content)
                        && route.path == path {
                            return Ok(Some(route));
                        }
        }

        Ok(None)
    }

    async fn delete_route(&self, id: i64) -> Result<(), StorageError> {
        let file_path = self.get_route_file_path(id);

        if file_path.exists() {
            // 删除前先备份
            if self.backup_enabled {
                self.backup_route_file(id)?;
            }

            fs::remove_file(&file_path).map_err(|e| {
                StorageError::FileError(format!("Failed to delete route file: {}", e))
            })?;
        }

        Ok(())
    }

    async fn list_routes(&self) -> Result<Vec<DynamicRoute>, StorageError> {
        let mut routes = Vec::new();

        for entry in fs::read_dir(&self.routes_dir).map_err(|e| {
            StorageError::FileError(format!("Failed to read routes directory: {}", e))
        })? {
            let entry = entry.map_err(|e| {
                StorageError::FileError(format!("Failed to read directory entry: {}", e))
            })?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(&path).map_err(|e| {
                    StorageError::FileError(format!("Failed to read route file: {}", e))
                })?;

                let route: DynamicRoute = serde_json::from_str(&content)?;
                routes.push(route);
            }
        }

        // 按 ID 排序
        routes.sort_by_key(|a| a.id);

        Ok(routes)
    }

    async fn count_routes(&self) -> Result<usize, StorageError> {
        let count = fs::read_dir(&self.routes_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("json"))
            .count();

        Ok(count)
    }

    async fn route_exists(&self, id: i64) -> Result<bool, StorageError> {
        let file_path = self.get_route_file_path(id);
        Ok(file_path.exists())
    }

    async fn clear_all(&self) -> Result<(), StorageError> {
        for entry in fs::read_dir(&self.routes_dir)?.filter_map(|entry| entry.ok()) {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                fs::remove_file(&path)?;
            }
        }

        Ok(())
    }
}

/// 路由存储统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct RouteStorageStats {
    /// 总路由数
    pub total_routes: usize,
    /// 启用的路由数
    pub enabled_routes: usize,
    /// 禁用的路由数
    pub disabled_routes: usize,
    /// 内存使用量（字节）
    pub memory_usage_bytes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::{DynamicRoute, HandlerType, RouteType};
    use chrono::Utc;
    use serde_json::json;

    #[tokio::test]
    async fn test_memory_storage() {
        let storage = MemoryRouteStorage::new(100, 60);

        // 测试保存路由
        let route = DynamicRoute {
            id: None,
            route_name: Some("测试路由".to_string()),
            route_type: RouteType::Memory,
            path: "/test".to_string(),
            handler_type: HandlerType::Static,
            handler_config: serde_json::json!({"content": "test"}),
            inline_template: Some("test content".to_string()),
            template_path: None,
            content_type_hint: Some("text/plain".to_string()),
            enabled: true,
            priority: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: Some("test_user".to_string()),
            group_id: None,
            is_primary_entry: None,
            metadata: None,
        };

        let id = storage.save_route(&route).await.unwrap();
        assert!(id > 0);

        // 测试加载路由
        let loaded = storage.load_route(id).await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().path, "/test");

        // 测试删除路由
        storage.delete_route(id).await.unwrap();
        let deleted = storage.load_route(id).await.unwrap();
        assert!(deleted.is_none());
    }

    #[test]
    fn test_file_storage() {
        let temp_dir = std::env::temp_dir().join("rustblog_test_routes");
        let _ = fs::remove_dir_all(&temp_dir); // 清理之前的测试

        let _storage = FileRouteStorage::new(&temp_dir, 1024 * 1024, true, 3).unwrap();

        // 测试目录创建
        assert!(temp_dir.exists());
        assert!(temp_dir.join("routes").exists());
        assert!(temp_dir.join("backups").exists());
        assert!(temp_dir.join(".gitignore").exists());

        // 清理
        let _ = fs::remove_dir_all(&temp_dir);
    }

    /// 测试内存存储的基本 CRUD 操作
    #[tokio::test]
    async fn test_memory_storage_crud() {
        let storage = MemoryRouteStorage::new(100, 60);

        // 创建路由
        let route = DynamicRoute {
            id: None,
            route_name: Some("测试路由".to_string()),
            route_type: RouteType::Memory,
            path: "/test/memory".to_string(),
            handler_type: HandlerType::Static,
            handler_config: json!({"content": "test content"}),
            inline_template: Some("test content".to_string()),
            template_path: None,
            content_type_hint: Some("text/plain".to_string()),
            enabled: true,
            priority: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: Some("test_user".to_string()),
            group_id: None,
            is_primary_entry: None,
            metadata: None,
        };

        // 保存路由
        let id = storage.save_route(&route).await.unwrap();
        assert!(id > 0);

        // 加载路由
        let loaded = storage.load_route(id).await.unwrap();
        assert!(loaded.is_some());
        let loaded_route = loaded.unwrap();
        assert_eq!(loaded_route.path, "/test/memory");
        assert_eq!(loaded_route.route_type, RouteType::Memory);

        // 根据路径加载路由
        let loaded_by_path = storage.load_route_by_path("/test/memory").await.unwrap();
        assert!(loaded_by_path.is_some());
        assert_eq!(loaded_by_path.unwrap().id.unwrap(), id);

        // 检查路由是否存在
        assert!(storage.route_exists(id).await.unwrap());

        // 列出所有路由
        let routes = storage.list_routes().await.unwrap();
        assert_eq!(routes.len(), 1);

        // 删除路由
        storage.delete_route(id).await.unwrap();
        assert!(!storage.route_exists(id).await.unwrap());
    }

    /// 测试内存存储的路径冲突检测
    #[tokio::test]
    async fn test_memory_storage_path_conflict() {
        let storage = MemoryRouteStorage::new(100, 60);

        let route1 = DynamicRoute {
            id: None,
            route_name: Some("路由1".to_string()),
            route_type: RouteType::Memory,
            path: "/test/conflict".to_string(),
            handler_type: HandlerType::Static,
            handler_config: json!({"content": "route1"}),
            inline_template: Some("route1".to_string()),
            template_path: None,
            content_type_hint: Some("text/plain".to_string()),
            enabled: true,
            priority: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: Some("test_user".to_string()),
            group_id: None,
            is_primary_entry: None,
            metadata: None,
        };

        let route2 = DynamicRoute {
            id: None,
            route_name: Some("路由2".to_string()),
            route_type: RouteType::Memory,
            path: "/test/conflict".to_string(), // 相同的路径
            handler_type: HandlerType::Static,
            handler_config: json!({"content": "route2"}),
            inline_template: Some("route2".to_string()),
            template_path: None,
            content_type_hint: Some("text/plain".to_string()),
            enabled: true,
            priority: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: Some("test_user".to_string()),
            group_id: None,
            is_primary_entry: None,
            metadata: None,
        };

        // 保存第一个路由
        let id1 = storage.save_route(&route1).await.unwrap();

        // 尝试保存相同路径的路由，应该失败
        let result = storage.save_route(&route2).await;
        assert!(result.is_err());

        // 清理
        storage.delete_route(id1).await.unwrap();
    }

    /// 测试内存存储的容量限制
    #[tokio::test]
    async fn test_memory_storage_capacity_limit() {
        let storage = MemoryRouteStorage::new(3, 60); // 最大 3 个路由

        // 创建 3 个路由
        for i in 1..=3 {
            let route = DynamicRoute {
                id: None,
                route_name: Some(format!("路由{}", i)),
                route_type: RouteType::Memory,
                path: format!("/test/capacity/{}", i),
                handler_type: HandlerType::Static,
                handler_config: json!({"content": format!("content{}", i)}),
                inline_template: Some(format!("content{}", i)),
                template_path: None,
                content_type_hint: Some("text/plain".to_string()),
                enabled: true,
                priority: 0,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                created_by: Some("test_user".to_string()),
                group_id: None,
                is_primary_entry: None,
                metadata: None,
            };
            storage.save_route(&route).await.unwrap();
        }

        // 尝试创建第 4 个路由，应该失败
        let route4 = DynamicRoute {
            id: None,
            route_name: Some("路由4".to_string()),
            route_type: RouteType::Memory,
            path: "/test/capacity/4".to_string(),
            handler_type: HandlerType::Static,
            handler_config: json!({"content": "content4"}),
            inline_template: Some("content4".to_string()),
            template_path: None,
            content_type_hint: Some("text/plain".to_string()),
            enabled: true,
            priority: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: Some("test_user".to_string()),
            group_id: None,
            is_primary_entry: None,
            metadata: None,
        };

        let result = storage.save_route(&route4).await;
        assert!(result.is_err());

        // 清理
        storage.clear_all().await.unwrap();
    }

    /// 测试路由类型枚举
    #[test]
    fn test_route_type_enum() {
        // 测试 Display
        assert_eq!(RouteType::Database.to_string(), "database");
        assert_eq!(RouteType::Memory.to_string(), "memory");
        assert_eq!(RouteType::File.to_string(), "file");

        // 测试 FromStr
        assert_eq!(RouteType::from_str("database"), Some(RouteType::Database));
        assert_eq!(RouteType::from_str("memory"), Some(RouteType::Memory));
        assert_eq!(RouteType::from_str("file"), Some(RouteType::File));
        assert_eq!(RouteType::from_str("invalid"), None);

        // 测试 AsRef
        assert_eq!(RouteType::Database.as_ref(), "database");
        assert_eq!(RouteType::Memory.as_ref(), "memory");
        assert_eq!(RouteType::File.as_ref(), "file");
    }

    /// 测试存储错误类型
    #[test]
    fn test_storage_error() {
        let error = StorageError::NotFound("test".to_string());
        assert!(error.to_string().contains("未找到"));

        let error = StorageError::CapacityExceeded("test".to_string());
        assert!(error.to_string().contains("容量超出"));

        let error = StorageError::InvalidPath("test".to_string());
        assert!(error.to_string().contains("无效路径"));
    }

    /// 测试存储统计信息
    #[test]
    fn test_storage_stats() {
        let stats = RouteStorageStats {
            total_routes: 100,
            enabled_routes: 80,
            disabled_routes: 20,
            memory_usage_bytes: 1024000,
        };

        assert_eq!(stats.total_routes, 100);
        assert_eq!(stats.enabled_routes, 80);
        assert_eq!(stats.disabled_routes, 20);
        assert_eq!(stats.memory_usage_bytes, 1024000);
    }
}
