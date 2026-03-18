use actix_web::{web, HttpResponse};
use crate::app_state::AppState;
use crate::middleware::auth::check_admin_auth;
use crate::db::models::{CreateRouteRequest, HandlerType};
use crate::routes::conflicts_with_static_route;
use actix_web::web::Bytes;

/// 测试路由配置
pub async fn test_route(
    req: actix_web::HttpRequest,
    route_data: Bytes,
    state: web::Data<AppState>,
) -> HttpResponse {
    // 权限检查
    if check_admin_auth(&req).is_none() {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "success": false,
            "message": "需要管理员权限"
        }));
    }

    // 使用允许控制字符的serde_json配置来解析JSON
    let route: CreateRouteRequest = match serde_json::from_slice(&route_data) {
        Ok(data) => data,
        Err(e) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "message": format!("无效的请求数据: {}", e)
            }));
        }
    };

    // 验证路由配置
    if let Err(e) = validate_route_config(&route) {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "message": format!("配置验证失败: {}", e)
        }));
    }

    // 检查是否与预定义静态路由冲突
    if conflicts_with_static_route(&route.path) {
        return HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "路由测试完成",
            "data": {
                "match": true,
                "conflict": true,
                "conflict_type": "static_route",
                "conflict_reason": format!("路径 '{}' 与预定义静态路由冲突", route.path),
                "response_preview": None::<serde_json::Value>
            }
        }));
    }

    // 检查路径是否已存在于动态路由表中
    let repo = state.dynamic_route_repository();
    if let Ok(Some(existing)) = repo.get_by_path(&route.path).await {
        return HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "路由测试完成",
            "data": {
                "match": true,
                "conflict": true,
                "conflict_type": "dynamic_route",
                "existing_route": existing,
                "response_preview": None::<serde_json::Value>
            }
        }));
    }

    // 预览响应
    let response_preview = preview_response(&route);

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "路由测试成功",
        "data": {
            "match": true,
            "conflict": false,
            "response_preview": response_preview
        }
    }))
}

/// 验证路由配置
fn validate_route_config(route: &CreateRouteRequest) -> Result<(), String> {
    // 验证路径格式
    if !route.path.starts_with('/') {
        return Err("路径必须以 / 开头".to_string());
    }

    // 验证路径不包含空格
    if route.path.contains(' ') {
        return Err("路径不能包含空格".to_string());
    }

    // 验证处理器配置
    if route.handler_config.is_null() || !route.handler_config.is_object() {
        return Err("处理器配置必须是有效的JSON对象".to_string());
    }

    // 根据处理器类型验证必需字段
    match route.handler_type {
        HandlerType::Redirect => {
            if route.handler_config.get("target").is_none() {
                return Err("重定向处理器需要target字段".to_string());
            }
        }
        HandlerType::Static => {
            if route.handler_config.get("content").is_none() {
                return Err("静态内容处理器需要content字段".to_string());
            }
        }
        HandlerType::Template => {
            if route.handler_config.get("template_name").is_none() {
                return Err("模板处理器需要template_name字段".to_string());
            }
        }
        HandlerType::Proxy => {
            if route.handler_config.get("target").is_none() {
                return Err("代理处理器需要target字段".to_string());
            }
        }
        HandlerType::Custom => {
            if route.handler_config.get("script").is_none() && route.handler_config.get("source").is_none() {
                return Err("自定义处理器需要script或source字段".to_string());
            }
        }
    }

    Ok(())
}

/// 预览响应
fn preview_response(route: &CreateRouteRequest) -> serde_json::Value {
    match route.handler_type {
        HandlerType::Redirect => {
            let target = route.handler_config.get("target")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let status_code = route.handler_config.get("status_code")
                .and_then(|v| v.as_i64())
                .unwrap_or(302);

            serde_json::json!({
                "status_code": status_code,
                "headers": {
                    "Location": target
                },
                "body": ""
            })
        }
        HandlerType::Static => {
            let content = route.handler_config.get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let content_type = route.handler_config.get("content_type")
                .and_then(|v| v.as_str())
                .unwrap_or("text/plain; charset=utf-8");

            serde_json::json!({
                "status_code": 200,
                "headers": {
                    "Content-Type": content_type
                },
                "body": content
            })
        }
        HandlerType::Template => {
            let template_name = route.handler_config.get("template_name")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let default_context: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
            let context = route.handler_config.get("context")
                .and_then(|v| v.as_object())
                .unwrap_or(&default_context);

            serde_json::json!({
                "status_code": 200,
                "headers": {
                    "Content-Type": "text/html; charset=utf-8"
                },
                "template": template_name,
                "context": context
            })
        }
        HandlerType::Proxy => {
            let target = route.handler_config.get("target")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let timeout = route.handler_config.get("timeout")
                .and_then(|v| v.as_i64())
                .unwrap_or(5000);

            serde_json::json!({
                "status_code": 200,
                "headers": {},
                "proxy_target": target,
                "timeout_ms": timeout
            })
        }
        HandlerType::Custom => {
            serde_json::json!({
                "status_code": 200,
                "headers": {},
                "custom_handler": true
            })
        }
    }
}