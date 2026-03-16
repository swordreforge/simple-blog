//! Cow优化模块
//!
//! 使用 `Cow<str>` (Copy-on-Write) 进一步优化字符串处理
//! 延迟字符串分配，只在必要时进行堆分配

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

/// 优化的字符串类型
///
/// 使用 Cow<str> 实现：
/// - 对于静态字符串，使用零拷贝的借用
/// - 对于需要修改的字符串，才进行堆分配
pub type OptimizedStr<'a> = Cow<'a, str>;

/// 优化的路径匹配器
///
/// 使用 Cow<str> 避免不必要的字符串分配
#[derive(Debug, Clone)]
pub struct OptimizedMatchResult<'a> {
    /// 匹配的路径（使用 Cow 延迟分配）
    pub path: OptimizedStr<'a>,
    /// 提取的路径参数（使用 Arc<str> 减少分配）
    pub params: HashMap<Arc<str>, Arc<str>>,
}

/// 路由模式（Cow 优化版本）
#[derive(Debug, Clone)]
pub enum CowRoutePattern<'a> {
    /// 精确匹配
    Exact(OptimizedStr<'a>),
    /// 路径参数匹配
    Parameterized {
        pattern: OptimizedStr<'a>,
        param_names: Vec<Arc<str>>,
    },
    /// 通配符匹配
    Wildcard {
        prefix: OptimizedStr<'a>,
        capture_name: Option<Arc<str>>,
    },
}

impl<'a> CowRoutePattern<'a> {
    /// 从路径字符串创建路由模式
    ///
    /// 根据字符串的生命周期选择最优的存储方式
    pub fn from_str(path: &'a str) -> Self {
        if path.contains('{') && path.contains('}') {
            let param_names = Self::extract_param_names(path);
            CowRoutePattern::Parameterized {
                pattern: Cow::Borrowed(path),
                param_names,
            }
        } else if path.contains('*') {
            if let Some(capture_name) = Self::extract_wildcard_name(path) {
                CowRoutePattern::Wildcard {
                    prefix: Cow::Borrowed(path.split('*').next().unwrap_or("")),
                    capture_name: Some(Arc::from(capture_name)),
                }
            } else {
                CowRoutePattern::Wildcard {
                    prefix: Cow::Borrowed(path.split('*').next().unwrap_or("")),
                    capture_name: None,
                }
            }
        } else {
            CowRoutePattern::Exact(Cow::Borrowed(path))
        }
    }

    /// 从拥有的字符串创建路由模式
    ///
    /// 注意：这将创建一个拥有的模式，生命周期为 'static
    pub fn from_owned(path: String) -> Self {
        // 将拥有的字符串转换为模式
        if path.contains('{') && path.contains('}') {
            let param_names = Self::extract_param_names(&path);
            CowRoutePattern::Parameterized {
                pattern: Cow::Owned(path.clone()),
                param_names,
            }
        } else if path.contains('*') {
            if let Some(capture_name) = Self::extract_wildcard_name(&path) {
                CowRoutePattern::Wildcard {
                    prefix: Cow::Owned(path.split('*').next().unwrap_or("").to_string()),
                    capture_name: Some(Arc::from(capture_name)),
                }
            } else {
                CowRoutePattern::Wildcard {
                    prefix: Cow::Owned(path.split('*').next().unwrap_or("").to_string()),
                    capture_name: None,
                }
            }
        } else {
            CowRoutePattern::Exact(Cow::Owned(path))
        }
    }

    /// 提取路径参数名称（使用 Arc<str>）
    fn extract_param_names(path: &str) -> Vec<Arc<str>> {
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
                    params.push(Arc::from(param.as_str()));
                }
            }
        }

        params
    }

    /// 提取通配符捕获名称
    fn extract_wildcard_name(path: &str) -> Option<String> {
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

    /// 匹配路径（优化版本）
    ///
    /// 使用 Cow<str> 和 Arc<str> 减少内存分配
    pub fn match_path(&self, path: &'a str) -> Option<OptimizedMatchResult<'a>> {
        match self {
            CowRoutePattern::Exact(pattern) => {
                if pattern.as_ref() == path {
                    Some(OptimizedMatchResult {
                        path: Cow::Borrowed(path),
                        params: HashMap::new(),
                    })
                } else {
                    None
                }
            }
            CowRoutePattern::Parameterized {
                pattern,
                param_names: _,
            } => {
                let mut params = HashMap::new();
                let mut pattern_idx = 0;
                let mut path_idx = 0;
                let pattern_bytes = pattern.as_bytes();
                let path_bytes = path.as_bytes();

                while pattern_idx < pattern_bytes.len() && path_idx < path_bytes.len() {
                    if pattern_bytes[pattern_idx] == b'{' {
                        pattern_idx += 1;
                        let param_start = pattern_idx;

                        while pattern_idx < pattern_bytes.len() && pattern_bytes[pattern_idx] != b'}' {
                            pattern_idx += 1;
                        }

                        if pattern_idx >= pattern_bytes.len() {
                            return None;
                        }

                        let param_name = &pattern[param_start..pattern_idx];
                        pattern_idx += 1;

                        let value_start = path_idx;
                        while path_idx < path_bytes.len() && path_bytes[path_idx] != b'/' {
                            path_idx += 1;
                        }

                        let param_value = &path[value_start..path_idx];

                        // 使用 Arc<str> 存储参数值
                        params.insert(
                            Arc::from(param_name),
                            Arc::from(param_value),
                        );

                        if pattern_idx < pattern_bytes.len() && pattern_bytes[pattern_idx] == b'/' {
                            pattern_idx += 1;
                        }

                        if path_idx < path_bytes.len() && path_bytes[path_idx] == b'/' {
                            path_idx += 1;
                        }
                    } else if pattern_bytes[pattern_idx] != path_bytes[path_idx] {
                        return None;
                    } else {
                        pattern_idx += 1;
                        path_idx += 1;
                    }
                }

                if pattern_idx < pattern_bytes.len() || path_idx < path_bytes.len() {
                    return None;
                }

                Some(OptimizedMatchResult {
                    path: Cow::Borrowed(path),
                    params,
                })
            }
            CowRoutePattern::Wildcard {
                prefix,
                capture_name,
            } => {
                if path.starts_with(prefix.as_ref()) {
                    let captured = &path[prefix.len()..];
                    let mut params = HashMap::new();

                    if let Some(name) = capture_name {
                        params.insert(Arc::clone(name), Arc::from(captured));
                    }

                    Some(OptimizedMatchResult {
                        path: Cow::Borrowed(path),
                        params,
                    })
                } else {
                    None
                }
            }
        }
    }

    /// 获取模式字符串
    pub fn as_str(&self) -> &str {
        match self {
            CowRoutePattern::Exact(s) => s.as_ref(),
            CowRoutePattern::Parameterized { pattern, .. } => pattern.as_ref(),
            CowRoutePattern::Wildcard { prefix, .. } => prefix.as_ref(),
        }
    }
}

/// 字符串片段构建器
///
/// 用于构建复杂的字符串，延迟最终分配
#[derive(Debug)]
pub struct StringFragmentBuilder<'a> {
    fragments: Vec<StringFragment<'a>>,
}

/// 字符串片段
#[derive(Debug, Clone)]
pub enum StringFragment<'a> {
    /// 借用的静态字符串
    Borrowed(&'a str),
    /// 拥有的字符串
    Owned(String),
    /// 数字
    Number(usize),
    /// Arc 共享字符串
    Arc(Arc<str>),
}

impl<'a> StringFragmentBuilder<'a> {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            fragments: Vec::new(),
        }
    }

    /// 添加借用字符串
    pub fn push_borrowed(&mut self, s: &'a str) {
        self.fragments.push(StringFragment::Borrowed(s));
    }

    /// 添加拥有的字符串
    pub fn push_owned(&mut self, s: String) {
        self.fragments.push(StringFragment::Owned(s));
    }

    /// 添加数字
    pub fn push_number(&mut self, n: usize) {
        self.fragments.push(StringFragment::Number(n));
    }

    /// 添加 Arc 字符串
    pub fn push_arc(&mut self, s: Arc<str>) {
        self.fragments.push(StringFragment::Arc(s));
    }

    /// 计算最终字符串的容量
    pub fn capacity(&self) -> usize {
        self.fragments.iter().map(|f| f.len()).sum()
    }

    /// 构建最终字符串
    pub fn build(&self) -> String {
        let capacity = self.capacity();
        let mut result = String::with_capacity(capacity);

        for fragment in &self.fragments {
            match fragment {
                StringFragment::Borrowed(s) => result.push_str(s),
                StringFragment::Owned(s) => result.push_str(s),
                StringFragment::Number(n) => result.push_str(&n.to_string()),
                StringFragment::Arc(s) => result.push_str(s.as_ref()),
            }
        }

        result
    }

    /// 构建为 Cow<str>
    pub fn build_cow(&self) -> Cow<'a, str> {
        if self.fragments.len() == 1 {
            match &self.fragments[0] {
                StringFragment::Borrowed(s) => Cow::Borrowed(*s),
                _ => Cow::Owned(self.build()),
            }
        } else {
            Cow::Owned(self.build())
        }
    }
}

impl<'a> Default for StringFragmentBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> StringFragment<'a> {
    fn len(&self) -> usize {
        match self {
            StringFragment::Borrowed(s) => s.len(),
            StringFragment::Owned(s) => s.len(),
            StringFragment::Number(n) => n.to_string().len(),
            StringFragment::Arc(s) => s.len(),
        }
    }
}

/// 路径参数提取器（Cow 优化版本）
///
/// 使用 Cow<str> 避免不必要的字符串分配
pub struct ParamExtractor<'a> {
    path: &'a str,
}

impl<'a> ParamExtractor<'a> {
    /// 创建新的参数提取器
    pub fn new(path: &'a str) -> Self {
        Self { path }
    }

    /// 提取路径段
    ///
    /// 返回路径段的 Cow<str> 切片，避免字符串分配
    pub fn extract_segments(&self) -> Vec<Cow<'a, str>> {
        self.path
            .split('/')
            .filter(|s| !s.is_empty())
            .map(Cow::Borrowed)
            .collect()
    }

    /// 提取参数（使用 Arc<str>）
    ///
    /// 从路径中提取参数值，返回 Arc<str> 减少分配
    pub fn extract_params(&self, pattern: &CowRoutePattern) -> Option<HashMap<Arc<str>, Arc<str>>> {
        if let Some(result) = pattern.match_path(self.path) {
            Some(result.params)
        } else {
            None
        }
    }

    /// 检查路径是否匹配模式
    pub fn matches(&self, pattern: &CowRoutePattern) -> bool {
        pattern.match_path(self.path).is_some()
    }
}

/// 字符串连接优化（使用 Cow）
///
/// 对于只读操作，返回借用的 Cow；对于需要修改的，才分配
pub fn join_cow<'a>(segments: &[Cow<'a, str>], separator: &str) -> Cow<'a, str> {
    if segments.is_empty() {
        return Cow::Borrowed("");
    }

    if segments.len() == 1 {
        return segments[0].clone();
    }

    let capacity = segments.iter().map(|s| s.len()).sum::<usize>() + separator.len() * (segments.len() - 1);
    let mut result = String::with_capacity(capacity);

    for (i, segment) in segments.iter().enumerate() {
        if i > 0 {
            result.push_str(separator);
        }
        result.push_str(segment.as_ref());
    }

    Cow::Owned(result)
}

/// 路径规范化（Cow 优化版本）
///
/// 规范化路径，移除多余的斜杠，返回 Cow<str>
pub fn normalize_path(path: &str) -> Cow<'_, str> {
    if path.is_empty() {
        return Cow::Borrowed("/");
    }

    // 如果路径已经是规范化的，直接返回借用
    if !path.contains("//") && !path.contains("/./") && !path.contains("/../") {
        return Cow::Borrowed(path);
    }

    // 需要规范化，分配新字符串
    let mut result = String::new();
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let mut normalized = Vec::new();

    for segment in segments {
        match segment {
            "." => continue,
            ".." => {
                normalized.pop();
            }
            _ => normalized.push(segment),
        }
    }

    // 总是以斜杠开头
    result.push('/');

    for (i, segment) in normalized.iter().enumerate() {
        if i > 0 {
            result.push('/');
        }
        result.push_str(segment);
    }

    if result == "/" {
        Cow::Borrowed("/")
    } else {
        Cow::Owned(result)
    }
}

/// 路径匹配缓存（使用 Cow 和 Arc）
///
/// 缓存常用的路径匹配结果
#[derive(Debug)]
pub struct PathMatchCache<'a> {
    cache: HashMap<Cow<'a, str>, Vec<Arc<str>>>,
    hits: usize,
    misses: usize,
}

impl<'a> PathMatchCache<'a> {
    /// 创建新的缓存
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            hits: 0,
            misses: 0,
        }
    }

    /// 获取缓存的路径段
    pub fn get(&mut self, path: Cow<'a, str>) -> Option<&Vec<Arc<str>>> {
        if let Some(cached) = self.cache.get(&path) {
            self.hits += 1;
            Some(cached)
        } else {
            self.misses += 1;
            None
        }
    }

    /// 缓存路径段
    pub fn insert(&mut self, path: Cow<'a, str>, segments: Vec<Arc<str>>) {
        self.cache.insert(path, segments);
    }

    /// 获取缓存命中率
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// 清空缓存
    pub fn clear(&mut self) {
        self.cache.clear();
        self.hits = 0;
        self.misses = 0;
    }
}

impl<'a> Default for PathMatchCache<'a> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cow_route_pattern_exact() {
        let pattern = CowRoutePattern::from_str("/users");
        assert!(matches!(pattern, CowRoutePattern::Exact(_)));

        let result = pattern.match_path("/users");
        assert!(result.is_some());
        assert_eq!(result.unwrap().path.as_ref(), "/users");
    }

    #[test]
    fn test_cow_route_pattern_parameterized() {
        let pattern = CowRoutePattern::from_str("/users/{id}");
        assert!(matches!(pattern, CowRoutePattern::Parameterized { .. }));

        let result = pattern.match_path("/users/123");
        assert!(result.is_some());
        let params = result.unwrap().params;
        assert_eq!(params.get("id").map(|s| s.as_ref()), Some("123"));
    }

    #[test]
    fn test_cow_route_pattern_wildcard() {
        let pattern = CowRoutePattern::from_str("/static/*");
        assert!(matches!(pattern, CowRoutePattern::Wildcard { .. }));

        let result = pattern.match_path("/static/css/style.css");
        assert!(result.is_some());
    }

    #[test]
    fn test_string_fragment_builder() {
        let mut builder = StringFragmentBuilder::new();
        builder.push_borrowed("api");
        builder.push_borrowed("v1");
        builder.push_borrowed("users");

        let result = builder.build();
        assert_eq!(result, "apiv1users");
    }

    #[test]
    fn test_string_fragment_builder_with_capacity() {
        let mut builder = StringFragmentBuilder::new();
        builder.push_borrowed("hello");
        builder.push_number(123);
        builder.push_borrowed("world");

        assert_eq!(builder.capacity(), 13);
    }

    #[test]
    fn test_string_fragment_builder_cow() {
        let mut builder = StringFragmentBuilder::new();
        builder.push_borrowed("single");

        let cow = builder.build_cow();
        assert!(matches!(cow, Cow::Borrowed(_)));
        assert_eq!(cow.as_ref(), "single");
    }

    #[test]
    fn test_param_extractor_segments() {
        let extractor = ParamExtractor::new("/api/v1/users");
        let segments = extractor.extract_segments();

        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0].as_ref(), "api");
        assert_eq!(segments[1].as_ref(), "v1");
        assert_eq!(segments[2].as_ref(), "users");
    }

    #[test]
    fn test_param_extractor_matches() {
        let extractor = ParamExtractor::new("/users/123");
        let pattern = CowRoutePattern::from_str("/users/{id}");

        assert!(extractor.matches(&pattern));
    }

    #[test]
    fn test_join_cow_empty() {
        let result = join_cow(&[], "/");
        assert!(matches!(result, Cow::Borrowed(_)));
        assert!(result.is_empty());
    }

    #[test]
    fn test_join_cow_single() {
        let segments = vec![Cow::Borrowed("users")];
        let result = join_cow(&segments, "/");
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(result.as_ref(), "users");
    }

    #[test]
    fn test_join_cow_multiple() {
        let segments = vec![
            Cow::Borrowed("api"),
            Cow::Borrowed("v1"),
            Cow::Borrowed("users"),
        ];
        let result = join_cow(&segments, "/");
        assert!(matches!(result, Cow::Owned(_)));
        assert_eq!(result.as_ref(), "api/v1/users");
    }

    #[test]
    fn test_normalize_path_already_normalized() {
        let result = normalize_path("/api/v1/users");
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(result.as_ref(), "/api/v1/users");
    }

    #[test]
    fn test_normalize_path_double_slash() {
        let result = normalize_path("/api//v1//users");
        assert!(matches!(result, Cow::Owned(_)));
        assert_eq!(result.as_ref(), "/api/v1/users");
    }

    #[test]
    fn test_normalize_path_dot_segments() {
        let result = normalize_path("/api/./v1/users");
        assert!(matches!(result, Cow::Owned(_)));
        assert_eq!(result.as_ref(), "/api/v1/users");
    }

    #[test]
    fn test_normalize_path_dot_dot_segments() {
        let result = normalize_path("/api/v1/../users");
        assert!(matches!(result, Cow::Owned(_)));
        assert_eq!(result.as_ref(), "/api/users");
    }

    #[test]
    fn test_normalize_path_empty() {
        let result = normalize_path("");
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(result.as_ref(), "/");
    }

    #[test]
    fn test_path_match_cache() {
        let mut cache = PathMatchCache::new();

        let path = Cow::Borrowed("/users");
        let segments = vec![Arc::from("users")];

        cache.insert(path.clone(), segments);

        let cached = cache.get(path);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().len(), 1);
    }

    #[test]
    fn test_path_match_cache_hit_rate() {
        let mut cache = PathMatchCache::new();

        let path = Cow::Borrowed("/users");
        let segments = vec![Arc::from("users")];

        cache.insert(path.clone(), segments.clone());
        cache.get(path.clone());
        cache.get(path.clone());

        assert!(cache.hit_rate() > 0.0);
    }

    #[test]
    fn test_optimized_match_result() {
        let result = OptimizedMatchResult {
            path: Cow::Borrowed("/users"),
            params: HashMap::new(),
        };

        assert_eq!(result.path.as_ref(), "/users");
        assert!(result.params.is_empty());
    }
}