use actix_web::{web, Responder};
use dynamic_route_actix::{RouteTable, core::SimpleRoute};
use std::sync::Arc;
use crate::models::FileRoute;
use crate::services::{save_route_to_file, delete_route_from_file, list_file_routes, clear_file_routes};

/// 文件路由添加
pub async fn add_file_route(
    route: web::Json<FileRoute>,
    data: web::Data<Arc<RouteTable>>,
) -> impl Responder {
    let route = route.into_inner();
    let simple_route = SimpleRoute::new(&route.body, &route.content_type);
    
    // 添加到内存路由表
    data.insert(route.path.clone(), Box::new(simple_route.clone()));
    
    // 保存到文件
    if let Err(e) = save_route_to_file(&route.path, &route.body, &route.content_type) {
        return actix_web::HttpResponse::InternalServerError().json(serde_json::json!({
            "status": "error",
            "message": format!("路由添加失败: {}", e)
        }));
    }

    actix_web::HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": format!("路由已添加到路径 '{}'（已保存到文件）", route.path),
        "path": route.path
    }))
}

/// 获取文件路由列表
pub async fn get_file_routes() -> impl Responder {
    match list_file_routes() {
        Ok(routes) => actix_web::HttpResponse::Ok().json(routes),
        Err(e) => actix_web::HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("读取文件路由失败: {}", e)
        }))
    }
}

/// 删除文件路由
pub async fn delete_file_route(
    path: web::Path<String>,
    data: web::Data<Arc<RouteTable>>,
) -> impl Responder {
    let path_str = path.into_inner();
    
    // 从内存中删除
    data.remove(&path_str);
    
    // 从文件中删除
    if let Err(e) = delete_route_from_file(&path_str) {
        return actix_web::HttpResponse::InternalServerError().json(serde_json::json!({
            "status": "error",
            "message": format!("路由删除失败: {}", e)
        }));
    }

    actix_web::HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": format!("路由 '{}' 已删除", path_str)
    }))
}

/// 查看文件路由详情
pub async fn view_file_route(
    path: web::Path<String>,
    data: web::Data<Arc<RouteTable>>,
) -> impl Responder {
    let path_str = path.into_inner();
    
    // Data<Arc<RouteTable>> 需要先解包才能使用
    let route_table = data.as_ref().as_ref();
    
    match route_table.get_arc(&path_str) {
        Some(route) => {
            let serializable = route.to_serializable();
            actix_web::HttpResponse::Ok().json(serde_json::json!({
                "path": path_str,
                "route": serializable
            }))
        }
        None => actix_web::HttpResponse::NotFound().json(serde_json::json!({
            "error": "路由不存在"
        }))
    }
}

/// 批量添加演示文件路由
pub async fn add_all_file_routes(data: web::Data<Arc<RouteTable>>) -> impl Responder {
    // 清空现有路由
    data.clear();
    if let Err(e) = clear_file_routes() {
        return actix_web::HttpResponse::InternalServerError().json(serde_json::json!({
            "status": "error",
            "message": format!("清空文件路由失败: {}", e)
        }));
    }

    // 添加演示路由
    let routes = vec![
        ("/", "欢迎使用文件路由系统！<br><br>这是一个基于 Rust 和 Actix-Web 的高性能动态路由库。<br>所有路由都会持久化存储到文件系统中。", "text/html"),
        ("/api/status", r#"{"status":"ok","version":"1.0.0","service":"file-route-service"}"#, "application/json"),
        ("/api/user", r#"{"id":1,"name":"张三","email":"zhangsan@example.com","role":"admin"}"#, "application/json"),
        ("/api/products", r#"{"products":[{"id":1,"name":"产品A","price":99.99},{"id":2,"name":"产品B","price":149.99}]}"#, "application/json"),
        ("/about", "<h1>关于我们</h1><p>这是一个基于文件存储的动态路由系统演示项目</p>", "text/html"),
    ];

    let count = routes.len();

    for (path, body, content_type) in &routes {
        let route = SimpleRoute::new(*body, *content_type);
        data.insert(path.to_string(), Box::new(route.clone()));
        
        // 保存到文件
        if let Err(e) = save_route_to_file(path, body, content_type) {
            return actix_web::HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": format!("保存路由 '{}' 到文件失败: {}", path, e)
            }));
        }
    }

    actix_web::HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": "已添加 5 个演示文件路由",
        "count": count
    }))
}

/// 清空所有文件路由
pub async fn clear_all_file_routes(data: web::Data<Arc<RouteTable>>) -> impl Responder {
    data.clear();
    
    if let Err(e) = clear_file_routes() {
        return actix_web::HttpResponse::InternalServerError().json(serde_json::json!({
            "status": "error",
            "message": format!("清空文件路由失败: {}", e)
        }));
    }

    actix_web::HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": "已清空所有文件路由"
    }))
}