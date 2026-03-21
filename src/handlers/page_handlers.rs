use crate::templates::{
    create_about_context, create_collect_context, create_friends_context, create_index_context,
    create_markdown_editor_context, create_passage_context, render_template,
};
use actix_files::NamedFile;
use actix_web::{HttpRequest, HttpResponse, web};

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
            let req = actix_web::test::TestRequest::default().to_http_request();
            file.into_response(&req)
        }
        Err(_) => HttpResponse::NotFound().body("Keyboard test page not found"),
    }
}

/// 管理后台
pub async fn admin(req: HttpRequest) -> HttpResponse {
    // 从 cookie 中获取 token
    let token = req.cookie("auth_token").map(|c| c.value().to_string());

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
            let template_settings =
                crate::templates::appearance_to_template_settings(&appearance_settings);
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

/// 动态路由管理页面
pub async fn dyn_routing(req: HttpRequest) -> HttpResponse {
    // 从 cookie 中获取 token
    let token = req.cookie("auth_token").map(|c| c.value().to_string());

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

    let mut context = tera::Context::new();
    context.insert("title", "动态路由管理");
    context.insert("page", "dyn-routing");

    render_template("admin/dyn-routing.html", &context).await
}
/// 状态页面
pub async fn status_page(path: web::Path<u16>) -> HttpResponse {
    render_status_page(path.into_inner()).await
}

/// 通用的状态码页面渲染函数
/// 可以在代码中任何需要返回状态码页面的地方调用
pub async fn render_status_page(status: u16) -> HttpResponse {
    // 根据状态码选择对应的模板文件
    let template_name = format!("status/{}.html", status);

    // 创建上下文
    let mut context = tera::Context::new();
    let status_text = match status {
        302 => "Found",
        401 => "Unauthorized",
        404 => "Not Found",
        405 => "Method Not Allowed",
        409 => "Conflict",
        423 => "Locked",
        500 => "Internal Server Error",
        999 => "Unknown Error",
        _ => "Unknown Status",
    };

    context.insert("status_code", &status);
    context.insert("status_text", &status_text);

    // 使用新的渲染函数，支持自定义状态码
    let http_status = actix_web::http::StatusCode::from_u16(status)
        .unwrap_or(actix_web::http::StatusCode::NOT_FOUND);

    crate::templates::render_template_with_status(&template_name, &context, http_status).await
}

/// 处理默认状态码（用于默认服务，如 404）
pub async fn handle_default_status(req: HttpRequest) -> HttpResponse {
    // 从请求中获取方法，如果是 GET 请求返回 404，其他方法返回 405
    let status_code = if req.method() == actix_web::http::Method::GET {
        404
    } else {
        405
    };

    let template_name = format!("status/{}.html", status_code);
    let mut context = tera::Context::new();
    let status_text = match status_code {
        404 => "Not Found",
        405 => "Method Not Allowed",
        _ => "Unknown Status",
    };

    context.insert("status_code", &status_code);
    context.insert("status_text", &status_text);

    let http_status = actix_web::http::StatusCode::from_u16(status_code)
        .unwrap_or(actix_web::http::StatusCode::NOT_FOUND);

    crate::templates::render_template_with_status(&template_name, &context, http_status).await
}

/// 健康检查
pub async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "rustblog"
    }))
}
