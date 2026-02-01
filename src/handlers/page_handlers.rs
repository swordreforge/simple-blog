use actix_web::{web, HttpResponse, HttpRequest};
use actix_files::NamedFile;
use crate::templates::{
    render_template,
    create_index_context,
    create_passage_context,
    create_collect_context,
    create_about_context,
    create_friends_context,
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

/// 文章详情页（通过 ID）
pub async fn passage_detail(path: web::Path<String>) -> HttpResponse {
    let _id = path.into_inner();
    let context = create_passage_context();
    render_template("passage.html", &context).await
}

/// 文章详情页（通过日期路径：/passage/{year}/{month}/{day}/{title}）
pub async fn collect() -> HttpResponse {
    let context = create_collect_context();
    render_template("collect.html", &context).await
}

/// 关于页面
pub async fn about() -> HttpResponse {
    let context = create_about_context();
    render_template("about.html", &context).await
}

/// 友链页面
pub async fn friends() -> HttpResponse {
    let context = create_friends_context();
    render_template("friends.html", &context).await
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
pub async fn admin(req: HttpRequest) -> HttpResponse {
    // 从 cookie 中获取 token
    let token = req.cookie("auth_token")
        .map(|c| c.value().to_string());

    if let Some(token_str) = token {
        // 验证 token
        match crate::jwt::validate_token(&token_str) {
            Ok(claims) => {
                // 检查是否为管理员
                if claims.role != "admin" {
                    // 非管理员，重定向到首页
                    return HttpResponse::Found()
                        .insert_header(("Location", "/"))
                        .finish();
                }
            }
            Err(_) => {
                // token 无效，重定向到首页
                return HttpResponse::Found()
                    .insert_header(("Location", "/"))
                    .finish();
            }
        }
    } else {
        // 没有 token，重定向到首页
        return HttpResponse::Found()
            .insert_header(("Location", "/"))
            .finish();
    }

    let mut context = crate::templates::create_admin_context();

    // 尝试从数据库加载外观设置
    match crate::templates::load_appearance_settings() {
        Ok(appearance_settings) => {
            // 将外观设置转换为模板设置
            let template_settings = crate::templates::appearance_to_template_settings(&appearance_settings);
            context.insert("settings", &template_settings);
        }
        Err(e) => {
            eprintln!("Failed to load appearance settings for admin page: {}", e);
            // 使用默认设置
            context.insert("settings", &crate::templates::TemplateSettings::default());
        }
    }

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