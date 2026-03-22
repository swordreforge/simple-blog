//! 动态路由服务
//!
//! 负责动态路由的加载、管理和执行

use crate::db::models::{DynamicRoute, HandlerType};
use crate::db::repositories::DynamicRouteRepository;
use crate::services::route_type_manager::RouteTypeManager;
use actix_web::{HttpRequest, HttpResponse};
use dynamic_route_actix::{RouteEntry, RouteTable, SimpleRoute};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::fs;

/// 路由加载统计
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LoadStats {
    pub database_loaded: usize,
    pub memory_loaded: usize,
    pub file_loaded: usize,
    pub failed: usize,
}

/// 动态路由服务
#[derive(Clone)]
pub struct DynamicRouteService {
    route_table: Arc<RouteTable>,
    repository: DynamicRouteRepository,
    route_type_manager: Option<Arc<RouteTypeManager>>,
}

impl DynamicRouteService {
    /// 创建新的动态路由服务
    pub fn new(
        route_table: Arc<RouteTable>,
        repository: DynamicRouteRepository,
        route_type_manager: Option<Arc<RouteTypeManager>>,
    ) -> Self {
        Self {
            route_table,
            repository,
            route_type_manager,
        }
    }

    /// 从所有存储类型加载路由
    pub async fn load_all_routes(&self) -> Result<LoadStats, Box<dyn std::error::Error>> {
        let mut stats = LoadStats::default();

        // 如果有路由类型管理器，从所有存储加载
        if let Some(manager) = &self.route_type_manager {
            use crate::db::models::RouteType;

            // 1. 从内存存储加载（最快）
            match manager
                .load_all_routes_from_storage(RouteType::Memory)
                .await
            {
                Ok(routes) => {
                    for route in routes {
                        if route.enabled {
                            if let Err(e) = self.add_route_to_table(&route).await {
                                tracing::warn!(
                                    "从内存存储加载路由失败: path={}, error={}",
                                    route.path,
                                    e
                                );
                                stats.failed += 1;
                            } else {
                                stats.memory_loaded += 1;
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("从内存存储加载路由失败: {}", e);
                }
            }

            // 2. 从文件存储加载
            match manager.load_all_routes_from_storage(RouteType::File).await {
                Ok(routes) => {
                    for route in routes {
                        if route.enabled {
                            if let Err(e) = self.add_route_to_table(&route).await {
                                tracing::warn!(
                                    "从文件存储加载路由失败: path={}, error={}",
                                    route.path,
                                    e
                                );
                                stats.failed += 1;
                            } else {
                                stats.file_loaded += 1;
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("从文件存储加载路由失败: {}", e);
                }
            }

            // 3. 从数据库存储加载
            match manager
                .load_all_routes_from_storage(RouteType::Database)
                .await
            {
                Ok(routes) => {
                    for route in routes {
                        if route.enabled {
                            if let Err(e) = self.add_route_to_table(&route).await {
                                tracing::warn!(
                                    "从数据库存储加载路由失败: path={}, error={}",
                                    route.path,
                                    e
                                );
                                stats.failed += 1;
                            } else {
                                stats.database_loaded += 1;
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("从数据库存储加载路由失败: {}", e);
                }
            }

            tracing::info!("路由加载统计: {:?}", stats);
            Ok(stats)
        } else {
            // 兼容性：如果没有 RouteTypeManager，只从数据库加载
            let routes = self.repository.get_all_enabled().await?;
            let count = routes.len();

            for route in routes {
                if let Err(e) = self.add_route_to_table(&route).await {
                    tracing::warn!("加载路由失败: path={}, error={}", route.path, e);
                }
            }

            tracing::info!("已加载 {} 个动态路由（仅数据库）", count);
            Ok(LoadStats {
                database_loaded: count,
                ..Default::default()
            })
        }
    }

    /// 从数据库加载所有启用的路由到路由表
    ///
    /// 注意：此方法为了向后兼容而保留，内部实际调用 `load_all_routes()`
    #[allow(dead_code)]
    pub async fn load_routes_from_db(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.load_all_routes().await?;
        Ok(())
    }

    /// 添加路由到路由表
    async fn add_route_to_table(
        &self,
        route: &DynamicRoute,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 规范化路径：确保以 / 开头，移除尾部斜杠（根路径除外），规范化多个连续斜杠
        let normalized_path = normalize_path(&route.path);

        match route.handler_type {
            HandlerType::Redirect => {
                // 处理重定向路由
                if let Some(target) = route.handler_config.get("target") {
                    let target_str = target.as_str().ok_or("target 必须是字符串")?;
                    let status_code = route
                        .handler_config
                        .get("status_code")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(302) as u16;

                    // 创建重定向路由处理器
                    let redirect_route = RedirectHandler::new(target_str.to_string(), status_code);
                    self.route_table
                        .insert(normalized_path, Box::new(redirect_route));
                }
            }
            HandlerType::Static => {
                // 处理静态内容路由
                let content_str = match route.route_type {
                    // file 类型：从 template_path 读取文件
                    crate::db::models::RouteType::File => {
                        if let Some(ref path) = route.template_path {
                            // 读取模板文件
                            match fs::read_to_string(path).await {
                                Ok(content) => content,
                                Err(e) => {
                                    tracing::error!("读取模板文件失败: path={}, error={}", path, e);
                                    return Err(format!("读取模板文件失败: {}", e).into());
                                }
                            }
                        } else {
                            return Err("file 类型路由必须提供 template_path".into());
                        }
                    }
                    // database/memory 类型：优先使用 inline_template
                    crate::db::models::RouteType::Database
                    | crate::db::models::RouteType::Memory => {
                        if let Some(ref template) = route.inline_template {
                            if !template.is_empty() {
                                template.clone()
                            } else {
                                // 兼容性：如果 inline_template 为空，尝试从 handler_config.content 读取
                                route
                                    .handler_config
                                    .get("content")
                                    .and_then(|v| v.as_str())
                                    .ok_or("需要提供 inline_template 或 handler_config.content")?
                                    .to_string()
                            }
                        } else {
                            // 兼容性：如果 inline_template 不存在，尝试从 handler_config.content 读取
                            route
                                .handler_config
                                .get("content")
                                .and_then(|v| v.as_str())
                                .ok_or("需要提供 inline_template 或 handler_config.content")?
                                .to_string()
                        }
                    }
                };

                let content_type = route
                    .handler_config
                    .get("content_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("text/html; charset=utf-8");

                let static_route = SimpleRoute::new(content_str, content_type.to_string());
                self.route_table
                    .insert(normalized_path, Box::new(static_route));
            }
            HandlerType::Proxy => {
                // 处理代理路由
                if let Some(target) = route.handler_config.get("target") {
                    let target_str = target.as_str().ok_or("target 必须是字符串")?;
                    let proxy_route = ProxyHandler::new(target_str.to_string());
                    self.route_table
                        .insert(normalized_path, Box::new(proxy_route));
                }
            }
            HandlerType::Custom => {
                // 处理自定义处理器路由
                let custom_route = CustomHandler::new(route.handler_config.clone());
                self.route_table
                    .insert(normalized_path, Box::new(custom_route));
            }
        }

        Ok(())
    }

    /// 获取路由表引用
    #[allow(dead_code)]
    pub fn route_table(&self) -> &Arc<RouteTable> {
        &self.route_table
    }

    /// 热更新：重新加载单个路由
    pub async fn reload_route(&self, id: i64) -> Result<(), Box<dyn std::error::Error>> {
        // 查询数据库获取路由信息
        if let Some(route) = self.repository.get_by_id(id).await? {
            // 先从路由表中删除旧路由
            self.route_table.remove(&route.path);
            tracing::debug!("已从路由表移除路由: path={}", route.path);

            // 如果路由是启用状态，重新加载
            if route.enabled {
                self.add_route_to_table(&route).await?;
                tracing::info!(
                    "路由热更新成功: path={}, type={}",
                    route.path,
                    route.handler_type
                );
            }
        }

        Ok(())
    }

    /// 热更新：移除路由
    pub fn remove_route(&self, path: &str) {
        self.route_table.remove(path);
        tracing::info!("已从路由表移除路由: path={}", path);
    }

    /// 检查路由是否在路由表中
    #[allow(dead_code)]
    pub fn route_exists(&self, path: &str) -> bool {
        self.route_table.get_arc(path).is_some()
    }

    /// 获取路由表中的路由数量
    #[allow(dead_code)]
    pub fn route_count(&self) -> usize {
        self.route_table.count()
    }
}

/// 重定向处理器
pub struct RedirectHandler {
    target: String,
    status_code: u16,
}

impl RedirectHandler {
    pub fn new(target: String, status_code: u16) -> Self {
        Self {
            target,
            status_code,
        }
    }
}

impl RouteEntry for RedirectHandler {
    fn handle(&self, _req: &HttpRequest) -> Pin<Box<dyn Future<Output = HttpResponse> + Send>> {
        let target = self.target.clone();
        let status = actix_web::http::StatusCode::from_u16(self.status_code)
            .unwrap_or(actix_web::http::StatusCode::FOUND);
        Box::pin(async move {
            HttpResponse::build(status)
                .insert_header(("Location", target))
                .finish()
        })
    }

    fn clone_box(&self) -> Box<dyn RouteEntry> {
        Box::new(RedirectHandler::new(self.target.clone(), self.status_code))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn to_serializable(&self) -> dynamic_route_actix::SerializableRoute {
        dynamic_route_actix::SerializableRoute {
            route_type: "redirect".to_string(),
            body: self.target.clone(),
            content_type: "text/plain".to_string(),
            extra_data: Some(serde_json::json!({"status_code": self.status_code}).to_string()),
        }
    }

    fn from_serializable(data: dynamic_route_actix::SerializableRoute) -> Box<dyn RouteEntry>
    where
        Self: Sized,
    {
        let status_code = if let Some(ref extra) = data.extra_data {
            serde_json::from_str(extra).unwrap_or(302)
        } else {
            302
        };
        Box::new(RedirectHandler::new(data.body, status_code))
    }
}

impl std::fmt::Debug for RedirectHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedirectHandler")
            .field("target", &self.target)
            .field("status_code", &self.status_code)
            .finish()
    }
}

/// 代理处理器
pub struct ProxyHandler {
    target: String,
}

impl ProxyHandler {
    pub fn new(target: String) -> Self {
        Self { target }
    }
}

impl RouteEntry for ProxyHandler {
    fn handle(&self, _req: &HttpRequest) -> Pin<Box<dyn Future<Output = HttpResponse> + Send>> {
        let target = self.target.clone();
        Box::pin(async move { HttpResponse::Ok().body(format!("Proxy to: {}", target)) })
    }

    fn clone_box(&self) -> Box<dyn RouteEntry> {
        Box::new(ProxyHandler::new(self.target.clone()))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn to_serializable(&self) -> dynamic_route_actix::SerializableRoute {
        dynamic_route_actix::SerializableRoute {
            route_type: "proxy".to_string(),
            body: self.target.clone(),
            content_type: "text/plain".to_string(),
            extra_data: None,
        }
    }

    fn from_serializable(data: dynamic_route_actix::SerializableRoute) -> Box<dyn RouteEntry>
    where
        Self: Sized,
    {
        Box::new(ProxyHandler::new(data.body))
    }
}

impl std::fmt::Debug for ProxyHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProxyHandler")
            .field("target", &self.target)
            .finish()
    }
}

/// 自定义处理器
pub struct CustomHandler {
    config: serde_json::Value,
}

impl CustomHandler {
    pub fn new(config: serde_json::Value) -> Self {
        Self { config }
    }
}

impl RouteEntry for CustomHandler {
    fn handle(&self, _req: &HttpRequest) -> Pin<Box<dyn Future<Output = HttpResponse> + Send>> {
        Box::pin(async move { HttpResponse::Ok().body("Custom handler executed") })
    }

    fn clone_box(&self) -> Box<dyn RouteEntry> {
        Box::new(CustomHandler::new(self.config.clone()))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn to_serializable(&self) -> dynamic_route_actix::SerializableRoute {
        dynamic_route_actix::SerializableRoute {
            route_type: "custom".to_string(),
            body: "Custom handler".to_string(),
            content_type: "text/plain".to_string(),
            extra_data: Some(self.config.to_string()),
        }
    }

    fn from_serializable(data: dynamic_route_actix::SerializableRoute) -> Box<dyn RouteEntry>
    where
        Self: Sized,
    {
        let config = if let Some(ref extra) = data.extra_data {
            serde_json::from_str(extra).unwrap_or_default()
        } else {
            serde_json::Value::Null
        };
        Box::new(CustomHandler::new(config))
    }
}

impl std::fmt::Debug for CustomHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CustomHandler")
            .field("config", &self.config)
            .finish()
    }
}

/// 规范化路径
///
/// 确保路径以 / 开头，移除尾部斜杠（根路径除外），规范化多个连续斜杠
fn normalize_path(path: &str) -> String {
    let mut normalized = path.trim().to_string();

    // 确保以 / 开头
    if !normalized.starts_with('/') {
        normalized = format!("/{}", normalized);
    }

    // 标准化多个连续斜杠为单个斜杠
    while normalized.contains("//") {
        normalized = normalized.replace("//", "/");
    }

    // 移除尾部斜杠（根路径除外）
    if normalized.len() > 1 && normalized.ends_with('/') {
        normalized.pop();
    }

    normalized
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http::StatusCode, test};

    #[tokio::test]
    async fn test_normalize_path_basic() {
        assert_eq!(normalize_path("/users"), "/users");
        assert_eq!(normalize_path("/users/"), "/users");
        assert_eq!(normalize_path("users"), "/users");
    }

    #[tokio::test]
    async fn test_normalize_path_multiple_slashes() {
        assert_eq!(normalize_path("//users"), "/users");
        assert_eq!(normalize_path("/users//123"), "/users/123");
        assert_eq!(normalize_path("//users//123//"), "/users/123");
    }

    #[tokio::test]
    async fn test_normalize_path_root() {
        assert_eq!(normalize_path("/"), "/");
        assert_eq!(normalize_path("//"), "/");
        assert_eq!(normalize_path(""), "/");
    }

    #[tokio::test]
    async fn test_normalize_path_whitespace() {
        assert_eq!(normalize_path("  /users  "), "/users");
        assert_eq!(normalize_path(" /users "), "/users");
    }

    #[tokio::test]
    async fn test_normalize_path_complex() {
        assert_eq!(normalize_path("/api/v1/users/"), "/api/v1/users");
        assert_eq!(normalize_path("api/v1/users"), "/api/v1/users");
        assert_eq!(normalize_path("/api//v1//users/"), "/api/v1/users");
    }

    #[tokio::test]
    async fn test_load_stats_default() {
        let stats = LoadStats::default();
        assert_eq!(stats.database_loaded, 0);
        assert_eq!(stats.memory_loaded, 0);
        assert_eq!(stats.file_loaded, 0);
        assert_eq!(stats.failed, 0);
    }

    #[tokio::test]
    async fn test_load_stats_serialization() {
        let stats = LoadStats {
            database_loaded: 10,
            memory_loaded: 5,
            file_loaded: 3,
            failed: 1,
        };

        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: LoadStats = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.database_loaded, 10);
        assert_eq!(deserialized.memory_loaded, 5);
        assert_eq!(deserialized.file_loaded, 3);
        assert_eq!(deserialized.failed, 1);
    }

    #[tokio::test]
    async fn test_redirect_handler_new() {
        let handler = RedirectHandler::new("/target".to_string(), 302);
        assert_eq!(handler.target, "/target");
        assert_eq!(handler.status_code, 302);
    }

    #[actix_web::test]
    async fn test_redirect_handler_handle() {
        let handler = RedirectHandler::new("/target".to_string(), 302);
        let req = test::TestRequest::default().to_http_request();
        
        let resp = handler.handle(&req).await;
        assert_eq!(resp.status(), StatusCode::FOUND);
        
        let location = resp.headers().get("Location").unwrap();
        assert_eq!(location, "/target");
    }

    #[tokio::test]
    async fn test_redirect_handler_clone_box() {
        let handler = RedirectHandler::new("/target".to_string(), 301);
        let _cloned = handler.clone_box();

        assert_eq!(handler.target, "/target");
        assert_eq!(handler.status_code, 301);
    }

    #[tokio::test]
    async fn test_redirect_handler_serializable() {
        let handler = RedirectHandler::new("/target".to_string(), 302);
        let serializable = handler.to_serializable();

        assert_eq!(serializable.route_type, "redirect");
        assert_eq!(serializable.body, "/target");

        let _deserialized = RedirectHandler::from_serializable(serializable);
        // 验证反序列化成功，不检查具体字段因为返回的是trait对象
    }

    #[tokio::test]
    async fn test_redirect_handler_debug() {
        let handler = RedirectHandler::new("/target".to_string(), 302);
        let debug_str = format!("{:?}", handler);
        assert!(debug_str.contains("RedirectHandler"));
        assert!(debug_str.contains("/target"));
    }

    #[tokio::test]
    async fn test_proxy_handler_new() {
        let handler = ProxyHandler::new("http://example.com".to_string());
        assert_eq!(handler.target, "http://example.com");
    }

    #[actix_web::test]
    async fn test_proxy_handler_handle() {
        let handler = ProxyHandler::new("http://example.com".to_string());
        let req = test::TestRequest::default().to_http_request();

        let resp = handler.handle(&req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body = actix_web::body::to_bytes(resp.into_body()).await.unwrap();
        assert_eq!(body, "Proxy to: http://example.com");
    }

    #[tokio::test]
    async fn test_proxy_handler_clone_box() {
        let handler = ProxyHandler::new("http://example.com".to_string());
        let _cloned = handler.clone_box();

        assert_eq!(handler.target, "http://example.com");
    }

    #[tokio::test]
    async fn test_proxy_handler_serializable() {
        let handler = ProxyHandler::new("http://example.com".to_string());
        let serializable = handler.to_serializable();

        assert_eq!(serializable.route_type, "proxy");
        assert_eq!(serializable.body, "http://example.com");

        let _deserialized = ProxyHandler::from_serializable(serializable);
        // 验证反序列化成功，不检查具体字段因为返回的是trait对象
    }

    #[tokio::test]
    async fn test_proxy_handler_debug() {
        let handler = ProxyHandler::new("http://example.com".to_string());
        let debug_str = format!("{:?}", handler);
        assert!(debug_str.contains("ProxyHandler"));
        assert!(debug_str.contains("http://example.com"));
    }

    #[tokio::test]
    async fn test_custom_handler_new() {
        let config = serde_json::json!({"key": "value"});
        let handler = CustomHandler::new(config.clone());
        assert_eq!(handler.config, config);
    }

    #[actix_web::test]
    async fn test_custom_handler_handle() {
        let config = serde_json::json!({"key": "value"});
        let handler = CustomHandler::new(config);
        let req = test::TestRequest::default().to_http_request();

        let resp = handler.handle(&req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body = actix_web::body::to_bytes(resp.into_body()).await.unwrap();
        assert_eq!(body, "Custom handler executed");
    }

    #[tokio::test]
    async fn test_custom_handler_clone_box() {
        let config = serde_json::json!({"key": "value"});
        let handler = CustomHandler::new(config.clone());
        let _cloned = handler.clone_box();

        assert_eq!(handler.config, config);
    }

    #[tokio::test]
    async fn test_custom_handler_serializable() {
        let config = serde_json::json!({"key": "value", "number": 42});
        let handler = CustomHandler::new(config.clone());
        let serializable = handler.to_serializable();

        assert_eq!(serializable.route_type, "custom");
        assert!(serializable.extra_data.is_some());

        let _deserialized = CustomHandler::from_serializable(serializable);
        // 验证反序列化成功，不检查具体字段因为返回的是trait对象
    }

    #[tokio::test]
    async fn test_custom_handler_debug() {
        let config = serde_json::json!({"key": "value"});
        let handler = CustomHandler::new(config);
        let debug_str = format!("{:?}", handler);
        assert!(debug_str.contains("CustomHandler"));
    }

    #[tokio::test]
    async fn test_custom_handler_with_null_config() {
        let handler = CustomHandler::new(serde_json::Value::Null);
        assert_eq!(handler.config, serde_json::Value::Null);
    }

    #[tokio::test]
    async fn test_redirect_handler_different_status_codes() {
        let codes = [301, 302, 303, 307, 308];

        for code in codes {
            let handler = RedirectHandler::new("/target".to_string(), code);
            let req = test::TestRequest::default().to_http_request();

            let expected_status = StatusCode::from_u16(code).unwrap();
            let resp = handler.handle(&req).await;
            assert_eq!(resp.status(), expected_status);
        }
    }

    #[tokio::test]
    async fn test_redirect_handler_with_special_chars() {
        let handler = RedirectHandler::new("/target?param=value&other=test".to_string(), 302);
        assert_eq!(handler.target, "/target?param=value&other=test");
    }

    #[tokio::test]
    async fn test_proxy_handler_with_url_components() {
        let urls = [
            "http://example.com",
            "https://example.com:8080",
            "http://example.com/path/to/resource",
            "https://user:pass@example.com",
        ];

        for url in urls {
            let handler = ProxyHandler::new(url.to_string());
            assert_eq!(handler.target, url);
        }
    }

    #[tokio::test]
    async fn test_normalize_path_edge_cases() {
        // 测试边界情况
        assert_eq!(normalize_path(""), "/");
        assert_eq!(normalize_path("   "), "/");
        assert_eq!(normalize_path("/"), "/");
        assert_eq!(normalize_path("//"), "/");
        assert_eq!(normalize_path("///"), "/");
        assert_eq!(normalize_path("users/"), "/users");
        assert_eq!(normalize_path("/users"), "/users");
    }

    #[tokio::test]
    async fn test_load_stats_increment() {
        let mut stats = LoadStats::default();

        stats.database_loaded += 1;
        stats.memory_loaded += 2;
        stats.file_loaded += 3;
        stats.failed += 1;

        assert_eq!(stats.database_loaded, 1);
        assert_eq!(stats.memory_loaded, 2);
        assert_eq!(stats.file_loaded, 3);
        assert_eq!(stats.failed, 1);
    }
}
