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
//!
//! 第四阶段：版本查询 API
//! - 分页查询支持
//! - 排序支持
//! - 缓存支持
//! - 公共 API
//!
//! 第五阶段：版本差异对比
//! - Diff 算法实现
//! - 版本对比 API
//! - 差异可视化支持

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use similar::{ChangeTag, TextDiff};
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

// ==================== 第四阶段：版本查询相关结构 ====================

/// 版本列表排序字段
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum VersionSortField {
    #[default]
    VersionNumber,
    CreatedAt,
    Title,
}

/// 版本列表排序方向
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SortOrder {
    #[default]
    Desc,
    Asc,
}

/// 版本列表查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionListQuery {
    /// 文章 ID
    pub passage_id: i64,
    /// 页码（从 1 开始）
    pub page: Option<i32>,
    /// 每页数量
    pub page_size: Option<i32>,
    /// 排序字段
    pub sort_by: Option<VersionSortField>,
    /// 排序方向
    pub sort_order: Option<SortOrder>,
    /// 变更类型过滤
    pub change_type: Option<String>,
    /// 标题搜索
    pub search_title: Option<String>,
}

impl Default for VersionListQuery {
    fn default() -> Self {
        Self {
            passage_id: 0,
            page: Some(1),
            page_size: Some(20),
            sort_by: Some(VersionSortField::VersionNumber),
            sort_order: Some(SortOrder::Desc),
            change_type: None,
            search_title: None,
        }
    }
}

impl VersionListQuery {
    /// 获取页码（默认为 1）
    pub fn get_page(&self) -> i32 {
        self.page.unwrap_or(1).max(1)
    }

    /// 获取每页数量（默认 20，最大 100）
    pub fn get_page_size(&self) -> i32 {
        self.page_size.unwrap_or(20).max(1).min(100)
    }

    /// 获取排序字段（默认为版本号）
    pub fn get_sort_by(&self) -> VersionSortField {
        self.sort_by.unwrap_or(VersionSortField::VersionNumber)
    }

    /// 获取排序方向（默认为降序）
    pub fn get_sort_order(&self) -> SortOrder {
        self.sort_order.unwrap_or(SortOrder::Desc)
    }
}

/// 版本列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionListResponse {
    /// 版本列表
    pub versions: Vec<PassageVersion>,
    /// 总数
    pub total: i64,
    /// 当前页码
    pub page: i32,
    /// 每页数量
    pub page_size: i32,
    /// 总页数
    pub total_pages: i32,
}

/// 缓存键生成
fn get_version_list_cache_key(passage_id: i64, query: &VersionListQuery) -> String {
    format!(
        "version_list:{}:p{}_s{}_{}_{}",
        passage_id,
        query.get_page(),
        query.get_page_size(),
        query.get_sort_by() as i32,
        query.get_sort_order() as i32
    )
}

fn get_version_count_cache_key(passage_id: i64) -> String {
    format!("version_count:{}", passage_id)
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
        self.clear_version_cache_internal(passage_id).await;
        
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

    /// 清除版本缓存（内部使用）
    async fn clear_version_cache_internal(&self, passage_id: i64) {
        self.clear_version_cache(passage_id).await;
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

    // ==================== 第四阶段：版本查询 API ====================

    /// 列出版本（分页、排序、过滤）
    ///
    /// # 参数
    /// - query: 查询参数
    ///
    /// # 返回
    /// 返回分页的版本列表
    pub async fn list_versions(&self, query: VersionListQuery) -> Result<VersionListResponse> {
        // 1. 构建排序字段
        let sort_by = match query.get_sort_by() {
            VersionSortField::VersionNumber => "version_number",
            VersionSortField::CreatedAt => "created_at",
            VersionSortField::Title => "title",
        };
        
        let sort_order = match query.get_sort_order() {
            SortOrder::Asc => "asc",
            SortOrder::Desc => "desc",
        };

        // 2. 计算分页偏移量
        let page = query.get_page();
        let page_size = query.get_page_size();
        let offset = (page - 1) as i64 * page_size as i64;
        let limit = page_size as i64;

        // 3. 尝试从缓存获取
        let cache_key = get_version_list_cache_key(query.passage_id, &query);
        if let Some(manager) = self.cache.manager() {
            if let Some(cached_str) = manager.get(&cache_key).await {
                if let Ok(cached) = serde_json::from_str::<VersionListResponse>(&cached_str) {
                    return Ok(cached);
                }
            }
        }

        // 4. 从数据库查询
        let versions = self.version_repo
            .list_versions(
                query.passage_id,
                offset,
                limit,
                sort_by,
                sort_order,
                query.change_type.as_deref(),
                query.search_title.as_deref(),
            )
            .await
            .map_err(to_send_sync_error)?;

        // 5. 获取总数
        let total = self.version_repo
            .get_version_count(query.passage_id)
            .await
            .map_err(to_send_sync_error)?;

        // 6. 计算总页数
        let total_pages = ((total as f64) / (page_size as f64)).ceil() as i32;

        // 7. 构建响应
        let response = VersionListResponse {
            versions,
            total,
            page,
            page_size,
            total_pages,
        };

        // 8. 缓存结果（5分钟 TTL）
        if let Some(manager) = self.cache.manager() {
            if let Ok(json_str) = serde_json::to_string(&response) {
                let _ = manager.set(&cache_key, &json_str).await;
            }
        }

        Ok(response)
    }

    /// 获取版本数量（带缓存）
    ///
    /// # 参数
    /// - passage_id: 文章 ID
    ///
    /// # 返回
    /// 返回版本总数
    pub async fn get_version_count_cached(&self, passage_id: i64) -> Result<i64> {
        // 1. 尝试从缓存获取
        let cache_key = get_version_count_cache_key(passage_id);
        if let Some(manager) = self.cache.manager() {
            if let Some(cached_str) = manager.get(&cache_key).await {
                if let Ok(count) = cached_str.parse::<i64>() {
                    return Ok(count);
                }
            }
        }

        // 2. 从数据库查询
        let count = self.version_repo
            .get_version_count(passage_id)
            .await
            .map_err(to_send_sync_error)?;

        // 3. 缓存结果（5分钟 TTL）
        if let Some(manager) = self.cache.manager() {
            let _ = manager.set(&cache_key, &count.to_string()).await;
        }

        Ok(count)
    }

    /// 获取版本简要信息列表（不包含内容，用于列表展示）
    ///
    /// # 参数
    /// - passage_id: 文章 ID
    ///
    /// # 返回
    /// 返回版本简要信息列表
    pub async fn get_version_summaries(&self, passage_id: i64) -> Result<Vec<VersionSummary>> {
        let versions = self.get_versions(passage_id).await?;
        
        Ok(versions.into_iter().map(|v| VersionSummary {
            id: v.id,
            passage_id: v.passage_id,
            version_number: v.version_number,
            title: v.title,
            change_type: v.change_type,
            change_reason: v.change_reason,
            created_at: v.created_at,
            created_by: v.created_by,
            file_size: v.file_size,
        }).collect())
    }

    /// 检查文章是否有历史版本
    ///
    /// # 参数
    /// - passage_id: 文章 ID
    ///
    /// # 返回
    /// 如果有历史版本返回 true，否则返回 false
    pub async fn has_versions(&self, passage_id: i64) -> Result<bool> {
        let count = self.get_version_count_cached(passage_id).await?;
        Ok(count > 0)
    }

    /// 清除版本缓存
    ///
    /// # 参数
    /// - passage_id: 文章 ID
    pub async fn clear_version_cache(&self, passage_id: i64) {
        // 清除版本列表缓存（使用通配符模式）
        if let Some(manager) = self.cache.manager() {
            let _ = manager.delete_pattern(&format!("version_list:{}:*", passage_id)).await;
            // 清除版本数量缓存
            let _ = manager.delete(&get_version_count_cache_key(passage_id)).await;
        }
    }
}

/// 版本简要信息（不包含内容）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionSummary {
    pub id: Option<i64>,
    pub passage_id: i64,
    pub version_number: i32,
    pub title: String,
    pub change_type: String,
    pub change_reason: Option<String>,
    pub created_at: chrono::DateTime<Utc>,
    pub created_by: String,
    pub file_size: i64,
}

// ==================== 第五阶段：版本差异对比 ====================

/// 差异类型
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffType {
    /// 行级差异
    Line,
    /// 字符级差异
    Char,
}

impl Default for DiffType {
    fn default() -> Self {
        DiffType::Line
    }
}

/// 差异输出格式
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffFormat {
    /// 统一格式
    Unified,
    /// 上下文格式
    Context,
    /// 侧边格式
    SideBySide,
}

impl Default for DiffFormat {
    fn default() -> Self {
        DiffFormat::Unified
    }
}

/// 单行差异
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffLine {
    /// 行号（旧版本）
    pub old_line_number: Option<i32>,
    /// 行号（新版本）
    pub new_line_number: Option<i32>,
    /// 行内容
    pub content: String,
    /// 行类型
    pub line_type: DiffLineType,
}

/// 单行差异类型
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffLineType {
    /// 上下文（未变化）
    Context,
    /// 添加
    Added,
    /// 删除
    Deleted,
    /// 修改
    Modified,
}

/// 字段差异详情
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldDiffDetail {
    /// 字段名
    pub field_name: String,
    /// 旧值
    pub old_value: String,
    /// 新值
    pub new_value: String,
    /// 是否变化
    pub changed: bool,
    /// 行级差异列表
    pub line_diffs: Vec<DiffLine>,
}

/// 版本差异响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionDiffResponse {
    /// 旧版本号
    pub from_version: i32,
    /// 新版本号
    pub to_version: i32,
    /// 字段差异列表
    pub field_diffs: Vec<FieldDiffDetail>,
    /// 总变更行数
    pub total_changes: i32,
    /// 变更统计
    pub stats: DiffStats,
}

/// 差异统计
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffStats {
    /// 添加的行数
    pub added: i32,
    /// 删除的行数
    pub deleted: i32,
    /// 修改的行数
    pub modified: i32,
    /// 未变化的行数
    pub unchanged: i32,
}

impl DiffStats {
    /// 从差异行计算统计信息
    fn from_lines(old_text: &str, new_text: &str) -> Self {
        let diff = TextDiff::from_lines(old_text, new_text);
        let mut added = 0;
        let mut deleted = 0;
        let mut unchanged = 0;

        for change in diff.iter_all_changes() {
            match change.tag() {
                ChangeTag::Delete => deleted += 1,
                ChangeTag::Insert => added += 1,
                ChangeTag::Equal => unchanged += 1,
            }
        }

        Self {
            added,
            deleted,
            modified: 0,
            unchanged,
        }
    }
}

/// 版本对比查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionDiffQuery {
    /// 文章 ID
    pub passage_id: i64,
    /// 旧版本号
    pub from_version: i32,
    /// 新版本号
    pub to_version: i32,
    /// 差异类型
    pub diff_type: Option<DiffType>,
    /// 输出格式
    pub format: Option<DiffFormat>,
    /// 是否包含所有字段（否则只包含内容）
    pub include_all_fields: Option<bool>,
}

impl PassageVersionService {
    /// 计算两个文本之间的行级差异
    ///
    /// # 参数
    /// - old_text: 旧文本
    /// - new_text: 新文本
    ///
    /// # 返回
    /// 返回差异行列表
    pub fn compute_line_diff(&self, old_text: &str, new_text: &str) -> Vec<DiffLine> {
        let diff = TextDiff::from_lines(old_text, new_text);
        let mut lines = Vec::new();
        let mut old_line_num = 1;
        let mut new_line_num = 1;

        for change in diff.iter_all_changes() {
            let line_type = match change.tag() {
                ChangeTag::Delete => {
                    old_line_num += 1;
                    DiffLineType::Deleted
                }
                ChangeTag::Insert => {
                    new_line_num += 1;
                    DiffLineType::Added
                }
                ChangeTag::Equal => {
                    old_line_num += 1;
                    new_line_num += 1;
                    DiffLineType::Context
                }
            };

            lines.push(DiffLine {
                old_line_number: if line_type == DiffLineType::Deleted || line_type == DiffLineType::Context {
                    Some(old_line_num - 1)
                } else {
                    None
                },
                new_line_number: if line_type == DiffLineType::Added || line_type == DiffLineType::Context {
                    Some(new_line_num - 1)
                } else {
                    None
                },
                content: change.value().to_string(),
                line_type,
            });
        }

        lines
    }

    /// 计算两个文本之间的字符级差异（单词级别，更适合代码）
    ///
    /// # 参数
    /// - old_text: 旧文本
    /// - new_text: 新文本
    ///
    /// # 返回
    /// 返回统一格式的差异字符串
    pub fn compute_word_diff(&self, old_text: &str, new_text: &str) -> String {
        let diff = TextDiff::from_words(old_text, new_text);
        diff.unified_diff()
            .context_radius(3)
            .header("old", "new")
            .to_string()
    }

    /// 计算统一格式的差异
    ///
    /// # 参数
    /// - old_text: 旧文本
    /// - new_text: 新文本
    /// - context_lines: 上下文行数
    ///
    /// # 返回
    /// 返回统一格式的差异字符串
    pub fn compute_unified_diff(&self, old_text: &str, new_text: &str, context_lines: usize) -> String {
        let diff = TextDiff::from_lines(old_text, new_text);
        diff.unified_diff()
            .context_radius(context_lines)
            .header("old", "new")
            .to_string()
    }

    /// 对比两个版本
    ///
    /// # 参数
    /// - query: 版本对比查询参数
    ///
    /// # 返回
    /// 返回版本差异响应
    pub async fn diff_versions(&self, query: VersionDiffQuery) -> Result<VersionDiffResponse> {
        // 1. 获取两个版本
        let from_version = self.version_repo
            .get_by_version_number(query.passage_id, query.from_version)
            .await
            .map_err(to_send_sync_error)?
            .ok_or_else(|| format!("版本 {} 不存在", query.from_version))?;

        let to_version = self.version_repo
            .get_by_version_number(query.passage_id, query.to_version)
            .await
            .map_err(to_send_sync_error)?
            .ok_or_else(|| format!("版本 {} 不存在", query.to_version))?;

        // 2. 确定是否包含所有字段
        let include_all = query.include_all_fields.unwrap_or(false);

        // 3. 构建字段差异列表
        let mut field_diffs = Vec::new();
        let mut total_changes = 0;

        // 内容差异（始终包含）
        let content_diffs = self.compute_line_diff(&from_version.content, &to_version.content);
        let added_lines = content_diffs.iter().filter(|l| l.line_type == DiffLineType::Added).count() as i32;
        let deleted_lines = content_diffs.iter().filter(|l| l.line_type == DiffLineType::Deleted).count() as i32;
        
        field_diffs.push(FieldDiffDetail {
            field_name: "content".to_string(),
            old_value: from_version.content.clone(),
            new_value: to_version.content.clone(),
            changed: from_version.content != to_version.content,
            line_diffs: content_diffs,
        });
        total_changes += added_lines + deleted_lines;

        // 其他字段差异（可选）
        if include_all {
            // 标题差异
            if from_version.title != to_version.title {
                field_diffs.push(FieldDiffDetail {
                    field_name: "title".to_string(),
                    old_value: from_version.title.clone(),
                    new_value: to_version.title.clone(),
                    changed: true,
                    line_diffs: vec![],
                });
                total_changes += 1;
            }

            // 标签差异
            if from_version.tags != to_version.tags {
                field_diffs.push(FieldDiffDetail {
                    field_name: "tags".to_string(),
                    old_value: from_version.tags.clone(),
                    new_value: to_version.tags.clone(),
                    changed: true,
                    line_diffs: vec![],
                });
                total_changes += 1;
            }

            // 分类差异
            if from_version.category != to_version.category {
                field_diffs.push(FieldDiffDetail {
                    field_name: "category".to_string(),
                    old_value: from_version.category.clone(),
                    new_value: to_version.category.clone(),
                    changed: true,
                    line_diffs: vec![],
                });
                total_changes += 1;
            }

            // 封面图片差异
            if from_version.cover_image != to_version.cover_image {
                let old_cover = from_version.cover_image.clone().unwrap_or_default();
                let new_cover = to_version.cover_image.clone().unwrap_or_default();
                field_diffs.push(FieldDiffDetail {
                    field_name: "cover_image".to_string(),
                    old_value: old_cover,
                    new_value: new_cover,
                    changed: true,
                    line_diffs: vec![],
                });
                total_changes += 1;
            }
        }

        // 4. 计算统计信息
        let stats = DiffStats {
            added: added_lines,
            deleted: deleted_lines,
            modified: 0,
            unchanged: 0,
        };

        Ok(VersionDiffResponse {
            from_version: query.from_version,
            to_version: query.to_version,
            field_diffs,
            total_changes,
            stats,
        })
    }

    /// 获取版本的快速差异摘要（不包含完整内容）
    ///
    /// # 参数
    /// - passage_id: 文章 ID
    /// - from_version: 旧版本号
    /// - to_version: 新版本号
    ///
    /// # 返回
    /// 返回差异摘要信息
    pub async fn get_diff_summary(
        &self,
        passage_id: i64,
        from_version: i32,
        to_version: i32,
    ) -> Result<DiffStats> {
        // 获取两个版本
        let from_version = self.version_repo
            .get_by_version_number(passage_id, from_version)
            .await
            .map_err(to_send_sync_error)?
            .ok_or_else(|| format!("版本 {} 不存在", from_version))?;

        let to_version = self.version_repo
            .get_by_version_number(passage_id, to_version)
            .await
            .map_err(to_send_sync_error)?
            .ok_or_else(|| format!("版本 {} 不存在", to_version))?;

        // 计算内容差异统计
        let stats = DiffStats::from_lines(&from_version.content, &to_version.content);

        Ok(stats)
    }

    /// 检查两个版本是否有差异
    ///
    /// # 参数
    /// - passage_id: 文章 ID
    /// - version_a: 版本 A
    /// - version_b: 版本 B
    ///
    /// # 返回
    /// 如果有差异返回 true，否则返回 false
    pub async fn versions_have_diff(
        &self,
        passage_id: i64,
        version_a: i32,
        version_b: i32,
    ) -> Result<bool> {
        let version_a = self.version_repo
            .get_by_version_number(passage_id, version_a)
            .await
            .map_err(to_send_sync_error)?
            .ok_or_else(|| format!("版本 {} 不存在", version_a))?;

        let version_b = self.version_repo
            .get_by_version_number(passage_id, version_b)
            .await
            .map_err(to_send_sync_error)?
            .ok_or_else(|| format!("版本 {} 不存在", version_b))?;

        Ok(version_a.content != version_b.content
            || version_a.title != version_b.title
            || version_a.tags != version_b.tags
            || version_a.category != version_b.category
            || version_a.cover_image != version_b.cover_image)
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

    // ==================== 第四阶段：版本查询 API 测试 ====================

    #[test]
    fn test_version_list_query_defaults() {
        let query = VersionListQuery {
            passage_id: 1,
            ..Default::default()
        };
        
        assert_eq!(query.get_page(), 1);
        assert_eq!(query.get_page_size(), 20);
        assert_eq!(query.get_sort_by(), VersionSortField::VersionNumber);
        assert_eq!(query.get_sort_order(), SortOrder::Desc);
    }

    #[test]
    fn test_version_list_query_custom() {
        let query = VersionListQuery {
            passage_id: 1,
            page: Some(3),
            page_size: Some(50),
            sort_by: Some(VersionSortField::CreatedAt),
            sort_order: Some(SortOrder::Asc),
            change_type: Some("auto".to_string()),
            search_title: Some("测试".to_string()),
        };
        
        assert_eq!(query.get_page(), 3);
        assert_eq!(query.get_page_size(), 50);
        assert_eq!(query.get_sort_by(), VersionSortField::CreatedAt);
        assert_eq!(query.get_sort_order(), SortOrder::Asc);
        assert_eq!(query.change_type, Some("auto".to_string()));
        assert_eq!(query.search_title, Some("测试".to_string()));
    }

    #[test]
    fn test_version_list_query_page_validation() {
        let query = VersionListQuery {
            passage_id: 1,
            page: Some(0), // 无效页码
            page_size: Some(200), // 超过最大值
            ..Default::default()
        };
        
        assert_eq!(query.get_page(), 1); // 最小值为 1
        assert_eq!(query.get_page_size(), 100); // 最大值为 100
    }

    #[test]
    fn test_cache_key_generation() {
        let query = VersionListQuery {
            passage_id: 123,
            page: Some(2),
            page_size: Some(10),
            sort_by: Some(VersionSortField::CreatedAt),
            sort_order: Some(SortOrder::Asc),
            ..Default::default()
        };
        
        let key = get_version_list_cache_key(123, &query);
        assert!(key.contains("version_list:123:"));
        assert!(key.contains("p2"));
        assert!(key.contains("s10"));
    }

    #[test]
    fn test_version_count_cache_key() {
        let key = get_version_count_cache_key(456);
        assert_eq!(key, "version_count:456");
    }

    #[test]
    fn test_version_sort_field_serialization() {
        let json = serde_json::to_string(&VersionSortField::CreatedAt).unwrap();
        assert!(json.contains("created_at"));
        
        let deserialized: VersionSortField = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, VersionSortField::CreatedAt);
    }

    #[test]
    fn test_sort_order_serialization() {
        let json = serde_json::to_string(&SortOrder::Asc).unwrap();
        assert!(json.contains("asc"));
        
        let deserialized: SortOrder = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, SortOrder::Asc);
    }

    #[test]
    fn test_version_summary_creation() {
        let version = PassageVersion {
            id: Some(1),
            passage_id: 100,
            passage_uuid: "uuid-123".to_string(),
            version_number: 5,
            file_path: "/path/to/file.md".to_string(),
            file_size: 1024,
            file_hash: Some("abc123".to_string()),
            title: "测试标题".to_string(),
            content: "# 内容".to_string(),
            tags: "[]".to_string(),
            category: "测试分类".to_string(),
            cover_image: None,
            change_type: "auto".to_string(),
            change_reason: Some("自动保存".to_string()),
            created_at: Utc::now(),
            created_by: "system".to_string(),
            parent_version_id: Some(4),
            branch_name: None,
        };
        
        let summary = VersionSummary {
            id: version.id,
            passage_id: version.passage_id,
            version_number: version.version_number,
            title: version.title.clone(),
            change_type: version.change_type.clone(),
            change_reason: version.change_reason.clone(),
            created_at: version.created_at,
            created_by: version.created_by.clone(),
            file_size: version.file_size,
        };
        
        assert_eq!(summary.id, Some(1));
        assert_eq!(summary.version_number, 5);
        assert_eq!(summary.title, "测试标题");
    }

    // ==================== 第五阶段：版本差异对比测试 ====================

    #[test]
    fn test_compute_line_diff() {
        let service = PassageVersionService::new(
            crate::db::repositories::PassageVersionRepository::new(
                Arc::new(r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::memory()).unwrap())
            ),
            Arc::new(crate::db::repositories::PassageRepository::new(
                Arc::new(r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::memory()).unwrap())
            )),
            Arc::new(crate::cache::AppCache::new(crate::cache::CacheConfig::default())),
        );
        
        let old_text = "第一行\n第二行\n第三行\n";
        let new_text = "第一行\n修改的行\n第三行\n第四行\n";
        
        let diff = service.compute_line_diff(old_text, new_text);
        
        assert!(!diff.is_empty());
        // 应该包含修改和新增的行
        assert!(diff.iter().any(|l| l.line_type == DiffLineType::Modified || l.line_type == DiffLineType::Deleted));
    }

    #[test]
    fn test_compute_line_diff_no_change() {
        let service = PassageVersionService::new(
            crate::db::repositories::PassageVersionRepository::new(
                Arc::new(r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::memory()).unwrap())
            ),
            Arc::new(crate::db::repositories::PassageRepository::new(
                Arc::new(r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::memory()).unwrap())
            )),
            Arc::new(crate::cache::AppCache::new(crate::cache::CacheConfig::default())),
        );
        
        let text = "第一行\n第二行\n第三行\n";
        
        let diff = service.compute_line_diff(text, text);
        
        // 所有行应该是上下文（未变化）
        assert!(diff.iter().all(|l| l.line_type == DiffLineType::Context));
    }

    #[test]
    fn test_compute_word_diff() {
        let service = PassageVersionService::new(
            crate::db::repositories::PassageVersionRepository::new(
                Arc::new(r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::memory()).unwrap())
            ),
            Arc::new(crate::db::repositories::PassageRepository::new(
                Arc::new(r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::memory()).unwrap())
            )),
            Arc::new(crate::cache::AppCache::new(crate::cache::CacheConfig::default())),
        );
        
        let old_text = "Hello World";
        let new_text = "Hello Rust";
        
        let diff = service.compute_word_diff(old_text, new_text);
        
        assert!(diff.contains("-World"));
        assert!(diff.contains("+Rust"));
    }

    #[test]
    fn test_compute_unified_diff() {
        let service = PassageVersionService::new(
            crate::db::repositories::PassageVersionRepository::new(
                Arc::new(r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::memory()).unwrap())
            ),
            Arc::new(crate::db::repositories::PassageRepository::new(
                Arc::new(r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::memory()).unwrap())
            )),
            Arc::new(crate::cache::AppCache::new(crate::cache::CacheConfig::default())),
        );
        
        let old_text = "line1\nline2\nline3\n";
        let new_text = "line1\nmodified\nline3\nline4\n";
        
        let diff = service.compute_unified_diff(old_text, new_text, 3);
        
        assert!(diff.contains("---"));
        assert!(diff.contains("+++"));
    }

    #[test]
    fn test_diff_stats_from_lines() {
        let old_text = "a\nb\nc\n";
        let new_text = "a\nb\nd\n";
        
        let stats = DiffStats::from_lines(old_text, new_text);
        
        assert!(stats.added > 0 || stats.deleted > 0);
    }

    #[test]
    fn test_diff_type_default() {
        assert_eq!(DiffType::default(), DiffType::Line);
    }

    #[test]
    fn test_diff_format_default() {
        assert_eq!(DiffFormat::default(), DiffFormat::Unified);
    }

    #[test]
    fn test_diff_line_type_serialization() {
        let json = serde_json::to_string(&DiffLineType::Added).unwrap();
        assert!(json.contains("added"));
        
        let deserialized: DiffLineType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, DiffLineType::Added);
    }

    #[test]
    fn test_version_diff_query() {
        let query = VersionDiffQuery {
            passage_id: 1,
            from_version: 1,
            to_version: 2,
            diff_type: Some(DiffType::Char),
            format: Some(DiffFormat::SideBySide),
            include_all_fields: Some(true),
        };
        
        assert_eq!(query.from_version, 1);
        assert_eq!(query.to_version, 2);
        assert_eq!(query.diff_type, Some(DiffType::Char));
        assert_eq!(query.format, Some(DiffFormat::SideBySide));
        assert_eq!(query.include_all_fields, Some(true));
    }

    #[test]
    fn test_version_diff_response() {
        let response = VersionDiffResponse {
            from_version: 1,
            to_version: 2,
            field_diffs: vec![],
            total_changes: 10,
            stats: DiffStats {
                added: 5,
                deleted: 3,
                modified: 2,
                unchanged: 100,
            },
        };
        
        assert_eq!(response.from_version, 1);
        assert_eq!(response.to_version, 2);
        assert_eq!(response.total_changes, 10);
        assert_eq!(response.stats.added, 5);
    }
}
