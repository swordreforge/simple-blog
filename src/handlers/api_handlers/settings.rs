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
    match crate::templates::load_appearance_settings() {
        Ok(settings) => {
            HttpResponse::Ok().json(settings)
        }
        Err(e) => {
            eprintln!("Failed to load appearance settings: {}", e);
            // 返回默认设置
            HttpResponse::Ok().json(crate::templates::AppearanceSettings::default())
        }
    }
}

/// 更新外观设置
pub async fn update_appearance(req: web::Json<serde_json::Value>) -> impl Responder {
    let updates = req.into_inner();
    
    // 获取数据库连接池
    let pool = match crate::db::get_db_pool().await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to get database pool: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "数据库连接失败"
            }));
        }
    };
    
    // 更新设置到数据库
    if let Ok(conn) = pool.get() {
        for (key, value) in updates.as_object().unwrap_or(&serde_json::Map::new()) {
            let key_str = key.as_str();
            let value_str = match value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Bool(b) => b.to_string(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Array(arr) => {
                    // 将数组转换为 JSON 字符串
                    if let Ok(json_str) = serde_json::to_string(arr) {
                        json_str
                    } else {
                        // 如果 JSON 序列化失败，用逗号连接
                        arr.iter()
                            .map(|v| v.as_str().unwrap_or(""))
                            .collect::<Vec<_>>()
                            .join(",")
                    }
                }
                _ => continue,
            };
            
            // 创建或更新设置
            let setting = crate::db::models::Setting {
                id: None,
                key: key_str.to_string(),
                value: value_str,
                r#type: "string".to_string(),
                description: Some("Appearance setting".to_string()),
                category: "appearance".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            
            if let Err(e) = crate::db::repositories::SettingRepository::set(&conn, &setting) {
                eprintln!("Failed to update setting {}: {}", key_str, e);
            }
        }
    }
    
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "外观设置已更新"
    }))
}

/// 获取音乐设置
pub async fn get_music() -> impl Responder {
    // 获取数据库连接池
    let pool = match crate::db::get_db_pool().await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to get database pool: {}", e);
            // 返回默认设置
            return HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "enabled": false,
                "auto_play": false,
                "volume": 0.7,
                "loop": true,
                "control_size": "medium",
                "player_color": "rgba(66, 133, 244, 0.9)",
                "position": "bottom-right",
                "custom_css": ""
            }));
        }
    };
    
    // 从数据库加载设置
    if let Ok(conn) = pool.get() {
        let keys = vec![
            "music_enabled", "music_auto_play", "music_volume", "music_loop",
            "music_control_size", "music_player_color", "music_position", "music_custom_css"
        ];
        
        let mut settings = serde_json::Map::new();
        
        for key in keys {
            if let Ok(Some(setting)) = crate::db::repositories::SettingRepository::get(&conn, key) {
                // 转换键名以匹配前端期望的格式
                let json_key = key.strip_prefix("music_").unwrap_or(key);
                let json_value: serde_json::Value = match key {
                    "music_enabled" | "music_auto_play" | "music_loop" => {
                        serde_json::Value::Bool(setting.value == "true")
                    }
                    "music_volume" => {
                        serde_json::Value::Number(
                            serde_json::Number::from_f64(setting.value.parse().unwrap_or(0.7))
                                .unwrap_or(serde_json::Number::from(0))
                        )
                    }
                    _ => serde_json::Value::String(setting.value),
                };
                settings.insert(json_key.to_string(), json_value);
            }
        }
        
        return HttpResponse::Ok().json(settings);
    }
    
    // 返回默认设置
    HttpResponse::Ok().json(serde_json::json!({
        "enabled": false,
        "auto_play": false,
        "volume": 0.7,
        "loop": true,
        "control_size": "medium",
        "player_color": "rgba(66, 133, 244, 0.9)",
        "position": "bottom-right",
        "custom_css": ""
    }))
}

/// 更新音乐设置
pub async fn update_music(req: web::Json<serde_json::Value>) -> impl Responder {
    let updates = req.into_inner();
    
    // 获取数据库连接池
    let pool = match crate::db::get_db_pool().await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to get database pool: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "数据库连接失败"
            }));
        }
    };
    
    // 键名映射：前端键名 -> 数据库键名
    let key_mapping: std::collections::HashMap<&str, &str> = [
        ("enabled", "music_enabled"),
        ("auto_play", "music_auto_play"),
        ("volume", "music_volume"),
        ("loop", "music_loop"),
        ("control_size", "music_control_size"),
        ("player_color", "music_player_color"),
        ("position", "music_position"),
        ("custom_css", "music_custom_css"),
    ].iter().cloned().collect();
    
    // 更新设置到数据库
    if let Ok(conn) = pool.get() {
        for (key, value) in updates.as_object().unwrap_or(&serde_json::Map::new()) {
            let frontend_key = key.as_str();
            let db_key = key_mapping.get(frontend_key).unwrap_or(&frontend_key);
            
            let value_str = match value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Bool(b) => b.to_string(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Array(arr) => {
                    // 将数组转换为 JSON 字符串
                    if let Ok(json_str) = serde_json::to_string(arr) {
                        json_str
                    } else {
                        // 如果 JSON 序列化失败，用逗号连接
                        arr.iter()
                            .map(|v| v.as_str().unwrap_or(""))
                            .collect::<Vec<_>>()
                            .join(",")
                    }
                }
                _ => continue,
            };
            
            // 创建或更新设置
            let setting = crate::db::models::Setting {
                id: None,
                key: db_key.to_string(),
                value: value_str,
                r#type: "string".to_string(),
                description: Some("Music setting".to_string()),
                category: "music".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            
            if let Err(e) = crate::db::repositories::SettingRepository::set(&conn, &setting) {
                eprintln!("Failed to update music setting {}: {}", db_key, e);
            }
        }
    }
    
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "音乐设置已更新"
    }))
}

/// 部分更新音乐设置
pub async fn update_music_partial(req: web::Json<serde_json::Value>) -> impl Responder {
    let updates = req.into_inner();
    
    // 获取数据库连接池
    let pool = match crate::db::get_db_pool().await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to get database pool: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "数据库连接失败"
            }));
        }
    };
    
    // 键名映射：前端键名 -> 数据库键名
    let key_mapping: std::collections::HashMap<&str, &str> = [
        ("enabled", "music_enabled"),
        ("auto_play", "music_auto_play"),
        ("volume", "music_volume"),
        ("loop", "music_loop"),
        ("control_size", "music_control_size"),
        ("player_color", "music_player_color"),
        ("position", "music_position"),
        ("custom_css", "music_custom_css"),
    ].iter().cloned().collect();
    
    // 更新设置到数据库
    if let Ok(conn) = pool.get() {
        for (key, value) in updates.as_object().unwrap_or(&serde_json::Map::new()) {
            let frontend_key = key.as_str();
            let db_key = key_mapping.get(frontend_key).unwrap_or(&frontend_key);
            
            let value_str = match value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Bool(b) => b.to_string(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Array(arr) => {
                    // 将数组转换为 JSON 字符串
                    if let Ok(json_str) = serde_json::to_string(arr) {
                        json_str
                    } else {
                        // 如果 JSON 序列化失败，用逗号连接
                        arr.iter()
                            .map(|v| v.as_str().unwrap_or(""))
                            .collect::<Vec<_>>()
                            .join(",")
                    }
                }
                _ => continue,
            };
            
            // 创建或更新设置
            let setting = crate::db::models::Setting {
                id: None,
                key: db_key.to_string(),
                value: value_str,
                r#type: "string".to_string(),
                description: Some("Music setting".to_string()),
                category: "music".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            
            if let Err(e) = crate::db::repositories::SettingRepository::set(&conn, &setting) {
                eprintln!("Failed to update music setting {}: {}", db_key, e);
            }
        }
    }
    
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "音乐设置已更新"
    }))
}

/// 获取模板设置
pub async fn get_template() -> HttpResponse {
    match crate::templates::load_template_settings() {
        Ok(settings) => {
            HttpResponse::Ok().json(settings)
        }
        Err(e) => {
            eprintln!("Failed to load template settings: {}", e);
            // 返回默认设置
            HttpResponse::Ok().json(crate::templates::TemplateSettings::default())
        }
    }
}

/// 更新模板设置
pub async fn update_template(req: web::Json<serde_json::Value>) -> HttpResponse {
    let updates = req.into_inner();
    
    // 获取数据库连接池
    let pool = match crate::db::get_db_pool().await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to get database pool: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "数据库连接失败"
            }));
        }
    };
    
    // 键名映射：前端键名 -> 数据库键名
    let key_mapping: std::collections::HashMap<&str, &str> = [
        ("name", "template_name"),
        ("greting", "template_greting"),
        ("year", "template_year"),
        ("foodes", "template_foods"),
        ("global_avatar", "global_avatar"),
        ("article_title", "template_article_title"),
        ("article_title_prefix", "template_article_title_prefix"),
        ("switch_notice", "template_switch_notice"),
        ("switch_notice_text", "template_switch_notice_text"),
        ("external_link_warning", "external_link_warning"),
        ("external_link_whitelist", "external_link_whitelist"),
        ("external_link_warning_text", "external_link_warning_text"),
        ("live2d_enabled", "live2d_enabled"),
        ("live2d_show_on_index", "live2d_show_on_index"),
        ("live2d_show_on_passage", "live2d_show_on_passage"),
        ("live2d_show_on_collect", "live2d_show_on_collect"),
        ("live2d_show_on_about", "live2d_show_on_about"),
        ("live2d_show_on_admin", "live2d_show_on_admin"),
        ("live2d_model_id", "live2d_model_id"),
        ("live2d_model_path", "live2d_model_path"),
        ("live2d_cdn_path", "live2d_cdn_path"),
        ("live2d_position", "live2d_position"),
        ("live2d_width", "live2d_width"),
        ("live2d_height", "live2d_height"),
        ("sponsor_enabled", "sponsor_enabled"),
        ("sponsor_title", "sponsor_title"),
        ("sponsor_image", "sponsor_image"),
        ("sponsor_description", "sponsor_description"),
        ("sponsor_button_text", "sponsor_button_text"),
        ("beian_enabled", "beian_enabled"),
        ("icp_number", "icp_number"),
        ("police_record_code", "police_record_code"),
        ("police_record_content", "police_record_content"),
    ].iter().cloned().collect();
    
    // 更新设置到数据库
    if let Ok(conn) = pool.get() {
        for (key, value) in updates.as_object().unwrap_or(&serde_json::Map::new()) {
            let frontend_key = key.as_str();
            let db_key = key_mapping.get(frontend_key).unwrap_or(&frontend_key);
            
            let value_str = match value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Bool(b) => b.to_string(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Array(arr) => {
                    // 将数组转换为 JSON 字符串
                    if let Ok(json_str) = serde_json::to_string(arr) {
                        json_str
                    } else {
                        // 如果 JSON 序列化失败，用逗号连接
                        arr.iter()
                            .map(|v| v.as_str().unwrap_or(""))
                            .collect::<Vec<_>>()
                            .join(",")
                    }
                }
                _ => continue,
            };
            
            // 创建或更新设置
            let setting = crate::db::models::Setting {
                id: None,
                key: db_key.to_string(),
                value: value_str,
                r#type: "string".to_string(),
                description: Some("Template setting".to_string()),
                category: "template".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            
            if let Err(e) = crate::db::repositories::SettingRepository::set(&conn, &setting) {
                eprintln!("Failed to update template setting {}: {}", db_key, e);
            }
        }
    }
    
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "模板设置已更新"
    }))
}

/// 更新设置（通用）
pub async fn update() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Settings updated"
    }))
}

/// 更新单个设置
pub async fn update_single(req: web::Json<serde_json::Value>) -> impl Responder {
    let updates = req.into_inner();
    
    // 获取 key 和 value
    let key = updates.get("key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "message": "Key is required"
        })));
    
    if key.is_err() {
        return key.unwrap_err();
    }
    
    let key = key.unwrap();
    
    let value = updates.get("value")
        .and_then(|v| v.as_str())
        .ok_or_else(|| HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "message": "Value is required"
        })));
    
    if value.is_err() {
        return value.unwrap_err();
    }
    
    let value = value.unwrap();
    
    // 获取数据库连接池
    let pool = match crate::db::get_db_pool().await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to get database pool: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "数据库连接失败"
            }));
        }
    };
    
    // 更新设置到数据库
    if let Ok(conn) = pool.get() {
        let setting = crate::db::models::Setting {
            id: None,
            key: key.to_string(),
            value: value.to_string(),
            r#type: "string".to_string(),
            description: None,
            category: "template".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        if let Err(e) = crate::db::repositories::SettingRepository::set(&conn, &setting) {
            eprintln!("Failed to update setting {}: {}", key, e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "更新设置失败"
            }));
        }
    }
    
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "设置已更新"
    }))
}