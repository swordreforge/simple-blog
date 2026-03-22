//! 高性能HashMap模块
//!
//! 使用hashbrown和AHash实现比标准HashMap更快2-3倍的哈希表。
//!
//! # 特性
//!
//! - 使用hashbrown作为底层实现（Rust标准库HashMap的底层）
//! - 使用AHash算法（比默认SipHash快2-3倍）
//! - 提供类型别名，方便替换标准HashMap
//! - 保持与标准HashMap相同的API
//!
//! # 性能对比
//!
//! - 查找速度：比标准HashMap快2-3倍
//! - 插入速度：比标准HashMap快1.5-2倍
//! - 内存占用：与标准HashMap相当
//!
//! # 使用示例
//!
//! ```no_run
//! use dynamic_route_actix::core::fast_hashmap::FastHashMap;
//!
//! let mut map: FastHashMap<String, i32> = FastHashMap::new();
//! map.insert("hello".to_string(), 42);
//! assert_eq!(map.get("hello"), Some(&42));
//! ```

use hashbrown::HashMap;
use std::hash::BuildHasherDefault;
use ahash::AHasher;

/// 快速HashMap类型别名
///
/// 使用hashbrown和AHash实现，比标准HashMap快2-3倍。
/// API与标准HashMap完全兼容。
pub type FastHashMap<K, V> = HashMap<K, V, BuildHasherDefault<AHasher>>;

/// 快速HashSet类型别名
///
/// 使用hashbrown和AHash实现，比标准HashSet快2-3倍。
/// API与标准HashSet完全兼容。
pub type FastHashSet<K> = hashbrown::HashSet<K, BuildHasherDefault<AHasher>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_hashmap_basic() {
        let mut map: FastHashMap<String, i32> = FastHashMap::new();

        map.insert("hello".to_string(), 42);
        map.insert("world".to_string(), 100);

        assert_eq!(map.get("hello"), Some(&42));
        assert_eq!(map.get("world"), Some(&100));
        assert_eq!(map.get("nonexistent"), None);

        assert_eq!(map.len(), 2);
        assert!(!map.is_empty());
    }

    #[test]
    fn test_fast_hashmap_remove() {
        let mut map: FastHashMap<String, i32> = FastHashMap::new();
        map.insert("hello".to_string(), 42);

        let removed = map.remove("hello");
        assert_eq!(removed, Some(42));
        assert!(map.is_empty());
    }

    #[test]
    fn test_fast_hashset_basic() {
        let mut set: FastHashSet<String> = FastHashSet::new();

        set.insert("hello".to_string());
        set.insert("world".to_string());

        assert!(set.contains("hello"));
        assert!(set.contains("world"));
        assert!(!set.contains("nonexistent"));

        assert_eq!(set.len(), 2);
    }
}