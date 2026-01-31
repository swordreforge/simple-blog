use actix_web::{web, HttpResponse};
use crate::db::repositories::Repository;

/// 获取数据库连接池状态
pub async fn get_pool_status() -> HttpResponse {
    match crate::db::get_pool_status() {
        Ok(status) => {
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "data": status
            }))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": format!("获取连接池状态失败: {}", e)
            }))
        }
    }
}

/// 数据库健康检查
pub async fn health_check(repo: web::Data<std::sync::Arc<dyn Repository>>) -> HttpResponse {
    // 获取连接池状态
    let pool_status = match crate::db::get_pool_status() {
        Ok(status) => status,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": format!("获取连接池状态失败: {}", e)
            }));
        }
    };

    // 检查数据库连接
    let connection_ok = repo.get_pool().get().is_ok();

    // 获取数据库文件大小
    let db_size = std::fs::metadata("data/blog.db")
        .ok()
        .map(|m| m.len());

    // 检查 WAL 模式是否启用
    let wal_enabled = if let Ok(conn) = repo.get_pool().get() {
        conn.query_row("PRAGMA journal_mode;", [], |row| {
            let mode: String = row.get(0)?;
            Ok(mode.to_uppercase() == "WAL")
        }).unwrap_or(false)
    } else {
        false
    };

    let status = if connection_ok {
        "healthy".to_string()
    } else {
        "unhealthy".to_string()
    };

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": {
            "status": status,
            "pool_status": pool_status,
            "database_size": db_size,
            "wal_enabled": wal_enabled
        }
    }))
}