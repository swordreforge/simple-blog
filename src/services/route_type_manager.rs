//! 路由类型管理器
//!
//! 统一管理三种路由存储类型：
//! - DatabaseRouteStorage: 数据库存储
//! - MemoryRouteStorage: 内存存储
//! - FileRouteStorage: 文件存储
//!
//! 提供路由类型切换、迁移和统一访问接口

use std::sync::Arc;
use crate::db::models::{DynamicRoute, RouteType};
use crate::services::route_storage::{RouteStorage, StorageError};
use crate::db::repositories::DynamicRouteRepository;
use crate::services::route_storage::{MemoryRouteStorage, FileRouteStorage};

/// 路由类型管理器
///
/// 管理三种存储类型，提供统一的访问接口和类型转换功能
#[derive(Clone)]
pub struct RouteTypeManager {
    #[allow(dead_code)]
    database_storage: Arc<DynamicRouteRepository>,
    #[allow(dead_code)]
    memory_storage: Arc<MemoryRouteStorage>,
    #[allow(dead_code)]
    file_storage: Arc<FileRouteStorage>,
    #[allow(dead_code)]
    default_type: RouteType,
}

impl RouteTypeManager {
    /// 创建新的路由类型管理器
    ///
    /// # 参数
    /// - `database_storage`: 数据库存储实现
    /// - `memory_storage`: 内存存储实现
    /// - `file_storage`: 文件存储实现
    /// - `default_type`: 默认存储类型
    pub fn new(
        database_storage: Arc<DynamicRouteRepository>,
        memory_storage: Arc<MemoryRouteStorage>,
        file_storage: Arc<FileRouteStorage>,
        default_type: RouteType,
    ) -> Self {
        Self {
            database_storage,
            memory_storage,
            file_storage,
            default_type,
        }
    }

    /// 获取指定类型的存储
    pub fn get_storage(&self, route_type: &RouteType) -> Arc<dyn RouteStorage> {
        match route_type {
            RouteType::Database => Arc::clone(&self.database_storage) as Arc<dyn RouteStorage>,
            RouteType::Memory => Arc::clone(&self.memory_storage) as Arc<dyn RouteStorage>,
            RouteType::File => Arc::clone(&self.file_storage) as Arc<dyn RouteStorage>,
        }
    }

    /// 保存路由
    ///
    /// 如果路由未指定类型，使用默认类型
    #[allow(dead_code)]
    pub async fn save_route(&self, route: DynamicRoute) -> Result<i64, StorageError> {
        // 路由类型，如果未指定则使用默认类型
        let route_type = route.route_type;

        // 根据路由类型选择存储
        let storage = self.get_storage(&route_type);

        // 保存路由
        let id = storage.save_route(&route).await?;

        Ok(id)
    }

    /// 加载路由
    ///
    /// 如果指定了路由类型，从指定类型加载；
    /// 否则依次从各存储类型查找
    pub async fn load_route(
        &self,
        id: i64,
        route_type: Option<RouteType>,
    ) -> Result<Option<DynamicRoute>, StorageError> {
        // 如果指定了路由类型，从指定类型加载
        if let Some(route_type) = route_type {
            let storage = self.get_storage(&route_type);
            return storage.load_route(id).await;
        }

        // 否则依次从各存储类型查找
        for storage_type in [RouteType::Database, RouteType::Memory, RouteType::File] {
            let storage = self.get_storage(&storage_type);
            if let Some(route) = storage.load_route(id).await? {
                return Ok(Some(route));
            }
        }

        Ok(None)
    }

    /// 根据路径加载路由
    ///
    /// 如果指定了路由类型，从指定类型加载；
    /// 否则依次从各存储类型查找
    #[allow(dead_code)]
    pub async fn load_route_by_path(
        &self,
        path: &str,
        route_type: Option<RouteType>,
    ) -> Result<Option<DynamicRoute>, StorageError> {
        // 如果指定了路由类型，从指定类型加载
        if let Some(route_type) = route_type {
            let storage = self.get_storage(&route_type);
            return storage.load_route_by_path(path).await;
        }

        // 否则依次从各存储类型查找
        for storage_type in [RouteType::Database, RouteType::Memory, RouteType::File] {
            let storage = self.get_storage(&storage_type);
            if let Some(route) = storage.load_route_by_path(path).await? {
                return Ok(Some(route));
            }
        }

        Ok(None)
    }

    /// 删除路由
    ///
    /// 如果指定了路由类型，从指定类型删除；
    /// 否则从所有存储类型中删除
    #[allow(dead_code)]
    pub async fn delete_route(
        &self,
        id: i64,
        route_type: Option<RouteType>,
    ) -> Result<(), StorageError> {
        // 如果指定了路由类型，从指定类型删除
        if let Some(route_type) = route_type {
            let storage = self.get_storage(&route_type);
            return storage.delete_route(id).await;
        }

        // 否则从所有存储类型中删除
        for storage_type in [RouteType::Database, RouteType::Memory, RouteType::File] {
            let storage = self.get_storage(&storage_type);
            let _ = storage.delete_route(id).await; // 忽略错误，可能路由不存在于该存储中
        }

        Ok(())
    }

    /// 列出所有路由
    ///
    /// 合并所有存储类型中的路由
    #[allow(dead_code)]
    pub async fn list_all_routes(&self) -> Result<Vec<DynamicRoute>, StorageError> {
        let mut all_routes = Vec::new();

        // 从数据库加载
        if let Ok(routes) = self.database_storage.get_all().await {
            all_routes.extend(routes);
        }

        // 从内存加载
        if let Ok(routes) = self.memory_storage.list_routes().await {
            all_routes.extend(routes);
        }

        // 从文件加载
        if let Ok(routes) = self.file_storage.list_routes().await {
            all_routes.extend(routes);
        }

        // 按 ID 排序
        all_routes.sort_by_key(|a| a.id);

        Ok(all_routes)
    }

    /// 列出指定类型的路由
    #[allow(dead_code)]
    pub async fn list_routes_by_type(
        &self,
        route_type: RouteType,
    ) -> Result<Vec<DynamicRoute>, StorageError> {
        let storage = self.get_storage(&route_type);
        storage.list_routes().await
    }

    /// 获取所有启用的路由
    ///
    /// 合并所有存储类型中启用的路由
    #[allow(dead_code)]
    pub async fn get_all_enabled(&self) -> Result<Vec<DynamicRoute>, StorageError> {
        let all_routes = self.list_all_routes().await?;
        let enabled_routes: Vec<DynamicRoute> = all_routes
            .into_iter()
            .filter(|route| route.enabled)
            .collect();

        Ok(enabled_routes)
    }

    /// 迁移路由
    ///
    /// 将路由从一种存储类型迁移到另一种存储类型
    pub async fn migrate_route(
        &self,
        id: i64,
        from_type: RouteType,
        to_type: RouteType,
    ) -> Result<(), StorageError> {
        if from_type == to_type {
            return Err(StorageError::InvalidData(
                "Source and destination types are the same".to_string(),
            ));
        }

        // 从源存储加载
        let from_storage = self.get_storage(&from_type);
        let mut route = from_storage
            .load_route(id)
            .await?
            .ok_or_else(|| {
                StorageError::NotFound(format!(
                    "Route {} not found in {} storage",
                    id, from_type
                ))
            })?;

        // 字段转换逻辑
        match (from_type, to_type) {
            // file -> database/memory: 读取文件内容转换为 inline_template
            (RouteType::File, RouteType::Database) |
            (RouteType::File, RouteType::Memory) => {
                let template_path = route.template_path.clone();
                if let Some(ref path) = template_path {
                    // 读取模板文件内容
                    let content = std::fs::read_to_string(path)
                        .map_err(|e| StorageError::FileError(
                            format!("Failed to read template file {}: {}", path, e)
                        ))?;

                    // 设置 inline_template
                    route.inline_template = Some(content);
                    // 清除 template_path（database/memory 类型不需要）
                    route.template_path = None;

                    tracing::info!(
                        "迁移路由 {}: 读取文件 {} 内容到 inline_template",
                        id,
                        path
                    );
                } else {
                    return Err(StorageError::InvalidData(
                        "File type route missing template_path".to_string()
                    ));
                }
            }

            // database/memory -> file: 将 inline_template 写入文件
            (RouteType::Database, RouteType::File) |
            (RouteType::Memory, RouteType::File) => {
                if let Some(ref inline_template) = route.inline_template {
                    // 确定文件路径
                    let template_path = if let Some(ref path) = route.template_path {
                        path.clone()
                    } else {
                        // 生成默认路径（存储在 data/routes/routes 目录）
                        format!("data/routes/routes/route_{}.html", route.id.unwrap_or(0))
                    };

                    // 确保目录存在
                    if let Some(parent_dir) = std::path::Path::new(&template_path).parent() {
                        std::fs::create_dir_all(parent_dir)
                            .map_err(|e| StorageError::FileError(
                                format!("Failed to create directory {}: {}", parent_dir.display(), e)
                            ))?;
                    }

                    // 写入文件
                    std::fs::write(&template_path, inline_template)
                        .map_err(|e| StorageError::FileError(
                            format!("Failed to write template file {}: {}", template_path, e)
                        ))?;

                    // 设置 template_path
                    route.template_path = Some(template_path);
                    // 清除 inline_template（file 类型不需要）
                    route.inline_template = None;

                    tracing::info!(
                        "迁移路由 {}: 将 inline_template 写入文件 {}",
                        id,
                        route.template_path.as_ref().unwrap()
                    );
                } else {
                    return Err(StorageError::InvalidData(
                        "Cannot migrate route without inline_template to file type".to_string()
                    ));
                }
            }

            // database <-> memory: 直接迁移，无需字段转换
            (RouteType::Database, RouteType::Memory) |
            (RouteType::Memory, RouteType::Database) => {
                tracing::info!("迁移路由 {}: database <-> memory 无需字段转换", id);
            }

            _ => {
                return Err(StorageError::InvalidData(
                    format!("Unsupported migration: {:?} -> {:?}", from_type, to_type)
                ));
            }
        }

        // 更新路由类型
        route.route_type = to_type;

        // 保存到数据库（所有路由都存储在数据库中）
        self.database_storage.update(id, &route).await
            .map_err(|e| StorageError::DatabaseError(format!("Failed to update route: {}", e)))?;

        // 如果迁移到 memory，同时加载到内存存储
        if to_type == RouteType::Memory {
            let memory_storage = self.get_storage(&RouteType::Memory);
            memory_storage.save_route(&route).await?;
            tracing::info!("迁移路由 {}: 已加载到内存存储", id);
        }

        // 如果从 memory 迁移走，从内存存储删除
        if from_type == RouteType::Memory {
            let memory_storage = self.get_storage(&RouteType::Memory);
            memory_storage.delete_route(id).await?;
            tracing::info!("迁移路由 {}: 已从内存存储删除", id);
        }

        tracing::info!("成功迁移路由 {} 从 {:?} 到 {:?}", id, from_type, to_type);

        Ok(())
    }

    /// 批量迁移路由
    ///
    /// 将所有路由从一种存储类型迁移到另一种存储类型
    pub async fn migrate_all_routes(
        &self,
        from_type: RouteType,
        to_type: RouteType,
    ) -> Result<usize, StorageError> {
        if from_type == to_type {
            return Err(StorageError::InvalidData(
                "Source and destination types are the same".to_string(),
            ));
        }

        let from_storage = self.get_storage(&from_type);
        let routes = from_storage.list_routes().await?;

        let mut migrated_count = 0;

        for route in routes {
            let id = route.id.ok_or_else(|| {
                StorageError::InvalidData("Route ID is required for migration".to_string())
            })?;

            if self.migrate_route(id, from_type, to_type).await.is_ok() {
                migrated_count += 1;
            }
        }

        Ok(migrated_count)
    }

    /// 检查路由是否存在
    ///
    /// 在所有存储类型中检查
    #[allow(dead_code)]
    pub async fn route_exists(&self, id: i64) -> Result<bool, StorageError> {
        for storage_type in [RouteType::Database, RouteType::Memory, RouteType::File] {
            let storage = self.get_storage(&storage_type);
            if storage.route_exists(id).await? {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// 获取路由总数
    ///
    /// 统计所有存储类型中的路由总数
    #[allow(dead_code)]
    pub async fn count_all_routes(&self) -> Result<usize, StorageError> {
        let mut total = 0;

        for storage_type in [RouteType::Database, RouteType::Memory, RouteType::File] {
            let storage = self.get_storage(&storage_type);
            total += storage.count_routes().await?;
        }

        Ok(total)
    }

    /// 获取存储统计信息
    ///
    /// 返回各存储类型的统计信息
    pub async fn get_storage_stats(
        &self,
    ) -> Result<StorageStatsSummary, StorageError> {
        // 从数据库中按类型分别统计路由
        let db_routes = self.database_storage.list(0, 0, Some(RouteType::Database), None).await
            .map_err(|e| StorageError::DatabaseError(format!("Failed to list database routes: {}", e)))?.1;
        let db_enabled = self.database_storage.list(0, 0, Some(RouteType::Database), Some(true)).await
            .map_err(|e| StorageError::DatabaseError(format!("Failed to list enabled database routes: {}", e)))?.1;

        let file_routes = self.database_storage.list(0, 0, Some(RouteType::File), None).await
            .map_err(|e| StorageError::DatabaseError(format!("Failed to list file routes: {}", e)))?.1;
        let file_enabled = self.database_storage.list(0, 0, Some(RouteType::File), Some(true)).await
            .map_err(|e| StorageError::DatabaseError(format!("Failed to list enabled file routes: {}", e)))?.1;

        // 数据库中的 memory 类型路由
        let memory_db_routes = self.database_storage.list(0, 0, Some(RouteType::Memory), None).await
            .map_err(|e| StorageError::DatabaseError(format!("Failed to list memory routes: {}", e)))?.1;
        let memory_db_enabled = self.database_storage.list(0, 0, Some(RouteType::Memory), Some(true)).await
            .map_err(|e| StorageError::DatabaseError(format!("Failed to list enabled memory routes: {}", e)))?.1;

        // 内存统计（真正的内存存储）
        let memory_stats = self.memory_storage.get_stats();

        // 文件系统统计（真正的文件存储）
        let file_fs_stats = self.file_storage.get_stats()?;

        // 组合file统计：数据库中的file类型路由 + 文件系统中的路由
        let total_file_routes = file_routes as usize + file_fs_stats.total_routes;
        let total_file_enabled = file_enabled as usize + file_fs_stats.enabled_routes;
        let total_file_disabled = total_file_routes - total_file_enabled;

        // 组合memory统计：数据库中的memory类型路由 + 真正的内存存储
        let total_memory_routes = memory_db_routes as usize + memory_stats.total_routes;
        let total_memory_enabled = memory_db_enabled as usize + memory_stats.enabled_routes;
        let total_memory_disabled = total_memory_routes - total_memory_enabled;

        Ok(StorageStatsSummary {
            database: StorageStats {
                total_routes: db_routes as usize,
                enabled_routes: db_enabled as usize,
                disabled_routes: (db_routes - db_enabled) as usize,
                memory_usage_bytes: 0, // 数据库不使用内存
            },
            memory: StorageStats {
                total_routes: total_memory_routes,
                enabled_routes: total_memory_enabled,
                disabled_routes: total_memory_disabled,
                memory_usage_bytes: memory_stats.memory_usage_bytes,
            },
            file: StorageStats {
                total_routes: total_file_routes,
                enabled_routes: total_file_enabled,
                disabled_routes: total_file_disabled,
                memory_usage_bytes: file_fs_stats.memory_usage_bytes,
            },
        })
    }

    /// 从指定存储类型加载所有路由
    pub async fn load_all_routes_from_storage(
        &self,
        route_type: RouteType,
    ) -> Result<Vec<DynamicRoute>, StorageError> {
        let storage = self.get_storage(&route_type);
        let routes = storage.list_routes().await?;

        Ok(routes)
    }

    /// 清空指定类型的所有路由
    pub async fn clear_storage(&self, route_type: RouteType) -> Result<(), StorageError> {
        // 获取要清空的路由数量
        let storage = self.get_storage(&route_type);
        let routes = storage.list_routes().await?;
        let route_count = routes.len();

        if route_count == 0 {
            tracing::info!("清空 {:?} 存储：无路由需要清空", route_type);
            return Ok(());
        }

        tracing::info!(
            "准备清空 {} 个 {:?} 类型的路由",
            route_count,
            route_type
        );

        // 类型专有处理
        match route_type {
            RouteType::Memory => {
                // 1. 从数据库删除所有 memory 类型的路由
                let deleted_count = self.database_storage
                    .delete_by_type(RouteType::Memory)
                    .await
                    .map_err(|e| StorageError::DatabaseError(format!("Failed to delete memory routes: {}", e)))?;

                // 2. 清空内存存储
                self.memory_storage.clear_all().await?;

                tracing::info!(
                    "成功清空 memory 存储：从数据库删除 {} 条记录，清空内存缓存",
                    deleted_count
                );
            }

            RouteType::File => {
                // 1. 从数据库删除所有 file 类型的路由
                let deleted_count = self.database_storage
                    .delete_by_type(RouteType::File)
                    .await
                    .map_err(|e| StorageError::DatabaseError(format!("Failed to delete file routes: {}", e)))?;

                // 2. 清空文件存储
                self.file_storage.clear_all().await?;

                // 注意：不删除实际的模板文件，保留作为备份
                tracing::info!(
                    "成功清空 file 存储：从数据库删除 {} 条记录，清空文件存储（模板文件已保留）",
                    deleted_count
                );
            }

            RouteType::Database => {
                // 1. 从数据库删除所有 database 类型的路由
                let deleted_count = self.database_storage
                    .delete_by_type(RouteType::Database)
                    .await
                    .map_err(|e| StorageError::DatabaseError(format!("Failed to delete database routes: {}", e)))?;

                tracing::info!(
                    "成功清空 database 存储：从数据库删除 {} 条记录",
                    deleted_count
                );
            }
        }

        tracing::info!("成功清空 {:?} 存储的所有路由", route_type);
        Ok(())
    }

    /// 清空所有存储
    #[allow(dead_code)]
    pub async fn clear_all_storages(&self) -> Result<(), StorageError> {
        for storage_type in [RouteType::Database, RouteType::Memory, RouteType::File] {
            self.clear_storage(storage_type).await?;
        }

        Ok(())
    }
}

/// 存储统计信息摘要
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StorageStatsSummary {
    /// 数据库统计
    pub database: StorageStats,
    /// 内存统计
    pub memory: StorageStats,
    /// 文件统计
    pub file: StorageStats,
}

/// 存储统计信息
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StorageStats {
    /// 总路由数
    pub total_routes: usize,
    /// 启用的路由数
    pub enabled_routes: usize,
    /// 禁用的路由数
    pub disabled_routes: usize,
    /// 内存使用量（字节）
    pub memory_usage_bytes: usize,
}

/// 为 DynamicRouteRepository 实现 RouteStorage trait
#[async_trait::async_trait]
impl RouteStorage for DynamicRouteRepository {
    async fn save_route(&self, route: &DynamicRoute) -> Result<i64, StorageError> {
        // 如果路由已有 ID，则更新；否则创建新路由
        if let Some(id) = route.id {
            self.update(id, route)
                .await
                .map(|_| id)
                .map_err(|e| StorageError::DatabaseError(e.to_string()))
        } else {
            self.create(route)
                .await
                .map_err(|e| StorageError::DatabaseError(e.to_string()))
        }
    }

    async fn load_route(&self, id: i64) -> Result<Option<DynamicRoute>, StorageError> {
        self.get_by_id(id)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))
    }

    async fn load_route_by_path(
        &self,
        path: &str,
    ) -> Result<Option<DynamicRoute>, StorageError> {
        self.get_by_path(path)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))
    }

    async fn delete_route(&self, id: i64) -> Result<(), StorageError> {
        self.delete(id)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))
    }

    async fn list_routes(&self) -> Result<Vec<DynamicRoute>, StorageError> {
        self.get_all()
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))
    }

    async fn count_routes(&self) -> Result<usize, StorageError> {
        let count = self
            .count()
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        Ok(count as usize)
    }

    async fn route_exists(&self, id: i64) -> Result<bool, StorageError> {
        let route = self
            .get_by_id(id)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        Ok(route.is_some())
    }

    async fn clear_all(&self) -> Result<(), StorageError> {
        self.delete_all()
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_type_display() {
        assert_eq!(RouteType::Database.to_string(), "database");
        assert_eq!(RouteType::Memory.to_string(), "memory");
        assert_eq!(RouteType::File.to_string(), "file");
    }

    #[test]
    fn test_route_type_from_str() {
        assert_eq!(RouteType::from_str("database"), Some(RouteType::Database));
        assert_eq!(RouteType::from_str("memory"), Some(RouteType::Memory));
        assert_eq!(RouteType::from_str("file"), Some(RouteType::File));
        assert_eq!(RouteType::from_str("invalid"), None);
    }
}