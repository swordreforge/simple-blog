use serde::{Deserialize, Serialize};
use dynamic_route_actix::RouteTable;
use std::sync::Arc;

/// 服务器状态
pub struct AppState {
    pub route_table: Arc<RouteTable>,
    pub file_route_table: Arc<RouteTable>,
}

/// 演示路由
#[derive(Deserialize)]
pub struct DemoRoute {
    pub name: String,
    pub path: String,
    pub body: String,
    pub content_type: String,
}

/// 文件路由（不包含 name 字段）
#[derive(Deserialize)]
pub struct FileRoute {
    pub path: String,
    pub body: String,
    pub content_type: String,
}

/// 演示路由信息
#[derive(Serialize)]
pub struct DemoRouteInfo {
    pub name: String,
    pub path: String,
    pub description: String,
    pub method: String,
}