//! 路由持久化抽象 trait
//!
//! 提供路由持久化和通用键值存储的抽象接口。

use crate::core::route_entry::RouteEntry;
use async_trait::async_trait;
use std::collections::HashMap;
use std::error::Error;

/// 路由持久化抽象
///
/// 定义了路由的加载和保存接口，用于支持不同的持久化后端。
///
/// # Examples
///
/// ```
/// use dynamic_route_actix::storage::RouteStorage;
/// use dynamic_route_actix::core::{RouteEntry, SimpleRoute};
/// use async_trait::async_trait;
///
/// struct MockStorage;
///
/// #[async_trait]
/// impl RouteStorage for MockStorage {
///     async fn load(&self) -> Result<std::collections::HashMap<String, Box<dyn RouteEntry>>, Box<dyn std::error::Error + Send + Sync>> {
///         Ok(std::collections::HashMap::new())
///     }
///
///     async fn save(&self, routes: &std::collections::HashMap<String, Box<dyn RouteEntry>>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait RouteStorage: Send + Sync {
    /// 从持久化存储加载所有路由
    ///
    /// # Returns
    ///
    /// 返回一个包含所有路由的 HashMap
    ///
    /// # Errors
    ///
    /// 如果加载失败，返回错误
    async fn load(&self) -> Result<HashMap<String, Box<dyn RouteEntry>>, Box<dyn Error + Send + Sync>>;

    /// 将所有路由保存到持久化存储
    ///
    /// # Arguments
    ///
    /// * `routes` - 要保存的路由表
    ///
    /// # Errors
    ///
    /// 如果保存失败，返回错误
    async fn save(
        &self,
        routes: &HashMap<String, Box<dyn RouteEntry>>,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
}

/// 简单的键值存储抽象，提供基础的 CRUD 操作
///
/// 提供统一的键值对存储接口，支持文件、内存、数据库等多种存储后端。
///
/// # Examples
///
/// ```
/// use dynamic_route_actix::storage::KeyValueStorage;
///
/// struct MemoryStorage {
///     data: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<String, String>>>,
/// }
///
/// impl MemoryStorage {
///     pub fn new() -> Self {
///         Self {
///             data: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
///         }
///     }
/// }
///
/// #[async_trait::async_trait]
/// impl KeyValueStorage for MemoryStorage {
///     async fn read(&self, key: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
///         let data = self.data.read().await;
///         data.get(key)
///             .cloned()
///             .ok_or_else(|| "Key not found".into())
///     }
///
///     async fn write(&self, key: &str, value: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
///         let mut data = self.data.write().await;
///         data.insert(key.to_string(), value.to_string());
///         Ok(())
///     }
///
///     async fn delete(&self, key: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
///         let mut data = self.data.write().await;
///         data.remove(key).map(|_| ()).ok_or_else(|| "Key not found".into())
///     }
///
///     async fn exists(&self, key: &str) -> bool {
///         let data = self.data.read().await;
///         data.contains_key(key)
///     }
/// }
/// ```
#[async_trait]
pub trait KeyValueStorage: Send + Sync {
    /// 读取指定键的值
    ///
    /// # Arguments
    ///
    /// * `key` - 要读取的键
    ///
    /// # Returns
    ///
    /// 返回键对应的值
    ///
    /// # Errors
    ///
    /// 如果键不存在或读取失败，返回错误
    async fn read(&self, key: &str) -> Result<String, Box<dyn Error + Send + Sync>>;

    /// 写入键值对
    ///
    /// # Arguments
    ///
    /// * `key` - 要写入的键
    /// * `value` - 要写入的值
    ///
    /// # Errors
    ///
    /// 如果写入失败，返回错误
    async fn write(&self, key: &str, value: &str) -> Result<(), Box<dyn Error + Send + Sync>>;

    /// 删除指定键
    ///
    /// # Arguments
    ///
    /// * `key` - 要删除的键
    ///
    /// # Errors
    ///
    /// 如果键不存在或删除失败，返回错误
    async fn delete(&self, key: &str) -> Result<(), Box<dyn Error + Send + Sync>>;

    /// 检查键是否存在
    ///
    /// # Arguments
    ///
    /// * `key` - 要检查的键
    ///
    /// # Returns
    ///
    /// 如果键存在返回 true，否则返回 false
    async fn exists(&self, key: &str) -> bool;
}
