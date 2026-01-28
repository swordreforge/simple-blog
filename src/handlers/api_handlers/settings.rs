use actix_web::{web, HttpResponse, Responder};

/// 获取所有设置
pub async fn get() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "settings": {
            "title": "RustBlog",
            "name": "Dango",
            "greeting": "Welcome to RustBlog",
            "background_image": "",
            "global_opacity": 0.9,
            "blur_amount": 20,
            "saturate_amount": 180
        }
    }))
}

/// 获取所有设置（完整版）
pub async fn get_all() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "settings": {
            "title": "RustBlog",
            "name": "Dango",
            "greeting": "Welcome to RustBlog",
            "background_image": "",
            "mobile_background_image": "",
            "global_opacity": "0.9",
            "background_size": "cover",
            "background_position": "center",
            "background_repeat": "no-repeat",
            "background_attachment": "fixed",
            "blur_amount": "20px",
            "saturate_amount": "180%",
            "dark_mode_enabled": false,
            "navbar_glass_color": "rgba(255, 255, 255, 0.85)",
            "navbar_text_color": "#333333",
            "card_glass_color": "rgba(255, 255, 255, 0.7)",
            "footer_glass_color": "rgba(255, 255, 255, 0.5)",
            "floating_text_enabled": false,
            "floating_texts": [],
            "music_enabled": false,
            "music_auto_play": false,
            "music_volume": 0.7,
            "music_loop": true
        }
    }))
}

/// 获取外观设置
pub async fn get_appearance() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "settings": {
            "background_image": "",
            "mobile_background_image": "",
            "global_opacity": "0.9",
            "background_size": "cover",
            "background_position": "center",
            "background_repeat": "no-repeat",
            "background_attachment": "fixed",
            "blur_amount": "20px",
            "saturate_amount": "180%",
            "dark_mode_enabled": false,
            "navbar_glass_color": "rgba(255, 255, 255, 0.85)",
            "navbar_text_color": "#333333",
            "card_glass_color": "rgba(255, 255, 255, 0.7)",
            "footer_glass_color": "rgba(255, 255, 255, 0.5)",
            "floating_text_enabled": false,
            "floating_texts": []
        }
    }))
}

/// 更新外观设置
pub async fn update_appearance(req: web::Json<serde_json::Value>) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "外观设置已更新",
        "settings": req.into_inner()
    }))
}

/// 获取音乐设置
pub async fn get_music() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "settings": {
            "music_enabled": false,
            "music_auto_play": false,
            "music_volume": 0.7,
            "music_loop": true,
            "playlist": []
        }
    }))
}

/// 更新音乐设置
pub async fn update_music(req: web::Json<serde_json::Value>) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "音乐设置已更新",
        "settings": req.into_inner()
    }))
}

/// 部分更新音乐设置
pub async fn update_music_partial(req: web::Json<serde_json::Value>) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "音乐设置已更新",
        "settings": req.into_inner()
    }))
}

/// 获取模板设置
pub async fn get_template() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "settings": {
            "name": "RustBlog",
            "greting": "欢迎来到 RustBlog",
            "year": "2026",
            "foodes": "RustBlog - 使用 Rust + Actix-web 构建",
            "global_avatar": "/img/avatar.webp",
            "article_title": false,
            "article_title_prefix": "",
            "switch_notice": true,
            "switch_notice_text": "🎉 新文章发布！",
            "external_link_warning": true,
            "external_link_whitelist": "github.com,rust-lang.org",
            "external_link_warning_text": "您即将离开本站",
            "live2d_enabled": false,
            "live2d_show_on_index": true,
            "live2d_show_on_passage": true,
            "live2d_show_on_collect": true,
            "live2d_show_on_about": true,
            "live2d_show_on_admin": false,
            "live2d_model_id": 1,
            "live2d_model_path": "",
            "live2d_cdn_path": "https://unpkg.com/live2d-widget-model@1.0.5/",
            "live2d_position": "right",
            "live2d_width": "280px",
            "live2d_height": "250px",
            "sponsor_enabled": false,
            "sponsor_title": "感谢您的支持",
            "sponsor_image": "/img/avatar.webp",
            "sponsor_description": "如果您觉得这个博客对您有帮助，欢迎赞助支持！",
            "sponsor_button_text": "❤️ 赞助支持"
        }
    }))
}

/// 更新模板设置
pub async fn update_template(req: web::Json<serde_json::Value>) -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "模板设置已更新",
        "settings": req.into_inner()
    }))
}

/// 更新设置（通用）
pub async fn update() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Settings updated"
    }))
}