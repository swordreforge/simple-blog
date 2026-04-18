use crate::utils::unsafe_utils::{format_date, format_datetime_short, format_year};
use actix_web::HttpResponse;
use std::collections::HashMap;
use std::sync::Arc;
use tera::{Context as TeraContext, Tera};

lazy_static::lazy_static! {
    static ref TERA: Arc<Tera> = {
        // 强制使用嵌入的文件系统，不回退到文件系统
        let tera = create_embedded_tera().expect(
            "Failed to create embedded Tera. 请确保从项目根目录编译：cargo build --release\n\
             如果仍然失败，请检查：\n\
             1. build.rs 是否正确配置了嵌入式资源\n\
             2. templates/ 目录下是否有 .html 文件\n\
             3. 编译时是否包含了所有必要的模板文件"
        );
        // 不启用自动转义，避免 CSS URL 中的字符被转义
        // 如需转义，在模板中使用 | escape 过滤器
        Arc::new(tera)
    };
}

/// 从内嵌文件系统创建 Tera 实例
fn create_embedded_tera() -> Result<Tera, Box<dyn std::error::Error>> {
    use crate::embedded::EmbeddedAssets;

    let mut tera = Tera::default();
    tera.autoescape_on(vec!["html"]);

    println!("🔍 调试：开始从内嵌文件加载模板...");
    let mut found_templates = Vec::new();

    // 遍历内嵌的模板文件
    for path in EmbeddedAssets::iter() {
        let path_str = path.as_ref();

        // 只处理 templates 目录下的 HTML 文件
        if path_str.starts_with("templates/") && path_str.ends_with(".html")
            && let Some(content) = EmbeddedAssets::get(&path) {
                // 移除 "templates/" 前缀，保留子目录结构
                // 例如: "templates/admin/admin.html" -> "admin/admin.html"
                let name = path_str.strip_prefix("templates/")
                    .ok_or("Path should start with 'templates/' after check")?;
                let content_str = std::str::from_utf8(&content.data)?;

                println!("  ✓ 加载模板: {} ({} bytes)", name, content.data.len());
                found_templates.push(name.to_string());

                // 使用 add_raw_template 方法直接添加模板内容
                tera.add_raw_template(name, content_str)?;
            }
    }

    if found_templates.is_empty() {
        eprintln!("❌ 错误: 没有找到任何内嵌模板文件！");
        eprintln!("🔍 所有嵌入的文件:");
        for path in EmbeddedAssets::iter() {
            eprintln!("  - {}", path.as_ref());
        }
        return Err("No embedded templates found".into());
    } else {
        println!("✅ 成功加载 {} 个内嵌模板", found_templates.len());
    }

    Ok(tera)
}

/// 模板设置
#[derive(Debug, Clone, serde::Serialize)]
pub struct TemplateSettings {
    // 基础模板设置
    pub name: String,
    pub greting: String,
    pub year: String,
    pub foodes: String,

    // 外观相关
    pub background_image: String,
    pub mobile_background_image: String,
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
    pub navbar_text_color: String,
    pub card_glass_color: String,
    pub footer_glass_color: String,
    pub dark_mode_enabled: bool,

    // 文章相关
    pub article_title: bool,
    pub article_title_prefix: String,

    // 切换提示
    pub switch_notice: bool,
    pub switch_notice_text: String,

    // 外部链接警告
    pub external_link_warning: bool,
    pub external_link_whitelist: String,
    pub external_link_warning_text: String,

    // Live2D 设置
    pub live2d_enabled: bool,
    pub live2d_show_on_index: bool,
    pub live2d_show_on_passage: bool,
    pub live2d_show_on_collect: bool,
    pub live2d_show_on_about: bool,
    pub live2d_show_on_admin: bool,
    pub live2d_model_id: String,
    pub live2d_model_path: String,
    pub live2d_cdn_path: String,
    pub live2d_position: String,
    pub live2d_width: String,
    pub live2d_height: String,

    // 赞助设置
    pub sponsor_enabled: bool,
    pub sponsor_title: String,
    pub sponsor_image: String,
    pub sponsor_description: String,
    pub sponsor_button_text: String,

    // 全局设置
    pub global_avatar: String,

    // 附件设置
    pub attachment_default_visibility: String,
    pub attachment_max_size: i64,
    pub attachment_allowed_types: String,

    // 文章摘要设置
    pub passage_summarize_enabled: bool,

    // 备案信息（针对中国内地）
    pub beian_enabled: bool,
    pub icp_number: String,
    pub police_record_code: String,
    pub police_record_content: String,
}

impl Default for TemplateSettings {
    fn default() -> Self {
        Self {
            // 基础模板设置
            name: "欢迎来到我的博客".to_string(),
            greting: "这是一个使用 Rust 语言构建的个人博客系统，支持文章管理、数据分析等功能。"
                .to_string(),
            year: "2026".to_string(),
            foodes: "我的博客".to_string(),

            // 外观相关
            background_image: "/img/test.webp".to_string(),
            mobile_background_image: "/img/mobile-test.webp".to_string(),
            background_color: "#ffffff".to_string(),
            background_size: "cover".to_string(),
            background_position: "center".to_string(),
            background_repeat: "no-repeat".to_string(),
            background_attachment: "fixed".to_string(),
            global_opacity: 0.15,
            blur_amount: 20,
            saturate_amount: 180,
            floating_text_enabled: false,

            // Admin 相关
            navbar_glass_color: "rgba(60, 60, 60, 0.6)".to_string(),
            navbar_text_color: "#ffffff".to_string(),
            card_glass_color: "rgba(220, 138, 221, 0.2)".to_string(),
            footer_glass_color: "rgba(220, 138, 221, 0.25)".to_string(),
            dark_mode_enabled: false,

            // 文章相关
            article_title: true,
            article_title_prefix: "文章".to_string(),

            // 切换提示
            switch_notice: true,
            switch_notice_text: "回来继续阅读".to_string(),

            // 外部链接警告
            external_link_warning: true,
            external_link_whitelist: "github.com,gitee.com,stackoverflow.com".to_string(),
            external_link_warning_text: "您即将离开本站，前往外部链接".to_string(),

            // Live2D 设置
            live2d_enabled: false,
            live2d_show_on_index: true,
            live2d_show_on_passage: true,
            live2d_show_on_collect: true,
            live2d_show_on_about: true,
            live2d_show_on_admin: false,
            live2d_model_id: "1".to_string(),
            live2d_model_path: "".to_string(),
            live2d_cdn_path: "https://unpkg.com/live2d-widget-model@1.0.5/".to_string(),
            live2d_position: "right".to_string(),
            live2d_width: "280px".to_string(),
            live2d_height: "250px".to_string(),

            // 赞助设置
            sponsor_enabled: false,
            sponsor_title: "感谢您的支持".to_string(),
            sponsor_image: "/img/avatar.webp".to_string(),
            sponsor_description: "如果您觉得这个博客对您有帮助，欢迎赞助支持！".to_string(),
            sponsor_button_text: "❤️ 赞助支持".to_string(),

            // 全局设置
            global_avatar: "/img/avatar.webp".to_string(),

            // 附件设置
            attachment_default_visibility: "public".to_string(),
            attachment_max_size: 524288000, // 500MB
            attachment_allowed_types:
                "jpg,jpeg,png,gif,mp4,mp3,pdf,doc,docx,xls,xlsx,ppt,pptx,zip,rar,7z,tar,gz"
                    .to_string(),

            // 文章摘要设置
            passage_summarize_enabled: true,

            // 备案信息（针对中国内地）
            beian_enabled: false,
            icp_number: "".to_string(),
            police_record_code: "".to_string(),
            police_record_content: "".to_string(),
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
            navbar_glass_color: "rgba(60, 60, 60, 0.6)".to_string(),
            navbar_text_color: "#ffffff".to_string(),
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

/// 从已获取的 settings map 构建 TemplateSettings（避免重复 DB 查询）
fn build_template_settings_from_map(map: &HashMap<String, String>) -> TemplateSettings {
    let mut settings = TemplateSettings::default();
    for (key, value) in map {
        match key.as_str() {
            "template_name" => settings.name = value.clone(),
            "template_greting" => settings.greting = value.clone(),
            "template_year" => settings.year = value.clone(),
            "template_foods" => settings.foodes = value.clone(),
            "template_article_title" => settings.article_title = value == "true",
            "template_article_title_prefix" => settings.article_title_prefix = value.clone(),
            "template_switch_notice" => settings.switch_notice = value == "true",
            "template_switch_notice_text" => settings.switch_notice_text = value.clone(),
            "external_link_warning" => settings.external_link_warning = value == "true",
            "external_link_whitelist" => settings.external_link_whitelist = value.clone(),
            "external_link_warning_text" => settings.external_link_warning_text = value.clone(),
            "background_image" => settings.background_image = value.clone(),
            "mobile_background_image" => settings.mobile_background_image = value.clone(),
            "background_color" => settings.background_color = value.clone(),
            "background_size" => settings.background_size = value.clone(),
            "background_position" => settings.background_position = value.clone(),
            "background_repeat" => settings.background_repeat = value.clone(),
            "background_attachment" => settings.background_attachment = value.clone(),
            "global_opacity" => settings.global_opacity = value.parse().unwrap_or(0.15),
            "blur_amount" => settings.blur_amount = value.parse().unwrap_or(20),
            "saturate_amount" => settings.saturate_amount = value.parse().unwrap_or(180),
            "floating_text_enabled" => settings.floating_text_enabled = value == "true",
            "navbar_glass_color" => settings.navbar_glass_color = value.clone(),
            "navbar_text_color" => settings.navbar_text_color = value.clone(),
            "card_glass_color" => settings.card_glass_color = value.clone(),
            "footer_glass_color" => settings.footer_glass_color = value.clone(),
            "dark_mode_enabled" => settings.dark_mode_enabled = value == "true",
            "live2d_enabled" => settings.live2d_enabled = value == "true",
            "live2d_show_on_index" => settings.live2d_show_on_index = value == "true",
            "live2d_show_on_passage" => settings.live2d_show_on_passage = value == "true",
            "live2d_show_on_collect" => settings.live2d_show_on_collect = value == "true",
            "live2d_show_on_about" => settings.live2d_show_on_about = value == "true",
            "live2d_show_on_admin" => settings.live2d_show_on_admin = value == "true",
            "live2d_model_id" => settings.live2d_model_id = value.clone(),
            "live2d_model_path" => settings.live2d_model_path = value.clone(),
            "live2d_cdn_path" => settings.live2d_cdn_path = value.clone(),
            "live2d_position" => settings.live2d_position = value.clone(),
            "live2d_width" => settings.live2d_width = value.clone(),
            "live2d_height" => settings.live2d_height = value.clone(),
            "sponsor_enabled" => settings.sponsor_enabled = value == "true",
            "sponsor_title" => settings.sponsor_title = value.clone(),
            "sponsor_image" => settings.sponsor_image = value.clone(),
            "sponsor_description" => settings.sponsor_description = value.clone(),
            "sponsor_button_text" => settings.sponsor_button_text = value.clone(),
            "global_avatar" => settings.global_avatar = value.clone(),
            "attachment_default_visibility" => {
                settings.attachment_default_visibility = value.clone()
            }
            "attachment_max_size" => {
                settings.attachment_max_size = value.parse().unwrap_or(524288000)
            }
            "attachment_allowed_types" => settings.attachment_allowed_types = value.clone(),
            "passage_summarize_enabled" => {
                settings.passage_summarize_enabled = value == "true"
            }
            "beian_enabled" => settings.beian_enabled = value == "true",
            "icp_number" => settings.icp_number = value.clone(),
            "police_record_code" => settings.police_record_code = value.clone(),
            "police_record_content" => settings.police_record_content = value.clone(),
            _ => {}
        }
    }
    settings
}

/// 从数据库加载外观设置
pub fn load_appearance_settings() -> Result<AppearanceSettings, Box<dyn std::error::Error>> {
    let pool = crate::db::get_db_pool_sync()?;
    let conn = pool.get()?;
    let map = crate::db::repositories::SettingRepository::get_all_as_map(&conn)?;

    let mut settings = AppearanceSettings::default();

    if let Some(v) = map.get("background_image") {
        settings.background_image = v.clone();
    }
    if let Some(v) = map.get("mobile_background_image") {
        settings.mobile_background_image = v.clone();
    }
    if let Some(v) = map.get("global_opacity") {
        settings.global_opacity = v.clone();
    }
    if let Some(v) = map.get("background_size") {
        settings.background_size = v.clone();
    }
    if let Some(v) = map.get("background_position") {
        settings.background_position = v.clone();
    }
    if let Some(v) = map.get("background_repeat") {
        settings.background_repeat = v.clone();
    }
    if let Some(v) = map.get("background_attachment") {
        settings.background_attachment = v.clone();
    }
    if let Some(v) = map.get("blur_amount") {
        settings.blur_amount = v.clone();
    }
    if let Some(v) = map.get("saturate_amount") {
        settings.saturate_amount = v.clone();
    }
    if let Some(v) = map.get("dark_mode_enabled") {
        settings.dark_mode_enabled = v == "true";
    }
    if let Some(v) = map.get("navbar_glass_color") {
        settings.navbar_glass_color = v.clone();
    }
    if let Some(v) = map.get("navbar_text_color") {
        settings.navbar_text_color = v.clone();
    }
    if let Some(v) = map.get("card_glass_color") {
        settings.card_glass_color = v.clone();
    }
    if let Some(v) = map.get("footer_glass_color") {
        settings.footer_glass_color = v.clone();
    }
    if let Some(v) = map.get("floating_text_enabled") {
        settings.floating_text_enabled = v == "true";
    }
    if let Some(v) = map.get("floating_texts") {
        // 尝试解析 JSON 数组
        if let Ok(arr) = serde_json::from_str::<Vec<String>>(v) {
            settings.floating_texts = arr;
        } else {
            // 如果不是有效的 JSON，尝试按逗号分割
            settings.floating_texts = v
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
    }

    Ok(settings)
}

/// 从数据库加载模板设置
pub fn load_template_settings() -> Result<TemplateSettings, Box<dyn std::error::Error>> {
    let pool = crate::db::get_db_pool_sync()?;
    let conn = pool.get()?;
    let map = crate::db::repositories::SettingRepository::get_all_as_map(&conn)?;
    Ok(build_template_settings_from_map(&map))
}

/// 将 AppearanceSettings 转换为 TemplateSettings
pub fn appearance_to_template_settings(appearance: &AppearanceSettings) -> TemplateSettings {
    TemplateSettings {
        background_image: appearance.background_image.clone(),
        mobile_background_image: appearance.mobile_background_image.clone(),
        background_color: "#1a1a2e".to_string(),
        background_size: appearance.background_size.clone(),
        background_position: appearance.background_position.clone(),
        background_repeat: appearance.background_repeat.clone(),
        background_attachment: appearance.background_attachment.clone(),
        global_opacity: appearance.global_opacity.parse().unwrap_or(0.9),
        blur_amount: appearance
            .blur_amount
            .trim_end_matches("px")
            .parse()
            .unwrap_or(20),
        saturate_amount: appearance
            .saturate_amount
            .trim_end_matches("%")
            .parse()
            .unwrap_or(180),
        floating_text_enabled: appearance.floating_text_enabled,
        navbar_glass_color: appearance.navbar_glass_color.clone(),
        card_glass_color: appearance.card_glass_color.clone(),
        footer_glass_color: appearance.footer_glass_color.clone(),
        ..TemplateSettings::default()
    }
}

/// 渲染模板（无锁，因为 Tera 只在启动时加载，运行时只读）
pub async fn render_template(template_name: &str, context: &TeraContext) -> HttpResponse {
    // 使用静态的 TERA 实例（内嵌的模板），无需加锁
    match TERA.render(template_name, context) {
        Ok(html) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .insert_header(("Cache-Control", "no-cache"))
            .body(html),
        Err(e) => {
            eprintln!("Template rendering error: {}", e);
            HttpResponse::InternalServerError().body(format!("Failed to render template: {}", e))
        }
    }
}

/// 渲染模板并返回指定的 HTTP 状态码
pub async fn render_template_with_status(
    template_name: &str,
    context: &TeraContext,
    status_code: actix_web::http::StatusCode,
) -> HttpResponse {
    // 使用静态的 TERA 实例（内嵌的模板），无需加锁
    match TERA.render(template_name, context) {
        Ok(html) => HttpResponse::build(status_code)
            .content_type("text/html; charset=utf-8")
            .insert_header(("Cache-Control", "no-cache"))
            .body(html),
        Err(e) => {
            eprintln!("Template rendering error: {}", e);
            HttpResponse::build(status_code)
                .content_type("text/plain; charset=utf-8")
                .body(format!("Failed to render template: {}", e))
        }
    }
}

/// 创建主页上下文
pub fn create_index_context() -> TeraContext {
    let mut context = TeraContext::new();
    let now = chrono::Local::now();

    // 一次性批量加载所有设置，构建 TemplateSettings
    let settings = if let Ok(pool) = crate::db::get_db_pool_sync()
        && let Ok(conn) = pool.get()
        && let Ok(map) = crate::db::repositories::SettingRepository::get_all_as_map(&conn)
    {
        build_template_settings_from_map(&map)
    } else {
        TemplateSettings::default()
    };

    context.insert("title", "RustBlog");
    context.insert("name", &settings.name);
    context.insert("greting", &settings.greting);
    context.insert("year", &format_year(&now));
    context.insert("foodes", &settings.foodes);
    context.insert("external_link_warning", &settings.external_link_warning);
    context.insert("external_link_whitelist", &settings.external_link_whitelist);
    context.insert("external_link_warning_text", &settings.external_link_warning_text);
    context.insert("settings", &settings);
    context.insert("switch_notice", &settings.switch_notice);
    context.insert("switch_notice_text", &settings.switch_notice_text);

    // Live2D
    context.insert("live2d_enabled", &settings.live2d_enabled);
    context.insert("live2d_show_on_index", &settings.live2d_show_on_index);
    context.insert("live2d_model_id", &settings.live2d_model_id);
    context.insert("live2d_model_name", "shizuku");
    context.insert("live2d_model_textures_id", &1);
    context.insert("live2d_cdn_path", &settings.live2d_cdn_path);
    context.insert("live2d_model_path", &settings.live2d_model_path);
    context.insert("live2d_position", &settings.live2d_position);
    context.insert("live2d_width", &settings.live2d_width);
    context.insert("live2d_height", &settings.live2d_height);
    context.insert("global_avatar", &settings.global_avatar);

    // 备案信息
    context.insert("beian_enabled", &settings.beian_enabled);
    context.insert("icp_number", &settings.icp_number);
    context.insert("police_record_code", &settings.police_record_code);
    context.insert("police_record_content", &settings.police_record_content);

    context
}

/// 创建文章上下文
pub fn create_passage_context() -> TeraContext {
    let mut context = TeraContext::new();
    let now = chrono::Local::now();

    // 一次性批量加载所有设置，构建 TemplateSettings
    let settings = if let Ok(pool) = crate::db::get_db_pool_sync()
        && let Ok(conn) = pool.get()
        && let Ok(map) = crate::db::repositories::SettingRepository::get_all_as_map(&conn)
    {
        build_template_settings_from_map(&map)
    } else {
        TemplateSettings::default()
    };

    context.insert("title", "文章 - RustBlog");
    context.insert("name", "Dango");
    context.insert("year", &format_year(&now));
    context.insert("foodes", &settings.foodes);
    context.insert("external_link_warning", &settings.external_link_warning);
    context.insert("external_link_whitelist", &settings.external_link_whitelist);
    context.insert("external_link_warning_text", &settings.external_link_warning_text);
    context.insert("settings", &settings);
    context.insert("switch_notice", &settings.switch_notice);
    context.insert("switch_notice_text", &settings.switch_notice_text);

    // 文章内容
    context.insert("content", "");
    context.insert("date", &format_date(&now));
    context.insert("passage_id", "");
    context.insert("published_at", &format_datetime_short(&now));
    context.insert("read_time", "5 分钟");
    context.insert("passage_status", "published");
    context.insert("is_scheduled", &false);
    context.insert("is_unpublished", &false);

    // 赞助
    context.insert("sponsor_enabled", &settings.sponsor_enabled);
    context.insert("sponsor_title", &settings.sponsor_title);
    context.insert("sponsor_description", &settings.sponsor_description);
    context.insert("sponsor_image", &settings.sponsor_image);
    context.insert("sponsor_button_text", &settings.sponsor_button_text);
    context.insert("global_avatar", &settings.global_avatar);

    // Live2D
    context.insert("live2d_enabled", &settings.live2d_enabled);
    context.insert("live2d_show_on_passage", &settings.live2d_show_on_passage);
    context.insert("live2d_cdn_path", &settings.live2d_cdn_path);
    context.insert("live2d_model_id", &settings.live2d_model_id);
    context.insert("live2d_model_path", &settings.live2d_model_path);
    context.insert("live2d_position", &settings.live2d_position);
    context.insert("live2d_width", &settings.live2d_width);
    context.insert("live2d_height", &settings.live2d_height);

    // 备案信息
    context.insert("beian_enabled", &settings.beian_enabled);
    context.insert("icp_number", &settings.icp_number);
    context.insert("police_record_code", &settings.police_record_code);
    context.insert("police_record_content", &settings.police_record_content);

    context
}

/// 创建归档上下文
pub fn create_collect_context() -> TeraContext {
    let mut context = TeraContext::new();
    let now = chrono::Local::now();

    // 一次性批量加载所有设置，构建 TemplateSettings
    let settings = if let Ok(pool) = crate::db::get_db_pool_sync()
        && let Ok(conn) = pool.get()
        && let Ok(map) = crate::db::repositories::SettingRepository::get_all_as_map(&conn)
    {
        build_template_settings_from_map(&map)
    } else {
        TemplateSettings::default()
    };

    context.insert("title", "归档 - RustBlog");
    context.insert("name", "Dango");
    context.insert("year", &format_year(&now));
    context.insert("foodes", &settings.foodes);
    context.insert("external_link_warning", &settings.external_link_warning);
    context.insert("external_link_whitelist", &settings.external_link_whitelist);
    context.insert("external_link_warning_text", &settings.external_link_warning_text);
    context.insert("settings", &settings);
    context.insert("switch_notice", &settings.switch_notice);
    context.insert("switch_notice_text", &settings.switch_notice_text);
    context.insert("global_avatar", &settings.global_avatar);

    // Live2D
    context.insert("live2d_enabled", &settings.live2d_enabled);
    context.insert("live2d_show_on_collect", &settings.live2d_show_on_collect);
    context.insert("live2d_cdn_path", &settings.live2d_cdn_path);
    context.insert("live2d_model_id", &settings.live2d_model_id);
    context.insert("live2d_model_path", &settings.live2d_model_path);
    context.insert("live2d_position", &settings.live2d_position);
    context.insert("live2d_width", &settings.live2d_width);
    context.insert("live2d_height", &settings.live2d_height);

    // 备案信息
    context.insert("beian_enabled", &settings.beian_enabled);
    context.insert("icp_number", &settings.icp_number);
    context.insert("police_record_code", &settings.police_record_code);
    context.insert("police_record_content", &settings.police_record_content);

    context
}

/// 创建关于上下文
pub fn create_about_context() -> TeraContext {
    let mut context = TeraContext::new();
    let now = chrono::Local::now();

    // 一次性批量加载所有设置，构建 TemplateSettings
    let settings = if let Ok(pool) = crate::db::get_db_pool_sync()
        && let Ok(conn) = pool.get()
        && let Ok(map) = crate::db::repositories::SettingRepository::get_all_as_map(&conn)
    {
        build_template_settings_from_map(&map)
    } else {
        TemplateSettings::default()
    };

    context.insert("title", "关于 - RustBlog");
    context.insert("name", "Dango");
    context.insert("year", &format_year(&now));
    context.insert("foodes", &settings.foodes);
    context.insert("external_link_warning", &settings.external_link_warning);
    context.insert("external_link_whitelist", &settings.external_link_whitelist);
    context.insert("external_link_warning_text", &settings.external_link_warning_text);
    context.insert("settings", &settings);
    context.insert("switch_notice", &settings.switch_notice);
    context.insert("switch_notice_text", &settings.switch_notice_text);
    context.insert("global_avatar", &settings.global_avatar);

    // Live2D
    context.insert("live2d_enabled", &settings.live2d_enabled);
    context.insert("live2d_show_on_about", &settings.live2d_show_on_about);
    context.insert("live2d_cdn_path", &settings.live2d_cdn_path);
    context.insert("live2d_model_id", &settings.live2d_model_id);
    context.insert("live2d_model_path", &settings.live2d_model_path);
    context.insert("live2d_position", &settings.live2d_position);
    context.insert("live2d_width", &settings.live2d_width);
    context.insert("live2d_height", &settings.live2d_height);

    // 备案信息
    context.insert("beian_enabled", &settings.beian_enabled);
    context.insert("icp_number", &settings.icp_number);
    context.insert("police_record_code", &settings.police_record_code);
    context.insert("police_record_content", &settings.police_record_content);

    context
}

/// 创建友链页面上下文
pub fn create_friends_context() -> TeraContext {
    let mut context = TeraContext::new();
    let now = chrono::Local::now();

    // 一次性批量加载所有设置，构建 TemplateSettings
    let settings = if let Ok(pool) = crate::db::get_db_pool_sync()
        && let Ok(conn) = pool.get()
        && let Ok(map) = crate::db::repositories::SettingRepository::get_all_as_map(&conn)
    {
        build_template_settings_from_map(&map)
    } else {
        TemplateSettings::default()
    };

    context.insert("title", "友链 - RustBlog");
    context.insert("year", &format_year(&now));
    context.insert("foodes", &settings.foodes);
    context.insert("external_link_warning", &settings.external_link_warning);
    context.insert("external_link_whitelist", &settings.external_link_whitelist);
    context.insert("external_link_warning_text", &settings.external_link_warning_text);
    context.insert("global_avatar", &settings.global_avatar);
    context.insert("settings", &settings);

    // 登录状态（前端会通过 API 检查）
    context.insert("is_logged_in", &false);
    context.insert("username", &"");

    // Live2D
    context.insert("live2d_enabled", &settings.live2d_enabled);
    context.insert("live2d_model_path", &settings.live2d_model_path);
    context.insert("live2d_position", &settings.live2d_position);
    context.insert("live2d_width", &settings.live2d_width);
    context.insert("live2d_height", &settings.live2d_height);

    // 备案信息
    context.insert("beian_enabled", &settings.beian_enabled);
    context.insert("icp_number", &settings.icp_number);
    context.insert("police_record_code", &settings.police_record_code);
    context.insert("police_record_content", &settings.police_record_content);

    context
}

/// 创建编辑器上下文
pub fn create_markdown_editor_context() -> TeraContext {
    let mut context = TeraContext::new();
    let now = chrono::Local::now();

    // 一次性批量加载所有设置，构建 TemplateSettings
    let settings = if let Ok(pool) = crate::db::get_db_pool_sync()
        && let Ok(conn) = pool.get()
        && let Ok(map) = crate::db::repositories::SettingRepository::get_all_as_map(&conn)
    {
        build_template_settings_from_map(&map)
    } else {
        TemplateSettings::default()
    };

    context.insert("title", "编辑器 - RustBlog");
    context.insert("name", "Dango");
    context.insert("year", &format_year(&now));
    context.insert("foodes", &settings.foodes);
    context.insert("external_link_warning", &settings.external_link_warning);
    context.insert("external_link_whitelist", &settings.external_link_whitelist);
    context.insert("external_link_warning_text", &settings.external_link_warning_text);
    context.insert("settings", &settings);
    context.insert("switch_notice", &settings.switch_notice);
    context.insert("switch_notice_text", &settings.switch_notice_text);
    context.insert("global_avatar", &settings.global_avatar);

    context
}

/// 创建管理后台上下文
pub fn create_admin_context() -> TeraContext {
    let mut context = TeraContext::new();
    let now = chrono::Local::now();

    // 一次性批量加载所有设置，构建 TemplateSettings
    let settings = if let Ok(pool) = crate::db::get_db_pool_sync()
        && let Ok(conn) = pool.get()
        && let Ok(map) = crate::db::repositories::SettingRepository::get_all_as_map(&conn)
    {
        build_template_settings_from_map(&map)
    } else {
        TemplateSettings::default()
    };

    context.insert("title", "管理后台 - RustBlog");
    context.insert("name", "Dango");
    context.insert("year", &format_year(&now));
    context.insert("foodes", &settings.foodes);
    context.insert("external_link_warning", &settings.external_link_warning);
    context.insert("external_link_whitelist", &settings.external_link_whitelist);
    context.insert("external_link_warning_text", &settings.external_link_warning_text);
    context.insert("settings", &settings);
    context.insert("switch_notice", &settings.switch_notice);
    context.insert("switch_notice_text", &settings.switch_notice_text);
    context.insert("global_avatar", &settings.global_avatar);

    // Live2D
    context.insert("live2d_enabled", &settings.live2d_enabled);
    context.insert("live2d_show_on_admin", &settings.live2d_show_on_admin);
    context.insert("live2d_cdn_path", &settings.live2d_cdn_path);
    context.insert("live2d_model_id", &settings.live2d_model_id);
    context.insert("live2d_model_path", &settings.live2d_model_path);
    context.insert("live2d_position", &settings.live2d_position);
    context.insert("live2d_width", &settings.live2d_width);
    context.insert("live2d_height", &settings.live2d_height);

    context
}
