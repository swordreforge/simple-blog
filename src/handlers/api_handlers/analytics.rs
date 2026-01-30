use actix_web::{web, HttpResponse};
use serde::Serialize;
use crate::db::repositories::{ArticleViewRepository, PassageRepository, Repository};
use std::sync::Arc;

/// 热门文章响应
#[derive(Debug, Serialize)]
pub struct PopularArticle {
    pub id: i64,
    pub title: String,
    pub view_count: i64,
}

/// 阅读来源响应
#[derive(Debug, Serialize)]
pub struct ViewSource {
    pub country: String,
    pub count: i64,
}

/// 阅读趋势响应
#[derive(Debug, Serialize)]
pub struct ViewTrend {
    pub date: String,
    pub count: i64,
}

/// 文章统计响应
#[derive(Debug, Serialize)]
pub struct ArticleStats {
    pub article_id: i64,
    pub title: String,
    pub total_views: i64,
    pub unique_visitors: i64,
    pub avg_duration: f64,
}

/// 城市统计响应
#[derive(Debug, Serialize)]
pub struct CityStats {
    pub city: String,
    pub country: String,
    pub count: i64,
}

/// IP 统计响应
#[derive(Debug, Serialize)]
pub struct IPStats {
    pub ip: String,
    pub country: String,
    pub city: String,
    pub count: i64,
}

/// 通用响应
#[derive(Debug, Serialize)]
pub struct AnalyticsResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

/// 获取最多阅读的文章
pub async fn most_viewed(
    query: web::Query<std::collections::HashMap<String, String>>,
    repo: web::Data<Arc<dyn Repository>>,
) -> HttpResponse {
    let limit: i64 = query.get("limit")
        .and_then(|l| l.parse().ok())
        .unwrap_or(10);
    
    let view_repo = ArticleViewRepository::new(repo.get_pool().clone());
    
    match view_repo.get_most_viewed_articles(limit).await {
        Ok(articles) => {
            let data: Vec<PopularArticle> = articles.into_iter().map(|a| PopularArticle {
                id: a.id.unwrap_or(0),
                title: a.title,
                view_count: a.view_count,
            }).collect();
            
            HttpResponse::Ok().json(AnalyticsResponse {
                success: true,
                data: Some(data),
                message: None,
            })
        }
        Err(_) => HttpResponse::InternalServerError().json(AnalyticsResponse::<()> {
            success: false,
            data: None,
            message: Some("获取热门文章失败".to_string()),
        })
    }
}

/// 获取阅读来源（按国家统计）
pub async fn view_sources(
    query: web::Query<std::collections::HashMap<String, String>>,
    repo: web::Data<Arc<dyn Repository>>,
) -> HttpResponse {
    let days: i64 = query.get("days")
        .and_then(|d| d.parse().ok())
        .unwrap_or(30);
    
    let view_repo = ArticleViewRepository::new(repo.get_pool().clone());
    
    match view_repo.get_view_sources(days).await {
        Ok(sources) => {
            let data: Vec<ViewSource> = sources.into_iter().map(|s| ViewSource {
                country: s.country,
                count: s.count,
            }).collect();
            
            HttpResponse::Ok().json(AnalyticsResponse {
                success: true,
                data: Some(data),
                message: None,
            })
        }
        Err(_) => HttpResponse::InternalServerError().json(AnalyticsResponse::<()> {
            success: false,
            data: None,
            message: Some("获取阅读来源失败".to_string()),
        })
    }
}

/// 获取阅读趋势
pub async fn view_trend(
    query: web::Query<std::collections::HashMap<String, String>>,
    repo: web::Data<Arc<dyn Repository>>,
) -> HttpResponse {
    let days: i64 = query.get("days")
        .and_then(|d| d.parse().ok())
        .unwrap_or(30);
    
    let view_repo = ArticleViewRepository::new(repo.get_pool().clone());
    
    match view_repo.get_view_trend(days).await {
        Ok(trend) => {
            let data: Vec<ViewTrend> = trend.into_iter().map(|t| ViewTrend {
                date: t.date,
                count: t.count,
            }).collect();
            
            HttpResponse::Ok().json(AnalyticsResponse {
                success: true,
                data: Some(data),
                message: None,
            })
        }
        Err(_) => HttpResponse::InternalServerError().json(AnalyticsResponse::<()> {
            success: false,
            data: None,
            message: Some("获取阅读趋势失败".to_string()),
        })
    }
}

/// 获取单篇文章的统计信息
pub async fn article_stats(
    query: web::Query<std::collections::HashMap<String, String>>,
    repo: web::Data<Arc<dyn Repository>>,
) -> HttpResponse {
    let id_str = query.get("id").cloned().unwrap_or_default();
    let id: i64 = match id_str.parse() {
        Ok(i) => i,
        Err(_) => {
            return HttpResponse::BadRequest().json(AnalyticsResponse::<()> {
                success: false,
                data: None,
                message: Some("无效的文章ID".to_string()),
            });
        }
    };
    
    if id <= 0 {
        return HttpResponse::BadRequest().json(AnalyticsResponse::<()> {
            success: false,
            data: None,
            message: Some("缺少文章ID参数".to_string()),
        });
    }
    
    let days: i64 = query.get("days")
        .and_then(|d| d.parse().ok())
        .unwrap_or(30);
    
    let view_repo = ArticleViewRepository::new(repo.get_pool().clone());
    
    // 先通过 ID 获取文章的 UUID
    let passage_repo = PassageRepository::new(repo.get_pool().clone());
    let passage = match passage_repo.get_by_id(id).await {
        Ok(p) => p,
        Err(_) => {
            return HttpResponse::NotFound().json(AnalyticsResponse::<()> {
                success: false,
                data: None,
                message: Some("文章不存在".to_string()),
            });
        }
    };
    
    let uuid = match &passage.uuid {
        Some(u) => u.as_str(),
        None => {
            return HttpResponse::InternalServerError().json(AnalyticsResponse::<()> {
                success: false,
                data: None,
                message: Some("文章 UUID 为空".to_string()),
            });
        }
    };
    
    match view_repo.get_article_stats(uuid, days).await {
        Ok(stats) => {
            let data = ArticleStats {
                article_id: stats.article_id,
                title: stats.title,
                total_views: stats.total_views,
                unique_visitors: stats.unique_visitors,
                avg_duration: stats.avg_duration,
            };
            
            HttpResponse::Ok().json(AnalyticsResponse {
                success: true,
                data: Some(data),
                message: None,
            })
        }
        Err(_) => HttpResponse::InternalServerError().json(AnalyticsResponse::<()> {
            success: false,
            data: None,
            message: Some("获取文章统计失败".to_string()),
        })
    }
}

/// 获取按城市统计的阅读数据
pub async fn view_by_city(
    query: web::Query<std::collections::HashMap<String, String>>,
    repo: web::Data<Arc<dyn Repository>>,
) -> HttpResponse {
    let days: i64 = query.get("days")
        .and_then(|d| d.parse().ok())
        .unwrap_or(30);
    
    let view_repo = ArticleViewRepository::new(repo.get_pool().clone());
    
    match view_repo.get_view_by_city(days).await {
        Ok(cities) => {
            let data: Vec<CityStats> = cities.into_iter().map(|c| CityStats {
                city: c.city,
                country: c.country,
                count: c.count,
            }).collect();
            
            HttpResponse::Ok().json(AnalyticsResponse {
                success: true,
                data: Some(data),
                message: None,
            })
        }
        Err(_) => HttpResponse::InternalServerError().json(AnalyticsResponse::<()> {
            success: false,
            data: None,
            message: Some("获取城市统计失败".to_string()),
        })
    }
}

/// 获取按IP统计的访问数据
pub async fn view_by_ip(
    query: web::Query<std::collections::HashMap<String, String>>,
    repo: web::Data<Arc<dyn Repository>>,
) -> HttpResponse {
    let days: i64 = query.get("days")
        .and_then(|d| d.parse().ok())
        .unwrap_or(30);
    
    let view_repo = ArticleViewRepository::new(repo.get_pool().clone());
    
    match view_repo.get_view_by_ip(days).await {
        Ok(ips) => {
            let data: Vec<IPStats> = ips.into_iter().map(|i| IPStats {
                ip: i.ip,
                country: i.country,
                city: i.city,
                count: i.count,
            }).collect();
            
            HttpResponse::Ok().json(AnalyticsResponse {
                success: true,
                data: Some(data),
                message: None,
            })
        }
        Err(_) => HttpResponse::InternalServerError().json(AnalyticsResponse::<()> {
            success: false,
            data: None,
            message: Some("获取IP统计失败".to_string()),
        })
    }
}