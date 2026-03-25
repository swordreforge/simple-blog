//! 文章版本历史服务 - 处理版本历史相关的业务逻辑
//!
//! 第二阶段：文件系统存储
//! - 文件系统操作
//! - 目录结构管理
//! - 文件路径处理

use chrono::Utc;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::cache::AppCache;
use crate::db::models::{Passage, PassageHistorySettings};
use crate::db::repositories::PassageRepository;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// 文章版本历史服务
#[derive(Clone)]
pub struct PassageVersionService {
    version_repo: crate::db::repositories::PassageVersionRepository,
    passage_repo: Arc<PassageRepository>,
    cache: Arc<AppCache>,
}

impl PassageVersionService {
    /// 创建新的文章版本历史服务
    pub fn new(
        version_repo: crate::db::repositories::PassageVersionRepository,
        passage_repo: Arc<PassageRepository>,
        cache: Arc<AppCache>,
    ) -> Self {
        Self {
            version_repo,
            passage_repo,
            cache,
        }
    }

    /// 加载历史版本配置
    pub async fn load_history_config(&self) -> Result<PassageHistorySettings> {
        Ok(PassageHistorySettings::default())
    }

    // ==================== 文件系统操作 ====================

    /// 生成历史文件路径
    ///
    /// 路径格式：{history_dir}/passages/{passage_uuid}/v{version_number}_{timestamp}.md
    pub fn generate_history_file_path(
        &self,
        passage_uuid: &str,
        version_number: i32,
        config: &PassageHistorySettings,
    ) -> Result<PathBuf> {
        let timestamp = Utc::now().format("%Y%m%d-%H%M%S");
        
        let path = PathBuf::from(&config.history_dir)
            .join("passages")
            .join(passage_uuid)
            .join(format!("v{}_{}.md", version_number, timestamp));
        
        Ok(path)
    }

    /// 验证路径安全性 - 防止路径遍历攻击
    pub fn validate_path(&self, path: &Path) -> Result<()> {
        let canonical = path.canonicalize()
            .map_err(|e| format!("路径无效: {}", e))?;
        
        let components: Vec<_> = canonical.components()
            .filter_map(|c| c.as_os_str().to_str())
            .collect();
        
        if components.contains(&"..") {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "路径包含非法字符")));
        }
        
        Ok(())
    }

    /// 规范化路径 - 支持相对路径和绝对路径
    pub fn normalize_path(&self, path: &str, base_dir: Option<&Path>) -> Result<PathBuf> {
        let path = PathBuf::from(path);
        
        if path.is_absolute() {
            return Ok(path);
        }
        
        if let Some(base) = base_dir {
            return Ok(base.join(path));
        }
        
        Ok(std::env::current_dir()?.join(path))
    }

    /// 写入历史文件
    pub async fn write_history_file(&self, path: &Path, content: &str) -> Result<()> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| format!("创建目录失败: {}", e))?;
        }
        
        tokio::fs::write(path, content).await
            .map_err(|e| format!("写入文件失败: {}", e))?;
        
        Ok(())
    }

    /// 读取历史文件
    pub async fn read_history_file(&self, path: &Path) -> Result<String> {
        if !path.exists() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other, 
                format!("文件不存在: {:?}", path)
            )));
        }
        
        self.validate_path(path)?;
        
        let content = tokio::fs::read_to_string(path).await
            .map_err(|e| format!("读取文件失败: {}", e))?;
        Ok(content)
    }

    /// 写入元数据文件
    pub async fn write_metadata_file(
        &self,
        passage_uuid: &str,
        version_number: i32,
        passage: &Passage,
        change_type: &str,
        change_reason: &Option<String>,
        config: &PassageHistorySettings,
    ) -> Result<()> {
        let metadata_path = PathBuf::from(&config.history_dir)
            .join("passages")
            .join(passage_uuid)
            .join(".metadata.json");
        
        let metadata = serde_json::json!({
            "version_number": version_number,
            "title": passage.title,
            "tags": passage.tags,
            "category": passage.category,
            "change_type": change_type,
            "change_reason": change_reason,
            "created_at": Utc::now().to_rfc3339(),
        });
        
        let mut all_metadata: serde_json::Value = if metadata_path.exists() {
            let content = tokio::fs::read_to_string(&metadata_path).await?;
            serde_json::from_str(&content)?
        } else {
            serde_json::json!({})
        };
        
        all_metadata[format!("v{}", version_number)] = metadata;
        
        if let Some(parent) = metadata_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        
        tokio::fs::write(
            &metadata_path,
            serde_json::to_string_pretty(&all_metadata)?
        ).await?;
        
        Ok(())
    }

    /// 计算文件哈希（用于内容去重）
    pub fn compute_file_hash(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    // ==================== 目录结构管理 ====================

    /// 确保目录存在
    pub async fn ensure_directory(&self, path: &Path) -> Result<()> {
        tokio::fs::create_dir_all(path).await
            .map_err(|e| format!("创建目录失败: {}", e))?;
        Ok(())
    }

    /// 清理空目录
    pub async fn cleanup_empty_directories(&self, base_path: &Path) -> Result<u32> {
        let mut removed_count = 0;
        
        // 使用栈来避免递归
        let mut stack: Vec<PathBuf> = vec![base_path.to_path_buf()];
        
        while let Some(current_path) = stack.pop() {
            if !current_path.is_dir() {
                continue;
            }
            
            // 收集所有子目录
            let mut entries = tokio::fs::read_dir(&current_path).await?;
            let mut subdirs = Vec::new();
            
            while let Some(entry) = entries.next_entry().await? {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    subdirs.push(entry_path);
                }
            }
            
            // 先处理子目录
            stack.extend(subdirs);
            
            // 检查当前目录是否为空
            let mut entries = tokio::fs::read_dir(&current_path).await?;
            if entries.next_entry().await?.is_none() {
                tokio::fs::remove_dir(&current_path).await.ok();
                removed_count += 1;
            }
        }
        
        Ok(removed_count)
    }

    /// 删除文章的版本目录
    pub async fn delete_version_directory(
        &self,
        passage_uuid: &str,
        config: &PassageHistorySettings,
    ) -> Result<()> {
        let version_dir = PathBuf::from(&config.history_dir)
            .join("passages")
            .join(passage_uuid);
        
        if version_dir.exists() {
            tokio::fs::remove_dir_all(&version_dir).await
                .map_err(|e| format!("删除版本目录失败: {}", e))?;
        }
        
        // 尝试清理父目录
        let passages_dir = PathBuf::from(&config.history_dir).join("passages");
        if passages_dir.exists() {
            let _ = self.cleanup_empty_directories(&passages_dir).await;
        }
        
        Ok(())
    }

    /// 删除单个历史文件
    pub async fn delete_history_file(&self, path: &Path) -> Result<()> {
        self.validate_path(path)?;
        
        if path.exists() {
            tokio::fs::remove_file(path).await
                .map_err(|e| format!("删除文件失败: {}", e))?;
        }
        
        // 尝试清理空目录
        if let Some(parent) = path.parent() {
            self.cleanup_empty_directories(parent).await.ok();
        }
        
        Ok(())
    }

    /// 获取版本目录大小
    pub async fn get_version_directory_size(
        &self,
        passage_uuid: &str,
        config: &PassageHistorySettings,
    ) -> Result<u64> {
        let version_dir = PathBuf::from(&config.history_dir)
            .join("passages")
            .join(passage_uuid);
        
        if !version_dir.exists() {
            return Ok(0);
        }
        
        self.calc_dir_size_iterative(&version_dir).await
    }

    /// 使用迭代方式计算目录大小（避免递归）
    async fn calc_dir_size_iterative(&self, path: &Path) -> Result<u64> {
        let mut total_size = 0u64;
        let mut stack: Vec<PathBuf> = vec![path.to_path_buf()];
        
        while let Some(current_path) = stack.pop() {
            let mut entries = tokio::fs::read_dir(&current_path).await?;
            
            while let Some(entry) = entries.next_entry().await? {
                let entry_path = entry.path();
                
                if entry_path.is_dir() {
                    stack.push(entry_path);
                } else if entry_path.is_file() {
                    let metadata = tokio::fs::metadata(&entry_path).await?;
                    total_size += metadata.len();
                }
            }
        }
        
        Ok(total_size)
    }

    // ==================== 文件路径处理 ====================

    /// 获取历史文件列表
    pub async fn list_history_files(
        &self,
        passage_uuid: &str,
        config: &PassageHistorySettings,
    ) -> Result<Vec<PathBuf>> {
        let version_dir = PathBuf::from(&config.history_dir)
            .join("passages")
            .join(passage_uuid);
        
        if !version_dir.exists() {
            return Ok(vec![]);
        }
        
        let mut files = Vec::new();
        let mut entries = tokio::fs::read_dir(&version_dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |e| e == "md") {
                files.push(path);
            }
        }
        
        // 按修改时间排序
        files.sort_by(|a, b| {
            let a_meta = std::fs::metadata(a).ok();
            let b_meta = std::fs::metadata(b).ok();
            
            match (a_meta, b_meta) {
                (Some(a), Some(b)) => {
                    let a_time = a.modified().ok();
                    let b_time = b.modified().ok();
                    match (a_time, b_time) {
                        (Some(a), Some(b)) => b.cmp(&a),
                        _ => std::cmp::Ordering::Equal,
                    }
                }
                _ => std::cmp::Ordering::Equal,
            }
        });
        
        Ok(files)
    }

    /// 检查文件是否存在
    #[allow(dead_code)]
    pub fn history_file_exists(&self, path: &Path) -> bool {
        path.exists() && path.is_file()
    }

    /// 获取文件元数据
    #[allow(dead_code)]
    pub async fn get_file_metadata(&self, path: &Path) -> Result<(u64, String)> {
        if !path.exists() {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "文件不存在")));
        }
        
        let metadata = tokio::fs::metadata(path).await
            .map_err(|e| format!("获取文件元数据失败: {}", e))?;
        
        let modified = metadata.modified()
            .map_err(|e| format!("获取修改时间失败: {}", e))?;
        
        let modified_str = chrono::DateTime::<Utc>::from(modified)
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string();
        
        Ok((metadata.len(), modified_str))
    }

    /// 规范化 UUID（处理特殊字符）
    #[allow(dead_code)]
    pub fn sanitize_uuid(&self, uuid: &str) -> String {
        uuid.chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect()
    }

    /// 验证 UUID 合法性
    #[allow(dead_code)]
    pub fn is_valid_uuid(&self, uuid: &str) -> bool {
        !uuid.is_empty() 
            && uuid.len() <= 255 
            && uuid.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sanitize_uuid() {
        let service = PassageVersionService::new(
            crate::db::repositories::PassageVersionRepository::new(
                Arc::new(r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::memory()).unwrap())
            ),
            Arc::new(crate::db::repositories::PassageRepository::new(
                Arc::new(r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::memory()).unwrap())
            )),
            Arc::new(crate::cache::AppCache::new(None)),
        );
        
        assert_eq!(service.sanitize_uuid("abc123-xyz"), "abc123-xyz");
        assert_eq!(service.sanitize_uuid("abc/..\\xyz"), "abcxyz");
        assert_eq!(service.sanitize_uuid(""), "");
    }

    #[test]
    fn test_is_valid_uuid() {
        let service = PassageVersionService::new(
            crate::db::repositories::PassageVersionRepository::new(
                Arc::new(r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::memory()).unwrap())
            ),
            Arc::new(crate::db::repositories::PassageRepository::new(
                Arc::new(r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::memory()).unwrap())
            )),
            Arc::new(crate::cache::AppCache::new(None)),
        );
        
        assert!(service.is_valid_uuid("abc123-xyz"));
        assert!(service.is_valid_uuid("abc_123"));
        assert!(!service.is_valid_uuid(""));
        assert!(!service.is_valid_uuid("abc/xyz"));
        assert!(!service.is_valid_uuid(&"a".repeat(256)));
    }

    #[test]
    fn test_compute_file_hash() {
        let service = PassageVersionService::new(
            crate::db::repositories::PassageVersionRepository::new(
                Arc::new(r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::memory()).unwrap())
            ),
            Arc::new(crate::db::repositories::PassageRepository::new(
                Arc::new(r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::memory()).unwrap())
            )),
            Arc::new(crate::cache::AppCache::new(None)),
        );
        
        let hash1 = service.compute_file_hash("hello world");
        let hash2 = service.compute_file_hash("hello world");
        let hash3 = service.compute_file_hash("hello world!");
        
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}
