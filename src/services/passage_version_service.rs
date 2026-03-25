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
//!
//! 第六阶段：版本恢复功能
//! - 软恢复实现
//! - 硬恢复实现
//! - 恢复 API
//!
//! 第七阶段：撤销/重做功能
//! - 撤销功能
//! - 重做功能
//!
//! 第八阶段：版本管理功能
//! - 删除单个版本
//! - 批量删除版本
//!
//! 第九阶段：缓存优化
//! - 缓存键定义
//! - 缓存读写
//! - 缓存策略 (TTL, 预热, 降级)

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use similar::{ChangeTag, TextDiff};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::cache::AppCache;
use crate::db::models::{Passage, PassageHistorySettings, PassageVersion, RestoreMode};
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

// ==================== 第九阶段：缓存优化 ====================

/// 版本历史缓存键生成器
pub mod cache_keys {
    use super::VersionListQuery;

    /// 缓存键前缀
    pub const VERSION_PREFIX: &str = "version";

    /// 版本列表缓存键
    pub fn version_list(passage_id: i64, query: &VersionListQuery) -> String {
        format!(
            "{}:list:{}:p{}_s{}_{}_{}",
            VERSION_PREFIX,
            passage_id,
            query.get_page(),
            query.get_page_size(),
            query.get_sort_by() as i32,
            query.get_sort_order() as i32
        )
    }

    /// 版本数量缓存键
    pub fn version_count(passage_id: i64) -> String {
        format!("{}:count:{}", VERSION_PREFIX, passage_id)
    }

    /// 版本详情缓存键
    pub fn version_detail(passage_id: i64, version_number: i32) -> String {
        format!("{}:detail:{}:v{}", VERSION_PREFIX, passage_id, version_number)
    }

    /// 最新版本缓存键
    pub fn version_latest(passage_id: i64) -> String {
        format!("{}:latest:{}", VERSION_PREFIX, passage_id)
    }

    /// 版本差异缓存键
    pub fn version_diff(passage_id: i64, from: i32, to: i32) -> String {
        format!("{}:diff:{}:{}->{}", VERSION_PREFIX, passage_id, from, to)
    }

    /// 撤销/重做状态缓存键
    pub fn undo_redo_status(passage_id: i64) -> String {
        format!("{}:undo_redo:{}", VERSION_PREFIX, passage_id)
    }

    /// 版本列表模式（用于批量删除）
    pub fn version_list_pattern(passage_id: i64) -> String {
        format!("{}:list:{}:*", VERSION_PREFIX, passage_id)
    }

    /// 所有版本相关缓存模式
    pub fn version_all_pattern(passage_id: i64) -> String {
        format!("{}:*:{}*", VERSION_PREFIX, passage_id)
    }
}

/// 缓存 TTL 配置（秒）
pub mod cache_ttl {
    /// 版本列表缓存 TTL（5分钟）
    pub const VERSION_LIST: u64 = 300;
    
    /// 版本数量缓存 TTL（5分钟）
    pub const VERSION_COUNT: u64 = 300;
    
    /// 版本详情缓存 TTL（10分钟）
    pub const VERSION_DETAIL: u64 = 600;
    
    /// 最新版本缓存 TTL（1分钟，较短因为变化频繁）
    pub const VERSION_LATEST: u64 = 60;
    
    /// 版本差异缓存 TTL（15分钟，差异计算较耗时）
    pub const VERSION_DIFF: u64 = 900;
    
    /// 撤销/重做状态缓存 TTL（30秒，较短因为状态变化频繁）
    pub const UNDO_REDO_STATUS: u64 = 30;
}

/// 缓存键生成（兼容旧接口）
fn get_version_list_cache_key(passage_id: i64, query: &VersionListQuery) -> String {
    cache_keys::version_list(passage_id, query)
}

fn get_version_count_cache_key(passage_id: i64) -> String {
    cache_keys::version_count(passage_id)
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

/// 版本恢复查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionRestoreQuery {
    /// 文章 ID
    pub passage_id: i64,
    /// 要恢复到的版本号
    pub version_number: i32,
    /// 恢复模式
    pub mode: RestoreMode,
    /// 是否创建备份版本
    pub create_backup: Option<bool>,
}

/// 版本恢复响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionRestoreResponse {
    /// 是否成功
    pub success: bool,
    /// 恢复后的版本号
    pub restored_version: i32,
    /// 新创建的备份版本号（如果有）
    pub backup_version: Option<i32>,
    /// 恢复模式
    pub mode: RestoreMode,
    /// 消息
    pub message: String,
}

/// 撤销/重做操作类型
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UndoRedoOperation {
    /// 撤销
    Undo,
    /// 重做
    Redo,
}

/// 撤销/重做查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoRedoQuery {
    /// 文章 ID
    pub passage_id: i64,
    /// 操作类型
    pub operation: UndoRedoOperation,
    /// 恢复模式
    pub mode: Option<RestoreMode>,
}

/// 撤销/重做响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoRedoResponse {
    /// 是否成功
    pub success: bool,
    /// 操作类型
    pub operation: UndoRedoOperation,
    /// 恢复到的版本号
    pub restored_version: i32,
    /// 之前的版本号
    pub from_version: i32,
    /// 消息
    pub message: String,
    /// 是否还有可撤销的内容
    pub can_undo: bool,
    /// 是否还有可重做的内容
    pub can_redo: bool,
}

/// 撤销/重做状态
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoRedoStatus {
    /// 是否可以撤销
    pub can_undo: bool,
    /// 是否可以重做
    pub can_redo: bool,
}

/// 版本删除查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionDeleteQuery {
    /// 文章 ID
    pub passage_id: i64,
    /// 要删除的版本 ID
    pub version_id: i64,
}

/// 版本批量删除查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionBatchDeleteQuery {
    /// 文章 ID
    pub passage_id: i64,
    /// 要删除的版本 ID 列表
    pub version_ids: Vec<i64>,
}

/// 版本删除响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionDeleteResponse {
    /// 是否成功
    pub success: bool,
    /// 删除的版本 ID
    pub deleted_version_id: i64,
    /// 消息
    pub message: String,
}

/// 版本批量删除响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionBatchDeleteResponse {
    /// 是否成功
    pub success: bool,
    /// 删除的数量
    pub deleted_count: u32,
    /// 消息
    pub message: String,
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

    // ==================== 第六阶段：版本恢复功能 ====================

    /// 恢复版本到指定版本
    ///
    /// # 参数
    /// - query: 恢复查询参数
    ///
    /// # 返回
    /// 返回版本恢复响应
    pub async fn restore_version(&self, query: VersionRestoreQuery) -> Result<VersionRestoreResponse> {
        // 1. 获取要恢复的版本
        let target_version = self.version_repo
            .get_by_version_number(query.passage_id, query.version_number)
            .await
            .map_err(to_send_sync_error)?
            .ok_or_else(|| format!("版本 {} 不存在", query.version_number))?;

        // 2. 获取当前文章
        let current_passage = self.passage_repo
            .get_by_id(query.passage_id)
            .await
            .map_err(to_send_sync_error)?;

        // 3. 根据恢复模式执行恢复
        match query.mode {
            RestoreMode::Soft => {
                self.soft_restore(query.passage_id, &target_version, &current_passage).await
            }
            RestoreMode::Hard => {
                let create_backup = query.create_backup.unwrap_or(true);
                self.hard_restore(query.passage_id, &target_version, &current_passage, create_backup).await
            }
            RestoreMode::HardWithBackup => {
                self.hard_restore_with_backup(query.passage_id, &target_version, &current_passage).await
            }
        }
    }

    /// 软恢复
    ///
    /// 只覆盖文件内容，不创建版本记录，不更新数据库
    async fn soft_restore(
        &self,
        passage_id: i64,
        target_version: &PassageVersion,
        _current_passage: &Passage,
    ) -> Result<VersionRestoreResponse> {
        // 1. 检查配置
        let config = self.load_history_config().await?;
        
        // 2. 如果使用文件系统模式，尝试从文件恢复
        if config.storage_mode == "filesystem" && !target_version.file_path.is_empty() {
            // 读取历史文件内容
            let history_path = Path::new(&target_version.file_path);
            
            if history_path.exists() {
                // 读取文件内容
                let content = self.read_history_file(history_path).await?;
                
                // 写入当前文章文件
                let current_file_path = PathBuf::from("markdown")
                    .join(format!("{}.md", passage_id));
                
                self.write_history_file(&current_file_path, &content).await?;
            }
        }

        Ok(VersionRestoreResponse {
            success: true,
            restored_version: target_version.version_number,
            backup_version: None,
            mode: RestoreMode::Soft,
            message: "软恢复成功（仅文件恢复）".to_string(),
        })
    }

    /// 硬恢复
    ///
    /// 保存当前版本到历史，更新数据库，创建恢复操作版本记录
    async fn hard_restore(
        &self,
        passage_id: i64,
        target_version: &PassageVersion,
        current_passage: &Passage,
        create_backup: bool,
    ) -> Result<VersionRestoreResponse> {
        let config = self.load_history_config().await?;

        // 1. 如果需要创建备份，保存当前版本
        let mut backup_version_number = None;
        
        if create_backup {
            let backup_id = self.save_version(
                passage_id,
                current_passage.uuid.as_deref().unwrap_or(""),
                current_passage,
                "pre_restore",
                Some("恢复前备份".to_string()),
            ).await?;
            
            if backup_id > 0 {
                backup_version_number = Some(target_version.version_number + 1);
            }
        }

        // 2. 构建更新后的文章
        let mut updated_passage = current_passage.clone();
        updated_passage.title = target_version.title.clone();
        updated_passage.content = target_version.content.clone();
        updated_passage.original_content = Some(target_version.content.clone());
        updated_passage.tags = target_version.tags.clone();
        updated_passage.category = target_version.category.clone();
        updated_passage.cover_image = target_version.cover_image.clone();
        updated_passage.updated_at = Utc::now();

        // 3. 更新数据库
        self.passage_repo
            .update(&updated_passage)
            .await
            .map_err(to_send_sync_error)?;

        // 4. 如果使用文件系统模式，更新文件
        if config.storage_mode == "filesystem" {
            let file_path = current_passage.file_path.as_ref()
                .ok_or("文章文件路径不存在")?;
            
            let full_path = PathBuf::from(file_path);
            
            if let Some(parent) = full_path.parent() {
                tokio::fs::create_dir_all(parent).await
                    .map_err(|e| format!("创建目录失败: {}", e))?;
            }
            
            tokio::fs::write(&full_path, &target_version.content).await
                .map_err(|e| format!("写入文件失败: {}", e))?;
        }

        // 5. 创建恢复操作版本记录
        self.save_version(
            passage_id,
            current_passage.uuid.as_deref().unwrap_or(""),
            &updated_passage,
            "restore",
            Some(format!("从版本 {} 恢复", target_version.version_number)),
        ).await?;

        // 6. 清除缓存
        self.clear_version_cache_internal(passage_id).await;

        Ok(VersionRestoreResponse {
            success: true,
            restored_version: target_version.version_number,
            backup_version: backup_version_number,
            mode: RestoreMode::Hard,
            message: "硬恢复成功".to_string(),
        })
    }

    /// 硬恢复 + 备份
    ///
    /// 保留更多元数据，包括恢复操作的详细信息
    async fn hard_restore_with_backup(
        &self,
        passage_id: i64,
        target_version: &PassageVersion,
        current_passage: &Passage,
    ) -> Result<VersionRestoreResponse> {
        // 1. 创建完整备份（包括所有字段）
        let backup_id = self.save_version(
            passage_id,
            current_passage.uuid.as_deref().unwrap_or(""),
            current_passage,
            "pre_restore_full_backup",
            Some(format!("完整备份 - 恢复到版本 {}", target_version.version_number)),
        ).await?;

        // 2. 执行硬恢复
        let mut response = self.hard_restore(passage_id, target_version, current_passage, false).await?;

        // 3. 更新备份版本号
        if backup_id > 0 {
            // 获取最新版本号作为备份版本号
            let latest_version = self.version_repo
                .get_latest_version(passage_id)
                .await
                .map_err(to_send_sync_error)?;
            
            if let Some(v) = latest_version {
                response.backup_version = Some(v.version_number);
            }
        }

        response.mode = RestoreMode::HardWithBackup;
        response.message = "硬恢复 + 备份成功".to_string();

        Ok(response)
    }

    /// 检查版本是否可以恢复
    ///
    /// # 参数
    /// - passage_id: 文章 ID
    /// - version_number: 版本号
    ///
    /// # 返回
    /// 如果可以恢复返回 true，否则返回 false
    pub async fn can_restore_version(&self, passage_id: i64, version_number: i32) -> Result<bool> {
        let version = self.version_repo
            .get_by_version_number(passage_id, version_number)
            .await
            .map_err(to_send_sync_error)?;
        
        Ok(version.is_some())
    }

    // ==================== 第七阶段：撤销/重做功能 ====================

    /// 检查是否可以撤销
    ///
    /// # 参数
    /// - passage_id: 文章 ID
    ///
    /// # 返回
    /// 如果可以撤销返回 true，否则返回 false
    pub async fn can_undo(&self, passage_id: i64) -> Result<bool> {
        let config = self.load_history_config().await?;
        
        if !config.enable_undo_redo {
            return Ok(false);
        }

        let latest = self.version_repo
            .get_latest_version(passage_id)
            .await
            .map_err(to_send_sync_error)?;
        
        if let Some(version) = latest {
            if version.change_type == "restore" || version.change_type == "undo" {
                return Ok(version.parent_version_id.is_some());
            }
            return Ok(version.parent_version_id.is_some());
        }
        
        Ok(false)
    }

    /// 检查是否可以重做
    ///
    /// # 参数
    /// - passage_id: 文章 ID
    ///
    /// # 返回
    /// 如果可以重做返回 true，否则返回 false
    pub async fn can_redo(&self, passage_id: i64) -> Result<bool> {
        let config = self.load_history_config().await?;
        
        if !config.enable_undo_redo {
            return Ok(false);
        }

        let versions = self.version_repo
            .get_by_passage_id(passage_id)
            .await
            .map_err(to_send_sync_error)?;
        
        let has_redo = versions.iter().any(|v| v.change_type == "redo");
        
        Ok(has_redo)
    }

    /// 撤销上一次操作
    ///
    /// # 参数
    /// - query: 撤销/重做查询参数
    ///
    /// # 返回
    /// 返回撤销/重做响应
    pub async fn undo_last_change(&self, query: UndoRedoQuery) -> Result<UndoRedoResponse> {
        let config = self.load_history_config().await?;
        
        if !config.enable_undo_redo {
            return Ok(UndoRedoResponse {
                success: false,
                operation: UndoRedoOperation::Undo,
                restored_version: 0,
                from_version: 0,
                message: "撤销/重做功能已禁用".to_string(),
                can_undo: false,
                can_redo: false,
            });
        }

        // 获取当前文章
        let _current_passage = self.passage_repo
            .get_by_id(query.passage_id)
            .await
            .map_err(to_send_sync_error)?;

        // 获取最新版本
        let latest_version = self.version_repo
            .get_latest_version(query.passage_id)
            .await
            .map_err(to_send_sync_error)?
            .ok_or("没有可撤销的版本")?;

        let from_version = latest_version.version_number;

        // 获取父版本
        let parent_version_id = latest_version.parent_version_id
            .ok_or("没有可撤销的版本")?;

        let parent_version = self.version_repo
            .get_by_id(parent_version_id)
            .await
            .map_err(to_send_sync_error)?
            .ok_or("父版本不存在")?;

        let restored_version_number = parent_version.version_number;

        // 执行恢复
        let mode = query.mode.unwrap_or(RestoreMode::Hard);
        
        let restore_query = VersionRestoreQuery {
            passage_id: query.passage_id,
            version_number: restored_version_number,
            mode,
            create_backup: Some(false), // 撤销操作不创建额外备份
        };

        let _restore_result = self.restore_version(restore_query).await?;

        // 获取更新后的状态
        let can_undo = self.can_undo(query.passage_id).await?;
        let can_redo = self.can_redo(query.passage_id).await?;

        Ok(UndoRedoResponse {
            success: true,
            operation: UndoRedoOperation::Undo,
            restored_version: restored_version_number,
            from_version,
            message: format!("已撤销到版本 {}", restored_version_number),
            can_undo,
            can_redo,
        })
    }

    /// 重做上一次撤销的操作
    ///
    /// # 参数
    /// - query: 撤销/重做查询参数
    ///
    /// # 返回
    /// 返回撤销/重做响应
    pub async fn redo_last_change(&self, query: UndoRedoQuery) -> Result<UndoRedoResponse> {
        let config = self.load_history_config().await?;
        
        if !config.enable_undo_redo {
            return Ok(UndoRedoResponse {
                success: false,
                operation: UndoRedoOperation::Redo,
                restored_version: 0,
                from_version: 0,
                message: "撤销/重做功能已禁用".to_string(),
                can_undo: false,
                can_redo: false,
            });
        }

        // 获取当前文章
        let _current_passage = self.passage_repo
            .get_by_id(query.passage_id)
            .await
            .map_err(to_send_sync_error)?;

        // 获取当前版本
        let current_version = self.version_repo
            .get_latest_version(query.passage_id)
            .await
            .map_err(to_send_sync_error)?
            .ok_or("没有可重做的版本")?;

        let from_version = current_version.version_number;

        // 查找需要重做到的版本
        // 在当前实现中，我们通过查找父版本来确定重做目标
        // 如果当前版本是 undo 产生的，我们可以恢复到 undo 之前的版本
        if current_version.change_type == "undo" || current_version.change_type == "restore" {
            // 查找父版本
            if let Some(parent_id) = current_version.parent_version_id {
                let parent_version = self.version_repo
                    .get_by_id(parent_id)
                    .await
                    .map_err(to_send_sync_error)?
                    .ok_or("父版本不存在")?;

                let restored_version_number = parent_version.version_number;

                // 执行恢复
                let mode = query.mode.unwrap_or(RestoreMode::Hard);
                
                let restore_query = VersionRestoreQuery {
                    passage_id: query.passage_id,
                    version_number: restored_version_number,
                    mode,
                    create_backup: Some(false),
                };

                let _restore_result = self.restore_version(restore_query).await?;

                let can_undo = self.can_undo(query.passage_id).await?;
                let can_redo = self.can_redo(query.passage_id).await?;

                return Ok(UndoRedoResponse {
                    success: true,
                    operation: UndoRedoOperation::Redo,
                    restored_version: restored_version_number,
                    from_version,
                    message: format!("已重做到版本 {}", restored_version_number),
                    can_undo,
                    can_redo,
                });
            }
        }

        Ok(UndoRedoResponse {
            success: false,
            operation: UndoRedoOperation::Redo,
            restored_version: from_version,
            from_version,
            message: "没有可重做的操作".to_string(),
            can_undo: self.can_undo(query.passage_id).await?,
            can_redo: false,
        })
    }

    /// 获取撤销/重做状态
    ///
    /// # 参数
    /// - passage_id: 文章 ID
    ///
    /// # 返回
    /// 返回当前是否可以撤销/重做
    pub async fn get_undo_redo_status(&self, passage_id: i64) -> Result<UndoRedoStatus> {
        Ok(UndoRedoStatus {
            can_undo: self.can_undo(passage_id).await?,
            can_redo: self.can_redo(passage_id).await?,
        })
    }

    // ==================== 第八阶段：版本管理功能 ====================

    /// 删除单个版本
    ///
    /// # 参数
    /// - query: 删除查询参数
    ///
    /// # 返回
    /// 返回删除响应
    pub async fn delete_version(&self, query: VersionDeleteQuery) -> Result<VersionDeleteResponse> {
        // 1. 验证版本存在
        let version = self.version_repo
            .get_by_id(query.version_id)
            .await
            .map_err(to_send_sync_error)?
            .ok_or_else(|| format!("版本 {} 不存在", query.version_id))?;
        
        // 2. 验证版本属于正确的文章
        if version.passage_id != query.passage_id {
            return Ok(VersionDeleteResponse {
                success: false,
                deleted_version_id: query.version_id,
                message: "版本不属于该文章".to_string(),
            });
        }
        
        // 3. 检查是否是最后一个版本
        let latest = self.version_repo
            .get_latest_version(query.passage_id)
            .await
            .map_err(to_send_sync_error)?;
        
        if let Some(latest_version) = latest {
            if latest_version.id == Some(query.version_id) && latest_version.version_number == version.version_number {
                return Ok(VersionDeleteResponse {
                    success: false,
                    deleted_version_id: query.version_id,
                    message: "不能删除最新版本".to_string(),
                });
            }
        }
        
        // 4. 删除版本
        self.version_repo
            .delete(query.version_id)
            .await
            .map_err(to_send_sync_error)?;
        
        // 5. 清除缓存
        self.clear_version_cache_internal(query.passage_id).await;
        
        Ok(VersionDeleteResponse {
            success: true,
            deleted_version_id: query.version_id,
            message: format!("版本 {} 已删除", version.version_number),
        })
    }

    /// 批量删除版本
    ///
    /// # 参数
    /// - query: 批量删除查询参数
    ///
    /// # 返回
    /// 返回批量删除响应
    pub async fn delete_versions_batch(&self, query: VersionBatchDeleteQuery) -> Result<VersionBatchDeleteResponse> {
        if query.version_ids.is_empty() {
            return Ok(VersionBatchDeleteResponse {
                success: false,
                deleted_count: 0,
                message: "没有要删除的版本".to_string(),
            });
        }
        
        // 1. 获取最新版本信息
        let latest = self.version_repo
            .get_latest_version(query.passage_id)
            .await
            .map_err(to_send_sync_error)?;
        
        let latest_version_id = latest.and_then(|v| v.id);
        
        // 2. 过滤掉最新版本
        let ids_to_delete: Vec<i64> = query.version_ids
            .into_iter()
            .filter(|id| Some(*id) != latest_version_id)
            .collect();
        
        if ids_to_delete.is_empty() {
            return Ok(VersionBatchDeleteResponse {
                success: false,
                deleted_count: 0,
                message: "没有可删除的版本（最新版本不能删除）".to_string(),
            });
        }
        
        // 3. 批量删除
        let deleted_count = self.version_repo
            .delete_batch(ids_to_delete)
            .await
            .map_err(to_send_sync_error)?;
        
        // 4. 清除缓存
        self.clear_version_cache_internal(query.passage_id).await;
        
        Ok(VersionBatchDeleteResponse {
            success: true,
            deleted_count,
            message: format!("已删除 {} 个版本", deleted_count),
        })
    }

    /// 删除文章的所有历史版本
    ///
    /// # 参数
    /// - passage_id: 文章 ID
    ///
    /// # 返回
    /// 返回删除的版本数量
    pub async fn delete_all_versions(&self, passage_id: i64) -> Result<u64> {
        // 获取版本数量
        let count = self.version_repo
            .get_version_count(passage_id)
            .await
            .map_err(to_send_sync_error)? as u64;
        
        if count == 0 {
            return Ok(0);
        }
        
        // 删除所有版本
        self.version_repo
            .delete_by_passage_id(passage_id)
            .await
            .map_err(to_send_sync_error)?;
        
        // 清除缓存
        self.clear_version_cache_internal(passage_id).await;
        
        Ok(count)
    }

    // ==================== 第九阶段：缓存优化 ====================

    /// 预热版本列表缓存
    ///
    /// 在文章访问高峰前预先加载常用数据到缓存
    ///
    /// # 参数
    /// - passage_id: 文章 ID
    /// - page: 页码
    /// - page_size: 每页数量
    pub async fn warm_version_list_cache(
        &self,
        passage_id: i64,
        page: i32,
        page_size: i32,
    ) -> Result<()> {
        let query = VersionListQuery {
            passage_id,
            page: Some(page),
            page_size: Some(page_size),
            sort_by: Some(VersionSortField::VersionNumber),
            sort_order: Some(SortOrder::Desc),
            change_type: None,
            search_title: None,
        };
        
        let _response = self.list_versions(query).await?;
        
        Ok(())
    }

    /// 预热最新版本缓存
    ///
    /// # 参数
    /// - passage_id: 文章 ID
    pub async fn warm_version_latest_cache(&self, passage_id: i64) -> Result<()> {
        let _ = self.get_latest_version(passage_id).await?;
        Ok(())
    }

    /// 批量预热文章版本缓存
    ///
    /// # 参数
    /// - passage_ids: 文章 ID 列表
    pub async fn warm_batch_cache(&self, passage_ids: Vec<i64>) -> Result<()> {
        for passage_id in passage_ids {
            // 预热版本列表
            let _ = self.warm_version_list_cache(passage_id, 1, 20).await;
            // 预热最新版本
            let _ = self.warm_version_latest_cache(passage_id).await;
        }
        Ok(())
    }

    /// 获取缓存命中率统计
    ///
    /// # 参数
    /// - passage_id: 文章 ID
    ///
    /// # 返回
    /// 返回缓存统计信息
    pub async fn get_cache_stats(&self, passage_id: i64) -> Result<VersionCacheStats> {
        let manager = match self.cache.manager() {
            Some(m) => m,
            None => {
                return Ok(VersionCacheStats {
                    cache_enabled: false,
                    version_list_cached: false,
                    version_count_cached: false,
                    latest_version_cached: false,
                    cache_keys: vec![],
                });
            }
        };

        let mut cache_keys = Vec::new();
        let version_list_key = cache_keys::version_list(passage_id, &VersionListQuery {
            passage_id,
            page: Some(1),
            page_size: Some(20),
            ..Default::default()
        });
        let version_count_key = cache_keys::version_count(passage_id);
        let version_latest_key = cache_keys::version_latest(passage_id);

        let version_list_cached = manager.get(&version_list_key).await.is_some();
        let version_count_cached = manager.get(&version_count_key).await.is_some();
        let latest_version_cached = manager.get(&version_latest_key).await.is_some();

        cache_keys.push(version_list_key);
        cache_keys.push(version_count_key);
        cache_keys.push(version_latest_key);

        Ok(VersionCacheStats {
            cache_enabled: true,
            version_list_cached,
            version_count_cached,
            latest_version_cached,
            cache_keys,
        })
    }

    /// 主动失效版本缓存（当发生版本变更时调用）
    ///
    /// # 参数
    /// - passage_id: 文章 ID
    pub async fn invalidate_version_cache(&self, passage_id: i64) -> Result<()> {
        self.clear_version_cache(passage_id).await;
        
        // 清除版本详情缓存
        if let Some(manager) = self.cache.manager() {
            let versions = self.version_repo
                .get_by_passage_id(passage_id)
                .await
                .map_err(to_send_sync_error)?;
            
            for version in versions {
                let key = cache_keys::version_detail(passage_id, version.version_number);
                let _ = manager.delete(&key).await;
            }
            
            // 清除差异缓存
            let diff_pattern = format!("{}:diff:{}*", cache_keys::VERSION_PREFIX, passage_id);
            let _ = manager.delete_pattern(&diff_pattern).await;
            
            // 清除撤销/重做状态缓存
            let undo_redo_key = cache_keys::undo_redo_status(passage_id);
            let _ = manager.delete(&undo_redo_key).await;
        }
        
        Ok(())
    }
}

/// 版本缓存统计
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionCacheStats {
    /// 是否启用缓存
    pub cache_enabled: bool,
    /// 版本列表是否已缓存
    pub version_list_cached: bool,
    /// 版本数量是否已缓存
    pub version_count_cached: bool,
    /// 最新版本是否已缓存
    pub latest_version_cached: bool,
    /// 缓存键列表
    pub cache_keys: Vec<String>,
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
        assert!(key.contains("version:list:123:"));
        assert!(key.contains("p2"));
        assert!(key.contains("s10"));
    }

    #[test]
    fn test_version_count_cache_key() {
        let key = get_version_count_cache_key(456);
        assert_eq!(key, "version:count:456");
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

    // ==================== 第六阶段：版本恢复功能测试 ====================

    #[test]
    fn test_version_restore_query() {
        let query = VersionRestoreQuery {
            passage_id: 1,
            version_number: 5,
            mode: RestoreMode::Hard,
            create_backup: Some(true),
        };
        
        assert_eq!(query.passage_id, 1);
        assert_eq!(query.version_number, 5);
        assert_eq!(query.mode, RestoreMode::Hard);
        assert_eq!(query.create_backup, Some(true));
    }

    #[test]
    fn test_version_restore_query_soft_mode() {
        let query = VersionRestoreQuery {
            passage_id: 1,
            version_number: 3,
            mode: RestoreMode::Soft,
            create_backup: None,
        };
        
        assert_eq!(query.mode, RestoreMode::Soft);
    }

    #[test]
    fn test_version_restore_query_hard_with_backup_mode() {
        let query = VersionRestoreQuery {
            passage_id: 1,
            version_number: 10,
            mode: RestoreMode::HardWithBackup,
            create_backup: Some(false),
        };
        
        assert_eq!(query.mode, RestoreMode::HardWithBackup);
    }

    #[test]
    fn test_version_restore_response() {
        let response = VersionRestoreResponse {
            success: true,
            restored_version: 5,
            backup_version: Some(6),
            mode: RestoreMode::Hard,
            message: "恢复成功".to_string(),
        };
        
        assert!(response.success);
        assert_eq!(response.restored_version, 5);
        assert_eq!(response.backup_version, Some(6));
        assert_eq!(response.mode, RestoreMode::Hard);
        assert_eq!(response.message, "恢复成功");
    }

    #[test]
    fn test_version_restore_response_no_backup() {
        let response = VersionRestoreResponse {
            success: true,
            restored_version: 3,
            backup_version: None,
            mode: RestoreMode::Soft,
            message: "软恢复成功".to_string(),
        };
        
        assert!(response.success);
        assert!(response.backup_version.is_none());
    }

    #[test]
    fn test_restore_mode_serialization() {
        let json = serde_json::to_string(&RestoreMode::Hard).unwrap();
        assert!(json.contains("hard"));
        
        let deserialized: RestoreMode = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, RestoreMode::Hard);
    }

    #[test]
    fn test_restore_mode_from_str() {
        assert_eq!(RestoreMode::from_str("soft"), Some(RestoreMode::Soft));
        assert_eq!(RestoreMode::from_str("hard"), Some(RestoreMode::Hard));
        assert_eq!(RestoreMode::from_str("hard_with_backup"), Some(RestoreMode::HardWithBackup));
        assert_eq!(RestoreMode::from_str("invalid"), None);
    }

    // ==================== 第七阶段：撤销/重做功能测试 ====================

    #[test]
    fn test_undo_redo_operation_serialization() {
        let json = serde_json::to_string(&UndoRedoOperation::Undo).unwrap();
        assert!(json.contains("undo"));
        
        let deserialized: UndoRedoOperation = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, UndoRedoOperation::Undo);
    }

    #[test]
    fn test_undo_redo_query() {
        let query = UndoRedoQuery {
            passage_id: 1,
            operation: UndoRedoOperation::Undo,
            mode: Some(RestoreMode::Hard),
        };
        
        assert_eq!(query.passage_id, 1);
        assert_eq!(query.operation, UndoRedoOperation::Undo);
        assert_eq!(query.mode, Some(RestoreMode::Hard));
    }

    #[test]
    fn test_undo_redo_query_redo_operation() {
        let query = UndoRedoQuery {
            passage_id: 1,
            operation: UndoRedoOperation::Redo,
            mode: None,
        };
        
        assert_eq!(query.operation, UndoRedoOperation::Redo);
    }

    #[test]
    fn test_undo_redo_response() {
        let response = UndoRedoResponse {
            success: true,
            operation: UndoRedoOperation::Undo,
            restored_version: 5,
            from_version: 6,
            message: "已撤销到版本 5".to_string(),
            can_undo: false,
            can_redo: true,
        };
        
        assert!(response.success);
        assert_eq!(response.operation, UndoRedoOperation::Undo);
        assert_eq!(response.restored_version, 5);
        assert_eq!(response.from_version, 6);
        assert!(!response.can_undo);
        assert!(response.can_redo);
    }

    #[test]
    fn test_undo_redo_response_failure() {
        let response = UndoRedoResponse {
            success: false,
            operation: UndoRedoOperation::Redo,
            restored_version: 0,
            from_version: 0,
            message: "没有可重做的操作".to_string(),
            can_undo: true,
            can_redo: false,
        };
        
        assert!(!response.success);
        assert_eq!(response.message, "没有可重做的操作");
        assert!(!response.can_redo);
    }

    #[test]
    fn test_undo_redo_status() {
        let status = UndoRedoStatus {
            can_undo: true,
            can_redo: false,
        };
        
        assert!(status.can_undo);
        assert!(!status.can_redo);
    }

    #[test]
    fn test_undo_redo_status_serialization() {
        let status = UndoRedoStatus {
            can_undo: true,
            can_redo: true,
        };
        
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("canUndo"));
        assert!(json.contains("canRedo"));
        
        let deserialized: UndoRedoStatus = serde_json::from_str(&json).unwrap();
        assert!(deserialized.can_undo);
        assert!(deserialized.can_redo);
    }

    // ==================== 第八阶段：版本管理功能测试 ====================

    #[test]
    fn test_version_delete_query() {
        let query = VersionDeleteQuery {
            passage_id: 1,
            version_id: 5,
        };
        
        assert_eq!(query.passage_id, 1);
        assert_eq!(query.version_id, 5);
    }

    #[test]
    fn test_version_batch_delete_query() {
        let query = VersionBatchDeleteQuery {
            passage_id: 1,
            version_ids: vec![1, 2, 3, 4, 5],
        };
        
        assert_eq!(query.passage_id, 1);
        assert_eq!(query.version_ids.len(), 5);
    }

    #[test]
    fn test_version_delete_query_empty_ids() {
        let query = VersionBatchDeleteQuery {
            passage_id: 1,
            version_ids: vec![],
        };
        
        assert!(query.version_ids.is_empty());
    }

    #[test]
    fn test_version_delete_response() {
        let response = VersionDeleteResponse {
            success: true,
            deleted_version_id: 5,
            message: "版本 5 已删除".to_string(),
        };
        
        assert!(response.success);
        assert_eq!(response.deleted_version_id, 5);
        assert_eq!(response.message, "版本 5 已删除");
    }

    #[test]
    fn test_version_delete_response_failure() {
        let response = VersionDeleteResponse {
            success: false,
            deleted_version_id: 0,
            message: "不能删除最新版本".to_string(),
        };
        
        assert!(!response.success);
        assert_eq!(response.message, "不能删除最新版本");
    }

    #[test]
    fn test_version_batch_delete_response() {
        let response = VersionBatchDeleteResponse {
            success: true,
            deleted_count: 3,
            message: "已删除 3 个版本".to_string(),
        };
        
        assert!(response.success);
        assert_eq!(response.deleted_count, 3);
    }

    #[test]
    fn test_version_batch_delete_response_failure() {
        let response = VersionBatchDeleteResponse {
            success: false,
            deleted_count: 0,
            message: "没有可删除的版本".to_string(),
        };
        
        assert!(!response.success);
        assert_eq!(response.deleted_count, 0);
    }

    // ==================== 第九阶段：缓存优化测试 ====================

    #[test]
    fn test_cache_keys_version_list() {
        let query = VersionListQuery {
            passage_id: 1,
            page: Some(2),
            page_size: Some(10),
            ..Default::default()
        };
        
        let key = cache_keys::version_list(1, &query);
        assert!(key.contains("version:list:1:"));
        assert!(key.contains("p2"));
        assert!(key.contains("s10"));
    }

    #[test]
    fn test_cache_keys_version_count() {
        let key = cache_keys::version_count(123);
        assert_eq!(key, "version:count:123");
    }

    #[test]
    fn test_cache_keys_version_detail() {
        let key = cache_keys::version_detail(1, 5);
        assert_eq!(key, "version:detail:1:v5");
    }

    #[test]
    fn test_cache_keys_version_latest() {
        let key = cache_keys::version_latest(100);
        assert_eq!(key, "version:latest:100");
    }

    #[test]
    fn test_cache_keys_version_diff() {
        let key = cache_keys::version_diff(1, 3, 5);
        assert_eq!(key, "version:diff:1:3->5");
    }

    #[test]
    fn test_cache_keys_undo_redo_status() {
        let key = cache_keys::undo_redo_status(50);
        assert_eq!(key, "version:undo_redo:50");
    }

    #[test]
    fn test_cache_keys_version_list_pattern() {
        let pattern = cache_keys::version_list_pattern(1);
        assert!(pattern.contains("version:list:1:*"));
    }

    #[test]
    fn test_cache_ttl_constants() {
        assert_eq!(cache_ttl::VERSION_LIST, 300);
        assert_eq!(cache_ttl::VERSION_COUNT, 300);
        assert_eq!(cache_ttl::VERSION_DETAIL, 600);
        assert_eq!(cache_ttl::VERSION_LATEST, 60);
        assert_eq!(cache_ttl::VERSION_DIFF, 900);
        assert_eq!(cache_ttl::UNDO_REDO_STATUS, 30);
    }

    #[test]
    fn test_version_cache_stats() {
        let stats = VersionCacheStats {
            cache_enabled: true,
            version_list_cached: true,
            version_count_cached: false,
            latest_version_cached: true,
            cache_keys: vec![
                "version:list:1:p1_s20".to_string(),
                "version:count:1".to_string(),
                "version:latest:1".to_string(),
            ],
        };
        
        assert!(stats.cache_enabled);
        assert!(stats.version_list_cached);
        assert!(!stats.version_count_cached);
        assert!(stats.latest_version_cached);
        assert_eq!(stats.cache_keys.len(), 3);
    }

    #[test]
    fn test_version_cache_stats_serialization() {
        let stats = VersionCacheStats {
            cache_enabled: false,
            version_list_cached: false,
            version_count_cached: false,
            latest_version_cached: false,
            cache_keys: vec![],
        };
        
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("cacheEnabled"));
        assert!(json.contains("versionListCached"));
        
        let deserialized: VersionCacheStats = serde_json::from_str(&json).unwrap();
        assert!(!deserialized.cache_enabled);
    }

    // ==================== 错误处理和边界情况测试 ====================

    #[test]
    fn test_detect_changes_tags() {
        let service = PassageVersionService::new(
            crate::db::repositories::PassageVersionRepository::new(
                Arc::new(r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::memory()).unwrap())
            ),
            Arc::new(crate::db::repositories::PassageRepository::new(
                Arc::new(r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::memory()).unwrap())
            )),
            Arc::new(crate::cache::AppCache::new(crate::cache::CacheConfig::default())),
        );
        
        let old_passage = create_test_passage("标题", "内容");
        let mut new_passage = create_test_passage("标题", "内容");
        new_passage.tags = "[\"新标签\"]".to_string();
        
        let mut config = PassageHistorySettings::default();
        config.save_on_tags_change = true;
        
        let changes = service.detect_changes(&old_passage, &new_passage, &config);
        
        assert!(changes.contains(&"标签"));
    }

    #[test]
    fn test_detect_changes_category() {
        let service = PassageVersionService::new(
            crate::db::repositories::PassageVersionRepository::new(
                Arc::new(r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::memory()).unwrap())
            ),
            Arc::new(crate::db::repositories::PassageRepository::new(
                Arc::new(r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::memory()).unwrap())
            )),
            Arc::new(crate::cache::AppCache::new(crate::cache::CacheConfig::default())),
        );
        
        let old_passage = create_test_passage("标题", "内容");
        let mut new_passage = create_test_passage("标题", "内容");
        new_passage.category = "新分类".to_string();
        
        let mut config = PassageHistorySettings::default();
        config.save_on_category_change = true;
        
        let changes = service.detect_changes(&old_passage, &new_passage, &config);
        
        assert!(changes.contains(&"分类"));
    }

    #[test]
    fn test_detect_changes_cover_image() {
        let service = PassageVersionService::new(
            crate::db::repositories::PassageVersionRepository::new(
                Arc::new(r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::memory()).unwrap())
            ),
            Arc::new(crate::db::repositories::PassageRepository::new(
                Arc::new(r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::memory()).unwrap())
            )),
            Arc::new(crate::cache::AppCache::new(crate::cache::CacheConfig::default())),
        );
        
        let old_passage = create_test_passage("标题", "内容");
        let mut new_passage = create_test_passage("标题", "内容");
        new_passage.cover_image = Some("new-cover.jpg".to_string());
        
        let mut config = PassageHistorySettings::default();
        config.save_on_cover_change = true;
        
        let changes = service.detect_changes(&old_passage, &new_passage, &config);
        
        assert!(changes.contains(&"封面图片"));
    }

    #[test]
    fn test_detect_changes_all_disabled() {
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
        
        // 禁用所有变更检测
        let mut config = PassageHistorySettings::default();
        config.save_on_title_change = false;
        config.save_on_content_change = false;
        config.save_on_tags_change = false;
        config.save_on_category_change = false;
        config.save_on_cover_change = false;
        
        let changes = service.detect_changes(&old_passage, &new_passage, &config);
        
        assert!(changes.is_empty());
    }

    #[test]
    fn test_history_settings_default_values() {
        let config = PassageHistorySettings::default();
        
        assert!(config.enabled);
        assert_eq!(config.storage_mode, "filesystem");
        assert_eq!(config.history_dir, "markdown/.history");
        assert_eq!(config.max_versions, 50);
        assert!(config.enable_deduplication);
        assert!(config.save_on_title_change);
        assert!(config.save_on_content_change);
        assert!(config.save_on_tags_change);
        assert!(config.enable_undo_redo);
    }

    #[test]
    fn test_history_settings_serialization() {
        let config = PassageHistorySettings {
            enabled: false,
            storage_mode: "database".to_string(),
            history_dir: "custom/history".to_string(),
            max_versions: 100,
            enable_deduplication: false,
            save_on_title_change: false,
            save_on_content_change: true,
            save_on_tags_change: true,
            save_on_summary_change: true,
            save_on_category_change: true,
            save_on_cover_change: true,
            enable_undo_redo: false,
        };
        
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"enabled\":false"));
        assert!(json.contains("\"storage_mode\":\"database\""));
        
        let deserialized: PassageHistorySettings = serde_json::from_str(&json).unwrap();
        assert!(!deserialized.enabled);
        assert_eq!(deserialized.max_versions, 100);
    }

    #[test]
    fn test_diff_stats_values() {
        let stats = DiffStats {
            added: 10,
            deleted: 5,
            modified: 3,
            unchanged: 100,
        };
        
        assert_eq!(stats.added, 10);
        assert_eq!(stats.deleted, 5);
        assert_eq!(stats.modified, 3);
        assert_eq!(stats.unchanged, 100);
    }

    #[test]
    fn test_version_restore_query_default_mode() {
        let query = VersionRestoreQuery {
            passage_id: 1,
            version_number: 5,
            mode: RestoreMode::Soft,
            create_backup: None,
        };
        
        assert_eq!(query.mode, RestoreMode::Soft);
    }

    #[test]
    fn test_undo_redo_query_default_mode() {
        let query = UndoRedoQuery {
            passage_id: 1,
            operation: UndoRedoOperation::Undo,
            mode: None,
        };
        
        assert_eq!(query.mode, None);
    }

    #[test]
    fn test_cache_keys_all_patterns() {
        let pattern = cache_keys::version_all_pattern(1);
        assert!(pattern.contains("version:*") || pattern.contains("*"));
    }

    #[test]
    fn test_version_list_query_with_all_filters() {
        let query = VersionListQuery {
            passage_id: 1,
            page: Some(5),
            page_size: Some(50),
            sort_by: Some(VersionSortField::Title),
            sort_order: Some(SortOrder::Asc),
            change_type: Some("manual".to_string()),
            search_title: Some("test".to_string()),
        };
        
        assert_eq!(query.get_page(), 5);
        assert_eq!(query.get_page_size(), 50);
        assert_eq!(query.get_sort_by(), VersionSortField::Title);
        assert_eq!(query.get_sort_order(), SortOrder::Asc);
    }

    #[test]
    fn test_version_list_query_page_boundary() {
        let query = VersionListQuery {
            passage_id: 1,
            page: Some(-1),
            page_size: Some(0),
            ..Default::default()
        };
        
        assert_eq!(query.get_page(), 1);
        assert_eq!(query.get_page_size(), 1);
    }

    #[test]
    fn test_diff_line_creation() {
        let line = DiffLine {
            old_line_number: Some(1),
            new_line_number: Some(1),
            content: "测试内容".to_string(),
            line_type: DiffLineType::Context,
        };
        
        assert_eq!(line.content, "测试内容");
        assert_eq!(line.line_type, DiffLineType::Context);
    }

    #[test]
    fn test_field_diff_detail_creation() {
        let detail = FieldDiffDetail {
            field_name: "title".to_string(),
            old_value: "旧标题".to_string(),
            new_value: "新标题".to_string(),
            changed: true,
            line_diffs: vec![],
        };
        
        assert_eq!(detail.field_name, "title");
        assert_eq!(detail.old_value, "旧标题");
        assert_eq!(detail.new_value, "新标题");
        assert!(detail.changed);
    }

    #[test]
    fn test_version_diff_response_with_fields() {
        let response = VersionDiffResponse {
            from_version: 1,
            to_version: 3,
            field_diffs: vec![
                FieldDiffDetail {
                    field_name: "title".to_string(),
                    old_value: "旧".to_string(),
                    new_value: "新".to_string(),
                    changed: true,
                    line_diffs: vec![],
                },
            ],
            total_changes: 5,
            stats: DiffStats {
                added: 2,
                deleted: 1,
                modified: 2,
                unchanged: 10,
            },
        };
        
        assert_eq!(response.field_diffs.len(), 1);
        assert_eq!(response.stats.added, 2);
    }
}
