use actix_web::{HttpResponse, web};
use serde::Serialize;

/// 归档响应
#[derive(Debug, Serialize)]
pub struct ArchiveResponse {
    pub year: String,
    pub month: String,
    pub count: i32,
}

/// 获取文章归档列表
pub async fn list(state: web::Data<crate::app_state::AppState>) -> HttpResponse {
    let passage_repo = state.passage_repository();

    // 使用聚合查询获取归档统计（优化：避免加载所有文章）
    match passage_repo.get_archive_stats().await {
        Ok(stats) => {
            // 转换为响应格式
            let data: Vec<ArchiveResponse> = stats
                .into_iter()
                .map(|stat| ArchiveResponse {
                    year: stat.year,
                    month: stat.month,
                    count: stat.count,
                })
                .collect();

            HttpResponse::Ok()
                .insert_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
                .insert_header(("Pragma", "no-cache"))
                .insert_header(("Expires", "0"))
                .json(serde_json::json!({
                    "success": true,
                    "data": data
                }))
        }
        Err(e) => {
            eprintln!("获取归档统计失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "获取归档失败"
            }))
        }
    }
}
