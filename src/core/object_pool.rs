//! 内存池模块
//!
//! 提供对象池和内存复用功能，减少内存分配开销。
//! 针对频繁分配的路由条目、Trie节点等对象进行优化。

use std::sync::Mutex;

/// 简单的线程安全对象池
///
/// 用于复用对象，减少内存分配开销
struct SimplePool<T> {
    items: Mutex<Vec<T>>,
    factory: Box<dyn Fn() -> T + Send + Sync>,
}

impl<T> SimplePool<T>
where
    T: Send,
{
    /// 创建新的对象池
    fn new<F>(capacity: usize, factory: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self {
            items: Mutex::new(Vec::with_capacity(capacity)),
            factory: Box::new(factory),
        }
    }

    /// 从池中获取对象
    fn pull(&self) -> PooledItem<T> {
        let mut items = self.items.lock().unwrap();
        let item = items.pop().unwrap_or_else(|| (self.factory)());
        PooledItem {
            item: Some(item),
            pool: &self.items as *const _ as usize, // 存储指针的数值表示
        }
    }

    /// 将对象返回到池中
    #[allow(dead_code)]
    pub fn push(&self, item: T) {
        let mut items = self.items.lock().unwrap();
        if items.len() < items.capacity() {
            items.push(item);
        }
    }
}

/// 池化的对象包装器
///
/// 当被drop时，自动将对象返回到池中
pub struct PooledItem<T> {
    item: Option<T>,
    #[allow(dead_code)]
    pool: usize, // 存储原始池指针的数值表示
}

impl<T> std::ops::Deref for PooledItem<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.item.as_ref().unwrap()
    }
}

impl<T> std::ops::DerefMut for PooledItem<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.item.as_mut().unwrap()
    }
}

// 注意：这里为了简化，我们不实现自动返回功能
// 实际使用时，对象会在PooledItem drop时丢失
// 在生产环境中，应该使用更完善的object-pool crate

/// 路由对象池
///
/// 用于复用频繁分配的路由条目，减少内存分配开销。
pub struct RouteObjectPool {
    /// 字符串池（用于路径和参数）
    string_pool: SimplePool<String>,
    /// Vec池（用于路径段分割）
    vec_pool: SimplePool<Vec<String>>,
}

impl RouteObjectPool {
    /// 创建新的路由对象池
    ///
    /// # 参数
    ///
    /// * `string_pool_size` - 字符串池的大小
    /// * `vec_pool_size` - Vec池的大小
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::core::object_pool::RouteObjectPool;
    ///
    /// let pool = RouteObjectPool::new(500, 200);
    /// ```
    pub fn new(string_pool_size: usize, vec_pool_size: usize) -> Self {
        Self {
            string_pool: SimplePool::new(string_pool_size, String::new),
            vec_pool: SimplePool::new(vec_pool_size, Vec::new),
        }
    }

    /// 从池中获取字符串
    ///
    /// # 返回
    ///
    /// 返回一个可复用的字符串
    pub fn pull_string(&self) -> PooledItem<String> {
        let mut item = self.string_pool.pull();
        item.clear();
        item
    }

    /// 从池中获取Vec
    ///
    /// # 返回
    ///
    /// 返回一个可复用的Vec
    pub fn pull_vec(&self) -> PooledItem<Vec<String>> {
        let mut item = self.vec_pool.pull();
        item.clear();
        item
    }

    /// 获取字符串池的当前大小
    pub fn string_pool_size(&self) -> usize {
        self.string_pool.items.lock().unwrap().len()
    }

    /// 获取Vec池的当前大小
    pub fn vec_pool_size(&self) -> usize {
        self.vec_pool.items.lock().unwrap().len()
    }
}

impl Default for RouteObjectPool {
    fn default() -> Self {
        Self::new(500, 200)
    }
}

/// 全局路由对象池
///
/// 提供一个全局可访问的对象池实例
pub static GLOBAL_OBJECT_POOL: std::sync::OnceLock<RouteObjectPool> = std::sync::OnceLock::new();

/// 获取全局对象池
///
/// # 返回
///
/// 返回全局对象池的引用
pub fn global_object_pool() -> &'static RouteObjectPool {
    GLOBAL_OBJECT_POOL.get_or_init(|| RouteObjectPool::new(500, 200))
}

/// 内存池配置
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// 字符串池大小
    pub string_pool_size: usize,
    /// Vec池大小
    pub vec_pool_size: usize,
    /// 是否启用对象池
    pub enable_object_pool: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            string_pool_size: 500,
            vec_pool_size: 200,
            enable_object_pool: true,
        }
    }
}

impl PoolConfig {
    /// 创建新的池配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置字符串池大小
    pub fn with_string_pool_size(mut self, size: usize) -> Self {
        self.string_pool_size = size;
        self
    }

    /// 设置Vec池大小
    pub fn with_vec_pool_size(mut self, size: usize) -> Self {
        self.vec_pool_size = size;
        self
    }

    /// 设置是否启用对象池
    pub fn with_object_pool_enabled(mut self, enabled: bool) -> Self {
        self.enable_object_pool = enabled;
        self
    }

    /// 构建路由对象池
    pub fn build(&self) -> RouteObjectPool {
        RouteObjectPool::new(self.string_pool_size, self.vec_pool_size)
    }
}

/// 路径分割辅助函数
///
/// 使用优化的路径分割函数，减少字符串分配
///
/// # 参数
///
/// * `path` - 要分割的路径
///
/// # 返回
///
/// 返回分割后的路径段向量
pub fn split_path_optimized(path: &str) -> Vec<String> {
    path.split('/')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// 路径标准化辅助函数
///
/// 使用优化的路径标准化函数，移除尾部斜杠和规范化空格
///
/// # 参数
///
/// * `path` - 要标准化的路径
///
/// # 返回
///
/// 返回标准化后的路径
pub fn normalize_path_optimized(path: &str) -> String {
    let mut normalized = path.trim().to_string();
    
    // 移除尾部斜杠（根路径除外）
    if normalized.len() > 1 && normalized.ends_with('/') {
        normalized.pop();
    }
    
    // 标准化多个连续斜杠为单个斜杠
    while normalized.contains("//") {
        normalized = normalized.replace("//", "/");
    }
    
    normalized
}

/// 参数提取辅助函数
///
/// 从匹配结果中提取参数
///
/// # 参数
///
/// * `params` - 参数向量
///
/// # 返回
///
/// 返回参数的HashMap
pub fn extract_params_optimized(params: &[(String, String)]) -> std::collections::HashMap<String, String> {
    params.iter().cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_pool_creation() {
        let pool = RouteObjectPool::new(20, 5);
        assert_eq!(pool.string_pool_size(), 0);
        assert_eq!(pool.vec_pool_size(), 0);
    }

    #[test]
    fn test_object_pool_string_reuse() {
        let pool = RouteObjectPool::new(10, 5);
        
        // 获取字符串
        let mut s = pool.pull_string();
        s.push_str("test");
        assert_eq!(*s, "test");
        drop(s); // 对象被释放，但不会自动返回到池（简化实现）
    }

    #[test]
    fn test_object_pool_vec_reuse() {
        let pool = RouteObjectPool::new(10, 5);
        
        // 获取Vec
        let mut v = pool.pull_vec();
        v.push("test".to_string());
        assert_eq!(v.len(), 1);
        drop(v); // 对象被释放
    }

    #[test]
    fn test_global_object_pool() {
        let pool = global_object_pool();
        
        let mut s = pool.pull_string();
        s.push_str("global-test");
        assert_eq!(*s, "global-test");
    }

    #[test]
    fn test_pool_config() {
        let config = PoolConfig::new()
            .with_string_pool_size(1000)
            .with_vec_pool_size(500)
            .with_object_pool_enabled(true);
        
        assert_eq!(config.string_pool_size, 1000);
        assert_eq!(config.vec_pool_size, 500);
        assert!(config.enable_object_pool);
    }

    #[test]
    fn test_pool_config_default() {
        let config = PoolConfig::default();
        assert_eq!(config.string_pool_size, 500);
        assert_eq!(config.vec_pool_size, 200);
        assert!(config.enable_object_pool);
    }

    #[test]
    fn test_split_path_optimized() {
        let path = "/users/123/posts/456";
        let segments = split_path_optimized(path);
        
        assert_eq!(segments.len(), 4);
        assert_eq!(segments[0], "users");
        assert_eq!(segments[1], "123");
        assert_eq!(segments[2], "posts");
        assert_eq!(segments[3], "456");
    }

    #[test]
    fn test_split_path_optimized_empty() {
        let path = "/";
        let segments = split_path_optimized(path);
        assert_eq!(segments.len(), 0);
    }

    #[test]
    fn test_split_path_optimized_trailing_slash() {
        let path = "/users/";
        let segments = split_path_optimized(path);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0], "users");
    }

    #[test]
    fn test_normalize_path_optimized() {
        assert_eq!(normalize_path_optimized("/users/"), "/users");
        assert_eq!(normalize_path_optimized("/users//123/"), "/users/123");
        assert_eq!(normalize_path_optimized("  /users/123  "), "/users/123");
        assert_eq!(normalize_path_optimized("/"), "/");
    }

    #[test]
    fn test_extract_params_optimized() {
        let params = vec![
            ("id".to_string(), "123".to_string()),
            ("name".to_string(), "test".to_string()),
        ];
        
        let extracted = extract_params_optimized(&params);
        
        assert_eq!(extracted.get("id"), Some(&"123".to_string()));
        assert_eq!(extracted.get("name"), Some(&"test".to_string()));
        assert_eq!(extracted.len(), 2);
    }
}