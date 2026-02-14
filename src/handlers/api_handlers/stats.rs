use actix_web::{web, HttpResponse};
use serde::Serialize;

/// 统计数据响应
#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub passages: i64,
    pub comments: i64,
    pub users: i64,
    pub views: i64,
}

/// 获取统计数据
pub async fn get_stats(state: web::Data<crate::app_state::AppState>, req: actix_web::HttpRequest) -> HttpResponse {
    // 鉴权检查
    if req.cookie("auth_token").is_none() {
        return crate::middleware::auth::missing_token_response();
    }
    if crate::middleware::auth::check_admin_auth(&req).is_none() {
        return crate::middleware::auth::forbidden_response();
    }

    let passage_repo = state.passage_repository();
    let comment_repo = state.comment_repository();
    let user_repo = state.user_repository();
    
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