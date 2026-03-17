use actix_web::{web, HttpResponse};
use crate::app_state::AppState;
use crate::middleware::auth::check_admin_auth;
use serde::Deserialize;

/// 批量操作请求
#[derive(Debug, Deserialize)]
#[serde(tag = "action")]
pub enum BatchRequest {
    #[serde(rename = "enable")]
    Enable { ids: Vec<i64> },
    #[serde(rename = "disable")]
    Disable { ids: Vec<i64> },
    #[serde(rename = "delete")]
    Delete { ids: Vec<i64> },
}

/// 批量操作
pub async fn batch_operations(
    req: actix_web::HttpRequest,
    batch_req: web::Json<BatchRequest>,
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

    match batch_req.into_inner() {
        BatchRequest::Enable { ids } => {
            batch_enable(&repo, ids, username).await
        }
        BatchRequest::Disable { ids } => {
            batch_disable(&repo, ids, username).await
        }
        BatchRequest::Delete { ids } => {
            batch_delete(&repo, ids, username).await
        }
    }
}

/// 批量启用
async fn batch_enable(
    repo: &crate::db::repositories::DynamicRouteRepository,
    ids: Vec<i64>,
    username: &str,
) -> HttpResponse {
    let mut success_count = 0;
    let mut failed_ids = Vec::new();

    for id in ids {
        match repo.get_by_id(id).await {
            Ok(Some(mut route)) => {
                if !route.enabled {
                    route.enabled = true;
                    route.updated_at = chrono::Utc::now();
                    match repo.update(id, &route).await {
                        Ok(_) => {
                            success_count += 1;
                            log_batch_operation(repo, id, "batch_enable", Some(&route), username);
                        }
                        Err(_) => {
                            failed_ids.push(id);
                        }
                    }
                } else {
                    success_count += 1;
                }
            }
            Ok(None) => {
                failed_ids.push(id);
            }
            Err(_) => {
                failed_ids.push(id);
            }
        }
    }

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": format!("批量启用完成: 成功 {}, 失败 {}", success_count, failed_ids.len()),
        "data": {
            "success_count": success_count,
            "failed_count": failed_ids.len(),
            "failed_ids": failed_ids
        }
    }))
}

/// 批量禁用
async fn batch_disable(
    repo: &crate::db::repositories::DynamicRouteRepository,
    ids: Vec<i64>,
    username: &str,
) -> HttpResponse {
    let mut success_count = 0;
    let mut failed_ids = Vec::new();

    for id in ids {
        match repo.get_by_id(id).await {
            Ok(Some(mut route)) => {
                if route.enabled {
                    route.enabled = false;
                    route.updated_at = chrono::Utc::now();
                    match repo.update(id, &route).await {
                        Ok(_) => {
                            success_count += 1;
                            log_batch_operation(repo, id, "batch_disable", Some(&route), username);
                        }
                        Err(_) => {
                            failed_ids.push(id);
                        }
                    }
                } else {
                    success_count += 1;
                }
            }
            Ok(None) => {
                failed_ids.push(id);
            }
            Err(_) => {
                failed_ids.push(id);
            }
        }
    }

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": format!("批量禁用完成: 成功 {}, 失败 {}", success_count, failed_ids.len()),
        "data": {
            "success_count": success_count,
            "failed_count": failed_ids.len(),
            "failed_ids": failed_ids
        }
    }))
}

/// 批量删除
async fn batch_delete(
    repo: &crate::db::repositories::DynamicRouteRepository,
    ids: Vec<i64>,
    username: &str,
) -> HttpResponse {
    let mut success_count = 0;
    let mut failed_ids = Vec::new();

    for id in ids {
        match repo.get_by_id(id).await {
            Ok(Some(route)) => {
                match repo.delete(id).await {
                    Ok(_) => {
                        success_count += 1;
                        log_batch_operation(repo, id, "batch_delete", Some(&route), username);
                    }
                    Err(_) => {
                        failed_ids.push(id);
                    }
                }
            }
            Ok(None) => {
                failed_ids.push(id);
            }
            Err(_) => {
                failed_ids.push(id);
            }
        }
    }

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": format!("批量删除完成: 成功 {}, 失败 {}", success_count, failed_ids.len()),
        "data": {
            "success_count": success_count,
            "failed_count": failed_ids.len(),
            "failed_ids": failed_ids
        }
    }))
}

/// 记录批量操作日志
fn log_batch_operation(
    repo: &crate::db::repositories::DynamicRouteRepository,
    route_id: i64,
    action: &str,
    route: Option<&crate::db::models::DynamicRoute>,
    username: &str,
) {
    use serde_json::to_string;

    let old_config_str = route.and_then(|r| to_string(r).ok());
    let new_config_str = route.and_then(|r| to_string(r).ok());

    // 记录日志（忽略错误）
    let _ = repo.log_operation(
        Some(route_id),
        action,
        old_config_str,
        new_config_str,
        Some(username.to_string()),
        None,
        None,
    );
}