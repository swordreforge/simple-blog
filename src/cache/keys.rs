//! 缓存键管理模块
//!
//! 提供结构化、类型安全的缓存键生成和管理
//! 支持版本控制，便于批量失效缓存

use std::fmt;

/// 缓存命名空间
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum CacheNamespace {
    /// 文章相关缓存
    Passage,
    /// 评论相关缓存
    Comment,
    /// 分类相关缓存
    Category,
    /// 标签相关缓存
    Tag,
    /// 设置相关缓存
    Settings,
    /// 用户相关缓存
    User,
    /// 统计相关缓存
    Stats,
    /// 音乐相关缓存
    Music,
    /// 附件相关缓存
    Attachment,
}

impl fmt::Display for CacheNamespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CacheNamespace::Passage => write!(f, "passage"),
            CacheNamespace::Comment => write!(f, "comment"),
            CacheNamespace::Category => write!(f, "category"),
            CacheNamespace::Tag => write!(f, "tag"),
            CacheNamespace::Settings => write!(f, "settings"),
            CacheNamespace::User => write!(f, "user"),
            CacheNamespace::Stats => write!(f, "stats"),
            CacheNamespace::Music => write!(f, "music"),
            CacheNamespace::Attachment => write!(f, "attachment"),
        }
    }
}

/// 缓存资源类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum CacheResource {
    /// 列表
    List,
    /// 单个资源
    Get,
    /// 统计
    Stats,
    /// 搜索
    Search,
    /// 归档
    Archive,
}

impl fmt::Display for CacheResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CacheResource::List => write!(f, "list"),
            CacheResource::Get => write!(f, "get"),
            CacheResource::Stats => write!(f, "stats"),
            CacheResource::Search => write!(f, "search"),
            CacheResource::Archive => write!(f, "archive"),
        }
    }
}

/// 缓存键构建器
///
/// # 示例
/// ```rust,ignore
/// use crate::cache::keys::{CacheKeyBuilder, CacheNamespace, CacheResource};
///
/// // 构建文章列表缓存键
/// let key = CacheKeyBuilder::new(CacheNamespace::Passage, CacheResource::List)
///     .with_param("page", "1")
///     .with_param("limit", "10")
///     .with_version(1)
///     .build();
/// ```
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CacheKeyBuilder {
    namespace: CacheNamespace,
    resource: CacheResource,
    params: Vec<(String, String)>,
    version: Option<u32>,
}

impl CacheKeyBuilder {
    /// 创建新的缓存键构建器
    #[allow(dead_code)]
    pub fn new(namespace: CacheNamespace, resource: CacheResource) -> Self {
        Self {
            namespace,
            resource,
            params: Vec::new(),
            version: None,
        }
    }

    /// 添加参数
    #[allow(dead_code)]
    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.params.push((key.into(), value.into()));
        self
    }

    /// 添加参数（数字版本，避免 to_string() 调用）
    #[allow(dead_code)]
    pub fn with_param_int(mut self, key: impl Into<String>, value: i64) -> Self {
        self.params.push((key.into(), value.to_string()));
        self
    }

    /// 添加多个参数
    #[allow(dead_code)]
    pub fn with_params(
        mut self,
        params: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        for (key, value) in params {
            self.params.push((key.into(), value.into()));
        }
        self
    }

    /// 设置版本号
    #[allow(dead_code)]
    pub fn with_version(mut self, version: u32) -> Self {
        self.version = Some(version);
        self
    }

    /// 构建缓存键
    #[allow(dead_code)]
    pub fn build(self) -> String {
        // 预分配容量：2 个固定部分 + params.len() + 可能的版本号
        let capacity = 2 + self.params.len() + if self.version.is_some() { 1 } else { 0 };
        let mut parts = Vec::with_capacity(capacity);
        parts.push(self.namespace.to_string());
        parts.push(self.resource.to_string());

        // 添加参数
        for (key, value) in self.params {
            parts.push(format!("{}:{}", key, value));
        }

        // 添加版本号
        if let Some(v) = self.version {
            parts.push(format!("v{}", v));
        }

        parts.join(":")
    }

    /// 构建模式（用于批量删除）
    #[allow(dead_code)]
    pub fn build_pattern(self) -> String {
        let mut parts = vec![self.namespace.to_string(), self.resource.to_string()];

        // 添加参数（最后一个使用通配符）
        if let Some((key, _)) = self.params.last() {
            for (k, _) in &self.params {
                if k == key {
                    parts.push(format!("{}:*", k));
                    break;
                } else {
                    parts.push(format!("{}:{{value}}", k));
                }
            }
        }

        parts.join(":")
    }
}

/// 文章缓存键生成器
#[allow(dead_code)]
pub struct PassageCacheKeys;

#[allow(dead_code)]
impl PassageCacheKeys {
    /// 生成文章列表缓存键
    pub fn list(page: i64, limit: i64) -> String {
        CacheKeyBuilder::new(CacheNamespace::Passage, CacheResource::List)
            .with_param_int("page", page)
            .with_param_int("limit", limit)
            .with_version(1)
            .build()
    }

    /// 生成文章列表缓存键（游标分页）
    pub fn list_cursor(cursor: Option<&str>, limit: i64) -> String {
        let builder = CacheKeyBuilder::new(CacheNamespace::Passage, CacheResource::List)
            .with_param_int("limit", limit)
            .with_version(1);

        if let Some(c) = cursor {
            builder.with_param("cursor", c).build()
        } else {
            builder.with_param("cursor", "first").build()
        }
    }

    /// 生成文章列表缓存键（日期筛选）
    pub fn list_by_date(
        year: Option<i32>,
        month: Option<i32>,
        day: Option<i32>,
        page: i64,
        limit: i64,
    ) -> String {
        let builder = CacheKeyBuilder::new(CacheNamespace::Passage, CacheResource::List)
            .with_param_int("page", page)
            .with_param_int("limit", limit)
            .with_version(1);

        let date_part = match (year, month, day) {
            (Some(y), Some(m), Some(d)) => format!("{}-{:02}-{:02}", y, m, d),
            (Some(y), Some(m), None) => format!("{}-{:02}", y, m),
            (Some(y), None, None) => format!("{}", y),
            _ => "all".to_string(),
        };

        builder.with_param("date", date_part).build()
    }

    /// 生成单篇文章缓存键（通过 ID）
    pub fn get_by_id(id: i64) -> String {
        CacheKeyBuilder::new(CacheNamespace::Passage, CacheResource::Get)
            .with_param_int("id", id)
            .with_version(1)
            .build()
    }

    /// 生成单篇文章缓存键（通过 UUID）
    pub fn get_by_uuid(uuid: &str) -> String {
        CacheKeyBuilder::new(CacheNamespace::Passage, CacheResource::Get)
            .with_param("uuid", uuid)
            .with_version(1)
            .build()
    }

    /// 生成最新文章缓存键
    pub fn latest() -> String {
        "passage:latest".to_string()
    }

    /// 生成最新文章缓存模式（用于删除）
    pub fn latest_pattern() -> String {
        "passage:latest".to_string()
    }

    /// 生成文章列表缓存模式（用于批量删除）
    pub fn list_pattern() -> String {
        "passage:list:*".to_string()
    }

    /// 生成文章详情缓存模式（用于批量删除）
    pub fn get_pattern() -> String {
        "passage:get:*".to_string()
    }

    /// 生成所有文章缓存模式
    pub fn all_pattern() -> String {
        "passage:*".to_string()
    }
}

/// 评论缓存键生成器
#[allow(dead_code)]
pub struct CommentCacheKeys;

#[allow(dead_code)]
impl CommentCacheKeys {
    /// 生成评论列表缓存键
    pub fn list(passage_uuid: Option<&str>, page: i64, limit: i64) -> String {
        let builder = CacheKeyBuilder::new(CacheNamespace::Comment, CacheResource::List)
            .with_param_int("page", page)
            .with_param_int("limit", limit)
            .with_version(1);

        if let Some(uuid) = passage_uuid {
            builder.with_param("passage_uuid", uuid).build()
        } else {
            builder.build()
        }
    }

    /// 生成评论列表缓存模式
    pub fn list_pattern() -> String {
        "comment:list:*".to_string()
    }
}

/// 分类缓存键生成器
#[allow(dead_code)]
pub struct CategoryCacheKeys;

#[allow(dead_code)]
impl CategoryCacheKeys {
    /// 生成分类列表缓存键
    pub fn list() -> String {
        CacheKeyBuilder::new(CacheNamespace::Category, CacheResource::List)
            .with_version(1)
            .build()
    }

    /// 生成单个分类缓存键
    pub fn get(id: i64) -> String {
        CacheKeyBuilder::new(CacheNamespace::Category, CacheResource::Get)
            .with_param_int("id", id)
            .with_version(1)
            .build()
    }
}

/// 标签缓存键生成器
#[allow(dead_code)]
pub struct TagCacheKeys;

#[allow(dead_code)]
impl TagCacheKeys {
    /// 生成标签列表缓存键
    pub fn list() -> String {
        CacheKeyBuilder::new(CacheNamespace::Tag, CacheResource::List)
            .with_version(1)
            .build()
    }

    /// 生成单个标签缓存键
    pub fn get(id: i64) -> String {
        CacheKeyBuilder::new(CacheNamespace::Tag, CacheResource::Get)
            .with_param_int("id", id)
            .with_version(1)
            .build()
    }
}

/// 设置缓存键生成器
#[allow(dead_code)]
pub struct SettingsCacheKeys;

#[allow(dead_code)]
impl SettingsCacheKeys {
    /// 生成所有设置缓存键
    pub fn all() -> String {
        CacheKeyBuilder::new(CacheNamespace::Settings, CacheResource::Get)
            .with_version(1)
            .build()
    }

    /// 生成外观设置缓存键
    pub fn appearance() -> String {
        CacheKeyBuilder::new(CacheNamespace::Settings, CacheResource::Get)
            .with_param("type", "appearance")
            .with_version(1)
            .build()
    }

    /// 生成音乐设置缓存键
    pub fn music() -> String {
        CacheKeyBuilder::new(CacheNamespace::Settings, CacheResource::Get)
            .with_param("type", "music")
            .with_version(1)
            .build()
    }
}

/// 统计缓存键生成器
#[allow(dead_code)]
pub struct StatsCacheKeys;

#[allow(dead_code)]
impl StatsCacheKeys {
    /// 生成统计信息缓存键
    pub fn general() -> String {
        CacheKeyBuilder::new(CacheNamespace::Stats, CacheResource::Stats)
            .with_version(1)
            .build()
    }

    /// 生成分析数据缓存键
    pub fn analytics(metric: &str) -> String {
        CacheKeyBuilder::new(CacheNamespace::Stats, CacheResource::Stats)
            .with_param("metric", metric)
            .with_version(1)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_builder_basic() {
        let key = CacheKeyBuilder::new(CacheNamespace::Passage, CacheResource::List)
            .with_param("page", "1")
            .with_param("limit", "10")
            .build();

        assert_eq!(key, "passage:list:page:1:limit:10");
    }

    #[test]
    fn test_cache_key_builder_with_version() {
        let key = CacheKeyBuilder::new(CacheNamespace::Passage, CacheResource::List)
            .with_param("page", "1")
            .with_version(2)
            .build();

        assert_eq!(key, "passage:list:page:1:v2");
    }

    #[test]
    fn test_passage_cache_keys() {
        // 测试文章列表缓存键
        let key = PassageCacheKeys::list(1, 10);
        assert_eq!(key, "passage:list:page:1:limit:10:v1");

        // 测试文章详情缓存键
        let key = PassageCacheKeys::get_by_id(123);
        assert_eq!(key, "passage:get:id:123:v1");

        let key = PassageCacheKeys::get_by_uuid("abc-123");
        assert_eq!(key, "passage:get:uuid:abc-123:v1");
    }

    #[test]
    fn test_passage_cache_keys_date_filter() {
        let key = PassageCacheKeys::list_by_date(Some(2026), Some(2), Some(14), 1, 10);
        assert_eq!(key, "passage:list:page:1:limit:10:date:2026-02-14:v1");

        let key = PassageCacheKeys::list_by_date(Some(2026), Some(2), None, 1, 10);
        assert_eq!(key, "passage:list:page:1:limit:10:date:2026-02:v1");

        let key = PassageCacheKeys::list_by_date(Some(2026), None, None, 1, 10);
        assert_eq!(key, "passage:list:page:1:limit:10:date:2026:v1");
    }

    #[test]
    fn test_passage_cache_keys_cursor() {
        let key = PassageCacheKeys::list_cursor(Some("2026-02-14 10:00:00+00:00|123"), 10);
        assert_eq!(
            key,
            "passage:list:limit:10:cursor:2026-02-14 10:00:00+00:00|123:v1"
        );

        let key = PassageCacheKeys::list_cursor(None, 10);
        assert_eq!(key, "passage:list:limit:10:cursor:first:v1");
    }

    #[test]
    fn test_cache_patterns() {
        assert_eq!(PassageCacheKeys::list_pattern(), "passage:list:*");
        assert_eq!(PassageCacheKeys::get_pattern(), "passage:get:*");
        assert_eq!(PassageCacheKeys::all_pattern(), "passage:*");
    }

    #[test]
    fn test_settings_cache_keys() {
        let key = SettingsCacheKeys::all();
        assert_eq!(key, "settings:get:v1");

        let key = SettingsCacheKeys::appearance();
        assert_eq!(key, "settings:get:type:appearance:v1");

        let key = SettingsCacheKeys::music();
        assert_eq!(key, "settings:get:type:music:v1");
    }

    #[test]
    fn test_stats_cache_keys() {
        let key = StatsCacheKeys::general();
        assert_eq!(key, "stats:stats:v1");

        let key = StatsCacheKeys::analytics("most-viewed");
        assert_eq!(key, "stats:stats:metric:most-viewed:v1");
    }
}
