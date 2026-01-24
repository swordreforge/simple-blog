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

/// 文章详情页
pub async fn passage_detail(path: web::Path<String>) -> HttpResponse {
    let _id = path.into_inner();
    let context = create_passage_context();
    render_template("passage.html", &context).await
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
            use actix_web::HttpMessage;
            let mut req = actix_web::test::TestRequest::default().to_http_request();
            file.into_response(&mut req)
        },
        Err(_) => HttpResponse::NotFound().body("Keyboard test page not found"),
    }
}

/// 管理后台
pub async fn admin() -> HttpResponse {
    HttpResponse::Ok().body("Admin page - TODO")
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