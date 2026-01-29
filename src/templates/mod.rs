use actix_web::HttpResponse;
use tera::{Tera, Context as TeraContext};
use std::sync::Arc;
use tokio::sync::RwLock;

lazy_static::lazy_static! {
    static ref TERA: Arc<RwLock<Tera>> = {
        let tera = match Tera::new("templates/**/*.html") {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Error parsing templates: {}", e);
                panic!("Failed to parse templates");
            }
        };
        // 不启用自动转义，避免 CSS URL 中的字符被转义
        // 如需转义，在模板中使用 | escape 过滤器
        Arc::new(RwLock::new(tera))
    };
}

/// 模板设置
#[derive(Debug, Clone, serde::Serialize)]
pub struct TemplateSettings {
    pub background_image: String,
    pub background_color: String,
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
            background_image: "/img/test.webp".to_string(),  // 使用默认背景图片
            background_color: "#1a1a2e".to_string(),
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

/// 外观设置结构（用于 API 和前端）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppearanceSettings {
    pub background_image: String,
    pub mobile_background_image: String,
    pub global_opacity: String,
    pub background_size: String,
    pub background_position: String,
    pub background_repeat: String,
    pub background_attachment: String,
    pub blur_amount: String,
    pub saturate_amount: String,
    pub dark_mode_enabled: bool,
    pub navbar_glass_color: String,
    pub navbar_text_color: String,
    pub card_glass_color: String,
    pub footer_glass_color: String,
    pub floating_text_enabled: bool,
    pub floating_texts: Vec<String>,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            background_image: "/img/test.webp".to_string(),
            mobile_background_image: "/img/mobile-test.webp".to_string(),
            global_opacity: "0.15".to_string(),
            background_size: "cover".to_string(),
            background_position: "center".to_string(),
            background_repeat: "no-repeat".to_string(),
            background_attachment: "fixed".to_string(),
            blur_amount: "20px".to_string(),
            saturate_amount: "180%".to_string(),
            dark_mode_enabled: false,
            navbar_glass_color: "rgba(220, 138, 221, 0.15)".to_string(),
            navbar_text_color: "#333333".to_string(),
            card_glass_color: "rgba(220, 138, 221, 0.2)".to_string(),
            footer_glass_color: "rgba(220, 138, 221, 0.25)".to_string(),
            floating_text_enabled: false,
            floating_texts: vec![
                "perfect".to_string(),
                "good".to_string(),
                "excellent".to_string(),
                "extraordinary".to_string(),
                "legend".to_string(),
            ],
        }
    }
}

/// 从数据库加载外观设置
pub fn load_appearance_settings() -> Result<AppearanceSettings, Box<dyn std::error::Error>> {
    // 使用同步方法获取数据库连接池
    let pool = crate::db::get_db_pool_sync()?;
    let conn = pool.get()?;
    
    let mut settings = AppearanceSettings::default();
    
    // 定义要加载的设置项
    let keys = vec![
        ("background_image", "background_image"),
        ("mobile_background_image", "mobile_background_image"),
        ("global_opacity", "global_opacity"),
        ("background_size", "background_size"),
        ("background_position", "background_position"),
        ("background_repeat", "background_repeat"),
        ("background_attachment", "background_attachment"),
        ("blur_amount", "blur_amount"),
        ("saturate_amount", "saturate_amount"),
        ("dark_mode_enabled", "dark_mode_enabled"),
        ("navbar_glass_color", "navbar_glass_color"),
        ("navbar_text_color", "navbar_text_color"),
        ("card_glass_color", "card_glass_color"),
        ("footer_glass_color", "footer_glass_color"),
        ("floating_text_enabled", "floating_text_enabled"),
        ("floating_texts", "floating_texts"),
    ];
    
    for (db_key, field_name) in keys {
        if let Some(setting) = crate::db::repositories::SettingRepository::get(&conn, db_key)? {
            match field_name {
                "background_image" => settings.background_image = setting.value,
                "mobile_background_image" => settings.mobile_background_image = setting.value,
                "global_opacity" => settings.global_opacity = setting.value,
                "background_size" => settings.background_size = setting.value,
                "background_position" => settings.background_position = setting.value,
                "background_repeat" => settings.background_repeat = setting.value,
                "background_attachment" => settings.background_attachment = setting.value,
                "blur_amount" => settings.blur_amount = setting.value,
                "saturate_amount" => settings.saturate_amount = setting.value,
                "dark_mode_enabled" => settings.dark_mode_enabled = setting.value == "true",
                "navbar_glass_color" => settings.navbar_glass_color = setting.value,
                "navbar_text_color" => settings.navbar_text_color = setting.value,
                "card_glass_color" => settings.card_glass_color = setting.value,
                "footer_glass_color" => settings.footer_glass_color = setting.value,
                "floating_text_enabled" => settings.floating_text_enabled = setting.value == "true",
                "floating_texts" => {
                    // 尝试解析 JSON 数组
                    if let Ok(arr) = serde_json::from_str::<Vec<String>>(&setting.value) {
                        settings.floating_texts = arr;
                    } else {
                        // 如果不是有效的 JSON，尝试按逗号分割
                        settings.floating_texts = setting.value
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                    }
                }
                _ => {}
            }
        }
    }
    
    Ok(settings)
}

/// 将 AppearanceSettings 转换为 TemplateSettings
pub fn appearance_to_template_settings(appearance: &AppearanceSettings) -> TemplateSettings {
    TemplateSettings {
        background_image: appearance.background_image.clone(),
        background_color: "#1a1a2e".to_string(),
        background_size: appearance.background_size.clone(),
        background_position: appearance.background_position.clone(),
        background_repeat: appearance.background_repeat.clone(),
        background_attachment: appearance.background_attachment.clone(),
        global_opacity: appearance.global_opacity.parse().unwrap_or(0.9),
        blur_amount: appearance.blur_amount.trim_end_matches("px").parse().unwrap_or(20),
        saturate_amount: appearance.saturate_amount.trim_end_matches("%").parse().unwrap_or(180),
        floating_text_enabled: appearance.floating_text_enabled,
        navbar_glass_color: appearance.navbar_glass_color.clone(),
        card_glass_color: appearance.card_glass_color.clone(),
        footer_glass_color: appearance.footer_glass_color.clone(),
    }
}

/// 渲染模板
pub async fn render_template(template_name: &str, context: &TeraContext) -> HttpResponse {
    // 开发模式：每次重新加载模板
    let tera = match Tera::new("templates/**/*.html") {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Template rendering error: {}", e);
            return HttpResponse::InternalServerError()
                .body(format!("Failed to parse templates: {}", e));
        }
    };
    
    match tera.render(template_name, context) {
        Ok(html) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .insert_header(("Cache-Control", "no-cache"))
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
    
    // 默认值
    let mut name = "Dango".to_string();
    let mut greting = "欢迎来到 RustBlog，一个基于 Rust 和 Actix-web 构建的现代化博客系统".to_string();
    
    // 尝试从数据库加载模板设置
    if let Ok(pool) = crate::db::get_db_pool_sync() {
        if let Ok(conn) = pool.get() {
            // 加载 name
            if let Ok(Some(setting)) = crate::db::repositories::SettingRepository::get(&conn, "template_name") {
                name = setting.value;
            }
            
            // 加载 greting
            if let Ok(Some(setting)) = crate::db::repositories::SettingRepository::get(&conn, "template_greting") {
                greting = setting.value;
            }
        }
    }
    
    context.insert("title", "RustBlog");
    context.insert("name", &name);
    context.insert("greting", &greting);
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

/// 创建管理后台上下文
pub fn create_admin_context() -> TeraContext {
    let mut context = TeraContext::new();
    let now = chrono::Local::now();
    
    context.insert("title", "管理后台 - RustBlog");
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
    context.insert("live2d_show_on_admin", &false);
    context.insert("live2d_cdn_path", "https://unpkg.com/live2d-widget@latest");
    context.insert("live2d_model_id", &1);
    context.insert("live2d_model_path", "https://unpkg.com/live2d-widget-model-shizuku@latest/assets/shizuku.model.json");
    context.insert("live2d_position", "right");
    context.insert("live2d_width", &280);
    context.insert("live2d_height", &260);
    
    context
}