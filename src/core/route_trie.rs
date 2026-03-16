//! 路由Trie树
//!
//! 使用前缀树（Trie）数据结构优化路由匹配性能，将查找复杂度从O(n)降低到O(k)，其中k为路径段数。

#![allow(clippy::type_complexity)]

use super::RouteEntry;
use std::collections::HashMap;

/// Trie节点类型
#[derive(Debug, Clone)]
enum TrieNodeType {
    /// 静态路径段，如 "users"
    Static(String),
    /// 参数化路径段，如 "{id}"
    Parameter(String),
    /// 通配符路径段，如 "*"
    Wildcard,
}

impl TrieNodeType {
    /// 从路径段创建节点类型
    fn from_segment(segment: &str) -> Self {
        if segment.starts_with('{') && segment.ends_with('}') {
            let param_name = segment[1..segment.len() - 1].to_string();
            TrieNodeType::Parameter(param_name)
        } else if segment == "*" {
            TrieNodeType::Wildcard
        } else {
            TrieNodeType::Static(segment.to_string())
        }
    }

    /// 检查是否匹配给定路径段
    #[allow(dead_code)]
    fn matches(&self, segment: &str) -> bool {
        match self {
            TrieNodeType::Static(s) => s == segment,
            TrieNodeType::Parameter(_) => true, // 参数匹配任何值
            TrieNodeType::Wildcard => true,      // 通配符匹配任何值
        }
    }
}

/// Trie节点
#[derive(Debug)]
struct TrieNode {
    /// 节点类型
    node_type: TrieNodeType,
    /// 子节点
    children: HashMap<String, TrieNode>,
    /// 参数化子节点
    param_child: Option<Box<TrieNode>>,
    /// 通配符子节点
    wildcard_child: Option<Box<TrieNode>>,
    /// 存储在此节点的路由处理器
    route: Option<Box<dyn RouteEntry>>,
}

impl TrieNode {
    /// 创建新的Trie节点
    fn new(node_type: TrieNodeType) -> Self {
        Self {
            node_type,
            children: HashMap::new(),
            param_child: None,
            wildcard_child: None,
            route: None,
        }
    }

    /// 插入路由路径
    fn insert(&mut self, path: &str, route: Box<dyn RouteEntry>) {
        use super::object_pool::split_path_optimized;
        let segments: Vec<String> = split_path_optimized(path);
        let segments_refs: Vec<&str> = segments.iter().map(|s| s.as_str()).collect();

        self.insert_segments(&segments_refs, 0, route);
    }

    /// 递归插入路径段
    fn insert_segments(&mut self, segments: &[&str], idx: usize, route: Box<dyn RouteEntry>) {
        if idx >= segments.len() {
            // 到达路径末尾，存储路由
            self.route = Some(route);
            return;
        }

        let segment = segments[idx];
        let node_type = TrieNodeType::from_segment(segment);

        match node_type {
            TrieNodeType::Static(key) => {
                // 在子节点中查找或创建
                if !self.children.contains_key(&key) {
                    self.children
                        .insert(key.clone(), TrieNode::new(TrieNodeType::Static(key.clone())));
                }
                self.children
                    .get_mut(&key)
                    .unwrap()
                    .insert_segments(segments, idx + 1, route);
            }
            TrieNodeType::Parameter(param_name) => {
                // 在参数化子节点中查找或创建
                if self.param_child.is_none() {
                    self.param_child = Some(Box::new(TrieNode::new(TrieNodeType::Parameter(
                        param_name,
                    ))));
                }
                self.param_child
                    .as_mut()
                    .unwrap()
                    .insert_segments(segments, idx + 1, route);
            }
            TrieNodeType::Wildcard => {
                // 在通配符子节点中查找或创建
                if self.wildcard_child.is_none() {
                    self.wildcard_child = Some(Box::new(TrieNode::new(TrieNodeType::Wildcard)));
                }
                self.wildcard_child
                    .as_mut()
                    .unwrap()
                    .insert_segments(segments, idx + 1, route);
            }
        }
    }

    /// 查找路由
    fn find(&self, path: &str) -> Option<(&Box<dyn RouteEntry>, Vec<(String, String)>)> {
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
    ) -> Option<(&Box<dyn RouteEntry>, Vec<(String, String)>)> {
        if idx >= segments.len() {
            // 到达路径末尾
            return self.route.as_ref().map(|route| (route, params));
        }

        let segment = segments[idx];

        // 1. 优先尝试精确匹配的静态节点
        if let Some(child) = self.children.get(segment) {
            if let Some(result) = child.find_segments(segments, idx + 1, params.clone()) {
                return Some(result);
            }
        }

        // 2. 尝试参数化匹配
        if let Some(ref param_child) = self.param_child {
            if let TrieNodeType::Parameter(param_name) = &param_child.node_type {
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
    fn remove(&mut self, path: &str) -> Option<Box<dyn RouteEntry>> {
        use super::object_pool::split_path_optimized;
        let segments: Vec<String> = split_path_optimized(path);
        let segments_refs: Vec<&str> = segments.iter().map(|s| s.as_str()).collect();

        if segments_refs.is_empty() {
            return self.route.take();
        }

        self.remove_segments(&segments_refs, 0)
    }

    /// 递归移除路径段
    fn remove_segments(&mut self, segments: &[&str], idx: usize) -> Option<Box<dyn RouteEntry>> {
        if idx >= segments.len() {
            return self.route.take();
        }

        let segment = segments[idx];

        // 尝试静态节点
        if let Some(child) = self.children.get_mut(segment) {
            if let Some(route) = child.remove_segments(segments, idx + 1) {
                // 如果子节点没有路由和子节点，可以删除
                if child.route.is_none() && child.children.is_empty() && child.param_child.is_none() && child.wildcard_child.is_none() {
                    self.children.remove(segment);
                }
                return Some(route);
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
        for child in self.children.values() {
            count += child.count_nodes();
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
        for child in self.children.values() {
            count += child.count_routes();
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

        // 递归遍历子节点
        for (key, child) in &self.children {
            let new_prefix = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{}/{}", prefix, key)
            };
            child.list_paths(&new_prefix, paths);
        }

        // 遍历参数化子节点
        if let Some(ref param_child) = self.param_child {
            if let TrieNodeType::Parameter(param_name) = &param_child.node_type {
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

/// 路由Trie树
///
/// 使用前缀树数据结构存储路由，提供O(k)的查找复杂度，其中k为路径段数。
#[derive(Debug)]
pub struct RouteTrie {
    /// 根节点
    root: TrieNode,
}

impl RouteTrie {
    /// 创建新的路由Trie树
    pub fn new() -> Self {
        Self {
            root: TrieNode::new(TrieNodeType::Static(String::new())),
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
    /// use dynamic_route_actix::core::route_trie::RouteTrie;
    /// use dynamic_route_actix::SimpleRoute;
    ///
    /// let mut trie = RouteTrie::new();
    /// trie.insert("/users", Box::new(SimpleRoute::new("users", "text/plain")));
    /// trie.insert("/users/{id}", Box::new(SimpleRoute::new("user", "text/plain")));
    ///
    /// assert!(trie.find("/users").is_some());
    /// assert!(trie.find("/users/123").is_some());
    /// ```
    pub fn insert(&mut self, path: &str, route: Box<dyn RouteEntry>) {
        self.root.insert(path, route);
    }

    /// 查找路由
    ///
    /// # 参数
    ///
    /// * `path` - 要查找的路径
    ///
    /// # 返回
    ///
    /// 返回路由处理器和提取的参数，如果未找到则返回None
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::core::route_trie::RouteTrie;
    /// use dynamic_route_actix::SimpleRoute;
    ///
    /// let mut trie = RouteTrie::new();
    /// trie.insert("/users/{id}", Box::new(SimpleRoute::new("user", "text/plain")));
    ///
    /// let (route, params) = trie.find("/users/123").unwrap();
    /// assert!(params.iter().any(|(k, v)| k == "id" && v == "123"));
    /// ```
    pub fn find(&self, path: &str) -> Option<(&Box<dyn RouteEntry>, Vec<(String, String)>)> {
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
    /// 返回被移除的路由处理器，如果未找到则返回None
    pub fn remove(&mut self, path: &str) -> Option<Box<dyn RouteEntry>> {
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
        self.root = TrieNode::new(TrieNodeType::Static(String::new()));
    }
}

impl Default for RouteTrie {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::SimpleRoute;
    use super::*;

    #[test]
    fn test_trie_insert_and_find() {
        let mut trie = RouteTrie::new();
        trie.insert("/users", Box::new(SimpleRoute::new("users", "text/plain")));

        let result = trie.find("/users");
        assert!(result.is_some());
    }

    #[test]
    fn test_parameterized_route() {
        let mut trie = RouteTrie::new();
        trie.insert("/users/{id}", Box::new(SimpleRoute::new("user", "text/plain")));

        let (route, params) = trie.find("/users/123").unwrap();
        assert!(params.iter().any(|(k, v)| k == "id" && v == "123"));

        let (route, params) = trie.find("/users/abc").unwrap();
        assert!(params.iter().any(|(k, v)| k == "id" && v == "abc"));
    }

    #[test]
    fn test_multiple_parameters() {
        let mut trie = RouteTrie::new();
        trie.insert(
            "/users/{id}/posts/{post_id}",
            Box::new(SimpleRoute::new("post", "text/plain")),
        );

        let (route, params) = trie.find("/users/123/posts/456").unwrap();
        assert!(params.iter().any(|(k, v)| k == "id" && v == "123"));
        assert!(params.iter().any(|(k, v)| k == "post_id" && v == "456"));
    }

    #[test]
    fn test_wildcard_route() {
        let mut trie = RouteTrie::new();
        trie.insert("/static/*", Box::new(SimpleRoute::new("static", "text/plain")));

        let result = trie.find("/static/css/style.css");
        assert!(result.is_some());

        let result = trie.find("/static/js/app.js");
        assert!(result.is_some());

        let result = trie.find("/static");
        assert!(result.is_none());
    }

    #[test]
    fn test_priority_matching() {
        let mut trie = RouteTrie::new();
        trie.insert("/users", Box::new(SimpleRoute::new("list", "text/plain")));
        trie.insert("/users/{id}", Box::new(SimpleRoute::new("detail", "text/plain")));

        // 精确匹配应该优先
        let (route, params) = trie.find("/users").unwrap();
        assert!(params.is_empty());

        // 参数化匹配
        let (route, params) = trie.find("/users/123").unwrap();
        assert!(params.iter().any(|(k, v)| k == "id" && v == "123"));
    }

    #[test]
    fn test_remove_route() {
        let mut trie = RouteTrie::new();
        trie.insert("/users", Box::new(SimpleRoute::new("users", "text/plain")));

        assert!(trie.contains("/users"));
        let removed = trie.remove("/users");
        assert!(removed.is_some());
        assert!(!trie.contains("/users"));
    }

    #[test]
    fn test_count_routes() {
        let mut trie = RouteTrie::new();
        assert_eq!(trie.count(), 0);

        trie.insert("/users", Box::new(SimpleRoute::new("users", "text/plain")));
        assert_eq!(trie.count(), 1);

        trie.insert("/users/{id}", Box::new(SimpleRoute::new("user", "text/plain")));
        assert_eq!(trie.count(), 2);

        trie.remove("/users");
        assert_eq!(trie.count(), 1);
    }

    #[test]
    fn test_list_paths() {
        let mut trie = RouteTrie::new();
        trie.insert("/users", Box::new(SimpleRoute::new("users", "text/plain")));
        trie.insert("/users/{id}", Box::new(SimpleRoute::new("user", "text/plain")));
        trie.insert("/posts", Box::new(SimpleRoute::new("posts", "text/plain")));

        let paths = trie.list_paths();
        assert_eq!(paths.len(), 3);
        assert!(paths.contains(&"/users".to_string()));
        assert!(paths.contains(&"/users/{id}".to_string()));
        assert!(paths.contains(&"/posts".to_string()));
    }

    #[test]
    fn test_complex_path() {
        let mut trie = RouteTrie::new();
        trie.insert(
            "/api/v1/users/{user_id}/posts/{post_id}/comments/{comment_id}",
            Box::new(SimpleRoute::new("comment", "text/plain")),
        );

        let (route, params) = trie
            .find("/api/v1/users/123/posts/456/comments/789")
            .unwrap();
        assert!(params.iter().any(|(k, v)| k == "user_id" && v == "123"));
        assert!(params.iter().any(|(k, v)| k == "post_id" && v == "456"));
        assert!(params.iter().any(|(k, v)| k == "comment_id" && v == "789"));
    }

    #[test]
    fn test_root_path() {
        let mut trie = RouteTrie::new();
        trie.insert("/", Box::new(SimpleRoute::new("root", "text/plain")));

        assert!(trie.find("/").is_some());
        // 空字符串路径分割后也是空segments，同样会匹配根路径
        assert!(trie.find("").is_some());
    }

    #[test]
    fn test_clear_routes() {
        let mut trie = RouteTrie::new();
        trie.insert("/users", Box::new(SimpleRoute::new("users", "text/plain")));
        trie.insert("/posts", Box::new(SimpleRoute::new("posts", "text/plain")));

        assert_eq!(trie.count(), 2);
        trie.clear();
        assert_eq!(trie.count(), 0);
    }

    #[test]
    fn test_trie_node_count() {
        let mut trie = RouteTrie::new();
        trie.insert("/users", Box::new(SimpleRoute::new("users", "text/plain")));
        trie.insert("/users/{id}", Box::new(SimpleRoute::new("user", "text/plain")));
        trie.insert("/posts", Box::new(SimpleRoute::new("posts", "text/plain")));

        // 根节点 + users节点 + 参数节点 + posts节点 = 4
        assert_eq!(trie.node_count(), 4);
    }
}