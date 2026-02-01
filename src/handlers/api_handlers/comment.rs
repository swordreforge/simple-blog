use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use crate::db::repositories::{CommentRepository, Repository};
use std::sync::Arc;

/// 将 Markdown 转换为 HTML
fn convert_markdown_to_html(markdown: &str) -> String {
    use pulldown_cmark::{Parser, html, Options};

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    html_output
}

/// 评论列表请求参数
#[derive(Debug, Deserialize)]
pub struct CommentListQuery {
    pub passage_uuid: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

/// 创建评论请求
#[derive(Debug, Deserialize)]
pub struct CreateCommentRequest {
    pub username: String,
    pub content: String,
    pub passage_uuid: String,
}

/// 评论响应
#[derive(Debug, Serialize)]
pub struct CommentResponse {
    pub id: i64,
    pub username: String,
    pub content: String,
    pub passage_uuid: String,
    pub created_at: String,
}

/// 通用响应
#[derive(Debug, Serialize)]
pub struct CommonResponse {
    pub success: bool,
    pub message: String,
}

/// 获取评论列表
pub async fn list(
    query: web::Query<CommentListQuery>,
    repo: web::Data<Arc<dyn Repository>>,
) -> HttpResponse {
    let comment_repo = CommentRepository::new(repo.get_pool().clone());
    
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(10);
    let offset = (page - 1) * limit;
    
    let comments = if let Some(ref passage_uuid) = query.passage_uuid {
        comment_repo.get_by_passage_uuid(passage_uuid, limit as i64, offset as i64).await
    } else {
        comment_repo.get_all(limit as i64, offset as i64).await
    };
    
    let total = if let Some(ref passage_uuid) = query.passage_uuid {
        comment_repo.count_by_passage_uuid(passage_uuid).await
    } else {
        comment_repo.count().await
    };
    
    match (comments, total) {
        (Ok(comments), Ok(total)) => {
            let data: Vec<CommentResponse> = comments.into_iter().map(|c| CommentResponse {
                id: c.id.unwrap_or(0),
                username: c.username,
                content: c.content,
                passage_uuid: c.passage_uuid,
                created_at: c.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            }).collect();
            
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "data": data,
                "pagination": {
                    "page": page,
                    "limit": limit,
                    "total": total,
                }
            }))
        }
        _ => HttpResponse::InternalServerError().json(CommonResponse {
            success: false,
            message: "获取评论列表失败".to_string(),
        })
    }
}

/// 创建评论
pub async fn create(
    req: web::Json<CreateCommentRequest>,
    repo: web::Data<Arc<dyn Repository>>,
) -> HttpResponse {
    // 验证必填字段
    if req.username.is_empty() || req.content.is_empty() || req.passage_uuid.is_empty() {
        return HttpResponse::BadRequest().json(CommonResponse {
            success: false,
            message: "用户名、内容和文章UUID不能为空".to_string(),
        });
    }
    
    let comment_repo = CommentRepository::new(repo.get_pool().clone());

    // 将 Markdown 转换为 HTML
    let html_content = convert_markdown_to_html(&req.content);

    let comment = crate::db::models::Comment {
        id: None,
        username: req.username.clone(),
        content: html_content,
        passage_uuid: req.passage_uuid.clone(),
        created_at: chrono::Utc::now(),
    };

    match comment_repo.create(&comment).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "评论创建成功",
            "data": CommentResponse {
                id: comment.id.unwrap_or(0),
                username: comment.username,
                content: comment.content,
                passage_uuid: comment.passage_uuid,
                created_at: comment.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            }
        })),
        Err(_) => HttpResponse::InternalServerError().json(CommonResponse {
            success: false,
            message: "创建评论失败".to_string(),
        })
    }
}

/// 删除评论
pub async fn delete(
    path: web::Path<i64>,
    repo: web::Data<Arc<dyn Repository>>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    // 鉴权检查
    if req.cookie("auth_token").is_none() {
        return crate::middleware::auth::missing_token_response();
    }
    if crate::middleware::auth::check_admin_auth(&req).is_none() {
        return crate::middleware::auth::forbidden_response();
    }

    let id = path.into_inner();
    
    if id <= 0 {
        return HttpResponse::BadRequest().json(CommonResponse {
            success: false,
            message: "无效的评论ID".to_string(),
        });
    }
    
    let comment_repo = CommentRepository::new(repo.get_pool().clone());
    
    match comment_repo.delete(id).await {
        Ok(_) => HttpResponse::Ok().json(CommonResponse {
            success: true,
            message: "评论删除成功".to_string(),
        }),
        Err(_) => HttpResponse::InternalServerError().json(CommonResponse {
            success: false,
            message: "删除评论失败".to_string(),
        })
    }
}

/// 批量删除评论请求
#[derive(Debug, Deserialize)]
pub struct BatchDeleteRequest {
    pub ids: Vec<i64>,
}

/// 批量删除评论
pub async fn delete_batch(
    req_json: web::Json<BatchDeleteRequest>,
    repo: web::Data<Arc<dyn Repository>>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    // 鉴权检查
    if req.cookie("auth_token").is_none() {
        return crate::middleware::auth::missing_token_response();
    }
    if crate::middleware::auth::check_admin_auth(&req).is_none() {
        return crate::middleware::auth::forbidden_response();
    }

    if req_json.ids.is_empty() {
        return HttpResponse::BadRequest().json(CommonResponse {
            success: false,
            message: "评论ID列表不能为空".to_string(),
        });
    }
    
    let comment_repo = CommentRepository::new(repo.get_pool().clone());

    match comment_repo.delete_batch(req_json.ids.clone()).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": format!("成功删除 {} 条评论", count),
            "deleted_count": count
        })),
        Err(_) => HttpResponse::InternalServerError().json(CommonResponse {
            success: false,
            message: "批量删除评论失败".to_string(),
        })
    }
}