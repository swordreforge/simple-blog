use actix_web::{web, HttpResponse};
use crate::app_state::AppState;
use crate::middleware::auth::check_admin_auth;

/// 导出路由配置
pub async fn export_routes(
    req: actix_web::HttpRequest,
    state: web::Data<AppState>,
) -> HttpResponse {
    // 权限检查
    if check_admin_auth(&req).is_none() {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "success": false,
            "message": "需要管理员权限"
        }));
    }

    let repo = state.dynamic_route_repository();

    // 获取所有路由
    match repo.list(0, 10000, None, None).await {
        Ok((routes, total)) => {
            let export_data = serde_json::json!({
                "version": "1.0",
                "exported_at": chrono::Utc::now().to_rfc3339(),
                "total": total,
                "routes": routes
            });

            HttpResponse::Ok()
                .content_type("application/json")
                .append_header(("Content-Disposition", "attachment; filename=\"routes-export.json\""))
                .json(serde_json::json!({
                    "success": true,
                    "data": export_data
                }))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": format!("导出失败: {}", e)
            }))
        }
    }
}

/// 导入路由配置
pub async fn import_routes(
    req: actix_web::HttpRequest,
    import_data: web::Json<serde_json::Value>,
    state: web::Data<AppState>,
) -> HttpResponse {
    // 权限检查
    let admin_info = match check_admin_auth(&req) {
        Some(info) => info,
        None => return HttpResponse::Forbidden().json(serde_json::json!({
            "success": false,
            "message": "需要管理员权限"
        })),
    };

    let repo = state.dynamic_route_repository();
    let username = &admin_info.1;

    // 解析导入数据
    let import_obj = import_data.into_inner();
    let routes = match import_obj.get("routes") {
        Some(r) if r.is_array() => r.as_array().unwrap(),
        _ => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "message": "无效的导入数据格式: 缺少routes数组"
            }));
        }
    };

    let mut imported_count = 0;
    let mut skipped_count = 0;
    let mut failed_count = 0;
    let mut errors = Vec::new();

    for (index, route_value) in routes.iter().enumerate() {
        // 解析路由配置
        let route: crate::db::models::DynamicRoute = match serde_json::from_value(route_value.clone()) {
            Ok(r) => r,
            Err(e) => {
                failed_count += 1;
                errors.push(format!("第{}条路由: {}", index + 1, e));
                continue;
            }
        };

        // 检查路径冲突
        if let Ok(Some(_)) = repo.get_by_path(&route.path).await {
            skipped_count += 1;
            continue;
        }

        // 导入路由
        let import_route = crate::db::models::DynamicRoute {
            id: None,
            route_name: route.route_name,
            route_type: route.route_type,
            path: route.path,
            handler_type: route.handler_type,
            handler_config: route.handler_config,
            content_source: route.content_source,
            content_template: route.content_template,
            content_type_hint: route.content_type_hint,
            enabled: route.enabled,
            priority: route.priority,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            created_by: Some(username.to_string()),
            metadata: route.metadata,
        };

        match repo.create(&import_route).await {
            Ok(id) => {
                imported_count += 1;
                log_import_operation(&repo, id, &import_route, username);
            }
            Err(e) => {
                failed_count += 1;
                errors.push(format!("第{}条路由: {}", index + 1, e));
            }
        }
    }

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": format!("导入完成: 成功 {}, 跳过 {}, 失败 {}", imported_count, skipped_count, failed_count),
        "data": {
            "imported": imported_count,
            "skipped": skipped_count,
            "failed": failed_count,
            "errors": errors
        }
    }))
}

/// 记录导入操作日志
fn log_import_operation(
    repo: &crate::db::repositories::DynamicRouteRepository,
    route_id: i64,
    route: &crate::db::models::DynamicRoute,
    username: &str,
) {
    use serde_json::to_string;

    let new_config_str = to_string(route).ok();

    // 记录日志（忽略错误）
    let _ = repo.log_operation(
        Some(route_id),
        "import",
        None,
        new_config_str,
        Some(username.to_string()),
        None,
        None,
    );
}