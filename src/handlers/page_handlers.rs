use actix_web::{web, HttpResponse};
use actix_files::NamedFile;
use crate::templates::{
    render_template,
    create_index_context,
    create_passage_context,
    create_collect_context,
    create_about_context,
    create_markdown_editor_context,
};
use crate::db::repositories::{PassageRepository, Repository};
use std::sync::Arc;
use crate::db::models::Passage;
use tera::{Context as TeraContext};

/// 主页处理器
pub async fn index() -> HttpResponse {
    let context = create_index_context();
    render_template("index.html", &context).await
}

/// 文章列表页
pub async fn passage_list() -> HttpResponse {
    let context = create_passage_context();
    render_template("passage.html", &context).await
}

/// 创建包含文章数据的上下文
fn create_passage_context_with_article(passage: &Passage) -> TeraContext {
    let mut context = create_passage_context();
    
    // 添加文章数据到上下文
    context.insert("passage_id", &passage.id.unwrap_or(0).to_string());
    context.insert("passage_title", &passage.title);
    context.insert("passage_content", &passage.content);
    context.insert("passage_date", &passage.created_at.format("%Y-%m-%d").to_string());
    
    context
}

/// 文章详情页（通过 ID）
pub async fn passage_detail(path: web::Path<String>) -> HttpResponse {
    let _id = path.into_inner();
    let context = create_passage_context();
    render_template("passage.html", &context).await
}

/// 文章详情页（通过日期路径：/passage/{year}/{month}/{day}/{title}）
pub async fn passage_detail_by_date(
    path: web::Path<(String, String, String, String)>,
    repo: web::Data<Arc<dyn crate::db::repositories::Repository>>,
) -> HttpResponse {
    let (_year, _month, _day, title) = path.into_inner();

    // 使用 urlencoding 正确解码标题
    let decoded_title = match urlencoding::decode(&title) {
        Ok(decoded) => decoded.to_string(),
        Err(_) => title.clone(),
    };

    // 从数据库获取所有文章
    let passage_repo = crate::db::repositories::PassageRepository::new(repo.get_pool().clone());
    
    match passage_repo.get_all(1000, 0).await {
        Ok(passages) => {
            // 查找匹配标题的文章
            let passage = passages.into_iter().find(|p| {
                p.title == decoded_title
            });
            
            if let Some(passage) = passage {
                // 渲染文章内容到模板
                let context = create_passage_context_with_article(&passage);
                render_template("passage.html", &context).await
            } else {
                // 文章未找到，渲染空页面
                let context = create_passage_context();
                render_template("passage.html", &context).await
            }
        }
        Err(_) => {
            let context = create_passage_context();
            render_template("passage.html", &context).await
        }
    }
}

/// 归档页面
pub async fn collect() -> HttpResponse {
    let context = create_collect_context();
    render_template("collect.html", &context).await
}

/// 关于页面
pub async fn about() -> HttpResponse {
    let context = create_about_context();
    render_template("about.html", &context).await
}

/// Markdown 编辑器
pub async fn markdown_editor() -> HttpResponse {
    let context = create_markdown_editor_context();
    render_template("markdown-editor.html", &context).await
}

/// 键盘测试页面
pub async fn keyboard_test() -> HttpResponse {
    match NamedFile::open_async("templates/keyboard-test.html").await {
        Ok(file) => {
            let mut req = actix_web::test::TestRequest::default().to_http_request();
            file.into_response(&mut req)
        },
        Err(_) => HttpResponse::NotFound().body("Keyboard test page not found"),
    }
}

/// 管理后台
pub async fn admin() -> HttpResponse {
    let context = crate::templates::create_admin_context();
    render_template("admin/admin.html", &context).await
}

/// 状态页面
pub async fn status_page(path: web::Path<u16>) -> HttpResponse {
    let status = path.into_inner();
    let status_text = match status {
        404 => "Not Found",
        500 => "Internal Server Error",
        403 => "Forbidden",
        _ => "Unknown Status",
    };
    
    HttpResponse::build(actix_web::http::StatusCode::from_u16(status).unwrap_or(actix_web::http::StatusCode::NOT_FOUND))
        .body(format!("{}: {}", status, status_text))
}

/// 健康检查
pub async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "rustblog"
    }))
}