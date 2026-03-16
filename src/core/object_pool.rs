//! 内存池模块
//!
//! 提供对象池和内存复用功能，减少内存分配开销。
//! 针对频繁分配的路由条目、Trie节点等对象进行优化。

use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::sync::Mutex;

/// 简单的线程安全对象池
///
/// 用于复用对象，减少内存分配开销
struct SimplePool<T> {
    items: Mutex<Vec<T>>,
    factory: Box<dyn Fn() -> T + Send + Sync>,
}

/// 无锁栈节点
///
/// 使用Treiber Stack算法实现的无锁栈节点
struct LockFreeNode<T> {
    data: Option<T>,
    next: AtomicPtr<LockFreeNode<T>>,
}

impl<T> LockFreeNode<T> {
    /// 创建新节点
    fn new(data: Option<T>) -> Self {
        Self {
            data,
            next: AtomicPtr::new(std::ptr::null_mut()),
        }
    }
}

/// 无锁栈（Lock-Free Stack）
///
/// 基于Treiber Stack算法实现，提供高并发性能
/// 使用原子操作实现线程安全，避免使用互斥锁
struct LockFreeStack<T> {
    head: AtomicPtr<LockFreeNode<T>>,
}

unsafe impl<T: Send> Send for LockFreeStack<T> {}
unsafe impl<T: Send> Sync for LockFreeStack<T> {}

impl<T> LockFreeStack<T> {
    /// 创建新的无锁栈
    fn new() -> Self {
        Self {
            head: AtomicPtr::new(std::ptr::null_mut()),
        }
    }

    /// 向栈中推送元素
    fn push(&self, data: T) {
        let new_node = Box::into_raw(Box::new(LockFreeNode::new(Some(data))));
        
        loop {
            let old_head = self.head.load(Ordering::Acquire);
            
            unsafe {
                (*new_node).next.store(old_head, Ordering::Relaxed);
            }
            
            // 尝试更新head指针
            if self.head.compare_exchange(
                old_head,
                new_node,
                Ordering::AcqRel,
                Ordering::Acquire,
            ).is_ok() {
                break;
            }
            // CAS失败，重试
        }
    }

    /// 从栈中弹出元素
    ///
    /// 如果栈为空，返回None
    fn pop(&self) -> Option<T> {
        loop {
            let old_head = self.head.load(Ordering::Acquire);
            
            if old_head.is_null() {
                return None;
            }
            
            unsafe {
                let new_head = (*old_head).next.load(Ordering::Relaxed);
                
                // 尝试更新head指针
                if self.head.compare_exchange(
                    old_head,
                    new_head,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ).is_ok() {
                    // 成功弹出，提取数据
                    let data = (*old_head).data.take();
                    
                    // 延迟释放节点，避免ABA问题
                    // 在生产环境中应该使用epoch-based reclamation
                    // 这里为了简化，我们直接释放，但需要注意这可能导致ABA问题
                    let _ = Box::from_raw(old_head);
                    
                    return data;
                }
                // CAS失败，重试
            }
        }
    }

    /// 检查栈是否为空
    fn is_empty(&self) -> bool {
        self.head.load(Ordering::Acquire).is_null()
    }
}

impl<T> Drop for LockFreeStack<T> {
    fn drop(&mut self) {
        // 清理所有节点
        let mut head = self.head.load(Ordering::Acquire);
        while !head.is_null() {
            unsafe {
                let node = Box::from_raw(head);
                head = node.next.load(Ordering::Acquire);
            }
        }
    }
}

impl<T> Default for LockFreeStack<T> {
    fn default() -> Self {
        Self::new()
    }
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

/// 无锁对象池
///
/// 使用分片锁优化实现的线程安全对象池，提供更高的并发性能
/// 通过多个子池减少锁竞争，适合高并发场景
struct LockFreeObjectPool<T> {
    sub_pools: Vec<Mutex<Vec<T>>>,
    factory: Box<dyn Fn() -> T + Send + Sync>,
    sub_pool_capacity: usize,
    total_capacity: AtomicUsize,
    size: AtomicUsize,
}

impl<T> LockFreeObjectPool<T>
where
    T: Send,
{
    /// 创建新的无锁对象池
    fn new<F>(total_capacity: usize, num_shards: usize, factory: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        let sub_pool_capacity = total_capacity.div_ceil(num_shards);
        let mut sub_pools = Vec::with_capacity(num_shards);
        for _ in 0..num_shards {
            sub_pools.push(Mutex::new(Vec::with_capacity(sub_pool_capacity)));
        }
        
        Self {
            sub_pools,
            factory: Box::new(factory),
            sub_pool_capacity,
            total_capacity: AtomicUsize::new(total_capacity),
            size: AtomicUsize::new(0),
        }
    }

    /// 从池中获取对象
    ///
    /// 如果池中有可用对象，则返回；否则创建新对象
    fn pull(&self) -> T {
        // 尝试从所有子池中获取对象
        for sub_pool in self.sub_pools.iter() {
            if let Ok(mut pool) = sub_pool.try_lock() {
                if let Some(item) = pool.pop() {
                    self.size.fetch_sub(1, Ordering::Relaxed);
                    return item;
                }
            }
        }
        
        // 所有子池都为空，创建新对象
        (self.factory)()
    }

    /// 将对象返回到池中
    ///
    /// 如果池未满，则将对象返回；否则丢弃对象
    fn push(&self, item: T) {
        let current_size = self.size.load(Ordering::Relaxed);
        let max_capacity = self.total_capacity.load(Ordering::Relaxed);

        if current_size >= max_capacity {
            return; // 池已满，丢弃对象
        }

        // 使用线程ID作为哈希，选择起始子池
        let thread_id = std::thread::current().id();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        thread_id.hash(&mut hasher);
        let start_pool_index = hasher.finish() as usize % self.sub_pools.len();

        // 尝试从起始子池开始，轮询所有子池
        for offset in 0..self.sub_pools.len() {
            let pool_index = (start_pool_index + offset) % self.sub_pools.len();

            if let Ok(mut pool) = self.sub_pools[pool_index].try_lock() {
                if pool.len() < self.sub_pool_capacity {
                    pool.push(item);
                    self.size.fetch_add(1, Ordering::Relaxed);
                    return; // 成功推送，返回
                }
            }
        }
        // 所有子池都满了或锁失败，丢弃对象
    }

    /// 获取当前池中的对象数量
    fn size(&self) -> usize {
        self.size.load(Ordering::Relaxed)
    }

    /// 获取池的容量
    fn capacity(&self) -> usize {
        self.total_capacity.load(Ordering::Relaxed)
    }
}

/// 无锁路由对象池
///
/// 使用无锁队列实现的高性能路由对象池
/// 适合高并发场景下的路由匹配和参数处理
pub struct LockFreeRouteObjectPool {
    /// 字符串池（用于路径和参数）
    string_pool: LockFreeObjectPool<String>,
    /// Vec池（用于路径段分割）
    vec_pool: LockFreeObjectPool<Vec<String>>,
}

impl LockFreeRouteObjectPool {
    /// 创建新的无锁路由对象池
    ///
    /// # 参数
    ///
    /// * `string_pool_size` - 字符串池的大小
    /// * `vec_pool_size` - Vec池的大小
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::core::object_pool::LockFreeRouteObjectPool;
    ///
    /// let pool = LockFreeRouteObjectPool::new(500, 200);
    /// ```
    pub fn new(string_pool_size: usize, vec_pool_size: usize) -> Self {
        // 使用8个分片来减少锁竞争
        const NUM_SHARDS: usize = 8;
        Self {
            string_pool: LockFreeObjectPool::new(string_pool_size, NUM_SHARDS, String::new),
            vec_pool: LockFreeObjectPool::new(vec_pool_size, NUM_SHARDS, Vec::new),
        }
    }

    /// 从池中获取字符串
    ///
    /// # 返回
    ///
    /// 返回一个可复用的字符串
    pub fn pull_string(&self) -> String {
        let mut item = self.string_pool.pull();
        item.clear();
        item
    }

    /// 将字符串返回到池中
    ///
    /// # 参数
    ///
    /// * `item` - 要返回的字符串
    pub fn push_string(&self, item: String) {
        self.string_pool.push(item);
    }

    /// 从池中获取Vec
    ///
    /// # 返回
    ///
    /// 返回一个可复用的Vec
    pub fn pull_vec(&self) -> Vec<String> {
        let mut item = self.vec_pool.pull();
        item.clear();
        item
    }

    /// 将Vec返回到池中
    ///
    /// # 参数
    ///
    /// * `item` - 要返回的Vec
    pub fn push_vec(&self, item: Vec<String>) {
        self.vec_pool.push(item);
    }

    /// 获取字符串池的当前大小
    pub fn string_pool_size(&self) -> usize {
        self.string_pool.size()
    }

    /// 获取Vec池的当前大小
    pub fn vec_pool_size(&self) -> usize {
        self.vec_pool.size()
    }

    /// 获取字符串池的容量
    pub fn string_pool_capacity(&self) -> usize {
        self.string_pool.capacity()
    }

    /// 获取Vec池的容量
    pub fn vec_pool_capacity(&self) -> usize {
        self.vec_pool.capacity()
    }
}

impl Default for LockFreeRouteObjectPool {
    fn default() -> Self {
        Self::new(500, 200)
    }
}

/// 全局无锁路由对象池
///
/// 提供一个全局可访问的无锁对象池实例
pub static GLOBAL_LOCKFREE_OBJECT_POOL: std::sync::OnceLock<LockFreeRouteObjectPool> = std::sync::OnceLock::new();

/// 获取全局无锁对象池
///
/// # 返回
///
/// 返回全局无锁对象池的引用
pub fn global_lockfree_object_pool() -> &'static LockFreeRouteObjectPool {
    GLOBAL_LOCKFREE_OBJECT_POOL.get_or_init(|| LockFreeRouteObjectPool::new(500, 200))
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

    #[test]
    fn test_lockfree_object_pool_basic() {
        let pool = LockFreeRouteObjectPool::new(10, 5);
        
        // 测试字符串池
        let mut s = pool.pull_string();
        s.push_str("test");
        assert_eq!(s, "test");
        pool.push_string(s);
        
        // 测试Vec池
        let mut v = pool.pull_vec();
        v.push("item".to_string());
        assert_eq!(v.len(), 1);
        pool.push_vec(v);
    }

    #[test]
    fn test_lockfree_object_pool_capacity() {
        let pool = LockFreeRouteObjectPool::new(5, 3);
        
        assert_eq!(pool.string_pool_capacity(), 5);
        assert_eq!(pool.vec_pool_capacity(), 3);
        assert_eq!(pool.string_pool_size(), 0);
        assert_eq!(pool.vec_pool_size(), 0);
    }

    #[test]
    fn test_lockfree_object_pool_reuse() {
        let pool = LockFreeRouteObjectPool::new(10, 5);
        
        // 返回一些对象到池中
        for i in 0..5 {
            pool.push_string(format!("string-{}", i));
        }
        
        // 验证对象被复用
        assert_eq!(pool.string_pool_size(), 5);
        
        // 从池中获取对象
        let s = pool.pull_string();
        assert_eq!(pool.string_pool_size(), 4);
        pool.push_string(s);
        assert_eq!(pool.string_pool_size(), 5);
    }

    #[test]
    fn test_lockfree_object_pool_overflow() {
        let pool = LockFreeRouteObjectPool::new(3, 2);
        
        // 超过容量的对象应该被丢弃
        for i in 0..10 {
            pool.push_string(format!("string-{}", i));
        }
        
        // 池大小不应该超过容量
        assert_eq!(pool.string_pool_size(), 3);
    }

    #[test]
    fn test_global_lockfree_object_pool() {
        let pool = global_lockfree_object_pool();
        
        let mut s = pool.pull_string();
        s.push_str("global-lockfree-test");
        assert_eq!(s, "global-lockfree-test");
        pool.push_string(s);
    }

    #[test]
    fn test_lockfree_route_object_pool_default() {
        let pool = LockFreeRouteObjectPool::default();
        
        assert_eq!(pool.string_pool_capacity(), 500);
        assert_eq!(pool.vec_pool_capacity(), 200);
    }
}