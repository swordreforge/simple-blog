//! 路由匹配器
//!
//! 提供高级路由匹配功能，支持路径参数、通配符和正则表达式匹配。

use std::collections::HashMap;

/// 路由匹配结果
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// 匹配的路由路径
    pub path: String,
    /// 提取的路径参数
    pub params: HashMap<String, String>,
}

/// 路由模式类型
#[derive(Debug, Clone, PartialEq)]
pub enum RoutePattern {
    /// 精确匹配
    Exact(String),
    /// 路径参数匹配，如 /user/{id}
    Parameterized {
        pattern: String,
        param_names: Vec<String>,
    },
    /// 通配符匹配，如 /static/*
    Wildcard {
        prefix: String,
        capture_name: Option<String>,
    },
}

impl RoutePattern {
    /// 从路径字符串创建路由模式
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::core::route_matcher::RoutePattern;
    ///
    /// let exact = RoutePattern::from("/users");
    /// assert!(matches!(exact, RoutePattern::Exact(_)));
    ///
    /// let param = RoutePattern::from("/users/{id}");
    /// assert!(matches!(param, RoutePattern::Parameterized { .. }));
    ///
    /// let wildcard = RoutePattern::from("/static/*");
    /// assert!(matches!(wildcard, RoutePattern::Wildcard { .. }));
    /// ```
    pub fn from(path: &str) -> Self {
        // 检查是否包含路径参数
        if path.contains('{') && path.contains('}') {
            let param_names = Self::extract_param_names(path);
            RoutePattern::Parameterized {
                pattern: path.to_string(),
                param_names,
            }
        }
        // 检查是否包含通配符
        else if path.contains('*') {
            if let Some(capture_name) = Self::extract_wildcard_name(path) {
                RoutePattern::Wildcard {
                    prefix: path.split('*').next().unwrap_or("").to_string(),
                    capture_name: Some(capture_name),
                }
            } else {
                RoutePattern::Wildcard {
                    prefix: path.split('*').next().unwrap_or("").to_string(),
                    capture_name: None,
                }
            }
        }
        // 精确匹配
        else {
            RoutePattern::Exact(path.to_string())
        }
    }

    /// 提取路径参数名称
    fn extract_param_names(path: &str) -> Vec<String> {
        let mut params = Vec::new();
        let mut chars = path.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '{' {
                let mut param = String::new();
                while let Some(&next) = chars.peek() {
                    if next == '}' {
                        chars.next();
                        break;
                    }
                    param.push(chars.next().unwrap());
                }
                if !param.is_empty() {
                    params.push(param);
                }
            }
        }

        params
    }

    /// 提取通配符捕获名称
    fn extract_wildcard_name(path: &str) -> Option<String> {
        // 检查是否是命名通配符，如 /static/{*path}
        if let Some(start) = path.find("{*") {
            if let Some(end) = path.find('}') {
                let name = path[start + 2..end].to_string();
                if !name.is_empty() {
                    return Some(name);
                }
            }
        }
        None
    }

    /// 匹配路径
    ///
    /// # 参数
    ///
    /// * `path` - 要匹配的路径
    ///
    /// # 返回
    ///
    /// 如果匹配成功，返回 `Some(MatchResult)`；否则返回 `None`
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::core::route_matcher::RoutePattern;
    ///
    /// let pattern = RoutePattern::from("/users/{id}");
    /// let result = pattern.match_path("/users/123");
    /// assert!(result.is_some());
    /// assert_eq!(result.unwrap().params.get("id"), Some(&"123".to_string()));
    /// ```
    pub fn match_path(&self, path: &str) -> Option<MatchResult> {
        match self {
            RoutePattern::Exact(pattern) => {
                if pattern == path {
                    Some(MatchResult {
                        path: path.to_string(),
                        params: HashMap::new(),
                    })
                } else {
                    None
                }
            }
            RoutePattern::Parameterized {
                pattern,
                param_names,
            } => {
                let mut params = HashMap::new();
                let pattern_parts: Vec<&str> = pattern.split('/').collect();
                let path_parts: Vec<&str> = path.split('/').collect();

                if pattern_parts.len() != path_parts.len() {
                    return None;
                }

                let mut matched_path = String::new();

                for (i, (pattern_part, path_part)) in pattern_parts.iter().zip(path_parts.iter()).enumerate() {
                    if pattern_part.starts_with('{') && pattern_part.ends_with('}') {
                        // 路径参数
                        let param_name = &pattern_part[1..pattern_part.len() - 1];
                        params.insert(param_name.to_string(), path_part.to_string());
                    } else if pattern_part != path_part {
                        // 不匹配
                        return None;
                    }

                    if i > 0 {
                        matched_path.push('/');
                    }
                    matched_path.push_str(path_part);
                }

                Some(MatchResult {
                    path: matched_path,
                    params,
                })
            }
            RoutePattern::Wildcard {
                prefix,
                capture_name,
            } => {
                if path.starts_with(prefix) {
                    let captured = &path[prefix.len()..];
                    let mut params = HashMap::new();

                    if let Some(name) = capture_name {
                        params.insert(name.clone(), captured.to_string());
                    }

                    Some(MatchResult {
                        path: path.to_string(),
                        params,
                    })
                } else {
                    None
                }
            }
        }
    }
}

/// 路由匹配器
///
/// 提供高效的路由匹配功能，支持多种匹配模式。
pub struct RouteMatcher {
    patterns: Vec<RoutePattern>,
}

impl RouteMatcher {
    /// 创建新的路由匹配器
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
        }
    }

    /// 添加路由模式
    pub fn add_pattern(&mut self, pattern: RoutePattern) {
        self.patterns.push(pattern);
    }

    /// 匹配路径
    ///
    /// # 参数
    ///
    /// * `path` - 要匹配的路径
    ///
    /// # 返回
    ///
    /// 返回所有匹配的结果
    ///
    /// # 示例
    ///
    /// ```
    /// use dynamic_route_actix::core::route_matcher::{RouteMatcher, RoutePattern};
    ///
    /// let mut matcher = RouteMatcher::new();
    /// matcher.add_pattern(RoutePattern::from("/users/{id}"));
    /// matcher.add_pattern(RoutePattern::from("/static/*"));
    ///
    /// let results = matcher.match_path("/users/123");
    /// assert!(!results.is_empty());
    /// ```
    pub fn match_path(&self, path: &str) -> Vec<MatchResult> {
        self.patterns
            .iter()
            .filter_map(|pattern| pattern.match_path(path))
            .collect()
    }

    /// 查找最佳匹配
    ///
    /// 优先级：精确匹配 > 参数化匹配 > 通配符匹配
    pub fn find_best_match(&self, path: &str) -> Option<MatchResult> {
        let mut best_match: Option<(MatchResult, i32)> = None;

        for pattern in &self.patterns {
            if let Some(result) = pattern.match_path(path) {
                let priority = match pattern {
                    RoutePattern::Exact(_) => 3,
                    RoutePattern::Parameterized { .. } => 2,
                    RoutePattern::Wildcard { .. } => 1,
                };

                match best_match {
                    Some((_, existing_priority)) if priority <= existing_priority => {
                        // 保留更高优先级的匹配
                    }
                    _ => {
                        best_match = Some((result, priority));
                    }
                }
            }
        }

        best_match.map(|(result, _)| result)
    }
}

impl Default for RouteMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_pattern() {
        let pattern = RoutePattern::from("/users");

        assert!(pattern.match_path("/users").is_some());
        assert!(pattern.match_path("/users/123").is_none());
        assert!(pattern.match_path("/posts").is_none());
    }

    #[test]
    fn test_parameterized_pattern() {
        let pattern = RoutePattern::from("/users/{id}");

        let result = pattern.match_path("/users/123");
        assert!(result.is_some());
        assert_eq!(result.unwrap().params.get("id"), Some(&"123".to_string()));

        let result = pattern.match_path("/users/abc");
        assert!(result.is_some());
        assert_eq!(result.unwrap().params.get("id"), Some(&"abc".to_string()));

        assert!(pattern.match_path("/users").is_none());
        assert!(pattern.match_path("/users/123/posts").is_none());
    }

    #[test]
    fn test_multiple_parameters() {
        let pattern = RoutePattern::from("/users/{id}/posts/{post_id}");

        let result = pattern.match_path("/users/123/posts/456");
        assert!(result.is_some());
        let params = result.unwrap().params;
        assert_eq!(params.get("id"), Some(&"123".to_string()));
        assert_eq!(params.get("post_id"), Some(&"456".to_string()));
    }

    #[test]
    fn test_wildcard_pattern() {
        let pattern = RoutePattern::from("/static/*");

        let result = pattern.match_path("/static/css/style.css");
        assert!(result.is_some());
        assert_eq!(result.unwrap().params.len(), 0);

        let result = pattern.match_path("/static/js/app.js");
        assert!(result.is_some());

        assert!(pattern.match_path("/static").is_none());
        assert!(pattern.match_path("/api/users").is_none());
    }

    #[test]
    fn test_route_matcher() {
        let mut matcher = RouteMatcher::new();
        matcher.add_pattern(RoutePattern::from("/users"));
        matcher.add_pattern(RoutePattern::from("/users/{id}"));
        matcher.add_pattern(RoutePattern::from("/static/*"));

        let results = matcher.match_path("/users/123");
        assert_eq!(results.len(), 1);

        let results = matcher.match_path("/static/css/style.css");
        assert_eq!(results.len(), 1);

        let results = matcher.match_path("/nonexistent");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_best_match_priority() {
        let mut matcher = RouteMatcher::new();
        matcher.add_pattern(RoutePattern::from("/static/*"));
        matcher.add_pattern(RoutePattern::from("/users/{id}"));
        matcher.add_pattern(RoutePattern::from("/users"));

        // 精确匹配应该优先
        let result = matcher.find_best_match("/users");
        assert!(result.is_some());
        assert!(result.unwrap().params.is_empty());

        // 参数化匹配应该优先于通配符
        let result = matcher.find_best_match("/users/123");
        assert!(result.is_some());
        assert_eq!(result.unwrap().params.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_complex_path() {
        let pattern = RoutePattern::from("/api/v1/users/{user_id}/posts/{post_id}/comments/{comment_id}");

        let result = pattern.match_path("/api/v1/users/123/posts/456/comments/789");
        assert!(result.is_some());
        let params = result.unwrap().params;
        assert_eq!(params.get("user_id"), Some(&"123".to_string()));
        assert_eq!(params.get("post_id"), Some(&"456".to_string()));
        assert_eq!(params.get("comment_id"), Some(&"789".to_string()));
    }

    #[test]
    fn test_empty_path() {
        let pattern = RoutePattern::from("/");
        assert!(pattern.match_path("/").is_some());
        assert!(pattern.match_path("").is_none());
    }

    #[test]
    fn test_special_characters() {
        let pattern = RoutePattern::from("/path/with-dash/{id}");

        let result = pattern.match_path("/path/with-dash/123");
        assert!(result.is_some());
        assert_eq!(result.unwrap().params.get("id"), Some(&"123".to_string()));
    }
}