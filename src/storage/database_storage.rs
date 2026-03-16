//! 数据库持久化存储实现
//!
//! 支持 SQLite 和 PostgreSQL 数据库，提供路由持久化和版本控制功能。

#![cfg(feature = "database")]

use crate::core::route_entry::RouteEntry;
use crate::core::SerializableRoute;
use crate::storage::traits::{KeyValueStorage, RouteStorage};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::pool::PoolOptions;
use sqlx::{Pool, Row, SqlitePool, Postgres};
use std::collections::HashMap;
use std::error::Error;
use thiserror::Error;

/// 数据库配置错误
#[derive(Error, Debug)]
pub enum DatabaseStorageError {
    #[error("Database connection error: {0}")]
    ConnectionError(String),
    #[error("Query execution error: {0}")]
    QueryError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Database not initialized")]
    NotInitialized,
    #[error("Route not found: {0}")]
    RouteNotFound(String),
    #[error("Invalid database URL: {0}")]
    InvalidDatabaseUrl(String),
}

/// 数据库类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseType {
    SQLite,
    PostgreSQL,
}

/// 路由版本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteVersion {
    pub version: i64,
    pub route_path: String,
    pub route_type: String,
    pub body: String,
    pub content_type: String,
    pub extra_data: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
}

/// 数据库存储配置
#[derive(Debug, Clone)]
pub struct DatabaseStorageConfig {
    pub database_type: DatabaseType,
    pub database_url: String,
    pub max_connections: u32,
    pub enable_versioning: bool,
    pub max_versions: Option<usize>,
}

impl Default for DatabaseStorageConfig {
    fn default() -> Self {
        Self {
            database_type: DatabaseType::SQLite,
            database_url: "sqlite:routes.db".to_string(),
            max_connections: 5,
            enable_versioning: true,
            max_versions: Some(10),
        }
    }
}

/// 数据库持久化存储
///
/// 支持 SQLite 和 PostgreSQL 数据库，提供路由持久化、版本控制和回滚功能。
///
/// # Examples
///
/// ```no_run
/// use dynamic_route_actix::storage::database_storage::{DatabaseStorage, DatabaseStorageConfig, DatabaseType};
/// use dynamic_route_actix::storage::RouteStorage;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = DatabaseStorageConfig {
///         database_type: DatabaseType::SQLite,
///         database_url: "sqlite:routes.db".to_string(),
///         ..Default::default()
///     };
///
///     let storage = DatabaseStorage::new(config).await?;
///
///     // 加载路由
///     let routes = storage.load().await?;
///
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub enum DatabaseStorage {
    Sqlite {
        pool: SqlitePool,
        config: DatabaseStorageConfig,
    },
    Postgres {
        pool: Pool<Postgres>,
        config: DatabaseStorageConfig,
    },
}

impl DatabaseStorage {
    /// 创建新的数据库存储实例
    ///
    /// # Arguments
    ///
    /// * `config` - 数据库配置
    ///
    /// # Returns
    ///
    /// 返回一个新的数据库存储实例
    ///
    /// # Errors
    ///
    /// 如果连接失败或初始化失败，返回错误
    pub async fn new(config: DatabaseStorageConfig) -> Result<Self, DatabaseStorageError> {
        match config.database_type {
            DatabaseType::SQLite => {
                Self::new_sqlite(config).await.map_err(|e| DatabaseStorageError::ConnectionError(e.to_string()))
            }
            DatabaseType::PostgreSQL => {
                Self::new_postgres(config).await.map_err(|e| DatabaseStorageError::ConnectionError(e.to_string()))
            }
        }
    }

    /// 创建 SQLite 数据库存储
    async fn new_sqlite(config: DatabaseStorageConfig) -> Result<Self, sqlx::Error> {
        let pool = PoolOptions::new()
            .max_connections(config.max_connections)
            .connect(&config.database_url)
            .await?;

        // 创建表结构
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS routes (
                path TEXT PRIMARY KEY,
                route_type TEXT NOT NULL,
                body TEXT NOT NULL,
                content_type TEXT NOT NULL,
                extra_data TEXT,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS route_versions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT NOT NULL,
                version INTEGER NOT NULL,
                route_type TEXT NOT NULL,
                body TEXT NOT NULL,
                content_type TEXT NOT NULL,
                extra_data TEXT,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                created_by TEXT,
                UNIQUE(path, version)
            );

            CREATE INDEX IF NOT EXISTS idx_route_versions_path ON route_versions(path);
            CREATE INDEX IF NOT EXISTS idx_route_versions_created_at ON route_versions(created_at);
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(DatabaseStorage::Sqlite { pool, config })
    }

    /// 创建 PostgreSQL 数据库存储
    async fn new_postgres(config: DatabaseStorageConfig) -> Result<Self, sqlx::Error> {
        let pool = PoolOptions::new()
            .max_connections(config.max_connections)
            .connect(&config.database_url)
            .await?;

        // 创建表结构
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS routes (
                path TEXT PRIMARY KEY,
                route_type TEXT NOT NULL,
                body TEXT NOT NULL,
                content_type TEXT NOT NULL,
                extra_data TEXT,
                updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS route_versions (
                id SERIAL PRIMARY KEY,
                path TEXT NOT NULL,
                version INTEGER NOT NULL,
                route_type TEXT NOT NULL,
                body TEXT NOT NULL,
                content_type TEXT NOT NULL,
                extra_data TEXT,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                created_by TEXT,
                UNIQUE(path, version)
            );

            CREATE INDEX IF NOT EXISTS idx_route_versions_path ON route_versions(path);
            CREATE INDEX IF NOT EXISTS idx_route_versions_created_at ON route_versions(created_at);
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(DatabaseStorage::Postgres {
            pool: pool.clone(),
            config,
        })
    }

    /// 获取数据库类型
    pub fn database_type(&self) -> DatabaseType {
        match self {
            DatabaseStorage::Sqlite { .. } => DatabaseType::SQLite,
            DatabaseStorage::Postgres { .. } => DatabaseType::PostgreSQL,
        }
    }

    /// 检查是否启用版本控制
    pub fn is_versioning_enabled(&self) -> bool {
        match self {
            DatabaseStorage::Sqlite { config, .. } => config.enable_versioning,
            DatabaseStorage::Postgres { config, .. } => config.enable_versioning,
        }
    }

    /// 保存路由版本
    async fn save_route_version(
        &self,
        path: &str,
        route: &SerializableRoute,
        version: i64,
        created_by: Option<String>,
    ) -> Result<(), DatabaseStorageError> {
        if !self.is_versioning_enabled() {
            return Ok(());
        }

        match self {
            DatabaseStorage::Sqlite { pool, config } => {
                sqlx::query(
                    r#"
                    INSERT INTO route_versions (path, version, route_type, body, content_type, extra_data, created_at, created_by)
                    VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, ?)
                    "#,
                )
                .bind(path)
                .bind(version)
                .bind(&route.route_type)
                .bind(&route.body)
                .bind(&route.content_type)
                .bind(&route.extra_data)
                .bind(created_by)
                .execute(pool)
                .await
                .map_err(|e| DatabaseStorageError::QueryError(e.to_string()))?;

                // 清理旧版本
                if let Some(max_versions) = config.max_versions {
                    sqlx::query(
                        r#"
                        DELETE FROM route_versions
                        WHERE path = ? AND id NOT IN (
                            SELECT id FROM route_versions
                            WHERE path = ?
                            ORDER BY created_at DESC
                            LIMIT ?
                        )
                        "#,
                    )
                    .bind(path)
                    .bind(path)
                    .bind(max_versions as i64)
                    .execute(pool)
                    .await
                    .map_err(|e| DatabaseStorageError::QueryError(e.to_string()))?;
                }

                Ok(())
            }
            DatabaseStorage::Postgres { pool, config } => {
                sqlx::query(
                    r#"
                    INSERT INTO route_versions (path, version, route_type, body, content_type, extra_data, created_at, created_by)
                    VALUES ($1, $2, $3, $4, $5, $6, CURRENT_TIMESTAMP, $7)
                    "#,
                )
                .bind(path)
                .bind(version)
                .bind(&route.route_type)
                .bind(&route.body)
                .bind(&route.content_type)
                .bind(&route.extra_data)
                .bind(created_by)
                .execute(pool)
                .await
                .map_err(|e| DatabaseStorageError::QueryError(e.to_string()))?;

                // 清理旧版本
                if let Some(max_versions) = config.max_versions {
                    sqlx::query(
                        r#"
                        DELETE FROM route_versions
                        WHERE path = $1 AND id NOT IN (
                            SELECT id FROM route_versions
                            WHERE path = $1
                            ORDER BY created_at DESC
                            LIMIT $2
                        )
                        "#,
                    )
                    .bind(path)
                    .bind(max_versions as i64)
                    .execute(pool)
                    .await
                    .map_err(|e| DatabaseStorageError::QueryError(e.to_string()))?;
                }

                Ok(())
            }
        }
    }

    /// 获取路由的下一个版本号
    async fn get_next_version(&self, path: &str) -> Result<i64, DatabaseStorageError> {
        match self {
            DatabaseStorage::Sqlite { pool, .. } => {
                let row = sqlx::query(
                    r#"
                    SELECT COALESCE(MAX(version), 0) as max_version
                    FROM route_versions
                    WHERE path = ?
                    "#,
                )
                .bind(path)
                .fetch_one(pool)
                .await
                .map_err(|e| DatabaseStorageError::QueryError(e.to_string()))?;

                Ok(row.get::<i64, _>("max_version") + 1)
            }
            DatabaseStorage::Postgres { pool, .. } => {
                let row = sqlx::query(
                    r#"
                    SELECT COALESCE(MAX(version), 0) as max_version
                    FROM route_versions
                    WHERE path = $1
                    "#,
                )
                .bind(path)
                .fetch_one(pool)
                .await
                .map_err(|e| DatabaseStorageError::QueryError(e.to_string()))?;

                Ok(row.get::<i64, _>("max_version") + 1)
            }
        }
    }

    /// 获取路由的所有版本
    pub async fn get_route_versions(&self, path: &str) -> Result<Vec<RouteVersion>, DatabaseStorageError> {
        match self {
            DatabaseStorage::Sqlite { pool, .. } => {
                let rows = sqlx::query(
                    r#"
                    SELECT version, path, route_type, body, content_type, extra_data, created_at, created_by
                    FROM route_versions
                    WHERE path = ?
                    ORDER BY version DESC
                    "#,
                )
                .bind(path)
                .fetch_all(pool)
                .await
                .map_err(|e| DatabaseStorageError::QueryError(e.to_string()))?;

                let versions = rows
                    .iter()
                    .map(|row| RouteVersion {
                        version: row.get("version"),
                        route_path: row.get("path"),
                        route_type: row.get("route_type"),
                        body: row.get("body"),
                        content_type: row.get("content_type"),
                        extra_data: row.get("extra_data"),
                        created_at: row.get("created_at"),
                        created_by: row.get("created_by"),
                    })
                    .collect();

                Ok(versions)
            }
            DatabaseStorage::Postgres { pool, .. } => {
                let rows = sqlx::query(
                    r#"
                    SELECT version, path, route_type, body, content_type, extra_data, created_at, created_by
                    FROM route_versions
                    WHERE path = $1
                    ORDER BY version DESC
                    "#,
                )
                .bind(path)
                .fetch_all(pool)
                .await
                .map_err(|e| DatabaseStorageError::QueryError(e.to_string()))?;

                let versions = rows
                    .iter()
                    .map(|row| RouteVersion {
                        version: row.get("version"),
                        route_path: row.get("path"),
                        route_type: row.get("route_type"),
                        body: row.get("body"),
                        content_type: row.get("content_type"),
                        extra_data: row.get("extra_data"),
                        created_at: row.get("created_at"),
                        created_by: row.get("created_by"),
                    })
                    .collect();

                Ok(versions)
            }
        }
    }

    /// 回滚路由到指定版本
    pub async fn rollback_route(
        &self,
        path: &str,
        version: i64,
    ) -> Result<(), DatabaseStorageError> {
        // 获取指定版本的路由
        let versions = self.get_route_versions(path).await?;
        let target_version = versions
            .iter()
            .find(|v| v.version == version)
            .ok_or_else(|| DatabaseStorageError::RouteNotFound(format!("Version {} not found", version)))?;

        // 创建可序列化的路由
        let serializable = SerializableRoute {
            route_type: target_version.route_type.clone(),
            body: target_version.body.clone(),
            content_type: target_version.content_type.clone(),
            extra_data: target_version.extra_data.clone(),
        };

        // 更新当前路由
        self.update_route(path, &serializable).await?;

        Ok(())
    }

    /// 更新单个路由
    async fn update_route(&self, path: &str, route: &SerializableRoute) -> Result<(), DatabaseStorageError> {
        match self {
            DatabaseStorage::Sqlite { pool, .. } => {
                sqlx::query(
                    r#"
                    INSERT OR REPLACE INTO routes (path, route_type, body, content_type, extra_data, updated_at)
                    VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
                    "#,
                )
                .bind(path)
                .bind(&route.route_type)
                .bind(&route.body)
                .bind(&route.content_type)
                .bind(&route.extra_data)
                .execute(pool)
                .await
                .map_err(|e| DatabaseStorageError::QueryError(e.to_string()))?;

                Ok(())
            }
            DatabaseStorage::Postgres { pool, .. } => {
                sqlx::query(
                    r#"
                    INSERT INTO routes (path, route_type, body, content_type, extra_data, updated_at)
                    VALUES ($1, $2, $3, $4, $5, CURRENT_TIMESTAMP)
                    ON CONFLICT (path) DO UPDATE SET
                        route_type = EXCLUDED.route_type,
                        body = EXCLUDED.body,
                        content_type = EXCLUDED.content_type,
                        extra_data = EXCLUDED.extra_data,
                        updated_at = CURRENT_TIMESTAMP
                    "#,
                )
                .bind(path)
                .bind(&route.route_type)
                .bind(&route.body)
                .bind(&route.content_type)
                .bind(&route.extra_data)
                .execute(pool)
                .await
                .map_err(|e| DatabaseStorageError::QueryError(e.to_string()))?;

                Ok(())
            }
        }
    }

    /// 删除路由
    pub async fn delete_route(&self, path: &str) -> Result<(), DatabaseStorageError> {
        match self {
            DatabaseStorage::Sqlite { pool, .. } => {
                // 删除当前路由
                sqlx::query("DELETE FROM routes WHERE path = ?")
                    .bind(path)
                    .execute(pool)
                    .await
                    .map_err(|e| DatabaseStorageError::QueryError(e.to_string()))?;

                // 删除所有版本
                sqlx::query("DELETE FROM route_versions WHERE path = ?")
                    .bind(path)
                    .execute(pool)
                    .await
                    .map_err(|e| DatabaseStorageError::QueryError(e.to_string()))?;

                Ok(())
            }
            DatabaseStorage::Postgres { pool, .. } => {
                // 删除当前路由
                sqlx::query("DELETE FROM routes WHERE path = $1")
                    .bind(path)
                    .execute(pool)
                    .await
                    .map_err(|e| DatabaseStorageError::QueryError(e.to_string()))?;

                // 删除所有版本
                sqlx::query("DELETE FROM route_versions WHERE path = $1")
                    .bind(path)
                    .execute(pool)
                    .await
                    .map_err(|e| DatabaseStorageError::QueryError(e.to_string()))?;

                Ok(())
            }
        }
    }

    /// 验证路由类型是否有效
    pub fn validate_route_type(&self, route_type: &str) -> Result<(), DatabaseStorageError> {
        // 使用注册表验证路由类型
        match crate::core::route_registry::RouteRegistry::list_types().contains(&route_type.to_string()) {
            true => Ok(()),
            false => Err(DatabaseStorageError::QueryError(format!(
                "Unknown route type: {}. Make sure to register the type before use.",
                route_type
            ))),
        }
    }
}

#[async_trait]
impl KeyValueStorage for DatabaseStorage {
    async fn read(&self, key: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        match self {
            DatabaseStorage::Sqlite { pool, .. } => {
                let row = sqlx::query("SELECT body FROM routes WHERE path = ?")
                    .bind(key)
                    .fetch_optional(pool)
                    .await
                    .map_err(|e| DatabaseStorageError::QueryError(e.to_string()))?;

                match row {
                    Some(row) => Ok(row.get("body")),
                    None => Err(DatabaseStorageError::RouteNotFound(key.to_string()).into()),
                }
            }
            DatabaseStorage::Postgres { pool, .. } => {
                let row = sqlx::query("SELECT body FROM routes WHERE path = $1")
                    .bind(key)
                    .fetch_optional(pool)
                    .await
                    .map_err(|e| DatabaseStorageError::QueryError(e.to_string()))?;

                match row {
                    Some(row) => Ok(row.get("body")),
                    None => Err(DatabaseStorageError::RouteNotFound(key.to_string()).into()),
                }
            }
        }
    }

    async fn write(&self, key: &str, value: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        match self {
            DatabaseStorage::Sqlite { pool, .. } => {
                sqlx::query(
                    r#"
                    INSERT OR REPLACE INTO routes (path, route_type, body, content_type, extra_data, updated_at)
                    VALUES (?, 'SimpleRoute', ?, 'text/plain', NULL, CURRENT_TIMESTAMP)
                    "#,
                )
                .bind(key)
                .bind(value)
                .execute(pool)
                .await
                .map_err(|e| DatabaseStorageError::QueryError(e.to_string()))?;

                Ok(())
            }
            DatabaseStorage::Postgres { pool, .. } => {
                sqlx::query(
                    r#"
                    INSERT INTO routes (path, route_type, body, content_type, extra_data, updated_at)
                    VALUES ($1, 'SimpleRoute', $2, 'text/plain', NULL, CURRENT_TIMESTAMP)
                    ON CONFLICT (path) DO UPDATE SET
                        body = EXCLUDED.body,
                        updated_at = CURRENT_TIMESTAMP
                    "#,
                )
                .bind(key)
                .bind(value)
                .execute(pool)
                .await
                .map_err(|e| DatabaseStorageError::QueryError(e.to_string()))?;

                Ok(())
            }
        }
    }

    async fn delete(&self, key: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.delete_route(key).await?;
        Ok(())
    }

    async fn exists(&self, key: &str) -> bool {
        match self {
            DatabaseStorage::Sqlite { pool, .. } => {
                let result = sqlx::query("SELECT 1 FROM routes WHERE path = ?")
                    .bind(key)
                    .fetch_optional(pool)
                    .await;

                result.is_ok() && result.unwrap().is_some()
            }
            DatabaseStorage::Postgres { pool, .. } => {
                let result = sqlx::query("SELECT 1 FROM routes WHERE path = $1")
                    .bind(key)
                    .fetch_optional(pool)
                    .await;

                result.is_ok() && result.unwrap().is_some()
            }
        }
    }
}

#[async_trait]
impl RouteStorage for DatabaseStorage {
    async fn load(&self) -> Result<HashMap<String, Box<dyn RouteEntry>>, Box<dyn Error + Send + Sync>> {
        let mut routes = HashMap::new();

        match self {
            DatabaseStorage::Sqlite { pool, .. } => {
                let rows = sqlx::query(
                    r#"
                    SELECT path, route_type, body, content_type, extra_data
                    FROM routes
                    ORDER BY path
                    "#,
                )
                .fetch_all(pool)
                .await
                .map_err(|e| DatabaseStorageError::QueryError(e.to_string()))?;

                for row in rows {
                    let path = row.get("path");
                    let serializable = SerializableRoute {
                        route_type: row.get("route_type"),
                        body: row.get("body"),
                        content_type: row.get("content_type"),
                        extra_data: row.get("extra_data"),
                    };

                    // 使用注册表创建路由实例
                    let route = crate::core::route_registry::RouteRegistry::create_route(serializable)
                        .map_err(|e| DatabaseStorageError::QueryError(format!("Failed to create route: {}", e)))?;

                    routes.insert(path, route);
                }

                Ok(routes)
            }
            DatabaseStorage::Postgres { pool, .. } => {
                let rows = sqlx::query(
                    r#"
                    SELECT path, route_type, body, content_type, extra_data
                    FROM routes
                    ORDER BY path
                    "#,
                )
                .fetch_all(pool)
                .await
                .map_err(|e| DatabaseStorageError::QueryError(e.to_string()))?;

                for row in rows {
                    let path = row.get("path");
                    let serializable = SerializableRoute {
                        route_type: row.get("route_type"),
                        body: row.get("body"),
                        content_type: row.get("content_type"),
                        extra_data: row.get("extra_data"),
                    };

                    // 使用注册表创建路由实例
                    let route = crate::core::route_registry::RouteRegistry::create_route(serializable)
                        .map_err(|e| DatabaseStorageError::QueryError(format!("Failed to create route: {}", e)))?;

                    routes.insert(path, route);
                }

                Ok(routes)
            }
        }
    }

    async fn save(&self, routes: &HashMap<String, Box<dyn RouteEntry>>) -> Result<(), Box<dyn Error + Send + Sync>> {
        for (path, route) in routes {
            let serializable = route.to_serializable();

            // 验证路由类型
            self.validate_route_type(&serializable.route_type)?;

            // 如果启用版本控制，先保存当前版本
            if self.is_versioning_enabled() && self.exists(path).await {
                // 读取当前路由
                let current_body = match self.read(path).await {
                    Ok(body) => body,
                    Err(_) => continue, // 如果读取失败，跳过版本保存
                };

                // 创建当前路由的可序列化对象
                let current_route = SerializableRoute {
                    route_type: serializable.route_type.clone(),
                    body: current_body,
                    content_type: serializable.content_type.clone(),
                    extra_data: serializable.extra_data.clone(),
                };

                // 获取下一个版本号
                let version = match self.get_next_version(path).await {
                    Ok(v) => v,
                    Err(_) => continue, // 如果获取版本号失败，跳过版本保存
                };

                // 保存版本
                let _ = self.save_route_version(path, &current_route, version, None).await;
            }

            // 更新当前路由
            self.update_route(path, &serializable).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_sqlite_storage_crud() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_str().unwrap();

        let config = DatabaseStorageConfig {
            database_type: DatabaseType::SQLite,
            database_url: format!("sqlite:{}", db_path),
            ..Default::default()
        };

        let storage = DatabaseStorage::new(config).await.unwrap();

        // 测试写入
        storage.write("test", "Hello, World!").await.unwrap();
        assert!(storage.exists("test").await);

        // 测试读取
        let content = storage.read("test").await.unwrap();
        assert_eq!(content, "Hello, World!");

        // 测试更新
        storage.write("test", "Updated content").await.unwrap();
        let content = storage.read("test").await.unwrap();
        assert_eq!(content, "Updated content");

        // 测试删除
        storage.delete("test").await.unwrap();
        assert!(!storage.exists("test").await);
    }

    #[tokio::test]
    async fn test_sqlite_storage_route_persistence() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_str().unwrap();

        let config = DatabaseStorageConfig {
            database_type: DatabaseType::SQLite,
            database_url: format!("sqlite:{}", db_path),
            ..Default::default()
        };

        let storage = DatabaseStorage::new(config).await.unwrap();

        // 创建并保存路由
        let mut routes = HashMap::new();
        routes.insert(
            "test1".into(),
            Box::new(crate::SimpleRoute::new("body1", "text/plain")) as Box<dyn RouteEntry>,
        );
        routes.insert(
            "test2".into(),
            Box::new(crate::SimpleRoute::new("body2", "application/json")) as Box<dyn RouteEntry>,
        );

        storage.save(&routes).await.unwrap();

        // 加载路由
        let loaded_routes = storage.load().await.unwrap();
        assert_eq!(loaded_routes.len(), 2);
        assert!(loaded_routes.contains_key("test1"));
        assert!(loaded_routes.contains_key("test2"));
    }

    #[tokio::test]
    async fn test_sqlite_version_control() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_str().unwrap();

        let config = DatabaseStorageConfig {
            database_type: DatabaseType::SQLite,
            database_url: format!("sqlite:{}", db_path),
            enable_versioning: true,
            max_versions: Some(3),
            ..Default::default()
        };

        let storage = DatabaseStorage::new(config).await.unwrap();

        // 创建初始路由
        let mut routes = HashMap::new();
        routes.insert(
            "test".into(),
            Box::new(crate::SimpleRoute::new("version1", "text/plain")) as Box<dyn RouteEntry>,
        );

        storage.save(&routes).await.unwrap();

        // 更新路由
        routes.insert(
            "test".into(),
            Box::new(crate::SimpleRoute::new("version2", "text/plain")) as Box<dyn RouteEntry>,
        );

        storage.save(&routes).await.unwrap();

        // 再次更新
        routes.insert(
            "test".into(),
            Box::new(crate::SimpleRoute::new("version3", "text/plain")) as Box<dyn RouteEntry>,
        );

        storage.save(&routes).await.unwrap();

        // 获取版本历史
        let versions = storage.get_route_versions("test").await.unwrap();
        assert_eq!(versions.len(), 2); // 第一个版本不会被记录（因为初始创建）
        assert_eq!(versions[0].body, "version2");
        assert_eq!(versions[1].body, "version1");
    }

    #[tokio::test]
    async fn test_sqlite_rollback() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_str().unwrap();

        let config = DatabaseStorageConfig {
            database_type: DatabaseType::SQLite,
            database_url: format!("sqlite:{}", db_path),
            enable_versioning: true,
            ..Default::default()
        };

        let storage = DatabaseStorage::new(config).await.unwrap();

        // 创建初始路由
        let mut routes = HashMap::new();
        routes.insert(
            "test".into(),
            Box::new(crate::SimpleRoute::new("version1", "text/plain")) as Box<dyn RouteEntry>,
        );

        storage.save(&routes).await.unwrap();

        // 更新路由
        routes.insert(
            "test".into(),
            Box::new(crate::SimpleRoute::new("version2", "text/plain")) as Box<dyn RouteEntry>,
        );

        storage.save(&routes).await.unwrap();

        // 回滚到版本 1
        storage.rollback_route("test", 1).await.unwrap();

        // 验证回滚成功
        let loaded_routes = storage.load().await.unwrap();
        let route = loaded_routes.get("test").unwrap();
        let serializable = route.to_serializable();
        assert_eq!(serializable.body, "version1");
    }

    #[tokio::test]
    async fn test_sqlite_route_validation() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_str().unwrap();

        let config = DatabaseStorageConfig {
            database_type: DatabaseType::SQLite,
            database_url: format!("sqlite:{}", db_path),
            ..Default::default()
        };

        let storage = DatabaseStorage::new(config).await.unwrap();

        // 测试有效的路由类型
        assert!(storage.validate_route_type("SimpleRoute").is_ok());

        // 测试无效的路由类型
        assert!(storage.validate_route_type("InvalidRoute").is_err());
    }
}