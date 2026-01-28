use actix_web::{web, HttpResponse};
use serde::Serialize;
use crate::db::repositories::{PassageRepository, CommentRepository, UserRepository, Repository};
use std::sync::Arc;

/// 统计数据响应
#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub passages: i64,
    pub comments: i64,
    pub users: i64,
    pub views: i64,
}

/// 获取统计数据
pub async fn get_stats(repo: web::Data<Arc<dyn Repository>>) -> HttpResponse {
    let passage_repo = PassageRepository::new(repo.get_pool().clone());
    let comment_repo = CommentRepository::new(repo.get_pool().clone());
    let user_repo = UserRepository::new(repo.get_pool().clone());
    
    // 获取各项统计
    let passages = passage_repo.count().await.unwrap_or(0);
    let comments = comment_repo.count().await.unwrap_or(0);
    let users = user_repo.count().await.unwrap_or(0);
    
    // 视图数（暂时返回0，需要从ArticleView表中查询）
    let views = 0;
    
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": StatsResponse {
            passages,
            comments,
            users,
            views,
        }
    }))
}