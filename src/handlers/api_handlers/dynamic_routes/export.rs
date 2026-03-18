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
        Ok((mut routes, total)) => {
            // 对 file 类型的路由进行特殊处理：读取模板文件内容到 inline_template
            for route in &mut routes {
                if route.route_type == crate::db::models::RouteType::File {
                    if let Some(ref template_path) = route.template_path {
                        // 尝试读取模板文件内容
                        match std::fs::read_to_string(template_path) {
                            Ok(content) => {
                                tracing::info!("导出 file 类型路由: 已读取模板文件 {}", template_path);
                                // 将文件内容放到 inline_template 中
                                route.inline_template = Some(content);
                            }
                            Err(e) => {
                                tracing::warn!("导出 file 类型路由: 无法读取模板文件 {}: {}", template_path, e);
                                // 如果读取失败，inline_template 保持为 None
                            }
                        }
                    }
                }
            }

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

        // 检查路径冲突（在所有存储类型中检查）
        let has_conflict = if let Some(manager) = state.route_type_manager() {
            // 检查所有存储类型
            let mut conflict = false;
            for route_type in [crate::db::models::RouteType::Database, crate::db::models::RouteType::Memory, crate::db::models::RouteType::File] {
                if let Ok(Some(_)) = manager.load_route_by_path(&route.path, Some(route_type)).await {
                    conflict = true;
                    break;
                }
            }
            conflict
        } else {
            // 兼容性：只检查数据库
            if let Ok(Some(_)) = repo.get_by_path(&route.path).await {
                true
            } else {
                false
            }
        };

        if has_conflict {
            skipped_count += 1;
            continue;
        }

        // 导入路由
        let mut import_route = crate::db::models::DynamicRoute {
            id: None,
            route_name: route.route_name,
            route_type: route.route_type,
            path: route.path,
            handler_type: route.handler_type,
            handler_config: route.handler_config,
            inline_template: route.inline_template,
            template_path: route.template_path,
            content_type_hint: route.content_type_hint,
            enabled: route.enabled,
            priority: route.priority,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            created_by: Some(username.to_string()),
            metadata: route.metadata,
        };

        // 使用 RouteTypeManager 导入路由
        let import_result = if let Some(manager) = state.route_type_manager() {
            // 先在数据库中创建记录
            let db_id = match repo.create(&import_route).await {
                Ok(id) => id,
                Err(e) => {
                    failed_count += 1;
                    errors.push(format!("第{}条路由（数据库）: {}", index + 1, e));
                    continue;
                }
            };

            // 更新 import_route 的 ID，避免后续重复插入
            import_route.id = Some(db_id);

            // 如果不是数据库类型，还需要在对应的存储后端中保存
            if import_route.route_type != crate::db::models::RouteType::Database {
                // 特殊处理 file 类型：需要创建模板文件
                if import_route.route_type == crate::db::models::RouteType::File {
                    // 如果 inline_template 有内容，将其写入 template_path 指定的文件
                    if let Some(ref inline_template) = import_route.inline_template {
                        let template_path = import_route.template_path.clone().unwrap_or_else(|| {
                            format!("data/routes/routes/route_{}.html", db_id)
                        });

                        // 确保目录存在
                        if let Some(parent_dir) = std::path::Path::new(&template_path).parent() {
                            if let Err(e) = std::fs::create_dir_all(parent_dir) {
                                tracing::error!("创建目录失败: {}", e);
                                failed_count += 1;
                                errors.push(format!("第{}条路由（创建目录）: {}", index + 1, e));
                                let _ = repo.delete(db_id).await;
                                continue;
                            }
                        }

                        // 写入模板文件
                        if let Err(e) = std::fs::write(&template_path, inline_template) {
                            tracing::error!("写入模板文件失败: {}", e);
                            failed_count += 1;
                            errors.push(format!("第{}条路由（写入模板文件）: {}", index + 1, e));
                            let _ = repo.delete(db_id).await;
                            continue;
                        }

                        tracing::info!("导入 file 类型路由: 已创建模板文件 {}", template_path);

                        // 清除 inline_template（file 类型不需要）
                        import_route.inline_template = None;
                        import_route.template_path = Some(template_path);
                    } else {
                        // 如果 inline_template 为空，检查 template_path 是否有值
                        let template_path = import_route.template_path.clone().unwrap_or_else(|| {
                            format!("data/routes/routes/route_{}.html", db_id)
                        });

                        // 检查文件是否存在
                        if !std::path::Path::new(&template_path).exists() {
                            // 创建空文件
                            if let Some(parent_dir) = std::path::Path::new(&template_path).parent() {
                                if let Err(e) = std::fs::create_dir_all(parent_dir) {
                                    tracing::error!("创建目录失败: {}", e);
                                    failed_count += 1;
                                    errors.push(format!("第{}条路由（创建目录）: {}", index + 1, e));
                                    let _ = repo.delete(db_id).await;
                                    continue;
                                }
                            }

                            if let Err(e) = std::fs::write(&template_path, "") {
                                tracing::error!("创建空模板文件失败: {}", e);
                                failed_count += 1;
                                errors.push(format!("第{}条路由（创建空模板文件）: {}", index + 1, e));
                                let _ = repo.delete(db_id).await;
                                continue;
                            }

                            tracing::warn!("导入 file 类型路由: 模板文件 {} 不存在，已创建空文件", template_path);
                        }

                        import_route.template_path = Some(template_path);
                    }

                    // 更新数据库记录以清除 inline_template（file 类型不需要）
                    if let Err(e) = repo.update(db_id, &import_route).await {
                        tracing::error!("更新数据库记录失败: {}", e);
                        failed_count += 1;
                        errors.push(format!("第{}条路由（更新数据库）: {}", index + 1, e));
                        let _ = repo.delete(db_id).await;
                        continue;
                    }
                }

                // 保存到存储后端
                let storage = manager.get_storage(&import_route.route_type);
                match storage.save_route(&import_route).await {
                    Ok(_) => {
                        tracing::info!("导入路由成功: id={}, type={}, path={}", db_id, import_route.route_type, import_route.path);
                    }
                    Err(e) => {
                        tracing::error!("导入路由失败（存储后端）: id={}, type={}, error={}", db_id, import_route.route_type, e);
                        // 删除数据库记录
                        let _ = repo.delete(db_id).await;
                        failed_count += 1;
                        errors.push(format!("第{}条路由（存储后端）: {}", index + 1, e));
                        continue;
                    }
                }
            } else {
                tracing::info!("导入路由成功（数据库）: id={}, path={}", db_id, import_route.path);
            }

            // 如果路由启用，热更新到路由表
            if import_route.enabled {
                if let Err(e) = state.dynamic_route_service().reload_route(db_id).await {
                    tracing::warn!("导入路由热更新失败: id={}, error={}", db_id, e);
                }
            }

            Ok(db_id)
        } else {
            // 兼容性：如果没有 RouteTypeManager，只使用数据库
            match repo.create(&import_route).await {
                Ok(id) => {
                    // 更新 import_route 的 ID
                    import_route.id = Some(id);

                    // 如果路由启用，热更新到路由表
                    if import_route.enabled {
                        if let Err(e) = state.dynamic_route_service().reload_route(id).await {
                            tracing::warn!("导入路由热更新失败: id={}, error={}", id, e);
                        }
                    }
                    Ok(id)
                }
                Err(e) => {
                    Err(e)
                }
            }
        };

        match import_result {
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