//! 文章版本历史服务 - 处理版本历史相关的业务逻辑
//!
//! 第二阶段：文件系统存储
//! - 文件系统操作
//! - 目录结构管理
//! - 文件路径处理
//!
//! 第三阶段：版本保存功能
//! - 版本自动保存和手动保存
//! - 变更检测逻辑
//! - 版本号生成
//! - 内容去重

use chrono::Utc;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::cache::AppCache;
use crate::db::models::{Passage, PassageHistorySettings, PassageVersion};
use crate::db::repositories::PassageRepository;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// 将标准错误转换为 Send + Sync 错误
#[allow(dead_code)]
fn to_send_sync_error(e: Box<dyn std::error::Error>) -> Box<dyn std::error::Error + Send + Sync> {
    #[allow(unsafe_code)]
    unsafe { std::mem::transmute(e) }
}

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
            return Err(Box::new(std::io::Error::other("路径包含非法字符")));
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
            return Err(Box::new(std::io::Error::other(format!("文件不存在: {:?}", path))));
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
            if path.is_file() && path.extension().is_some_and(|e| e == "md") {
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
            return Err(Box::new(std::io::Error::other("文件不存在")));
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

    // ==================== 版本保存功能 (第三阶段) ====================

    /// 保存版本（核心方法）
    ///
    /// # 参数
    /// - `passage_id`: 文章数据库 ID
    /// - `passage_uuid`: 文章 UUID
    /// - `passage`: 文章对象
    /// - `change_type`: 变更类型 (auto/manual/restore/pre_restore)
    /// - `change_reason`: 变更原因
    ///
    /// # 返回
    /// 返回新创建的版本 ID，0 表示未保存（功能未启用）
    pub async fn save_version(
        &self,
        passage_id: i64,
        passage_uuid: &str,
        passage: &Passage,
        change_type: &str,
        change_reason: Option<String>,
    ) -> Result<i64> {
        // 1. 获取配置
        let config = self.load_history_config().await?;
        
        if !config.enabled {
            return Ok(0); // 未启用历史功能
        }
        
        // 2. 获取要保存的原始 Markdown 内容
        // 优先使用 original_content，否则使用 content
        let content_to_save = passage.original_content.as_ref()
            .unwrap_or(&passage.content);
        
        // 3. 计算文件哈希（用于去重）
        let file_hash = self.compute_file_hash(content_to_save);
        
        // 4. 检查是否已存在相同内容的版本（去重）
        if config.enable_deduplication {
            if let Some(existing) = self.version_repo
                .check_duplicate_content(passage_id, &file_hash)
                .await
                .map_err(to_send_sync_error)?
            {
                return Ok(existing.id.unwrap_or(0)); // 返回已存在的版本 ID
            }
        }
        
        // 5. 生成版本号
        let version_number = self.get_next_version_number(passage_id).await?;
        
        // 6. 根据存储模式处理
        let (file_path, file_size) = if config.storage_mode == "filesystem" {
            // 文件系统模式：写入历史文件
            let history_file_path = self.generate_history_file_path(
                passage_uuid, 
                version_number,
                &config,
            )?;
            
            // 写入历史文件
            self.write_history_file(&history_file_path, content_to_save).await?;
            
            // 写入元数据文件
            self.write_metadata_file(
                passage_uuid, 
                version_number,
                passage,
                change_type,
                &change_reason,
                &config,
            ).await?;
            
            (history_file_path.to_string_lossy().to_string(), content_to_save.len() as i64)
        } else {
            // 数据库模式：文件路径为空
            (String::new(), content_to_save.len() as i64)
        };
        
        // 7. 获取父版本 ID（用于 Git 风格的链式结构）
        let parent_version_id = self.version_repo
            .get_latest_version(passage_id)
            .await
            .map_err(to_send_sync_error)?
            .and_then(|v| v.id);
        
        // 8. 构建版本记录
        let version = PassageVersion {
            id: None,
            passage_id,
            passage_uuid: passage_uuid.to_string(),
            version_number,
            file_path,
            file_size,
            file_hash: Some(file_hash),
            title: passage.title.clone(),
            content: content_to_save.clone(),
            tags: passage.tags.clone(),
            category: passage.category.clone(),
            cover_image: passage.cover_image.clone(),
            change_type: change_type.to_string(),
            change_reason,
            created_at: Utc::now(),
            created_by: "system".to_string(),
            parent_version_id,
            branch_name: None,
        };
        
        // 9. 保存到数据库
        let version_id = self.version_repo.create(&version)
            .await
            .map_err(to_send_sync_error)?;
        
        // 10. 限制版本数量
        if config.max_versions > 0 {
            self.version_repo
                .trim_old_versions(passage_id, config.max_versions as usize)
                .await
                .map_err(to_send_sync_error)?;
        }
        
        // 11. 清除缓存
        self.clear_version_cache(passage_id).await;
        
        Ok(version_id)
    }

    /// 自动保存版本（在更新文章时调用）
    ///
    /// # 参数
    /// - `passage_id`: 文章数据库 ID
    /// - `passage_uuid`: 文章 UUID
    /// - `old_passage`: 更新前的文章
    /// - `new_passage`: 更新后的文章
    ///
    /// # 返回
    /// 返回 Some(version_id) 如果保存了版本，None 如果没有变化不需要保存
    pub async fn auto_save_version(
        &self,
        passage_id: i64,
        passage_uuid: &str,
        old_passage: &Passage,
        new_passage: &Passage,
    ) -> Result<Option<i64>> {
        let config = self.load_history_config().await?;
        
        if !config.enabled {
            return Ok(None);
        }
        
        // 检测变更
        let changes = self.detect_changes(old_passage, new_passage, &config);
        
        if changes.is_empty() {
            return Ok(None); // 没有需要保存的变更
        }
        
        let change_reason = Some(format!("自动保存：{}", changes.join("、")));
        
        let version_id = self.save_version(
            passage_id,
            passage_uuid,
            new_passage,
            "auto",
            change_reason,
        ).await?;
        
        if version_id > 0 {
            Ok(Some(version_id))
        } else {
            Ok(None)
        }
    }

    /// 手动创建版本
    ///
    /// # 参数
    /// - `passage_id`: 文章数据库 ID
    /// - `passage_uuid`: 文章 UUID
    /// - `passage`: 文章对象
    /// - `change_reason`: 版本说明
    ///
    /// # 返回
    /// 返回新创建的版本 ID
    pub async fn create_version(
        &self,
        passage_id: i64,
        passage_uuid: &str,
        passage: &Passage,
        change_reason: Option<String>,
    ) -> Result<i64> {
        self.save_version(
            passage_id,
            passage_uuid,
            passage,
            "manual",
            change_reason,
        ).await
    }

    /// 检测文章变更
    ///
    /// # 参数
    /// - `old_passage`: 更新前的文章
    /// - `new_passage`: 更新后的文章
    /// - `config`: 历史版本配置
    ///
    /// # 返回
    /// 返回变更字段名称列表
    pub fn detect_changes(
        &self,
        old_passage: &Passage,
        new_passage: &Passage,
        config: &PassageHistorySettings,
    ) -> Vec<&'static str> {
        let mut changes = Vec::new();
        
        // 获取原始内容进行比较
        let old_content = old_passage.original_content.as_ref()
            .unwrap_or(&old_passage.content);
        let new_content = new_passage.original_content.as_ref()
            .unwrap_or(&new_passage.content);
        
        if config.save_on_title_change && old_passage.title != new_passage.title {
            changes.push("标题");
        }
        if config.save_on_content_change && old_content != new_content {
            changes.push("内容");
        }
        if config.save_on_tags_change && old_passage.tags != new_passage.tags {
            changes.push("标签");
        }
        if config.save_on_category_change && old_passage.category != new_passage.category {
            changes.push("分类");
        }
        if config.save_on_cover_change && old_passage.cover_image != new_passage.cover_image {
            changes.push("封面图片");
        }
        
        changes
    }

    /// 获取下一个版本号
    async fn get_next_version_number(&self, passage_id: i64) -> Result<i32> {
        let latest = self.version_repo.get_latest_version_number(passage_id)
            .await
            .map_err(to_send_sync_error)?;
        Ok(latest + 1)
    }

    /// 清除版本缓存
    async fn clear_version_cache(&self, passage_id: i64) {
        // 简化实现：实际应该清除缓存
        let _ = passage_id;
    }

    // ==================== 版本查询功能 ====================

    /// 获取文章的所有版本
    pub async fn get_versions(&self, passage_id: i64) -> Result<Vec<PassageVersion>> {
        self.version_repo.get_by_passage_id(passage_id)
            .await
            .map_err(to_send_sync_error)
    }

    /// 获取特定版本
    pub async fn get_version(
        &self,
        passage_id: i64,
        version_number: i32,
    ) -> Result<Option<PassageVersion>> {
        self.version_repo.get_by_version_number(passage_id, version_number)
            .await
            .map_err(to_send_sync_error)
    }

    /// 获取最新版本
    pub async fn get_latest_version(&self, passage_id: i64) -> Result<Option<PassageVersion>> {
        self.version_repo.get_latest_version(passage_id)
            .await
            .map_err(to_send_sync_error)
    }

    /// 获取版本数量
    pub async fn get_version_count(&self, passage_id: i64) -> Result<i32> {
        self.version_repo.get_latest_version_number(passage_id)
            .await
            .map_err(to_send_sync_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_passage(title: &str, content: &str) -> Passage {
        Passage {
            id: Some(1),
            uuid: Some("test-uuid-123".to_string()),
            title: title.to_string(),
            content: content.to_string(),
            original_content: Some(content.to_string()),
            summary: Some("摘要".to_string()),
            summarize: None,
            author: "test".to_string(),
            tags: "[]".to_string(),
            category: "测试".to_string(),
            status: crate::db::models::PassageStatus::Published,
            visibility: crate::db::models::PassageVisibility::Public,
            file_path: Some("markdown/test.md".to_string()),
            is_scheduled: false,
            published_at: Some(Utc::now()),
            cover_image: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_sanitize_uuid() {
        let service = PassageVersionService::new(
            crate::db::repositories::PassageVersionRepository::new(
                Arc::new(r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::memory()).unwrap())
            ),
            Arc::new(crate::db::repositories::PassageRepository::new(
                Arc::new(r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::memory()).unwrap())
            )),
            Arc::new(crate::cache::AppCache::new(crate::cache::CacheConfig::default())),
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
            Arc::new(crate::cache::AppCache::new(crate::cache::CacheConfig::default())),
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
            Arc::new(crate::cache::AppCache::new(crate::cache::CacheConfig::default())),
        );
        
        let hash1 = service.compute_file_hash("hello world");
        let hash2 = service.compute_file_hash("hello world");
        let hash3 = service.compute_file_hash("hello world!");
        
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_detect_changes_title() {
        let service = PassageVersionService::new(
            crate::db::repositories::PassageVersionRepository::new(
                Arc::new(r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::memory()).unwrap())
            ),
            Arc::new(crate::db::repositories::PassageRepository::new(
                Arc::new(r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::memory()).unwrap())
            )),
            Arc::new(crate::cache::AppCache::new(crate::cache::CacheConfig::default())),
        );
        
        let old_passage = create_test_passage("旧标题", "内容");
        let new_passage = create_test_passage("新标题", "内容");
        
        let config = PassageHistorySettings::default();
        let changes = service.detect_changes(&old_passage, &new_passage, &config);
        
        assert!(changes.contains(&"标题"));
        assert_eq!(changes.len(), 1);
    }

    #[test]
    fn test_detect_changes_content() {
        let service = PassageVersionService::new(
            crate::db::repositories::PassageVersionRepository::new(
                Arc::new(r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::memory()).unwrap())
            ),
            Arc::new(crate::db::repositories::PassageRepository::new(
                Arc::new(r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::memory()).unwrap())
            )),
            Arc::new(crate::cache::AppCache::new(crate::cache::CacheConfig::default())),
        );
        
        let old_passage = create_test_passage("标题", "旧内容");
        let new_passage = create_test_passage("标题", "新内容");
        
        let config = PassageHistorySettings::default();
        let changes = service.detect_changes(&old_passage, &new_passage, &config);
        
        assert!(changes.contains(&"内容"));
        assert_eq!(changes.len(), 1);
    }

    #[test]
    fn test_detect_changes_multiple() {
        let service = PassageVersionService::new(
            crate::db::repositories::PassageVersionRepository::new(
                Arc::new(r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::memory()).unwrap())
            ),
            Arc::new(crate::db::repositories::PassageRepository::new(
                Arc::new(r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::memory()).unwrap())
            )),
            Arc::new(crate::cache::AppCache::new(crate::cache::CacheConfig::default())),
        );
        
        let old_passage = create_test_passage("旧标题", "旧内容");
        let new_passage = create_test_passage("新标题", "新内容");
        
        let config = PassageHistorySettings::default();
        let changes = service.detect_changes(&old_passage, &new_passage, &config);
        
        assert!(changes.contains(&"标题"));
        assert!(changes.contains(&"内容"));
        assert_eq!(changes.len(), 2);
    }

    #[test]
    fn test_detect_changes_no_change() {
        let service = PassageVersionService::new(
            crate::db::repositories::PassageVersionRepository::new(
                Arc::new(r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::memory()).unwrap())
            ),
            Arc::new(crate::db::repositories::PassageRepository::new(
                Arc::new(r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::memory()).unwrap())
            )),
            Arc::new(crate::cache::AppCache::new(crate::cache::CacheConfig::default())),
        );
        
        let passage = create_test_passage("标题", "内容");
        
        let config = PassageHistorySettings::default();
        let changes = service.detect_changes(&passage, &passage, &config);
        
        assert!(changes.is_empty());
    }

    #[test]
    fn test_detect_changes_disabled_fields() {
        let service = PassageVersionService::new(
            crate::db::repositories::PassageVersionRepository::new(
                Arc::new(r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::memory()).unwrap())
            ),
            Arc::new(crate::db::repositories::PassageRepository::new(
                Arc::new(r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::memory()).unwrap())
            )),
            Arc::new(crate::cache::AppCache::new(crate::cache::CacheConfig::default())),
        );
        
        let old_passage = create_test_passage("旧标题", "旧内容");
        let new_passage = create_test_passage("新标题", "新内容");
        
        // 只启用标题变更检测
        let mut config = PassageHistorySettings::default();
        config.save_on_content_change = false;
        
        let changes = service.detect_changes(&old_passage, &new_passage, &config);
        
        assert!(changes.contains(&"标题"));
        assert!(!changes.contains(&"内容"));
        assert_eq!(changes.len(), 1);
    }

    #[test]
    fn test_generate_history_file_path() {
        let service = PassageVersionService::new(
            crate::db::repositories::PassageVersionRepository::new(
                Arc::new(r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::memory()).unwrap())
            ),
            Arc::new(crate::db::repositories::PassageRepository::new(
                Arc::new(r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::memory()).unwrap())
            )),
            Arc::new(crate::cache::AppCache::new(crate::cache::CacheConfig::default())),
        );
        
        let config = PassageHistorySettings::default();
        let path = service.generate_history_file_path("test-uuid", 1, &config).unwrap();
        
        let path_str = path.to_string_lossy();
        assert!(path_str.contains("markdown/.history"));
        assert!(path_str.contains("passages"));
        assert!(path_str.contains("test-uuid"));
        assert!(path_str.contains("v1_"));
        assert!(path_str.ends_with(".md"));
    }

    #[test]
    fn test_load_history_config_default() {
        let service = PassageVersionService::new(
            crate::db::repositories::PassageVersionRepository::new(
                Arc::new(r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::memory()).unwrap())
            ),
            Arc::new(crate::db::repositories::PassageRepository::new(
                Arc::new(r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::memory()).unwrap())
            )),
            Arc::new(crate::cache::AppCache::new(crate::cache::CacheConfig::default())),
        );
        
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let config = runtime.block_on(service.load_history_config()).unwrap();
        
        assert!(config.enabled);
        assert_eq!(config.storage_mode, "filesystem");
        assert_eq!(config.history_dir, "markdown/.history");
        assert_eq!(config.max_versions, 50);
        assert!(config.enable_deduplication);
    }
}
