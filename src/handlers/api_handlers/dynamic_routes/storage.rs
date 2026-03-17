//! 路由存储 API handlers
//!
//! 提供路由存储类型管理和迁移的 API 端点

use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use crate::app_state::AppState;
use crate::db::models::RouteType;

/// 获取存储统计信息
#[derive(Serialize)]
pub struct StorageStatsResponse {
    pub database: StorageTypeStats,
    pub memory: StorageTypeStats,
    pub file: StorageTypeStats,
}

/// 单个存储类型的统计信息
#[derive(Serialize)]
pub struct StorageTypeStats {
    pub total_routes: usize,
    pub enabled_routes: usize,
    pub disabled_routes: usize,
    pub memory_usage_bytes: usize,
}

/// 迁移路由请求
#[derive(Deserialize)]
pub struct MigrateRouteRequest {
    pub route_id: i64,
    pub target_type: RouteType,
}

/// 批量迁移路由请求
#[derive(Deserialize)]
pub struct BatchMigrateRoutesRequest {
    pub source_type: RouteType,
    pub target_type: RouteType,
}

/// 获取存储统计信息
pub async fn get_storage_stats(
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let route_type_manager = state.route_type_manager.as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("路由类型管理器未初始化"))?;
    
    match route_type_manager.get_storage_stats().await {
        Ok(stats) => {
            let response = StorageStatsResponse {
                database: StorageTypeStats {
                    total_routes: stats.database.total_routes,
                    enabled_routes: stats.database.enabled_routes,
                    disabled_routes: stats.database.disabled_routes,
                    memory_usage_bytes: stats.database.memory_usage_bytes,
                },
                memory: StorageTypeStats {
                    total_routes: stats.memory.total_routes,
                    enabled_routes: stats.memory.enabled_routes,
                    disabled_routes: stats.memory.disabled_routes,
                    memory_usage_bytes: stats.memory.memory_usage_bytes,
                },
                file: StorageTypeStats {
                    total_routes: stats.file.total_routes,
                    enabled_routes: stats.file.enabled_routes,
                    disabled_routes: stats.file.disabled_routes,
                    memory_usage_bytes: stats.file.memory_usage_bytes,
                },
            };
            
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            tracing::error!("获取存储统计失败: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "获取存储统计失败",
                "message": e.to_string()
            })))
        }
    }
}

/// 迁移单个路由
pub async fn migrate_route(
    state: web::Data<AppState>,
    req: web::Json<MigrateRouteRequest>,
) -> Result<HttpResponse> {
    let route_type_manager = state.route_type_manager.as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("路由类型管理器未初始化"))?;
    
    // 首先查找路由的当前存储类型
    let from_type = match route_type_manager.load_route(req.route_id, None).await {
        Ok(Some(route)) => route.route_type,
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "路由不存在",
                "route_id": req.route_id
            })));
        }
        Err(e) => {
            tracing::error!("查找路由失败: {}", e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "查找路由失败",
                "message": e.to_string()
            })));
        }
    };
    
    // 执行迁移
    match route_type_manager.migrate_route(req.route_id, from_type, req.target_type).await {
        Ok(_) => {
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": format!("路由 {} 从 {:?} 迁移到 {:?} 成功", req.route_id, from_type, req.target_type)
            })))
        }
        Err(e) => {
            tracing::error!("迁移路由失败: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "迁移路由失败",
                "message": e.to_string()
            })))
        }
    }
}

/// 批量迁移路由
pub async fn batch_migrate_routes(
    state: web::Data<AppState>,
    req: web::Json<BatchMigrateRoutesRequest>,
) -> Result<HttpResponse> {
    let route_type_manager = state.route_type_manager.as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("路由类型管理器未初始化"))?;
    
    match route_type_manager.migrate_all_routes(req.source_type, req.target_type).await {
        Ok(count) => {
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": format!("成功迁移 {} 条路由", count),
                "count": count
            })))
        }
        Err(e) => {
            tracing::error!("批量迁移路由失败: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "批量迁移路由失败",
                "message": e.to_string()
            })))
        }
    }
}

/// 清空指定存储类型的路由
pub async fn clear_storage(
    state: web::Data<AppState>,
    route_type: web::Path<RouteType>,
) -> Result<HttpResponse> {
    let route_type_manager = state.route_type_manager.as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("路由类型管理器未初始化"))?;
    
    match route_type_manager.clear_storage(*route_type).await {
        Ok(_) => {
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": format!("清空 {:?} 存储成功", route_type)
            })))
        }
        Err(e) => {
            tracing::error!("清空存储失败: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "清空存储失败",
                "message": e.to_string()
            })))
        }
    }
}

/// 获取路由的存储类型
pub async fn get_route_storage_type(
    state: web::Data<AppState>,
    route_id: web::Path<i64>,
) -> Result<HttpResponse> {
    let route_type_manager = state.route_type_manager.as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("路由类型管理器未初始化"))?;
    
    // 尝试从各个存储类型中查找路由
    let route_id = route_id.into_inner();
    
    // 先尝试从数据库加载
    if let Ok(Some(_)) = route_type_manager.load_route(route_id, Some(RouteType::Database)).await {
        return Ok(HttpResponse::Ok().json(serde_json::json!({
            "route_id": route_id,
            "storage_type": "database"
        })));
    }
    
    // 尝试从内存加载
    if let Ok(Some(_)) = route_type_manager.load_route(route_id, Some(RouteType::Memory)).await {
        return Ok(HttpResponse::Ok().json(serde_json::json!({
            "route_id": route_id,
            "storage_type": "memory"
        })));
    }
    
    // 尝试从文件加载
    if let Ok(Some(_)) = route_type_manager.load_route(route_id, Some(RouteType::File)).await {
        return Ok(HttpResponse::Ok().json(serde_json::json!({
            "route_id": route_id,
            "storage_type": "file"
        })));
    }
    
    // 都找不到
    Ok(HttpResponse::NotFound().json(serde_json::json!({
        "error": "路由不存在",
        "route_id": route_id
    })))
}