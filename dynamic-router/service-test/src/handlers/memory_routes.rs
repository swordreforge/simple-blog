use actix_web::{web, Responder};
use dynamic_route_actix::{RouteTable, core::SimpleRoute};
use std::sync::Arc;
use crate::models::DemoRoute;

/// 添加单个演示路由
pub async fn add_demo_route(
    route: web::Json<DemoRoute>,
    data: web::Data<Arc<RouteTable>>,
) -> impl Responder {
    let route = route.into_inner();
    let simple_route = SimpleRoute::new(&route.body, &route.content_type);
    data.insert(route.path.clone(), Box::new(simple_route));

    actix_web::HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": format!("路由 '{}' 已添加到路径 '{}'", route.name, route.path),
        "path": route.path
    }))
}

/// 获取演示路由列表
pub async fn get_demo_routes() -> impl Responder {
    let demo_routes = vec![
        crate::models::DemoRouteInfo {
            name: "首页".to_string(),
            path: "/".to_string(),
            description: "欢迎页面".to_string(),
            method: "GET".to_string(),
        },
        crate::models::DemoRouteInfo {
            name: "API状态".to_string(),
            path: "/api/status".to_string(),
            description: "返回API状态信息".to_string(),
            method: "GET, POST, PUT, DELETE".to_string(),
        },
        crate::models::DemoRouteInfo {
            name: "用户信息".to_string(),
            path: "/api/user".to_string(),
            description: "用户数据接口".to_string(),
            method: "GET, POST".to_string(),
        },
        crate::models::DemoRouteInfo {
            name: "产品列表".to_string(),
            path: "/api/products".to_string(),
            description: "产品列表接口".to_string(),
            method: "GET".to_string(),
        },
        crate::models::DemoRouteInfo {
            name: "关于我们".to_string(),
            path: "/about".to_string(),
            description: "关于页面".to_string(),
            method: "GET".to_string(),
        },
    ];

    actix_web::HttpResponse::Ok().json(demo_routes)
}

/// 批量添加演示路由
pub async fn add_all_demo_routes(data: web::Data<Arc<RouteTable>>) -> impl Responder {
    // 清空现有路由
    data.clear();

    // 添加演示路由
    let routes = vec![
        ("/", "欢迎使用动态路由系统！<br><br>这是一个基于 Rust 和 Actix-Web 的高性能动态路由库。<br>所有路由都可以通过管理界面动态添加、修改和删除。", "text/html"),
        ("/api/status", r#"{"status":"ok","version":"1.0.0","service":"dynamic-route-service"}"#, "application/json"),
        ("/api/user", r#"{"id":1,"name":"张三","email":"zhangsan@example.com","role":"admin"}"#, "application/json"),
        ("/api/products", r#"{"products":[{"id":1,"name":"产品A","price":99.99},{"id":2,"name":"产品B","price":149.99}]}"#, "application/json"),
        ("/about", "<h1>关于我们</h1><p>这是一个动态路由系统演示项目</p>", "text/html"),
    ];

    let count = routes.len();

    for (path, body, content_type) in &routes {
        let route = SimpleRoute::new(*body, *content_type);
        data.insert(path.to_string(), Box::new(route));
    }

    actix_web::HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": "已添加 5 个演示路由",
        "count": count
    }))
}

/// 清空所有路由
pub async fn clear_all_routes(data: web::Data<Arc<RouteTable>>) -> impl Responder {
    data.clear();

    actix_web::HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": "已清空所有路由"
    }))
}

/// 获取路由统计信息
pub async fn get_stats(data: web::Data<Arc<RouteTable>>) -> impl Responder {
    let count = data.count();
    let paths = data.list_paths();

    actix_web::HttpResponse::Ok().json(serde_json::json!({
        "total_routes": count,
        "routes": paths
    }))
}