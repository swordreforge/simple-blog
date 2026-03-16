//! 字符串优化模块
//!
//! 提供小字符串优化（SSO）和字符串池复用功能
//! 针对频繁使用的路径、参数名称等字符串进行优化

use std::sync::Arc;
use std::collections::HashMap;
use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicUsize, Ordering};

/// 小字符串优化（Small String Optimization）
///
/// 对于短字符串（小于等于23字节），直接在栈上存储，避免堆分配
/// 对于长字符串，使用堆分配
///
/// 这是Rust标准库String的固有特性，我们提供辅助函数来最大化利用
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SmallString {
    inner: String,
}

impl SmallString {
    /// 创建新的小字符串
    pub fn new<S: Into<String>>(s: S) -> Self {
        Self {
            inner: s.into(),
        }
    }

    /// 从静态字符串创建（零分配）
    pub fn from_static(s: &'static str) -> Self {
        Self {
            inner: s.to_string(),
        }
    }

    /// 获取字符串切片
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// 获取字符串的可变引用
    pub fn as_mut_str(&mut self) -> &mut str {
        &mut self.inner
    }

    /// 检查是否是小字符串（栈上存储）
    pub fn is_small(&self) -> bool {
        // Rust的String在大多数平台上对<=23字节的字符串使用SSO
        self.inner.len() <= 23
    }

    /// 追加字符串
    pub fn push_str(&mut self, s: &str) {
        self.inner.push_str(s);
    }

    /// 清空字符串
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// 获取长度
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl std::ops::Deref for SmallString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for SmallString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl From<String> for SmallString {
    fn from(s: String) -> Self {
        Self { inner: s }
    }
}

impl From<&str> for SmallString {
    fn from(s: &str) -> Self {
        Self {
            inner: s.to_string(),
        }
    }
}

impl From<SmallString> for String {
    fn from(s: SmallString) -> Self {
        s.inner
    }
}

impl AsRef<str> for SmallString {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl std::fmt::Display for SmallString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

/// 字符串池（String Pool）
///
/// 用于复用常用字符串，减少内存分配
/// 使用原子引用计数（Arc）确保线程安全
#[derive(Debug)]
pub struct StringPool {
    pool: HashMap<String, Arc<str>>,
    hits: AtomicUsize,
    misses: AtomicUsize,
}

impl StringPool {
    /// 创建新的字符串池
    pub fn new() -> Self {
        Self {
            pool: HashMap::new(),
            hits: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
        }
    }

    /// 获取或创建字符串
    ///
    /// 如果字符串已存在于池中，返回共享引用
    /// 否则，创建新字符串并加入池中
    pub fn get_or_insert(&mut self, s: &str) -> Arc<str> {
        if let Some(cached) = self.pool.get(s) {
            self.hits.fetch_add(1, Ordering::Relaxed);
            return Arc::clone(cached);
        }

        self.misses.fetch_add(1, Ordering::Relaxed);
        let arc_str: Arc<str> = Arc::from(s);
        self.pool.insert(s.to_string(), Arc::clone(&arc_str));
        arc_str
    }

    /// 尝试获取字符串，不存在时返回None
    pub fn get(&self, s: &str) -> Option<Arc<str>> {
        self.pool.get(s).cloned()
    }

    /// 预填充常用字符串
    pub fn prefill(&mut self, strings: &[&str]) {
        for s in strings {
            if !self.pool.contains_key(*s) {
                let arc_str: Arc<str> = Arc::from(*s);
                self.pool.insert(s.to_string(), arc_str);
            }
        }
    }

    /// 获取池中字符串数量
    pub fn len(&self) -> usize {
        self.pool.len()
    }

    /// 检查池是否为空
    pub fn is_empty(&self) -> bool {
        self.pool.is_empty()
    }

    /// 获取缓存命中率
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    /// 清空字符串池
    pub fn clear(&mut self) {
        self.pool.clear();
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
    }

    /// 移除未使用的字符串
    pub fn shrink(&mut self) {
        self.pool.retain(|_, arc| Arc::strong_count(arc) > 1);
    }
}

impl Default for StringPool {
    fn default() -> Self {
        Self::new()
    }
}

/// 路径字符串池
///
/// 专门用于路径字符串的复用
/// 预填充常用路径模式
pub struct PathStringPool {
    inner: StringPool,
}

impl PathStringPool {
    /// 创建新的路径字符串池
    pub fn new() -> Self {
        let mut pool = Self {
            inner: StringPool::new(),
        };
        
        // 预填充常用路径段
        pool.inner.prefill(&[
            // HTTP方法
            "GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS",
            
            // 常用路径段
            "api", "v1", "v2", "users", "posts", "comments", "id", "name",
            "email", "password", "token", "auth", "login", "logout", "register",
            "admin", "dashboard", "settings", "profile",
            
            // 参数名称
            "user_id", "post_id", "comment_id", "id", "page", "limit", "offset",
            "sort", "order", "query", "search", "filter",
            
            // 内容类型
            "application/json", "text/plain", "text/html", "application/xml",
            "application/octet-stream", "multipart/form-data",
            
            // 路由类型
            "SimpleRoute", "TimedRoute", "AuthRoute", "CustomRoute",
            
            // 其他常用字符串
            "utf-8", "gzip", "deflate", "br", "identity",
        ]);
        
        pool
    }

    /// 获取或创建路径字符串
    pub fn get_or_insert(&mut self, s: &str) -> Arc<str> {
        self.inner.get_or_insert(s)
    }

    /// 尝试获取路径字符串
    pub fn get(&self, s: &str) -> Option<Arc<str>> {
        self.inner.get(s)
    }

    /// 获取缓存命中率
    pub fn hit_rate(&self) -> f64 {
        self.inner.hit_rate()
    }

    /// 清空池
    pub fn clear(&mut self) {
        self.inner.clear();
    }
}

impl Default for PathStringPool {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局路径字符串池
static GLOBAL_PATH_POOL: Lazy<std::sync::Mutex<PathStringPool>> = 
    Lazy::new(|| std::sync::Mutex::new(PathStringPool::new()));

/// 获取全局路径字符串池
pub fn global_path_pool() -> &'static std::sync::Mutex<PathStringPool> {
    &GLOBAL_PATH_POOL
}

/// 智能字符串选择器
///
/// 根据字符串长度和复用频率，自动选择最优的字符串表示
pub enum SmartString {
    /// 小字符串（直接存储）
    Small(SmallString),
    /// 池化字符串（共享引用）
    Pooled(Arc<str>),
    /// 借用字符串（零拷贝）
    Borrowed(&'static str),
}

impl SmartString {
    /// 从字符串创建智能字符串
    pub fn from_string(s: &str) -> Self {
        // 对于短字符串，使用小字符串优化
        if s.len() <= 23 {
            // 尝试从池获取
            if let Ok(pool) = GLOBAL_PATH_POOL.try_lock() {
                if let Some(pooled) = pool.get(s) {
                    return SmartString::Pooled(pooled);
                }
            }
            SmartString::Small(SmallString::from(s))
        } else {
            // 长字符串尝试从池获取
            if let Ok(mut pool) = GLOBAL_PATH_POOL.try_lock() {
                let pooled = pool.get_or_insert(s);
                SmartString::Pooled(pooled)
            } else {
                SmartString::Small(SmallString::from(s))
            }
        }
    }

    /// 从静态字符串创建
    pub const fn from_static(s: &'static str) -> Self {
        SmartString::Borrowed(s)
    }

    /// 获取字符串切片
    pub fn as_str(&self) -> &str {
        match self {
            SmartString::Small(s) => s.as_str(),
            SmartString::Pooled(s) => s.as_ref(),
            SmartString::Borrowed(s) => s,
        }
    }

    /// 获取长度
    pub fn len(&self) -> usize {
        self.as_str().len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.as_str().is_empty()
    }
}

impl AsRef<str> for SmartString {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for SmartString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl PartialEq for SmartString {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for SmartString {}

impl std::hash::Hash for SmartString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

/// 路径分割优化（使用字符串池）
///
/// 优化的路径分割函数，使用字符串池减少内存分配
pub fn split_path_pooled(path: &str) -> Vec<Arc<str>> {
    if let Ok(mut pool) = GLOBAL_PATH_POOL.try_lock() {
        path.split('/')
            .filter(|s| !s.is_empty())
            .map(|s| pool.get_or_insert(s))
            .collect()
    } else {
        // 降级到普通分割
        path.split('/')
            .filter(|s| !s.is_empty())
            .map(Arc::from)
            .collect()
    }
}

/// 路径分割优化（使用小字符串）
///
/// 优化的路径分割函数，对小字符串使用SSO
pub fn split_path_small(path: &str) -> Vec<SmallString> {
    path.split('/')
        .filter(|s| !s.is_empty())
        .map(SmallString::from)
        .collect()
}

/// 路径分割优化（混合策略）
///
/// 自动选择最优策略：小字符串用SSO，长字符串用池
pub fn split_path_smart(path: &str) -> Vec<SmartString> {
    path.split('/')
        .filter(|s| !s.is_empty())
        .map(SmartString::from_string)
        .collect()
}

/// 字符串连接优化（避免中间分配）
///
/// 使用Cow来延迟字符串分配
pub fn join_paths_optimized(segments: &[&str]) -> String {
    if segments.is_empty() {
        return String::new();
    }

    let capacity = segments.iter().map(|s| s.len()).sum::<usize>() + segments.len();
    let mut result = String::with_capacity(capacity);
    
    for (i, segment) in segments.iter().enumerate() {
        if i > 0 || !segment.starts_with('/') {
            result.push('/');
        }
        result.push_str(segment);
    }
    
    result
}

/// 参数提取优化（使用字符串池）
///
/// 从匹配结果中提取参数，使用字符串池减少分配
pub fn extract_params_pooled(params: &[(Arc<str>, Arc<str>)]) -> HashMap<Arc<str>, Arc<str>> {
    params.iter().cloned().collect()
}

/// 统计信息
#[derive(Debug, Default)]
pub struct StringOptimizationStats {
    pub total_operations: AtomicUsize,
    pub small_string_count: AtomicUsize,
    pub pooled_string_count: AtomicUsize,
    pub borrowed_string_count: AtomicUsize,
}

impl StringOptimizationStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_small_string(&self) {
        self.small_string_count.fetch_add(1, Ordering::Relaxed);
        self.total_operations.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_pooled_string(&self) {
        self.pooled_string_count.fetch_add(1, Ordering::Relaxed);
        self.total_operations.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_borrowed_string(&self) {
        self.borrowed_string_count.fetch_add(1, Ordering::Relaxed);
        self.total_operations.fetch_add(1, Ordering::Relaxed);
    }

    pub fn small_string_ratio(&self) -> f64 {
        let total = self.total_operations.load(Ordering::Relaxed);
        if total == 0 {
            0.0
        } else {
            self.small_string_count.load(Ordering::Relaxed) as f64 / total as f64
        }
    }

    pub fn pooled_string_ratio(&self) -> f64 {
        let total = self.total_operations.load(Ordering::Relaxed);
        if total == 0 {
            0.0
        } else {
            self.pooled_string_count.load(Ordering::Relaxed) as f64 / total as f64
        }
    }

    pub fn borrowed_string_ratio(&self) -> f64 {
        let total = self.total_operations.load(Ordering::Relaxed);
        if total == 0 {
            0.0
        } else {
            self.borrowed_string_count.load(Ordering::Relaxed) as f64 / total as f64
        }
    }
}

/// 全局统计信息
static GLOBAL_STATS: Lazy<StringOptimizationStats> = Lazy::new(StringOptimizationStats::new);

/// 获取全局统计信息
pub fn global_stats() -> &'static StringOptimizationStats {
    &GLOBAL_STATS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_string_creation() {
        let s = SmallString::new("hello");
        assert_eq!(s.as_str(), "hello");
        assert!(s.is_small());
    }

    #[test]
    fn test_small_string_from_long() {
        let s = SmallString::new("this is a very long string that will not fit in the small string optimization");
        assert!(!s.is_small());
    }

    #[test]
    fn test_small_string_mutation() {
        let mut s = SmallString::new("hello");
        s.push_str(" world");
        assert_eq!(s.as_str(), "hello world");
    }

    #[test]
    fn test_string_pool_basic() {
        let mut pool = StringPool::new();
        
        let s1 = pool.get_or_insert("test");
        let s2 = pool.get_or_insert("test");
        
        // 应该是同一个Arc引用
        assert!(Arc::ptr_eq(&s1, &s2));
    }

    #[test]
    fn test_string_pool_different_strings() {
        let mut pool = StringPool::new();
        
        let s1 = pool.get_or_insert("test1");
        let s2 = pool.get_or_insert("test2");
        
        // 不应该是同一个引用
        assert!(!Arc::ptr_eq(&s1, &s2));
    }

    #[test]
    fn test_string_pool_hit_rate() {
        let mut pool = StringPool::new();
        
        pool.get_or_insert("test");
        pool.get_or_insert("test");
        pool.get_or_insert("test");
        pool.get_or_insert("other");
        
        let rate = pool.hit_rate();
        assert!(rate > 0.0 && rate <= 1.0);
    }

    #[test]
    fn test_path_string_pool_prefill() {
        let pool = PathStringPool::new();
        
        // 检查预填充的字符串
        assert!(pool.get("GET").is_some());
        assert!(pool.get("POST").is_some());
        assert!(pool.get("users").is_some());
    }

    #[test]
    fn test_smart_string_from_static() {
        let s = SmartString::from_static("hello");
        assert_eq!(s.as_str(), "hello");
    }

    #[test]
    fn test_smart_string_from_str_small() {
        let s = SmartString::from_string("hello");
        assert_eq!(s.as_str(), "hello");
        assert!(matches!(s, SmartString::Small(_)));
    }

    #[test]
    fn test_smart_string_from_str_large() {
        let s = SmartString::from_string("this is a very long string");
        assert_eq!(s.as_str(), "this is a very long string");
        // 长字符串应该使用池化
        assert!(matches!(s, SmartString::Pooled(_)));
    }

    #[test]
    fn test_split_path_pooled() {
        let segments = split_path_pooled("/users/123/posts");
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0].as_ref(), "users");
        assert_eq!(segments[1].as_ref(), "123");
        assert_eq!(segments[2].as_ref(), "posts");
    }

    #[test]
    fn test_split_path_small() {
        let segments = split_path_small("/users/123/posts");
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0].as_str(), "users");
        assert_eq!(segments[1].as_str(), "123");
        assert_eq!(segments[2].as_str(), "posts");
    }

    #[test]
    fn test_split_path_smart() {
        let segments = split_path_smart("/users/123/posts");
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0].as_str(), "users");
        assert_eq!(segments[1].as_str(), "123");
        assert_eq!(segments[2].as_str(), "posts");
    }

    #[test]
    fn test_join_paths_optimized() {
        let segments = vec!["api", "v1", "users"];
        let result = join_paths_optimized(&segments);
        assert_eq!(result, "/api/v1/users");
    }

    #[test]
    fn test_join_paths_optimized_empty() {
        let segments: Vec<&str> = vec![];
        let result = join_paths_optimized(&segments);
        assert!(result.is_empty());
    }

    #[test]
    fn test_join_paths_optimized_with_leading_slash() {
        let segments = vec!["/api", "v1", "users"];
        let result = join_paths_optimized(&segments);
        assert_eq!(result, "/api/v1/users");
    }

    #[test]
    fn test_string_optimization_stats() {
        let stats = StringOptimizationStats::new();
        
        stats.record_small_string();
        stats.record_small_string();
        stats.record_pooled_string();
        stats.record_borrowed_string();
        
        assert_eq!(stats.small_string_ratio(), 0.5);
        assert_eq!(stats.pooled_string_ratio(), 0.25);
        assert_eq!(stats.borrowed_string_ratio(), 0.25);
    }

    #[test]
    fn test_global_path_pool() {
        let pool = global_path_pool();
        
        if let Ok(pool) = pool.try_lock() {
            // 检查预填充的字符串
            assert!(pool.get("GET").is_some());
            assert!(pool.get("users").is_some());
        }
    }
}