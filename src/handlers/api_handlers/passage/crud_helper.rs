//! 文章缓存辅助函数
//! 用于缓存击穿防护的数据库查询函数

use crate::db::repositories::PassageRepository;
use crate::cache::CacheError;
use crate::utils::format_datetime_optimized;

/// 从数据库获取文章列表（用于缓存加载函数）
#[allow(clippy::too_many_arguments)]
pub async fn fetch_passage_list_from_db(
    passage_repo: std::sync::Arc<PassageRepository>,
    use_cursor: bool,
    cursor: Option<String>,
    year: Option<i32>,
    month: Option<i32>,
    day: Option<i32>,
    limit: i64,
    page: i64,
    offset: i64,
) -> Result<Option<String>, CacheError> {
    use serde_json::json;

    if use_cursor {
        // 游标分页
        match passage_repo.get_published_cursor(cursor, limit).await {
            Ok((passages, next_cursor)) => {
                let data: Vec<crate::handlers::api_handlers::passage::crud::PassageResponse> = passages
                    .into_iter()
                    .map(|p| crate::handlers::api_handlers::passage::crud::PassageResponse {
                        id: p.id.unwrap_or(0),
                        uuid: p.uuid.unwrap_or_default(),
                        title: p.title,
                        content: p.original_content.unwrap_or_default(),
                        html_content: None,
                        summary: p.summary,
                        author: p.author,
                        tags: p.tags,
                        category: p.category,
                        status: p.status,
                        file_path: p.file_path,
                        visibility: p.visibility,
                        is_scheduled: p.is_scheduled,
                        published_at: p.published_at
                            .map(|d: chrono::DateTime<chrono::Utc>| format_datetime_optimized(&d)),
                        cover_image: p.cover_image,
                        created_at: format_datetime_optimized(&p.created_at),
                        updated_at: format_datetime_optimized(&p.updated_at),
                    })
                    .collect();

                let response = json!({
                    "success": true,
                    "data": data,
                    "pagination": {
                        "has_more": next_cursor.is_some() && data.len() >= limit as usize,
                        "next_cursor": next_cursor,
                        "limit": limit
                    }
                });

                serde_json::to_string(&response).map(Some).map_err(|e| CacheError::Unknown(e.to_string()))
            }
            Err(e) => Err(CacheError::ConnectionError(e.to_string())),
        }
    } else {
        // 传统分页
        let result = if year.is_some() || month.is_some() || day.is_some() {
            passage_repo.get_published_by_date(year, month, day, limit, offset).await
        } else {
            passage_repo.get_published(limit, offset).await
        };

        match result {
            Ok(passages) => {
                // 获取总数
                let total: i64 = if year.is_some() || month.is_some() || day.is_some() {
                    match passage_repo.count_published_by_date(year, month, day).await {
                        Ok(c) => c,
                        Err(_) => passages.len() as i64,
                    }
                } else {
                    match passage_repo.count_published().await {
                        Ok(c) => c,
                        Err(_) => passages.len() as i64,
                    }
                };

                // 如果当前页超出了实际页数，返回空数据
                let total_pages = (total + limit - 1) / limit;
                if page > total_pages && total_pages > 0 {
                    let response = json!({
                        "success": true,
                        "data": [],
                        "pagination": {
                            "page": page,
                            "limit": limit,
                            "total": total,
                            "total_pages": total_pages,
                            "has_more": false
                        }
                    });
                    return serde_json::to_string(&response).map(Some).map_err(|e| CacheError::Unknown(e.to_string()));
                }

                // 计算下一页游标
                let next_cursor = passages.last().map(|p| {
                    format!(
                        "{}|{}",
                        p.created_at.format("%Y-%m-%d %H:%M:%S%:z"),
                        p.id.unwrap_or(0)
                    )
                });

                let data: Vec<crate::handlers::api_handlers::passage::crud::PassageResponse> = passages
                    .into_iter()
                    .map(|p| crate::handlers::api_handlers::passage::crud::PassageResponse {
                        id: p.id.unwrap_or(0),
                        uuid: p.uuid.unwrap_or_default(),
                        title: p.title,
                        content: p.original_content.unwrap_or_default(),
                        html_content: None,
                        summary: p.summary,
                        author: p.author,
                        tags: p.tags,
                        category: p.category,
                        status: p.status,
                        file_path: p.file_path,
                        visibility: p.visibility,
                        is_scheduled: p.is_scheduled,
                        published_at: p.published_at
                            .map(|d| format_datetime_optimized(&d)),
                        cover_image: p.cover_image,
                        created_at: format_datetime_optimized(&p.created_at),
                        updated_at: format_datetime_optimized(&p.updated_at),
                    })
                    .collect();

                let response = json!({
                    "success": true,
                    "data": data,
                    "pagination": {
                        "page": page,
                        "limit": limit,
                        "total": total,
                        "total_pages": total_pages,
                        "has_more": page < total_pages,
                        "next_cursor": next_cursor
                    }
                });

                serde_json::to_string(&response).map(Some).map_err(|e| CacheError::Unknown(e.to_string()))
            }
            Err(e) => Err(CacheError::ConnectionError(e.to_string())),
        }
    }
}