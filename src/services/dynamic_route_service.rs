//! 动态路由服务
//!
//! 负责动态路由的加载、管理和执行

use std::sync::Arc;
use std::future::Future;
use std::pin::Pin;
use actix_web::{HttpRequest, HttpResponse};
use dynamic_route_actix::{RouteTable, SimpleRoute, RouteEntry};
use crate::db::repositories::DynamicRouteRepository;
use crate::db::models::{DynamicRoute, HandlerType};

/// 动态路由服务
#[derive(Clone)]
pub struct DynamicRouteService {
    route_table: Arc<RouteTable>,
    repository: DynamicRouteRepository,
}

impl DynamicRouteService {
    /// 创建新的动态路由服务
    pub fn new(route_table: Arc<RouteTable>, repository: DynamicRouteRepository) -> Self {
        Self {
            route_table,
            repository,
        }
    }

    /// 从数据库加载所有启用的路由到路由表
    pub async fn load_routes_from_db(&self) -> Result<(), Box<dyn std::error::Error>> {
        let routes = self.repository.get_all_enabled().await?;
        let count = routes.len();
        
        for route in routes {
            if let Err(e) = self.add_route_to_table(&route) {
                tracing::warn!("加载路由失败: path={}, error={}", route.path, e);
            } else {
                tracing::debug!("加载路由成功: path={}, type={}", route.path, route.handler_type);
            }
        }
        
        tracing::info!("已加载 {} 个动态路由", count);
        Ok(())
    }

    /// 添加路由到路由表
    fn add_route_to_table(&self, route: &DynamicRoute) -> Result<(), Box<dyn std::error::Error>> {
        match route.handler_type {
            HandlerType::Redirect => {
                // 处理重定向路由
                if let Some(target) = route.handler_config.get("target") {
                    let target_str = target.as_str().ok_or("target 必须是字符串")?;
                    let status_code = route.handler_config
                        .get("status_code")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(302) as u16;
                    
                    // 创建重定向路由处理器
                    let redirect_route = RedirectHandler::new(target_str.to_string(), status_code);
                    self.route_table.insert(route.path.clone(), Box::new(redirect_route));
                }
            }
            HandlerType::Static => {
                // 处理静态内容路由
                if let Some(content) = route.handler_config.get("content") {
                    let content_str = content.as_str().ok_or("content 必须是字符串")?;
                    let content_type = route.handler_config
                        .get("content_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("text/plain");
                    
                    let static_route = SimpleRoute::new(content_str.to_string(), content_type.to_string());
                    self.route_table.insert(route.path.clone(), Box::new(static_route));
                }
            }
            HandlerType::Template => {
                // 处理模板渲染路由
                let template_name = route.handler_config
                    .get("template_name")
                    .and_then(|v| v.as_str())
                    .ok_or("template_name 是必需的")?;
                
                let template_route = TemplateHandler::new(
                    template_name.to_string(),
                    route.handler_config.clone(),
                );
                self.route_table.insert(route.path.clone(), Box::new(template_route));
            }
            HandlerType::Proxy => {
                // 处理代理路由
                if let Some(target) = route.handler_config.get("target") {
                    let target_str = target.as_str().ok_or("target 必须是字符串")?;
                    let proxy_route = ProxyHandler::new(target_str.to_string());
                    self.route_table.insert(route.path.clone(), Box::new(proxy_route));
                }
            }
            HandlerType::Custom => {
                // 处理自定义处理器路由
                let custom_route = CustomHandler::new(route.handler_config.clone());
                self.route_table.insert(route.path.clone(), Box::new(custom_route));
            }
        }
        
        Ok(())
    }

    /// 获取路由表引用
    pub fn route_table(&self) -> &Arc<RouteTable> {
        &self.route_table
    }

    /// 热更新：重新加载单个路由
    pub async fn reload_route(&self, id: i64) -> Result<(), Box<dyn std::error::Error>> {
        // 先从路由表中删除旧路由
        if let Some(route) = self.repository.get_by_id(id).await? {
            self.route_table.remove(&route.path);
            tracing::debug!("已从路由表移除路由: path={}", route.path);
        }

        // 如果路由是启用状态，重新加载
        if let Some(route) = self.repository.get_by_id(id).await? {
            if route.enabled {
                self.add_route_to_table(&route)?;
                tracing::info!("路由热更新成功: path={}, type={}", route.path, route.handler_type);
            }
        }

        Ok(())
    }

    /// 热更新：移除路由
    pub fn remove_route(&self, path: &str) {
        self.route_table.remove(path);
        tracing::info!("已从路由表移除路由: path={}", path);
    }

    /// 热更新：批量重新加载所有路由
    pub async fn reload_all_routes(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 清空路由表
        self.route_table.clear();
        tracing::info!("已清空路由表");

        // 重新加载所有启用的路由
        self.load_routes_from_db().await?;

        Ok(())
    }

    /// 检查路由是否在路由表中
    pub fn route_exists(&self, path: &str) -> bool {
        self.route_table.get_arc(path).is_some()
    }

    /// 获取路由表中的路由数量
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
        Self { target, status_code }
    }

    pub fn target(&self) -> &str {
        &self.target
    }

    pub fn status_code(&self) -> u16 {
        self.status_code
    }
}

impl RouteEntry for RedirectHandler {
    fn handle(&self, _req: &HttpRequest) -> Pin<Box<dyn Future<Output = HttpResponse> + Send>> {
        let target = self.target.clone();
        let status = actix_web::http::StatusCode::from_u16(self.status_code).unwrap_or(actix_web::http::StatusCode::FOUND);
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

/// 模板处理器
pub struct TemplateHandler {
    template_name: String,
    context: serde_json::Value,
}

impl TemplateHandler {
    pub fn new(template_name: String, context: serde_json::Value) -> Self {
        Self { template_name, context }
    }

    pub fn template_name(&self) -> &str {
        &self.template_name
    }

    pub fn context(&self) -> &serde_json::Value {
        &self.context
    }
}

impl RouteEntry for TemplateHandler {
    fn handle(&self, _req: &HttpRequest) -> Pin<Box<dyn Future<Output = HttpResponse> + Send>> {
        let template_name = self.template_name.clone();
        Box::pin(async move {
            HttpResponse::Ok()
                .content_type("text/html")
                .body(format!("Template: {}", template_name))
        })
    }

    fn clone_box(&self) -> Box<dyn RouteEntry> {
        Box::new(TemplateHandler::new(self.template_name.clone(), self.context.clone()))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn to_serializable(&self) -> dynamic_route_actix::SerializableRoute {
        dynamic_route_actix::SerializableRoute {
            route_type: "template".to_string(),
            body: self.template_name.clone(),
            content_type: "text/html".to_string(),
            extra_data: Some(self.context.to_string()),
        }
    }

    fn from_serializable(data: dynamic_route_actix::SerializableRoute) -> Box<dyn RouteEntry>
    where
        Self: Sized,
    {
        let context = if let Some(ref extra) = data.extra_data {
            serde_json::from_str(extra).unwrap_or_default()
        } else {
            serde_json::Value::Null
        };
        Box::new(TemplateHandler::new(data.body, context))
    }
}

impl std::fmt::Debug for TemplateHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TemplateHandler")
            .field("template_name", &self.template_name)
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

    pub fn target(&self) -> &str {
        &self.target
    }
}

impl RouteEntry for ProxyHandler {
    fn handle(&self, _req: &HttpRequest) -> Pin<Box<dyn Future<Output = HttpResponse> + Send>> {
        let target = self.target.clone();
        Box::pin(async move {
            HttpResponse::Ok()
                .body(format!("Proxy to: {}", target))
        })
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

    pub fn config(&self) -> &serde_json::Value {
        &self.config
    }
}

impl RouteEntry for CustomHandler {
    fn handle(&self, _req: &HttpRequest) -> Pin<Box<dyn Future<Output = HttpResponse> + Send>> {
        Box::pin(async move {
            HttpResponse::Ok()
                .body("Custom handler executed")
        })
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