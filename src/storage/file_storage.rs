//! 基于文件的持久化存储实现
//!
//! 提供文件系统的键值存储和路由持久化功能。

use crate::core::route_entry::RouteEntry;
use crate::core::simple_route::SimpleRoute;
use crate::core::SerializableRoute;
use crate::storage::traits::{KeyValueStorage, RouteStorage};
use async_trait::async_trait;
use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// 基于文件的持久化存储
///
/// 提供文件系统的键值对存储功能，支持 CRUD 操作。
///
/// # Examples
///
/// ```no_run
/// use dynamic_route_actix::storage::{FileStorage, KeyValueStorage};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let storage = FileStorage::new("./data");
///
///     // 写入文件
///     storage.write("config.json", "{\"theme\":\"dark\"}").await?;
///
///     // 读取文件
///     let content = storage.read("config.json").await?;
///     println!("Config: {}", content);
///
///     // 检查文件是否存在
///     if storage.exists("config.json").await {
///         println!("Config file exists");
///     }
///
///     // 删除文件
///     storage.delete("config.json").await?;
///
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct FileStorage {
    base_path: PathBuf,
}

impl FileStorage {
    /// 创建一个新的文件存储实例
    ///
    /// # Arguments
    ///
    /// * `base_path` - 基础目录路径
    ///
    /// # Examples
    ///
    /// ```
    /// use dynamic_route_actix::storage::FileStorage;
    ///
    /// let storage = FileStorage::new("./data");
    /// ```
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    /// 获取文件的完整路径
    fn get_file_path(&self, key: &str) -> PathBuf {
        // 简单的路径处理，防止路径遍历攻击
        let normalized_key = key.replace("..", "").replace("\\", "/");
        self.base_path.join(normalized_key)
    }

    /// 确保基础目录存在
    async fn ensure_base_dir(&self) -> Result<(), Box<dyn Error>> {
        if !self.base_path.exists() {
            fs::create_dir_all(&self.base_path).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl KeyValueStorage for FileStorage {
    async fn read(&self, key: &str) -> Result<String, Box<dyn Error>> {
        let file_path = self.get_file_path(key);

        if !file_path.exists() {
            return Err(format!("File not found: {}", file_path.display()).into());
        }

        let mut file = fs::File::open(&file_path).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;

        Ok(contents)
    }

    async fn write(&self, key: &str, value: &str) -> Result<(), Box<dyn Error>> {
        self.ensure_base_dir().await?;

        let file_path = self.get_file_path(key);
        let mut file = fs::File::create(&file_path).await?;
        file.write_all(value.as_bytes()).await?;
        file.flush().await?;

        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), Box<dyn Error>> {
        let file_path = self.get_file_path(key);

        if !file_path.exists() {
            return Err(format!("File not found: {}", file_path.display()).into());
        }

        fs::remove_file(&file_path).await?;
        Ok(())
    }

    async fn exists(&self, key: &str) -> bool {
        let file_path = self.get_file_path(key);
        file_path.exists()
    }
}

#[async_trait]
impl RouteStorage for FileStorage {
    async fn load(&self) -> Result<HashMap<String, Box<dyn RouteEntry>>, Box<dyn Error>> {
        let mut routes = HashMap::new();

        if !self.base_path.exists() {
            return Ok(routes);
        }

        let mut entries = fs::read_dir(&self.base_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            // 只处理 .json 文件
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }

            // 从文件名提取路由路径（去掉 .json 扩展名）
            let route_key = path
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| format!("Invalid filename: {}", path.display()))?;

            // 读取文件内容
            let content = fs::read_to_string(&path).await?;
            let serializable: SerializableRoute = serde_json::from_str(&content)?;

            // 根据路由类型创建对应的 RouteEntry
            // 目前只支持 SimpleRoute，未来可以扩展支持更多类型
            let route: Box<dyn RouteEntry> = match serializable.route_type.as_str() {
                "SimpleRoute" => SimpleRoute::from_serializable(serializable),
                _ => {
                    return Err(format!("Unknown route type: {}", serializable.route_type).into());
                }
            };

            routes.insert(route_key.to_string(), route);
        }

        Ok(routes)
    }

    async fn save(
        &self,
        routes: &HashMap<String, Box<dyn RouteEntry>>,
    ) -> Result<(), Box<dyn Error>> {
        self.ensure_base_dir().await?;

        // 使用新的序列化接口
        for (path, route) in routes {
            let serializable = route.to_serializable();
            let json = serde_json::to_string_pretty(&serializable)?;

            let file_path = self.get_file_path(&format!("{}.json", path));
            let mut file = fs::File::create(&file_path).await?;
            file.write_all(json.as_bytes()).await?;
            file.flush().await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_file_storage_write_and_read() {
        let temp_dir = tempdir().unwrap();
        let storage = FileStorage::new(temp_dir.path());

        // 写入文件
        storage.write("test.txt", "Hello, World!").await.unwrap();

        // 读取文件
        let content = storage.read("test.txt").await.unwrap();
        assert_eq!(content, "Hello, World!");
    }

    #[tokio::test]
    async fn test_file_storage_delete() {
        let temp_dir = tempdir().unwrap();
        let storage = FileStorage::new(temp_dir.path());

        // 写入文件
        storage.write("test.txt", "Hello, World!").await.unwrap();
        assert!(storage.exists("test.txt").await);

        // 删除文件
        storage.delete("test.txt").await.unwrap();
        assert!(!storage.exists("test.txt").await);
    }

    #[tokio::test]
    async fn test_file_storage_overwrite() {
        let temp_dir = tempdir().unwrap();
        let storage = FileStorage::new(temp_dir.path());

        // 写入初始内容
        storage.write("test.txt", "Original content").await.unwrap();

        // 覆盖写入
        storage.write("test.txt", "Updated content").await.unwrap();

        // 验证内容已更新
        let content = storage.read("test.txt").await.unwrap();
        assert_eq!(content, "Updated content");
    }

    #[tokio::test]
    async fn test_file_storage_path_normalization() {
        let temp_dir = tempdir().unwrap();
        let storage = FileStorage::new(temp_dir.path());

        // 测试路径规范化 - 文件名中的特殊字符会被处理
        // 写入包含特殊字符的文件名
        storage.write("test..file.txt", "content").await.unwrap();

        // 验证能够读取该文件
        let content = storage.read("test..file.txt").await.unwrap();
        assert_eq!(content, "content");

        // 验证文件存在
        assert!(storage.exists("test..file.txt").await);
    }

    #[tokio::test]
    async fn test_file_storage_nonexistent_read() {
        let temp_dir = tempdir().unwrap();
        let storage = FileStorage::new(temp_dir.path());

        let result = storage.read("nonexistent.txt").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_file_storage_nonexistent_delete() {
        let temp_dir = tempdir().unwrap();
        let storage = FileStorage::new(temp_dir.path());

        let result = storage.delete("nonexistent.txt").await;
        assert!(result.is_err());
    }
}
