//! 缓存工具模块
//! 提供统一的缓存失效操作，消除代码重复

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
pub async fn invalidate_all_passage_cache(manager: Option<&crate::cache::manager::CacheManager>) {
    invalidate_cache_patterns(
        manager,
        &[
            &PassageCacheKeys::list_pattern(),
            &PassageCacheKeys::get_pattern(),
            &PassageCacheKeys::latest_pattern(),
        ],
    )
    .await;
}

/// 细粒度失效文章缓存
///
/// 当单篇文章被更新时调用，只失效相关的缓存，避免清除所有缓存。
///
/// 相比 `invalidate_all_passage_cache`，此函数更精确，只失效：
/// - 文章详情缓存
/// - 文章列表缓存（所有）
/// - 如果有分类，失效该分类的列表缓存
/// - 如果有日期，失效该日期的列表缓存
///
/// # 参数
/// - `manager`: 缓存管理器（可选）
/// - `passage_id`: 文章 ID
/// - `passage_uuid`: 文章 UUID
/// - `category`: 文章分类（可选）
/// - `year`: 发布年份（可选）
/// - `month`: 发布月份（可选）
/// - `day`: 发布日期（可选）
///
/// # 示例
/// ```rust,ignore
/// use crate::cache::utils::invalidate_passage_cache_granular;
///
/// invalidate_passage_cache_granular(
///     app_cache.manager(),
///     123,
///     "abc-123-def-456",
///     Some("技术"),
///     Some(2026),
///     Some(3),
///     Some(21)
/// ).await;
/// ```
pub async fn invalidate_passage_cache_granular(
    manager: Option<&crate::cache::manager::CacheManager>,
    passage_id: i64,
    passage_uuid: &str,
    category: Option<&str>,
    year: Option<i32>,
    month: Option<i32>,
    day: Option<i32>,
) {
    // 预分配容量，最多 7 个键（3 个固定 + 1 个分类 + 3 个日期）
    let mut keys_to_invalidate = Vec::with_capacity(7);
    keys_to_invalidate.push(PassageCacheKeys::get_by_uuid(passage_uuid));
    keys_to_invalidate.push(PassageCacheKeys::get_by_id(passage_id));
    keys_to_invalidate.push(PassageCacheKeys::list_pattern().to_string());

    // 清除分类列表缓存
    if let Some(cat) = category {
        keys_to_invalidate.push(format!("passage:list:category:{}", cat));
    }

    // 清除日期列表缓存
    if let Some(y) = year {
        // 年份缓存
        keys_to_invalidate.push(format!("passage:list:date:{}:*", y));

        // 年月缓存
        if let Some(m) = month {
            keys_to_invalidate.push(format!("passage:list:date:{}-{:02}:*", y, m));

            // 年月日缓存
            if let Some(d) = day {
                keys_to_invalidate.push(format!("passage:list:date:{}-{:02}-{:02}:*", y, m, d));
            }
        }
    }

    // 批量删除缓存
    if let Some(mgr) = manager {
        for key in keys_to_invalidate {
            // 如果是模式（包含 *），使用 delete_pattern
            if key.contains('*') {
                let _ = mgr.delete_pattern(&key).await;
            } else {
                let _ = mgr.delete(&key).await;
            }
        }
    }
}

/// 失效特定分类缓存
///
/// 当特定分类被更新时调用，只失效该分类相关的缓存。
///
/// # 参数
/// - `manager`: 缓存管理器（可选）
/// - `category_name`: 分类名称
///
/// # 示例
/// ```rust,ignore
/// use crate::cache::utils::invalidate_specific_category_cache;
///
/// invalidate_specific_category_cache(app_cache.manager(), "技术").await;
/// ```
pub async fn invalidate_specific_category_cache(
    manager: Option<&crate::cache::manager::CacheManager>,
    category_name: &str,
) {
    if let Some(mgr) = manager {
        // 清除该分类的列表缓存
        let _ = mgr
            .delete_pattern(&format!("passage:list:category:{}", category_name))
            .await;

        // 清除分类本身的缓存
        let _ = mgr.delete_pattern("category:get:*").await;

        // 清除分类列表缓存
        let _ = mgr.delete_pattern("category:list:*").await;
    }
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
        let patterns = [
            PassageCacheKeys::list_pattern(),
            PassageCacheKeys::get_pattern(),
        ];
        assert_eq!(patterns.len(), 2);
        assert_eq!(patterns[0], "passage:list:*");
        assert_eq!(patterns[1], "passage:get:*");
    }
}
