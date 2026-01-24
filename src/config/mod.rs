use serde::{Deserialize, Serialize};

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub templates: TemplateConfig,
    pub static_files: StaticConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            templates: TemplateConfig::default(),
            static_files: StaticConfig::default(),
        }
    }
}

/// 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
        }
    }
}

/// 模板配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    pub dir: String,
    pub cache_enabled: bool,
}

impl Default for TemplateConfig {
    fn default() -> Self {
        Self {
            dir: "templates".to_string(),
            cache_enabled: true,
        }
    }
}

/// 静态文件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticConfig {
    pub dir: String,
    pub cache_max_age: u32,
}

impl Default for StaticConfig {
    fn default() -> Self {
        Self {
            dir: "static".to_string(),
            cache_max_age: 86400, // 24小时
        }
    }
}