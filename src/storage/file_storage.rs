//! 基于文件的持久化存储实现
//!
//! 提供文件系统的键值存储和路由持久化功能。

use crate::core::route_entry::RouteEntry;
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
/// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
    async fn ensure_base_dir(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        if !self.base_path.exists() {
            fs::create_dir_all(&self.base_path).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl KeyValueStorage for FileStorage {
    async fn read(&self, key: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let file_path = self.get_file_path(key);

        if !file_path.exists() {
            return Err(format!("File not found: {}", file_path.display()).into());
        }

        let mut file = fs::File::open(&file_path).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;

        Ok(contents)
    }

    async fn write(&self, key: &str, value: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.ensure_base_dir().await?;

        let file_path = self.get_file_path(key);
        let mut file = fs::File::create(&file_path).await?;
        file.write_all(value.as_bytes()).await?;
        file.flush().await?;

        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
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
    async fn load(
        &self,
    ) -> Result<HashMap<String, Box<dyn RouteEntry>>, Box<dyn Error + Send + Sync>> {
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

            // 使用注册表创建路由实例
            let route: Box<dyn RouteEntry> =
                crate::core::route_registry::RouteRegistry::create_route(serializable)
                    .map_err(|e| format!("Failed to create route: {}", e))?;

            routes.insert(route_key.to_string(), route);
        }

        Ok(routes)
    }

    async fn save(
        &self,
        routes: &HashMap<String, Box<dyn RouteEntry>>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
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

    #[tokio::test]
    async fn test_file_storage_route_persistence_with_registry() {
        let temp_dir = tempdir().unwrap();
        let storage = FileStorage::new(temp_dir.path());

        // 创建并保存路由
        let mut routes = std::collections::HashMap::new();
        routes.insert(
            "test1".into(),
            Box::new(crate::SimpleRoute::new("body1", "text/plain")) as Box<dyn RouteEntry>,
        );
        routes.insert(
            "test2".into(),
            Box::new(crate::SimpleRoute::new("body2", "application/json")) as Box<dyn RouteEntry>,
        );

        storage.save(&routes).await.expect("Failed to save routes");

        // 加载路由
        let loaded_routes = storage.load().await.expect("Failed to load routes");

        assert_eq!(loaded_routes.len(), 2);
        assert!(loaded_routes.contains_key("test1"));
        assert!(loaded_routes.contains_key("test2"));
    }

    #[tokio::test]
    async fn test_file_storage_with_custom_route_type() {
        use crate::core::{RouteEntry, RouteRegistry, SerializableRoute};
        use actix_web::{HttpRequest, HttpResponse};
        use std::future::Future;
        use std::pin::Pin;

        // 定义自定义路由类型
        #[derive(Debug, Clone)]
        struct TimedRoute {
            body: String,
            content_type: String,
            timeout_ms: u64,
        }

        impl TimedRoute {
            fn new(body: &str, content_type: &str, timeout_ms: u64) -> Self {
                Self {
                    body: body.to_string(),
                    content_type: content_type.to_string(),
                    timeout_ms,
                }
            }
        }

        impl RouteEntry for TimedRoute {
            fn handle(
                &self,
                _req: &HttpRequest,
            ) -> Pin<Box<dyn Future<Output = HttpResponse> + Send>> {
                let body = self.body.clone();
                let content_type = self.content_type.clone();
                Box::pin(async move { HttpResponse::Ok().content_type(content_type).body(body) })
            }

            fn clone_box(&self) -> Box<dyn RouteEntry> {
                Box::new(self.clone())
            }

            fn to_serializable(&self) -> SerializableRoute {
                let extra_data = serde_json::json!({
                    "timeout_ms": self.timeout_ms
                })
                .to_string();

                SerializableRoute {
                    route_type: "TimedRoute".to_string(),
                    body: self.body.clone(),
                    content_type: self.content_type.clone(),
                    extra_data: Some(extra_data),
                }
            }

            fn from_serializable(data: SerializableRoute) -> Box<dyn RouteEntry>
            where
                Self: Sized,
            {
                let timeout_ms = if let Some(ref extra) = data.extra_data {
                    serde_json::from_str(extra).unwrap_or(1000)
                } else {
                    1000
                };

                Box::new(TimedRoute::new(&data.body, &data.content_type, timeout_ms))
            }
        }

        // 注册自定义路由类型
        let result = RouteRegistry::register("TimedRoute", TimedRoute::from_serializable);
        assert!(result.is_ok());

        let temp_dir = tempdir().unwrap();
        let storage = FileStorage::new(temp_dir.path());

        // 创建并保存自定义路由
        let mut routes = std::collections::HashMap::new();
        routes.insert(
            "timed".into(),
            Box::new(TimedRoute::new("timed response", "text/plain", 5000)) as Box<dyn RouteEntry>,
        );

        storage
            .save(&routes)
            .await
            .expect("Failed to save custom routes");

        // 加载路由
        let loaded_routes = storage.load().await.expect("Failed to load custom routes");
        assert_eq!(loaded_routes.len(), 1);
        assert!(loaded_routes.contains_key("timed"));

        // 验证加载的路由具有正确的类型
        if let Some(route) = loaded_routes.get("timed") {
            let serializable = route.to_serializable();
            assert_eq!(serializable.route_type, "TimedRoute");
        }

        // 清理注册表
        RouteRegistry::unregister("TimedRoute");
    }
}
