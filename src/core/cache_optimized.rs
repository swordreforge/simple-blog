//! 缓存优化模块
//!
//! 提供缓存友好的数据结构优化，减少缓存未命中，提升整体性能。
//! 使用更紧凑的数据结构、内存布局优化和缓存感知的算法。

use std::hash::{Hash, Hasher};

/// 紧凑的Radix Tree节点边
///
/// 优化内存布局，减少内存占用和缓存未命中
#[derive(Debug, Clone)]
pub struct CompactRadixEdge {
    /// 边的前缀（用于路径压缩）
    prefix: String,
    /// 子节点索引（使用索引而非指针，提高缓存局部性）
    node_index: usize,
}

/// 紧凑的Radix Tree节点
///
/// 使用扁平化的存储方式，减少指针追踪，提升缓存命中率
#[derive(Debug)]
pub struct CompactRadixNode {
    /// 节点类型
    node_type: RadixNodeType,
    /// 子边（使用扁平数组存储）
    children: Vec<CompactRadixEdge>,
    /// 参数化子节点索引
    param_child_index: Option<usize>,
    /// 通配符子节点索引
    wildcard_child_index: Option<usize>,
    /// 路由处理器索引（存储在单独的路由处理器数组中）
    route_index: Option<usize>,
}

impl CompactRadixNode {
    /// 创建新的紧凑Radix节点
    pub fn new(node_type: RadixNodeType) -> Self {
        Self {
            node_type,
            children: Vec::new(),
            param_child_index: None,
            wildcard_child_index: None,
            route_index: None,
        }
    }
}

/// 紧凑的Radix Tree
///
/// 使用扁平化的节点存储，减少内存碎片和缓存未命中
#[derive(Debug)]
pub struct CompactRadixTree {
    /// 扁平化的节点存储（连续内存，缓存友好）
    nodes: Vec<CompactRadixNode>,
    /// 路由处理器存储（单独的数组，保持紧凑性）
    routes: Vec<Box<dyn super::RouteEntry>>,
    /// 根节点索引
    root_index: usize,
}

impl Default for CompactRadixTree {
    fn default() -> Self {
        Self::new()
    }
}

impl CompactRadixTree {
    /// 创建新的紧凑Radix Tree
    pub fn new() -> Self {
        let mut nodes = Vec::new();
        let root_index = nodes.len();
        nodes.push(CompactRadixNode::new(RadixNodeType::Static(String::new())));
        
        Self {
            nodes,
            routes: Vec::new(),
            root_index,
        }
    }
    
    /// 插入路由
    pub fn insert(&mut self, path: &str, route: Box<dyn super::RouteEntry>) {
        let route_index = self.routes.len();
        self.routes.push(route);
        
        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let mut current_index = self.root_index;
        
        for segment in segments {
            let node_type = RadixNodeType::from_segment(segment);
            let child_index = self.find_or_create_child(current_index, node_type);
            current_index = child_index;
        }
        
        // 在最终节点设置路由
        if let Some(node) = self.nodes.get_mut(current_index) {
            node.route_index = Some(route_index);
        }
    }
    
    /// 查找路由
    pub fn find(&self, path: &str) -> Option<(&Box<dyn super::RouteEntry>, Vec<(String, String)>)> {
        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let mut current_index = self.root_index;
        let mut params = Vec::new();
        
        for segment in segments {
            if let Some(node) = self.nodes.get(current_index) {
                // 首先尝试精确匹配静态子节点
                let mut found = false;
                for edge in &node.children {
                    if edge.prefix == segment {
                        current_index = edge.node_index;
                        found = true;
                        break;
                    }
                }
                
                // 如果没有找到，尝试参数化子节点
                if !found {
                    if let Some(param_index) = node.param_child_index {
                        if let Some(param_node) = self.nodes.get(param_index) {
                            if let RadixNodeType::Parameter(param_name) = &param_node.node_type {
                                params.push((param_name.clone(), segment.to_string()));
                                current_index = param_index;
                                found = true;
                            }
                        }
                    }
                }
                
                // 如果还是没有找到，尝试通配符子节点
                if !found {
                    if let Some(wildcard_index) = node.wildcard_child_index {
                        current_index = wildcard_index;
                        found = true;
                    }
                }
                
                if !found {
                    return None;
                }
            } else {
                return None;
            }
        }
        
        if let Some(node) = self.nodes.get(current_index) {
            if let Some(route_index) = node.route_index {
                if let Some(route) = self.routes.get(route_index) {
                    return Some((route, params));
                }
            }
        }
        
        None
    }
    
    /// 查找或创建子节点
    fn find_or_create_child(&mut self, parent_index: usize, node_type: RadixNodeType) -> usize {
        let prefix = match &node_type {
            RadixNodeType::Static(s) => s.clone(),
            RadixNodeType::Parameter(_) => String::new(),
            RadixNodeType::Wildcard => String::new(),
        };
        
        match &node_type {
            RadixNodeType::Static(_) => {
                // 查找匹配的静态子节点
                if let Some(parent) = self.nodes.get(parent_index) {
                    for edge in &parent.children {
                        if edge.prefix == prefix {
                            return edge.node_index;
                        }
                    }
                }
                
                // 创建新的静态子节点
                let new_index = self.nodes.len();
                self.nodes.push(CompactRadixNode::new(node_type.clone()));
                
                if let Some(parent) = self.nodes.get_mut(parent_index) {
                    parent.children.push(CompactRadixEdge {
                        prefix,
                        node_index: new_index,
                    });
                }
                
                new_index
            }
            RadixNodeType::Parameter(_) => {
                // 查找或创建参数化子节点
                if let Some(parent) = self.nodes.get(parent_index) {
                    if let Some(param_index) = parent.param_child_index {
                        return param_index;
                    }
                }
                
                let new_index = self.nodes.len();
                self.nodes.push(CompactRadixNode::new(node_type));
                
                if let Some(parent) = self.nodes.get_mut(parent_index) {
                    parent.param_child_index = Some(new_index);
                }
                
                new_index
            }
            RadixNodeType::Wildcard => {
                // 查找或创建通配符子节点
                if let Some(parent) = self.nodes.get(parent_index) {
                    if let Some(wildcard_index) = parent.wildcard_child_index {
                        return wildcard_index;
                    }
                }
                
                let new_index = self.nodes.len();
                self.nodes.push(CompactRadixNode::new(node_type));
                
                if let Some(parent) = self.nodes.get_mut(parent_index) {
                    parent.wildcard_child_index = Some(new_index);
                }
                
                new_index
            }
        }
    }
    
    /// 检查是否包含路径
    pub fn contains(&self, path: &str) -> bool {
        self.find(path).is_some()
    }
    
    /// 移除路由
    pub fn remove(&mut self, path: &str) -> Option<Box<dyn super::RouteEntry>> {
        // 简化实现：只移除路由，不清理空节点
        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let mut current_index = self.root_index;
        
        for segment in segments {
            if let Some(node) = self.nodes.get(current_index) {
                let mut found = false;
                for edge in &node.children {
                    if edge.prefix == segment {
                        current_index = edge.node_index;
                        found = true;
                        break;
                    }
                }
                
                if !found {
                    if let Some(param_index) = node.param_child_index {
                        current_index = param_index;
                        found = true;
                    }
                }
                
                if !found {
                    if let Some(wildcard_index) = node.wildcard_child_index {
                        current_index = wildcard_index;
                        found = true;
                    }
                }
                
                if !found {
                    return None;
                }
            } else {
                return None;
            }
        }
        
        if let Some(node) = self.nodes.get_mut(current_index) {
            if let Some(route_index) = node.route_index.take() {
                return self.routes.get(route_index).map(|_| {
                    // 注意：这里简化处理，实际应该从routes中移除
                    // 但为了保持索引一致性，这里不实际移除
                    Box::new(super::SimpleRoute::new("removed", "text/plain")) as Box<dyn super::RouteEntry>
                });
            }
        }
        
        None
    }
    
    /// 列出所有路径
    pub fn list_paths(&self) -> Vec<String> {
        let mut paths = Vec::new();
        self.collect_paths(self.root_index, String::new(), &mut paths);
        paths
    }
    
    /// 递归收集路径
    fn collect_paths(&self, node_index: usize, prefix: String, paths: &mut Vec<String>) {
        if let Some(node) = self.nodes.get(node_index) {
            if node.route_index.is_some() {
                paths.push(prefix.clone());
            }
            
            for edge in &node.children {
                let new_prefix = if prefix.is_empty() {
                    format!("/{}", edge.prefix)
                } else {
                    format!("{}/{}", prefix, edge.prefix)
                };
                self.collect_paths(edge.node_index, new_prefix, paths);
            }
            
            if let Some(param_index) = node.param_child_index {
                let new_prefix = if prefix.is_empty() {
                    "/{param}".to_string()
                } else {
                    format!("{}/{{param}}", prefix)
                };
                self.collect_paths(param_index, new_prefix, paths);
            }
            
            if let Some(wildcard_index) = node.wildcard_child_index {
                let new_prefix = if prefix.is_empty() {
                    "/*".to_string()
                } else {
                    format!("{}/*", prefix)
                };
                self.collect_paths(wildcard_index, new_prefix, paths);
            }
        }
    }
    
    /// 清空树
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.routes.clear();
        let root_index = self.nodes.len();
        self.nodes.push(CompactRadixNode::new(RadixNodeType::Static(String::new())));
        self.root_index = root_index;
    }
}

/// Radix节点类型
#[derive(Debug, Clone)]
pub enum RadixNodeType {
    /// 静态路径段，如 "users"
    Static(String),
    /// 参数化路径段，如 "{id}"
    Parameter(String),
    /// 通配符路径段，如 "*"
    Wildcard,
}

impl RadixNodeType {
    /// 从路径段创建节点类型
    pub fn from_segment(segment: &str) -> Self {
        if segment.starts_with('{') && segment.ends_with('}') {
            let param_name = segment[1..segment.len() - 1].to_string();
            RadixNodeType::Parameter(param_name)
        } else if segment == "*" {
            RadixNodeType::Wildcard
        } else {
            RadixNodeType::Static(segment.to_string())
        }
    }
}

/// 缓存友好的路由表分片
///
/// 使用紧凑的数据结构，优化内存布局和缓存命中率
#[derive(Debug)]
pub struct CacheOptimizedShard {
    /// 紧凑的Radix Tree
    inner: CompactRadixTree,
    /// 路由数量
    count: usize,
}

impl CacheOptimizedShard {
    /// 创建新的缓存优化分片
    pub fn new() -> Self {
        Self {
            inner: CompactRadixTree::new(),
            count: 0,
        }
    }
    
    /// 插入路由
    pub fn insert(&mut self, path: &str, route: Box<dyn super::RouteEntry>) {
        let existed = self.inner.contains(path);
        self.inner.insert(path, route);
        if !existed {
            self.count += 1;
        }
    }
    
    /// 查找路由
    pub fn find(&self, path: &str) -> Option<(&Box<dyn super::RouteEntry>, Vec<(String, String)>)> {
        self.inner.find(path)
    }
    
    /// 移除路由
    pub fn remove(&mut self, path: &str) -> bool {
        if self.inner.remove(path).is_some() {
            self.count -= 1;
            true
        } else {
            false
        }
    }
    
    /// 检查是否包含路径
    pub fn contains(&self, path: &str) -> bool {
        self.inner.contains(path)
    }
    
    /// 列出所有路径
    pub fn list_paths(&self) -> Vec<String> {
        self.inner.list_paths()
    }
    
    /// 清空分片
    pub fn clear(&mut self) {
        self.inner.clear();
        self.count = 0;
    }
    
    /// 获取路由数量
    pub fn count(&self) -> usize {
        self.count
    }
}

impl Default for CacheOptimizedShard {
    fn default() -> Self {
        Self::new()
    }
}

/// 缓存优化的路由表
///
/// 使用缓存友好的数据结构实现，减少缓存未命中，提升整体性能
pub struct CacheOptimizedRouteTable {
    /// 分片数组，使用缓存优化的分片
    shards: Vec<CacheOptimizedShard>,
    /// 路由数量
    count: usize,
}

impl CacheOptimizedRouteTable {
    /// 创建新的缓存优化路由表
    pub fn new(num_shards: usize) -> Self {
        Self {
            shards: (0..num_shards).map(|_| CacheOptimizedShard::new()).collect(),
            count: 0,
        }
    }
    
    /// 根据路径计算分片索引
    fn shard_index(&self, path: &str) -> usize {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        path.hash(&mut hasher);
        (hasher.finish() as usize) % self.shards.len()
    }
    
    /// 插入路由
    pub fn insert(&mut self, path: String, route: Box<dyn super::RouteEntry>) {
        let shard_idx = self.shard_index(&path);
        let shard = &mut self.shards[shard_idx];
        let existed = shard.contains(&path);
        shard.insert(&path, route);
        if !existed {
            self.count += 1;
        }
    }
    
    /// 查找路由
    pub fn find(&self, path: &str) -> Option<(&Box<dyn super::RouteEntry>, Vec<(String, String)>)> {
        // 首先尝试在哈希分片中查找（对于静态路由，这是最优的）
        let shard_idx = self.shard_index(path);
        if let Some(result) = self.shards[shard_idx].find(path) {
            return Some(result);
        }

        // 如果在哈希分片中找不到，则需要在所有分片中搜索
        // 这对于参数化路由是必要的，因为 /user/{id} 和 /user/123 的哈希值不同
        for (i, shard) in self.shards.iter().enumerate() {
            if i != shard_idx {
                if let Some(result) = shard.find(path) {
                    return Some(result);
                }
            }
        }

        None
    }
    
    /// 移除路由
    pub fn remove(&mut self, path: &str) -> bool {
        let shard_idx = self.shard_index(path);
        let removed = self.shards[shard_idx].remove(path);
        if removed {
            self.count -= 1;
        }
        removed
    }
    
    /// 检查是否包含路径
    pub fn contains(&self, path: &str) -> bool {
        let shard_idx = self.shard_index(path);
        self.shards[shard_idx].contains(path)
    }
    
    /// 获取路由数量
    pub fn count(&self) -> usize {
        self.count
    }
    
    /// 列出所有路径
    pub fn list_paths(&self) -> Vec<String> {
        let mut paths = Vec::new();
        for shard in &self.shards {
            paths.extend(shard.list_paths());
        }
        paths
    }
    
    /// 清空路由表
    pub fn clear(&mut self) {
        for shard in &mut self.shards {
            shard.clear();
        }
        self.count = 0;
    }
}

impl Default for CacheOptimizedRouteTable {
    fn default() -> Self {
        Self::new(16)
    }
}

#[cfg(test)]
mod tests {
    use super::super::SimpleRoute;
    use super::*;

    #[test]
    fn test_compact_radix_tree_insert_and_find() {
        let mut tree = CompactRadixTree::new();
        let route = SimpleRoute::new("Hello", "text/plain");
        tree.insert("/hello", Box::new(route));
        
        let result = tree.find("/hello");
        assert!(result.is_some());
    }
    
    #[test]
    fn test_compact_radix_tree_parameter() {
        let mut tree = CompactRadixTree::new();
        let route = SimpleRoute::new("User", "text/plain");
        tree.insert("/user/{id}", Box::new(route));
        
        let result = tree.find("/user/123");
        assert!(result.is_some());
        let (_route, params) = result.unwrap();
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].0, "id");
        assert_eq!(params[0].1, "123");
    }
    
    #[test]
    fn test_cache_optimized_shard() {
        let mut shard = CacheOptimizedShard::new();
        let route = SimpleRoute::new("Test", "text/plain");
        shard.insert("/test", Box::new(route));
        
        assert!(shard.contains("/test"));
        assert_eq!(shard.count(), 1);
        
        let result = shard.find("/test");
        assert!(result.is_some());
        
        assert!(shard.remove("/test"));
        assert!(!shard.contains("/test"));
        assert_eq!(shard.count(), 0);
    }
    
    #[test]
    fn test_compact_radix_tree_list_paths() {
        let mut tree = CompactRadixTree::new();
        tree.insert("/route1", Box::new(SimpleRoute::new("1", "text/plain")));
        tree.insert("/route2", Box::new(SimpleRoute::new("2", "text/plain")));
        
        let paths = tree.list_paths();
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&"/route1".to_string()));
        assert!(paths.contains(&"/route2".to_string()));
    }
    
    #[test]
    fn test_cache_optimized_route_table() {
        let mut table = CacheOptimizedRouteTable::new(8);
        let route = SimpleRoute::new("Hello", "text/plain");
        table.insert("/hello".to_string(), Box::new(route));
        
        assert!(table.contains("/hello"));
        assert_eq!(table.count(), 1);
        
        let result = table.find("/hello");
        assert!(result.is_some());
        
        assert!(table.remove("/hello"));
        assert!(!table.contains("/hello"));
        assert_eq!(table.count(), 0);
    }
    
    #[test]
    fn test_cache_optimized_route_table_parameter() {
        let mut table = CacheOptimizedRouteTable::new(8);
        let route = SimpleRoute::new("User", "text/plain");
        table.insert("/user/{id}".to_string(), Box::new(route));
        
        let result = table.find("/user/123");
        assert!(result.is_some());
        let (_route, params) = result.unwrap();
        assert_eq!(params.len(), 1);
    }
}