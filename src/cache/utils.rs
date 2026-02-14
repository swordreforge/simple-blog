/// 缓存工具模块
/// 提供统一的缓存失效操作，消除代码重复

use super::PassageCacheKeys;

/// 失效指定的缓存模式
///
/// # 参数
/// - `manager`: 缓存管理器（可选）
/// - `patterns`: 要失效的缓存模式列表
///
/// # 示例
/// ```rust,ignore
/// use crate::cache::utils::invalidate_cache_patterns;
///
/// invalidate_cache_patterns(
///     app_cache.manager(),
///     &[PassageCacheKeys::list_pattern(), PassageCacheKeys::get_pattern()]
/// ).await;
/// ```
pub async fn invalidate_cache_patterns(
    manager: Option<&crate::cache::manager::CacheManager>,
    patterns: &[&str],
) {
    if let Some(mgr) = manager {
        for pattern in patterns {
            let _ = mgr.delete_pattern(pattern).await;
        }
    }
}

/// 失效所有文章相关缓存
///
/// 这是最常用的缓存失效操作，当文章被创建、更新或删除时调用。
///
/// # 参数
/// - `manager`: 缓存管理器（可选）
///
/// # 示例
/// ```rust,ignore
/// use crate::cache::utils::invalidate_all_passage_cache;
///
/// // 文章创建、更新或删除后调用
/// invalidate_all_passage_cache(app_cache.manager()).await;
/// ```
pub async fn invalidate_all_passage_cache(
    manager: Option<&crate::cache::manager::CacheManager>,
) {
    invalidate_cache_patterns(
        manager,
        &[
            &PassageCacheKeys::list_pattern(),
            &PassageCacheKeys::get_pattern(),
        ]
    ).await;
}

/// 失效单篇文章缓存
///
/// 当单篇文章被更新或删除时调用，比 `invalidate_all_passage_cache` 更精确。
///
/// # 参数
/// - `manager`: 缓存管理器（可选）
/// - `passage_id`: 文章 ID
/// - `passage_uuid`: 文章 UUID
///
/// # 示例
/// ```rust,ignore
/// use crate::cache::utils::invalidate_passage_cache;
///
/// invalidate_passage_cache(
///     app_cache.manager(),
///     123,
///     "abc-123-def-456"
/// ).await;
/// ```
pub async fn invalidate_passage_cache(
    manager: Option<&crate::cache::manager::CacheManager>,
    passage_id: i64,
    passage_uuid: &str,
) {
    let cache_keys = vec![
        PassageCacheKeys::get_by_uuid(passage_uuid),
        PassageCacheKeys::get_by_id(passage_id),
    ];

    if let Some(mgr) = manager {
        for key in cache_keys {
            let _ = mgr.delete(&key).await;
        }
    }
}

/// 失效单篇文章缓存及其列表缓存
///
/// 当单篇文章被更新或删除时调用，同时失效文章详情和列表缓存。
///
/// # 参数
/// - `manager`: 缓存管理器（可选）
/// - `passage_id`: 文章 ID
/// - `passage_uuid`: 文章 UUID
///
/// # 示例
/// ```rust,ignore
/// use crate::cache::utils::invalidate_passage_and_list_cache;
///
/// invalidate_passage_and_list_cache(
///     app_cache.manager(),
///     123,
///     "abc-123-def-456"
/// ).await;
/// ```
pub async fn invalidate_passage_and_list_cache(
    manager: Option<&crate::cache::manager::CacheManager>,
    passage_id: i64,
    passage_uuid: &str,
) {
    // 先失效单篇文章缓存
    invalidate_passage_cache(manager, passage_id, passage_uuid).await;

    // 再失效列表缓存（因为文章状态可能改变）
    invalidate_cache_patterns(
        manager,
        &[
            &PassageCacheKeys::list_pattern(),
        ]
    ).await;
}

/// 失效分类相关缓存
///
/// 当分类被创建、更新或删除时调用。
///
/// # 参数
/// - `manager`: 缓存管理器（可选）
///
/// # 示例
/// ```rust,ignore
/// use crate::cache::utils::invalidate_category_cache;
///
/// invalidate_category_cache(app_cache.manager()).await;
/// ```
pub async fn invalidate_category_cache(
    manager: Option<&crate::cache::manager::CacheManager>,
) {
    invalidate_all_passage_cache(manager).await;
}

/// 失效标签相关缓存
///
/// 当标签被创建、更新或删除时调用。
///
/// # 参数
/// - `manager`: 缓存管理器（可选）
///
/// # 示例
/// ```rust,ignore
/// use crate::cache::utils::invalidate_tag_cache;
///
/// invalidate_tag_cache(app_cache.manager()).await;
/// ```
pub async fn invalidate_tag_cache(
    manager: Option<&crate::cache::manager::CacheManager>,
) {
    invalidate_all_passage_cache(manager).await;
}

/// 失效评论相关缓存
///
/// 当评论被创建、更新或删除时调用。
///
/// # 参数
/// - `manager`: 缓存管理器（可选）
/// - `passage_uuid`: 关联的文章 UUID（可选）
///
/// # 示例
/// ```rust,ignore
/// use crate::cache::utils::invalidate_comment_cache;
///
/// invalidate_comment_cache(app_cache.manager(), Some("abc-123")).await;
/// ```
pub async fn invalidate_comment_cache(
    manager: Option<&crate::cache::manager::CacheManager>,
    passage_uuid: Option<&str>,
) {
    if let Some(uuid) = passage_uuid {
        // 失效关联文章的缓存
        let key = PassageCacheKeys::get_by_uuid(uuid);
        if let Some(mgr) = manager {
            let _ = mgr.delete(&key).await;
        }
    }

    // 失效评论列表缓存
    invalidate_cache_patterns(
        manager,
        &["comment:*"]
    ).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_format() {
        // 测试缓存键格式
        let passage_id = 123;
        let passage_uuid = "abc-123-def-456";

        let key1 = PassageCacheKeys::get_by_uuid(passage_uuid);
        let key2 = PassageCacheKeys::get_by_id(passage_id);

        assert_eq!(key1, "passage:get:uuid:abc-123-def-456:v1");
        assert_eq!(key2, "passage:get:id:123:v1");
    }

    #[test]
    fn test_invalidate_all_passage_cache_patterns() {
        // 测试缓存模式字符串是否正确
        let patterns = [PassageCacheKeys::list_pattern(), PassageCacheKeys::get_pattern()];
        assert_eq!(patterns.len(), 2);
        assert_eq!(patterns[0], "passage:list:*");
        assert_eq!(patterns[1], "passage:get:*");
    }
}