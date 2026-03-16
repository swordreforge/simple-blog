//! 基于内存的临时存储实现
//!
//! 提供纯内存的键值存储功能，适用于测试和缓存场景。

use crate::core::route_entry::RouteEntry;
use crate::storage::traits::{KeyValueStorage, RouteStorage};
use async_trait::async_trait;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 基于内存的临时存储
///
/// 提供纯内存的键值对存储功能，适用于测试和缓存场景。
///
/// # Examples
///
/// ```
/// use dynamic_route_actix::storage::{MemoryStorage, KeyValueStorage};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let storage = MemoryStorage::new();
///
///     // 写入键值对
///     storage.write("user:1", "Alice").await?;
///
///     // 读取值
///     let user = storage.read("user:1").await?;
///     assert_eq!(user, "Alice");
///
///     // 删除键值对
///     storage.delete("user:1").await?;
///
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct MemoryStorage {
    data: Arc<RwLock<HashMap<String, String>>>,
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryStorage {
    /// 创建一个新的内存存储实例
    ///
    /// # Examples
    ///
    /// ```
    /// use dynamic_route_actix::storage::MemoryStorage;
    ///
    /// let storage = MemoryStorage::new();
    /// ```
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 清空所有数据
    ///
    /// # Examples
    ///
    /// ```
    /// use dynamic_route_actix::storage::{MemoryStorage, KeyValueStorage};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let storage = MemoryStorage::new();
    ///     storage.write("key", "value").await.unwrap();
    ///     storage.clear().await;
    ///     assert!(!storage.exists("key").await);
    /// }
    /// ```
    pub async fn clear(&self) {
        let mut data = self.data.write().await;
        data.clear();
    }

    /// 获取当前存储的键数量
    ///
    /// # Examples
    ///
    /// ```
    /// use dynamic_route_actix::storage::{MemoryStorage, KeyValueStorage};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let storage = MemoryStorage::new();
    ///     storage.write("key1", "value1").await.unwrap();
    ///     storage.write("key2", "value2").await.unwrap();
    ///     assert_eq!(storage.len().await, 2);
    /// }
    /// ```
    pub async fn len(&self) -> usize {
        let data = self.data.read().await;
        data.len()
    }

    /// 检查存储是否为空
    ///
    /// # Examples
    ///
    /// ```
    /// use dynamic_route_actix::storage::{MemoryStorage, KeyValueStorage};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let storage = MemoryStorage::new();
    ///     assert!(storage.is_empty().await);
    ///     storage.write("key", "value").await.unwrap();
    ///     assert!(!storage.is_empty().await);
    /// }
    /// ```
    pub async fn is_empty(&self) -> bool {
        let data = self.data.read().await;
        data.is_empty()
    }
}

#[async_trait]
impl KeyValueStorage for MemoryStorage {
    async fn read(&self, key: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let data = self.data.read().await;
        data.get(key)
            .cloned()
            .ok_or_else(|| format!("Key not found: {}", key).into())
    }

    async fn write(&self, key: &str, value: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut data = self.data.write().await;
        data.insert(key.to_string(), value.to_string());
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut data = self.data.write().await;
        data.remove(key)
            .map(|_| ())
            .ok_or_else(|| format!("Key not found: {}", key).into())
    }

    async fn exists(&self, key: &str) -> bool {
        let data = self.data.read().await;
        data.contains_key(key)
    }
}

#[async_trait]
impl RouteStorage for MemoryStorage {
    async fn load(&self) -> Result<HashMap<String, Box<dyn RouteEntry>>, Box<dyn Error + Send + Sync>> {
        // MemoryStorage 不支持持久化路由，返回空 HashMap
        // 如果需要支持，可以添加额外的存储结构来保存序列化的路由
        Ok(HashMap::new())
    }

    async fn save(
        &self,
        routes: &HashMap<String, Box<dyn RouteEntry>>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        // MemoryStorage 不支持持久化路由，返回成功但不做任何操作
        // 如果需要支持，可以将路由序列化后存储到内部结构中
        let _ = routes;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_storage_write_and_read() {
        let storage = MemoryStorage::new();

        // 写入键值对
        storage.write("key1", "value1").await.unwrap();

        // 读取值
        let value = storage.read("key1").await.unwrap();
        assert_eq!(value, "value1");
    }

    #[tokio::test]
    async fn test_memory_storage_delete() {
        let storage = MemoryStorage::new();

        // 写入键值对
        storage.write("key1", "value1").await.unwrap();
        assert!(storage.exists("key1").await);

        // 删除键值对
        storage.delete("key1").await.unwrap();
        assert!(!storage.exists("key1").await);
    }

    #[tokio::test]
    async fn test_memory_storage_overwrite() {
        let storage = MemoryStorage::new();

        // 写入初始值
        storage.write("key1", "value1").await.unwrap();

        // 覆盖写入
        storage.write("key1", "value2").await.unwrap();

        // 验证值已更新
        let value = storage.read("key1").await.unwrap();
        assert_eq!(value, "value2");
    }

    #[tokio::test]
    async fn test_memory_storage_multiple_keys() {
        let storage = MemoryStorage::new();

        // 写入多个键值对
        for i in 0..10 {
            storage
                .write(&format!("key-{}", i), &format!("value-{}", i))
                .await
                .unwrap();
        }

        // 验证所有键值对
        for i in 0..10 {
            assert!(storage.exists(&format!("key-{}", i)).await);
            let value = storage.read(&format!("key-{}", i)).await.unwrap();
            assert_eq!(value, format!("value-{}", i));
        }

        assert_eq!(storage.len().await, 10);
    }

    #[tokio::test]
    async fn test_memory_storage_clear() {
        let storage = MemoryStorage::new();

        // 写入多个键值对
        for i in 0..5 {
            storage
                .write(&format!("key-{}", i), &format!("value-{}", i))
                .await
                .unwrap();
        }

        assert_eq!(storage.len().await, 5);

        // 清空所有数据
        storage.clear().await;

        assert_eq!(storage.len().await, 0);
        assert!(storage.is_empty().await);
    }

    #[tokio::test]
    async fn test_memory_storage_nonexistent_read() {
        let storage = MemoryStorage::new();

        let result = storage.read("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_memory_storage_nonexistent_delete() {
        let storage = MemoryStorage::new();

        let result = storage.delete("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_memory_storage_clone() {
        let storage1 = MemoryStorage::new();
        storage1.write("key1", "value1").await.unwrap();

        let storage2 = storage1.clone();

        // 两个实例共享同一份数据
        assert!(storage2.exists("key1").await);
        assert_eq!(storage2.read("key1").await.unwrap(), "value1");

        // 通过 storage2 修改数据
        storage2.write("key2", "value2").await.unwrap();

        // storage1 也能看到修改
        assert!(storage1.exists("key2").await);
        assert_eq!(storage1.read("key2").await.unwrap(), "value2");
    }

    #[tokio::test]
    async fn test_memory_storage_is_empty() {
        let storage = MemoryStorage::new();
        assert!(storage.is_empty().await);

        storage.write("key", "value").await.unwrap();
        assert!(!storage.is_empty().await);

        storage.clear().await;
        assert!(storage.is_empty().await);
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        let storage = Arc::new(MemoryStorage::new());
        let mut handles = vec![];

        // 并发写入
        for i in 0..10 {
            let storage_clone = Arc::clone(&storage);
            let handle = tokio::spawn(async move {
                let result = storage_clone
                    .write(&format!("key-{}", i), &format!("value-{}", i))
                    .await;
                if let Err(e) = result {
                    eprintln!("Error writing: {}", e);
                }
            });
            handles.push(handle);
        }

        // 等待所有写入完成
        for handle in handles {
            handle.await.unwrap();
        }

        // 验证所有数据都已写入
        for i in 0..10 {
            assert!(storage.exists(&format!("key-{}", i)).await);
            assert_eq!(
                storage.read(&format!("key-{}", i)).await.unwrap(),
                format!("value-{}", i)
            );
        }

        assert_eq!(storage.len().await, 10);
    }
}
