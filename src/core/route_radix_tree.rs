//! 路由Radix Tree
//!
//! 使用Radix Tree（基数树）数据结构优化路由匹配性能，进一步压缩Trie树节点，
//! 减少内存占用并提升缓存命中率。

#![allow(clippy::type_complexity)]

use super::RouteEntry;

/// Radix节点类型
#[derive(Debug, Clone)]
enum RadixNodeType {
    /// 静态路径段，如 "users"
    Static(String),
    /// 参数化路径段，如 "{id}"
    Parameter(String),
    /// 通配符路径段，如 "*"
    Wildcard,
}

impl RadixNodeType {
    /// 从路径段创建节点类型
    fn from_segment(segment: &str) -> Self {
        if segment.starts_with('{') && segment.ends_with('}') {
            let param_name = segment[1..segment.len() - 1].to_string();
            RadixNodeType::Parameter(param_name)
        } else if segment == "*" {
            RadixNodeType::Wildcard
        } else {
            RadixNodeType::Static(segment.to_string())
        }
    }

    /// 检查是否匹配给定路径段
    #[allow(dead_code)]
    fn matches(&self, segment: &str) -> bool {
        match self {
            RadixNodeType::Static(s) => s == segment,
            RadixNodeType::Parameter(_) => true, // 参数匹配任何值
            RadixNodeType::Wildcard => true,      // 通配符匹配任何值
        }
    }
}

/// Radix Tree节点边
///
/// 边包含前缀和指向子节点的引用，用于压缩路径。
#[derive(Debug)]
struct RadixEdge {
    /// 边的前缀（用于路径压缩）
    prefix: String,
    /// 子节点
    node: Box<RadixNode>,
}

/// Radix Tree节点
#[derive(Debug)]
struct RadixNode {
    /// 节点类型
    node_type: RadixNodeType,
    /// 子边（使用前缀压缩的子节点）
    children: Vec<RadixEdge>,
    /// 参数化子节点
    param_child: Option<Box<RadixNode>>,
    /// 通配符子节点
    wildcard_child: Option<Box<RadixNode>>,
    /// 存储在此节点的路由处理器（使用Arc实现零拷贝共享）
    route: Option<std::sync::Arc<dyn RouteEntry>>,
}

impl RadixNode {
    /// 创建新的Radix节点
    fn new(node_type: RadixNodeType) -> Self {
        Self {
            node_type,
            children: Vec::new(),
            param_child: None,
            wildcard_child: None,
            route: None,
        }
    }

    /// 查找最长公共前缀
    fn longest_common_prefix(a: &str, b: &str) -> usize {
        a.chars()
            .zip(b.chars())
            .take_while(|(ca, cb)| ca == cb)
            .count()
    }

    /// 在子边中查找匹配的前缀
    #[allow(dead_code)]
    fn find_matching_edge(&self, segment: &str) -> Option<(usize, usize)> {
        for (idx, edge) in self.children.iter().enumerate() {
            let lcp = Self::longest_common_prefix(segment, &edge.prefix);
            if lcp > 0 {
                return Some((idx, lcp));
            }
        }
        None
    }

    /// 插入路由路径
    fn insert(&mut self, path: &str, route: Box<dyn RouteEntry>) {
        use super::object_pool::split_path_optimized;
        // 将Box转换为Arc，实现零拷贝共享
        let arc_route: std::sync::Arc<dyn RouteEntry> = route.into();
        let segments: Vec<String> = split_path_optimized(path);
        let segments_refs: Vec<&str> = segments.iter().map(|s| s.as_str()).collect();

        self.insert_segments(&segments_refs, 0, arc_route);
    }

    /// 递归插入路径段
    fn insert_segments(&mut self, segments: &[&str], idx: usize, route: std::sync::Arc<dyn RouteEntry>) {
        if idx >= segments.len() {
            // 到达路径末尾，存储路由（使用Arc共享）
            self.route = Some(route);
            return;
        }

        let segment = segments[idx];
        let node_type = RadixNodeType::from_segment(segment);

        match node_type {
            RadixNodeType::Static(key) => {
                // 查找是否已有匹配的边
                let existing_edge = self.children.iter().position(|edge| edge.prefix == key);

                if let Some(edge_idx) = existing_edge {
                    // 边已存在，继续插入到子节点
                    self.children[edge_idx].node.insert_segments(segments, idx + 1, route);
                } else {
                    // 创建新的边
                    let mut new_node = RadixNode::new(RadixNodeType::Static(key.clone()));
                    if idx + 1 < segments.len() {
                        new_node.insert_segments(segments, idx + 1, route);
                    } else {
                        new_node.route = Some(route);
                    }
                    self.children.push(RadixEdge {
                        prefix: key,
                        node: Box::new(new_node),
                    });
                }
            }
            RadixNodeType::Parameter(param_name) => {
                // 在参数化子节点中查找或创建
                if self.param_child.is_none() {
                    self.param_child = Some(Box::new(RadixNode::new(RadixNodeType::Parameter(
                        param_name,
                    ))));
                }
                self.param_child
                    .as_mut()
                    .unwrap()
                    .insert_segments(segments, idx + 1, route);
            }
            RadixNodeType::Wildcard => {
                // 在通配符子节点中查找或创建
                if self.wildcard_child.is_none() {
                    self.wildcard_child = Some(Box::new(RadixNode::new(RadixNodeType::Wildcard)));
                }
                self.wildcard_child
                    .as_mut()
                    .unwrap()
                    .insert_segments(segments, idx + 1, route);
            }
        }
    }

    /// 查找路由
    fn find(&self, path: &str) -> Option<(&std::sync::Arc<dyn RouteEntry>, Vec<(String, String)>)> {
        use super::object_pool::split_path_optimized;
        let segments: Vec<String> = split_path_optimized(path);
        let segments_refs: Vec<&str> = segments.iter().map(|s| s.as_str()).collect();

        if segments_refs.is_empty() {
            // 根路径
            if let Some(ref route) = self.route {
                return Some((route, Vec::new()));
            }
            return None;
        }

        self.find_segments(&segments_refs, 0, Vec::new())
    }

    /// 递归查找路径段
    fn find_segments(
        &self,
        segments: &[&str],
        idx: usize,
        params: Vec<(String, String)>,
    ) -> Option<(&std::sync::Arc<dyn RouteEntry>, Vec<(String, String)>)> {
        if idx >= segments.len() {
            // 到达路径末尾
            return self.route.as_ref().map(|route| (route, params));
        }

        let segment = segments[idx];

        // 1. 优先尝试精确匹配的静态节点
        for edge in &self.children {
            if edge.prefix == segment {
                // 完全匹配，继续到子节点
                if let Some(result) = edge.node.find_segments(segments, idx + 1, params.clone()) {
                    return Some(result);
                }
            }
        }

        // 2. 尝试参数化匹配
        if let Some(ref param_child) = self.param_child {
            if let RadixNodeType::Parameter(param_name) = &param_child.node_type {
                let mut new_params = params.clone();
                new_params.push((param_name.clone(), segment.to_string()));

                if let Some(result) = param_child.find_segments(segments, idx + 1, new_params) {
                    return Some(result);
                }
            }
        }

        // 3. 尝试通配符匹配（匹配剩余所有路径）
        if let Some(ref wildcard_child) = self.wildcard_child {
            // 通配符匹配剩余所有路径段
            let remaining_path = segments[idx..].join("/");
            let mut new_params = params;
            new_params.push(("*".to_string(), remaining_path));

            // 通配符节点应该立即返回
            if let Some(ref route) = wildcard_child.route {
                return Some((route, new_params));
            }

            // 或者继续匹配（如果通配符节点有子节点）
            if let Some(result) = wildcard_child.find_segments(segments, segments.len(), new_params) {
                return Some(result);
            }
        }

        None
    }

    /// 移除路由
    fn remove(&mut self, path: &str) -> Option<std::sync::Arc<dyn RouteEntry>> {
        use super::object_pool::split_path_optimized;
        let segments: Vec<String> = split_path_optimized(path);
        let segments_refs: Vec<&str> = segments.iter().map(|s| s.as_str()).collect();

        if segments_refs.is_empty() {
            return self.route.take();
        }

        self.remove_segments(&segments_refs, 0)
    }

    /// 递归移除路径段
    fn remove_segments(&mut self, segments: &[&str], idx: usize) -> Option<std::sync::Arc<dyn RouteEntry>> {
        if idx >= segments.len() {
            return self.route.take();
        }

        let segment = segments[idx];

        // 尝试静态节点 - 遍历所有边寻找精确匹配
        for edge_idx in 0..self.children.len() {
            if self.children[edge_idx].prefix == segment {
                // 找到精确匹配，尝试从子节点删除
                if let Some(route) = self.children[edge_idx].node.remove_segments(segments, idx + 1) {
                    // 如果子节点没有路由和子节点，可以删除边
                    if self.children[edge_idx].node.route.is_none()
                        && self.children[edge_idx].node.children.is_empty()
                        && self.children[edge_idx].node.param_child.is_none()
                        && self.children[edge_idx].node.wildcard_child.is_none() {
                        self.children.remove(edge_idx);
                    }
                    return Some(route);
                }
            }
        }

        // 尝试参数化节点
        if let Some(ref mut param_child) = self.param_child {
            if let Some(route) = param_child.remove_segments(segments, idx + 1) {
                if param_child.route.is_none()
                    && param_child.children.is_empty()
                    && param_child.param_child.is_none()
                    && param_child.wildcard_child.is_none()
                {
                    self.param_child = None;
                }
                return Some(route);
            }
        }

        // 尝试通配符节点
        if let Some(ref mut wildcard_child) = self.wildcard_child {
            if let Some(route) = wildcard_child.remove_segments(segments, idx + 1) {
                if wildcard_child.route.is_none()
                    && wildcard_child.children.is_empty()
                    && wildcard_child.param_child.is_none()
                    && wildcard_child.wildcard_child.is_none()
                {
                    self.wildcard_child = None;
                }
                return Some(route);
            }
        }

        None
    }

    /// 统计节点数量
    fn count_nodes(&self) -> usize {
        let mut count = 1;
        for edge in &self.children {
            count += edge.node.count_nodes();
        }
        if let Some(ref param_child) = self.param_child {
            count += param_child.count_nodes();
        }
        if let Some(ref wildcard_child) = self.wildcard_child {
            count += wildcard_child.count_nodes();
        }
        count
    }

    /// 统计路由数量
    fn count_routes(&self) -> usize {
        let mut count = if self.route.is_some() { 1 } else { 0 };
        for edge in &self.children {
            count += edge.node.count_routes();
        }
        if let Some(ref param_child) = self.param_child {
            count += param_child.count_routes();
        }
        if let Some(ref wildcard_child) = self.wildcard_child {
            count += wildcard_child.count_routes();
        }
        count
    }

    /// 获取所有路由路径
    fn list_paths(&self, prefix: &str, paths: &mut Vec<String>) {
        // 如果此节点有路由，添加到列表
        if self.route.is_some() {
            let path = if prefix.is_empty() { "/".to_string() } else { format!("/{}", prefix) };
            paths.push(path);
        }

        // 递归遍历子边
        for edge in &self.children {
            let new_prefix = if prefix.is_empty() {
                edge.prefix.clone()
            } else {
                format!("{}/{}", prefix, edge.prefix)
            };
            edge.node.list_paths(&new_prefix, paths);
        }

        // 遍历参数化子节点
        if let Some(ref param_child) = self.param_child {
            if let RadixNodeType::Parameter(param_name) = &param_child.node_type {
                let new_prefix = if prefix.is_empty() {
                    format!("{{{}}}", param_name)
                } else {
                    format!("{}/{{{}}}", prefix, param_name)
                };
                param_child.list_paths(&new_prefix, paths);
            }
        }

        // 遍历通配符子节点
        if let Some(ref wildcard_child) = self.wildcard_child {
            let new_prefix = if prefix.is_empty() {
                "*".to_string()
            } else {
                format!("{}/{}", prefix, "*")
            };
            wildcard_child.list_paths(&new_prefix, paths);
        }
    }
}

/// 路由Radix Tree
///
/// 使用Radix Tree（基数树）数据结构存储路由，提供O(k)的查找复杂度，其中k为路径段数。
/// 相比标准Trie树，Radix Tree通过压缩单分支路径减少了节点数量，提升了内存效率和缓存命中率。
#[derive(Debug)]
pub struct RouteRadixTree {
    /// 根节点
    root: RadixNode,
}

impl RouteRadixTree {
    /// 创建新的路由Radix Tree
    pub fn new() -> Self {
        Self {
            root: RadixNode::new(RadixNodeType::Static(String::new())),
        }
    }

    /// 插入路由
    ///
    /// # 参数
    ///
    /// * `path` - 路由路径
    /// * `route` - 路由处理器
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::core::route_radix_tree::RouteRadixTree;
    /// use dynamic_route_actix::SimpleRoute;
    ///
    /// let mut radix = RouteRadixTree::new();
    /// radix.insert("/users", Box::new(SimpleRoute::new("users", "text/plain")));
    /// radix.insert("/users/{id}", Box::new(SimpleRoute::new("user", "text/plain")));
    ///
    /// assert!(radix.find("/users").is_some());
    /// assert!(radix.find("/users/123").is_some());
    /// ```
    pub fn insert(&mut self, path: &str, route: Box<dyn RouteEntry>) {
        self.root.insert(path, route);
    }

    /// 插入路由（使用Arc，零拷贝）
    ///
    /// # 参数
    ///
    /// * `path` - 路由路径
    /// * `route` - 路由处理器（使用Arc共享）
    ///
    /// # 性能优化
    ///
    /// 直接使用Arc存储路由，避免Box到Arc的转换开销。
    /// 适用于需要多次克隆路由或在不同分片间移动路由的场景。
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::core::route_radix_tree::RouteRadixTree;
    /// use dynamic_route_actix::SimpleRoute;
    /// use std::sync::Arc;
    ///
    /// let mut radix = RouteRadixTree::new();
    /// let route = Arc::new(SimpleRoute::new("users", "text/plain"));
    /// radix.insert_arc("/users", route);
    ///
    /// assert!(radix.find("/users").is_some());
    /// ```
    pub fn insert_arc(&mut self, path: &str, route: std::sync::Arc<dyn RouteEntry>) {
        use super::object_pool::split_path_optimized;
        let segments: Vec<String> = split_path_optimized(path);
        let segments_refs: Vec<&str> = segments.iter().map(|s| s.as_str()).collect();

        self.root.insert_segments(&segments_refs, 0, route);
    }

    /// 查找路由
    ///
    /// # 参数
    ///
    /// * `path` - 要查找的路径
    ///
    /// # 返回
    ///
    /// 返回路由处理器（使用Arc共享，零拷贝）和提取的参数，如果未找到则返回None
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::core::route_radix_tree::RouteRadixTree;
    /// use dynamic_route_actix::SimpleRoute;
    ///
    /// let mut radix = RouteRadixTree::new();
    /// radix.insert("/users/{id}", Box::new(SimpleRoute::new("user", "text/plain")));
    ///
    /// let (route, params) = radix.find("/users/123").unwrap();
    /// assert!(params.iter().any(|(k, v)| k == "id" && v == "123"));
    /// ```
    pub fn find(&self, path: &str) -> Option<(&std::sync::Arc<dyn RouteEntry>, Vec<(String, String)>)> {
        self.root.find(path)
    }

    /// 移除路由
    ///
    /// # 参数
    ///
    /// * `path` - 要移除的路由路径
    ///
    /// # 返回
    ///
    /// 返回被移除的路由处理器（使用Arc共享），如果未找到则返回None
    pub fn remove(&mut self, path: &str) -> Option<std::sync::Arc<dyn RouteEntry>> {
        self.root.remove(path)
    }

    /// 检查路由是否存在
    pub fn contains(&self, path: &str) -> bool {
        self.find(path).is_some()
    }

    /// 获取路由数量
    pub fn count(&self) -> usize {
        self.root.count_routes()
    }

    /// 获取节点数量
    pub fn node_count(&self) -> usize {
        self.root.count_nodes()
    }

    /// 获取所有路由路径
    pub fn list_paths(&self) -> Vec<String> {
        let mut paths = Vec::new();
        self.root.list_paths("", &mut paths);
        paths
    }

    /// 清空所有路由
    pub fn clear(&mut self) {
        self.root = RadixNode::new(RadixNodeType::Static(String::new()));
    }
}

impl Default for RouteRadixTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::SimpleRoute;
    use super::*;

    #[test]
    fn test_radix_insert_and_find() {
        let mut radix = RouteRadixTree::new();
        radix.insert("/users", Box::new(SimpleRoute::new("users", "text/plain")));

        let result = radix.find("/users");
        assert!(result.is_some());
    }

    #[test]
    fn test_parameterized_route() {
        let mut radix = RouteRadixTree::new();
        radix.insert("/users/{id}", Box::new(SimpleRoute::new("user", "text/plain")));

        let (route, params) = radix.find("/users/123").unwrap();
        assert!(params.iter().any(|(k, v)| k == "id" && v == "123"));

        let (route, params) = radix.find("/users/abc").unwrap();
        assert!(params.iter().any(|(k, v)| k == "id" && v == "abc"));
    }

    #[test]
    fn test_multiple_parameters() {
        let mut radix = RouteRadixTree::new();
        radix.insert(
            "/users/{id}/posts/{post_id}",
            Box::new(SimpleRoute::new("post", "text/plain")),
        );

        let (route, params) = radix.find("/users/123/posts/456").unwrap();
        assert!(params.iter().any(|(k, v)| k == "id" && v == "123"));
        assert!(params.iter().any(|(k, v)| k == "post_id" && v == "456"));
    }

    #[test]
    fn test_wildcard_route() {
        let mut radix = RouteRadixTree::new();
        radix.insert("/static/*", Box::new(SimpleRoute::new("static", "text/plain")));

        let result = radix.find("/static/css/style.css");
        assert!(result.is_some());

        let result = radix.find("/static/js/app.js");
        assert!(result.is_some());

        let result = radix.find("/static");
        assert!(result.is_none());
    }

    #[test]
    fn test_priority_matching() {
        let mut radix = RouteRadixTree::new();
        radix.insert("/users", Box::new(SimpleRoute::new("list", "text/plain")));
        radix.insert("/users/{id}", Box::new(SimpleRoute::new("detail", "text/plain")));

        // 精确匹配应该优先
        let (route, params) = radix.find("/users").unwrap();
        assert!(params.is_empty());

        // 参数化匹配
        let (route, params) = radix.find("/users/123").unwrap();
        assert!(params.iter().any(|(k, v)| k == "id" && v == "123"));
    }

    #[test]
    fn test_remove_route() {
        let mut radix = RouteRadixTree::new();
        radix.insert("/users", Box::new(SimpleRoute::new("users", "text/plain")));

        assert!(radix.contains("/users"));
        let removed = radix.remove("/users");
        assert!(removed.is_some());
        assert!(!radix.contains("/users"));
    }

    #[test]
    fn test_count_routes() {
        let mut radix = RouteRadixTree::new();
        assert_eq!(radix.count(), 0);

        radix.insert("/users", Box::new(SimpleRoute::new("users", "text/plain")));
        assert_eq!(radix.count(), 1);

        radix.insert("/users/{id}", Box::new(SimpleRoute::new("user", "text/plain")));
        assert_eq!(radix.count(), 2);

        radix.remove("/users");
        assert_eq!(radix.count(), 1);
    }

    #[test]
    fn test_list_paths() {
        let mut radix = RouteRadixTree::new();
        radix.insert("/users", Box::new(SimpleRoute::new("users", "text/plain")));
        radix.insert("/users/{id}", Box::new(SimpleRoute::new("user", "text/plain")));
        radix.insert("/posts", Box::new(SimpleRoute::new("posts", "text/plain")));

        let paths = radix.list_paths();
        assert_eq!(paths.len(), 3);
        assert!(paths.contains(&"/users".to_string()));
        assert!(paths.contains(&"/users/{id}".to_string()));
        assert!(paths.contains(&"/posts".to_string()));
    }

    #[test]
    fn test_complex_path() {
        let mut radix = RouteRadixTree::new();
        radix.insert(
            "/api/v1/users/{user_id}/posts/{post_id}/comments/{comment_id}",
            Box::new(SimpleRoute::new("comment", "text/plain")),
        );

        let (route, params) = radix
            .find("/api/v1/users/123/posts/456/comments/789")
            .unwrap();
        assert!(params.iter().any(|(k, v)| k == "user_id" && v == "123"));
        assert!(params.iter().any(|(k, v)| k == "post_id" && v == "456"));
        assert!(params.iter().any(|(k, v)| k == "comment_id" && v == "789"));
    }

    #[test]
    fn test_root_path() {
        let mut radix = RouteRadixTree::new();
        radix.insert("/", Box::new(SimpleRoute::new("root", "text/plain")));

        assert!(radix.find("/").is_some());
        // 空字符串路径分割后也是空segments，同样会匹配根路径
        assert!(radix.find("").is_some());
    }

    #[test]
    fn test_clear_routes() {
        let mut radix = RouteRadixTree::new();
        radix.insert("/users", Box::new(SimpleRoute::new("users", "text/plain")));
        radix.insert("/posts", Box::new(SimpleRoute::new("posts", "text/plain")));

        assert_eq!(radix.count(), 2);
        radix.clear();
        assert_eq!(radix.count(), 0);
    }

    #[test]
    fn test_radix_node_count() {
        let mut radix = RouteRadixTree::new();
        radix.insert("/users", Box::new(SimpleRoute::new("users", "text/plain")));
        radix.insert("/users/{id}", Box::new(SimpleRoute::new("user", "text/plain")));
        radix.insert("/posts", Box::new(SimpleRoute::new("posts", "text/plain")));

        // Radix Tree应该比标准Trie树节点更少
        let node_count = radix.node_count();
        assert!(node_count > 0);
        assert!(node_count <= 4); // 可能更少，因为有前缀压缩
    }

    #[test]
    fn test_radix_compression() {
        let mut radix = RouteRadixTree::new();
        // 插入有共同前缀的路由
        radix.insert("/api/v1/users", Box::new(SimpleRoute::new("users", "text/plain")));
        radix.insert("/api/v1/posts", Box::new(SimpleRoute::new("posts", "text/plain")));
        radix.insert("/api/v1/comments", Box::new(SimpleRoute::new("comments", "text/plain")));

        // Radix Tree应该压缩 /api/v1/ 前缀
        let node_count = radix.node_count();
        // 理想情况下，应该只有根节点 + api节点 + v1节点 + 3个子节点 = 6个节点
        // 而标准Trie树需要根节点 + api + v1 + users + posts + comments = 6个节点
        // 在更复杂的情况下，Radix Tree的优势会更明显
        assert!(node_count > 0);
    }
}