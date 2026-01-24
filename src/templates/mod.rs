use actix_web::HttpResponse;
use tera::{Tera, Context as TeraContext};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 全局 Tera 实例
lazy_static::lazy_static! {
    static ref TERA: Arc<RwLock<Tera>> = {
        let mut tera = match Tera::new("templates/**/*.html") {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Error parsing templates: {}", e);
                panic!("Failed to parse templates");
            }
        };
        // 开发模式下启用自动重载
        tera.autoescape_on(vec![".html", ".htm"]);
        Arc::new(RwLock::new(tera))
    };
}

/// 模板设置
#[derive(Debug, Clone, serde::Serialize)]
pub struct TemplateSettings {
    pub background_image: String,
    pub background_size: String,
    pub background_position: String,
    pub background_repeat: String,
    pub background_attachment: String,
    pub global_opacity: f64,
    pub blur_amount: u32,
    pub saturate_amount: u32,
    pub floating_text_enabled: bool,
    // Admin 相关
    pub navbar_glass_color: String,
    pub card_glass_color: String,
    pub footer_glass_color: String,
}

impl Default for TemplateSettings {
    fn default() -> Self {
        Self {
            background_image: String::new(),
            background_size: "cover".to_string(),
            background_position: "center".to_string(),
            background_repeat: "no-repeat".to_string(),
            background_attachment: "fixed".to_string(),
            global_opacity: 0.9,
            blur_amount: 20,
            saturate_amount: 180,
            floating_text_enabled: false,
            navbar_glass_color: "rgba(255, 255, 255, 0.85)".to_string(),
            card_glass_color: "rgba(255, 255, 255, 0.7)".to_string(),
            footer_glass_color: "rgba(255, 255, 255, 0.5)".to_string(),
        }
    }
}

/// 渲染模板
pub async fn render_template(template_name: &str, context: &TeraContext) -> HttpResponse {
    let tera = TERA.read().await;
    
    match tera.render(template_name, context) {
        Ok(html) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .insert_header(("Cache-Control", "public, max-age=300"))
            .body(html),
        Err(e) => {
            eprintln!("Template rendering error: {}", e);
            HttpResponse::InternalServerError()
                .body(format!("Failed to render template: {}", e))
        }
    }
}

/// 创建主页上下文
pub fn create_index_context() -> TeraContext {
    let mut context = TeraContext::new();
    let now = chrono::Local::now();
    
    context.insert("title", "RustBlog");
    context.insert("name", "Dango");
    context.insert("greting", "欢迎来到 RustBlog，一个基于 Rust 和 Actix-web 构建的现代化博客系统");
    context.insert("year", &now.format("%Y").to_string());
    context.insert("foodes", "RustBlog - 使用 Rust + Actix-web 构建");
    context.insert("settings", &TemplateSettings::default());
    context.insert("switch_notice", &true);
    context.insert("switch_notice_text", "🎉 新文章发布！");
    context.insert("external_link_warning", &true);
    context.insert("external_link_warning_text", "您即将离开本站");
    context.insert("external_link_whitelist", "github.com,rust-lang.org");
    
    // Live2D
    context.insert("live2d_enabled", &false);
    context.insert("live2d_show_on_index", &false);
    context.insert("live2d_model_id", &1);
    context.insert("live2d_model_name", "shizuku");
    context.insert("live2d_model_textures_id", &1);
    context.insert("live2d_cdn_path", "https://unpkg.com/live2d-widget@latest");
    context.insert("live2d_model_path", "https://unpkg.com/live2d-widget-model-shizuku@latest/assets/shizuku.model.json");
    context.insert("live2d_position", "right");
    context.insert("live2d_width", &280);
    context.insert("live2d_height", &260);
    
    context
}

/// 创建文章上下文
pub fn create_passage_context() -> TeraContext {
    let mut context = TeraContext::new();
    let now = chrono::Local::now();
    
    context.insert("title", "文章 - RustBlog");
    context.insert("name", "Dango");
    context.insert("year", &now.format("%Y").to_string());
    context.insert("foodes", "RustBlog - 使用 Rust + Actix-web 构建");
    context.insert("settings", &TemplateSettings::default());
    context.insert("switch_notice", &true);
    context.insert("switch_notice_text", "🎉 新文章发布！");
    context.insert("external_link_warning", &true);
    context.insert("external_link_warning_text", "您即将离开本站");
    context.insert("external_link_whitelist", "github.com,rust-lang.org");
    
    // 文章内容
    context.insert("content", "");
    context.insert("date", &now.format("%Y-%m-%d").to_string());
    context.insert("passage_id", "");
    context.insert("published_at", &now.format("%Y-%m-%d %H:%M").to_string());
    context.insert("read_time", "5 分钟");
    context.insert("passage_status", "published");
    context.insert("is_scheduled", &false);
    context.insert("is_unpublished", &false);
    
    // 赞助
    context.insert("sponsor_enabled", &false);
    context.insert("sponsor_title", "");
    context.insert("sponsor_description", "");
    context.insert("sponsor_image", "");
    context.insert("sponsor_button_text", "");
    
    // Live2D
    context.insert("live2d_enabled", &false);
    context.insert("live2d_show_on_passage", &false);
    context.insert("live2d_cdn_path", "https://unpkg.com/live2d-widget@latest");
    context.insert("live2d_model_id", &1);
    context.insert("live2d_model_path", "https://unpkg.com/live2d-widget-model-shizuku@latest/assets/shizuku.model.json");
    context.insert("live2d_position", "right");
    context.insert("live2d_width", &280);
    context.insert("live2d_height", &260);
    
    context
}

/// 创建归档上下文
pub fn create_collect_context() -> TeraContext {
    let mut context = TeraContext::new();
    let now = chrono::Local::now();
    
    context.insert("title", "归档 - RustBlog");
    context.insert("name", "Dango");
    context.insert("year", &now.format("%Y").to_string());
    context.insert("foodes", "RustBlog - 使用 Rust + Actix-web 构建");
    context.insert("settings", &TemplateSettings::default());
    context.insert("switch_notice", &true);
    context.insert("switch_notice_text", "🎉 新文章发布！");
    context.insert("external_link_warning", &true);
    context.insert("external_link_warning_text", "您即将离开本站");
    context.insert("external_link_whitelist", "github.com,rust-lang.org");
    
    // Live2D
    context.insert("live2d_enabled", &false);
    context.insert("live2d_show_on_collect", &false);
    context.insert("live2d_cdn_path", "https://unpkg.com/live2d-widget@latest");
    context.insert("live2d_model_id", &1);
    context.insert("live2d_model_path", "https://unpkg.com/live2d-widget-model-shizuku@latest/assets/shizuku.model.json");
    context.insert("live2d_position", "right");
    context.insert("live2d_width", &280);
    context.insert("live2d_height", &260);
    
    context
}

/// 创建关于上下文
pub fn create_about_context() -> TeraContext {
    let mut context = TeraContext::new();
    let now = chrono::Local::now();
    
    context.insert("title", "关于 - RustBlog");
    context.insert("name", "Dango");
    context.insert("year", &now.format("%Y").to_string());
    context.insert("foodes", "RustBlog - 使用 Rust + Actix-web 构建");
    context.insert("settings", &TemplateSettings::default());
    context.insert("switch_notice", &true);
    context.insert("switch_notice_text", "🎉 新文章发布！");
    context.insert("external_link_warning", &true);
    context.insert("external_link_warning_text", "您即将离开本站");
    context.insert("external_link_whitelist", "github.com,rust-lang.org");
    
    // Live2D
    context.insert("live2d_enabled", &false);
    context.insert("live2d_show_on_about", &false);
    context.insert("live2d_cdn_path", "https://unpkg.com/live2d-widget@latest");
    context.insert("live2d_model_id", &1);
    context.insert("live2d_model_path", "https://unpkg.com/live2d-widget-model-shizuku@latest/assets/shizuku.model.json");
    context.insert("live2d_position", "right");
    context.insert("live2d_width", &280);
    context.insert("live2d_height", &260);
    
    context
}

/// 创建编辑器上下文
pub fn create_markdown_editor_context() -> TeraContext {
    let mut context = TeraContext::new();
    let now = chrono::Local::now();
    
    context.insert("title", "编辑器 - RustBlog");
    context.insert("name", "Dango");
    context.insert("year", &now.format("%Y").to_string());
    context.insert("foodes", "RustBlog - 使用 Rust + Actix-web 构建");
    context.insert("settings", &TemplateSettings::default());
    context.insert("switch_notice", &true);
    context.insert("switch_notice_text", "🎉 新文章发布！");
    context.insert("external_link_warning", &true);
    context.insert("external_link_warning_text", "您即将离开本站");
    context.insert("external_link_whitelist", "github.com,rust-lang.org");
    
    context
}