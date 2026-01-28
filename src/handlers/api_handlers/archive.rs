use actix_web::{web, HttpResponse};
use serde::Serialize;
use crate::db::repositories::{PassageRepository, Repository};
use std::sync::Arc;
use std::collections::HashMap;

/// 归档响应
#[derive(Debug, Serialize)]
pub struct ArchiveResponse {
    pub year: String,
    pub month: String,
    pub count: i32,
}

/// 获取文章归档（按年月分组）
pub async fn list(repo: web::Data<Arc<dyn Repository>>) -> HttpResponse {
    let passage_repo = PassageRepository::new(repo.get_pool().clone());
    
    // 获取所有文章
    let passages = match passage_repo.get_all(1000, 0).await {
        Ok(p) => p,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "获取归档失败"
            }));
        }
    };
    
    // 按年月分组
    let mut archive_map: HashMap<String, i32> = HashMap::new();
    
    for passage in passages {
        let year = passage.created_at.format("%Y").to_string();
        let month = passage.created_at.format("%m").to_string();
        let key = format!("{}-{}", year, month);
        *archive_map.entry(key).or_insert(0) += 1;
    }
    
    // 转换为响应格式
    let mut data: Vec<ArchiveResponse> = archive_map.into_iter()
        .map(|(key, count)| {
            let parts: Vec<&str> = key.split('-').collect();
            ArchiveResponse {
                year: parts[0].to_string(),
                month: parts[1].to_string(),
                count,
            }
        })
        .collect();
    
    // 按年份和月份降序排序
    data.sort_by(|a, b| {
        let year_cmp = b.year.cmp(&a.year);
        if year_cmp != std::cmp::Ordering::Equal {
            return year_cmp;
        }
        b.month.cmp(&a.month)
    });
    
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": data
    }))
}