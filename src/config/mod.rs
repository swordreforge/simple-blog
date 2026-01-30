use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use clap::Parser;

/// 命令行参数配置
#[derive(Parser, Debug, Clone)]
#[command(name = "rustblog")]
#[command(about = "A simple blog system written in Rust", long_about = None)]
#[command(version)]
pub struct CliArgs {
    /// Port to listen on
    #[arg(short, long, default_value = "8080")]
    pub port: u16,

    /// Host to bind to
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,

    /// Database file path (SQLite)
    #[arg(short = 'd', long, default_value = "./data/blog.db")]
    pub db_path: String,

    /// Template directory
    #[arg(short = 't', long, default_value = "templates")]
    pub templates_dir: String,

    /// Static files directory
    #[arg(short = 's', long, default_value = "static")]
    pub static_dir: String,

    /// Log level (debug, info, warn, error)
    #[arg(short = 'l', long, default_value = "info")]
    pub log_level: String,

    /// Enable TLS (HTTPS)
    #[arg(long)]
    pub enable_tls: bool,

    /// Path to TLS certificate file
    #[arg(long)]
    pub tls_cert: Option<String>,

    /// Path to TLS private key file
    #[arg(long)]
    pub tls_key: Option<String>,

    /// GeoIP database file path
    #[arg(long, default_value = "./data/GeoLite2-City.mmdb")]
    pub geoip_db_path: String,

    /// Disable template caching
    #[arg(long)]
    pub disable_template_cache: bool,
}

impl CliArgs {
    /// 将相对路径转换为绝对路径
    pub fn resolve_paths(&mut self) {
        // 尝试找到项目根目录
        // 策略：从可执行文件目录开始，向上查找包含 Cargo.toml 的目录
        let base_dir = if let Ok(exe_path) = std::env::current_exe() {
            let exe_dir = exe_path.parent().unwrap_or_else(|| Path::new("."));
            let mut current = exe_dir.to_path_buf();
            
            // 向上查找项目根目录（包含 Cargo.toml）
            let mut found = false;
            for _ in 0..10 { // 最多向上查找 10 层
                if current.join("Cargo.toml").exists() {
                    found = true;
                    break;
                }
                if current.parent().is_none() {
                    break;
                }
                current = current.parent().unwrap().to_path_buf();
            }
            
            if found {
                current
            } else {
                // 如果找不到，使用可执行文件目录
                exe_dir.to_path_buf()
            }
        } else {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"))
        };

        // 数据库路径
        self.db_path = Self::make_absolute(&base_dir, &self.db_path);
        
        // 模板目录
        self.templates_dir = Self::make_absolute(&base_dir, &self.templates_dir);
        
        // 静态文件目录
        self.static_dir = Self::make_absolute(&base_dir, &self.static_dir);
        
        // GeoIP 数据库路径
        self.geoip_db_path = Self::make_absolute(&base_dir, &self.geoip_db_path);

        // TLS 证书和密钥
        if let Some(ref mut cert) = self.tls_cert {
            *cert = Self::make_absolute(&base_dir, cert.as_str());
        }
        if let Some(ref mut key) = self.tls_key {
            *key = Self::make_absolute(&base_dir, key.as_str());
        }
    }

    /// 将路径转换为绝对路径
    fn make_absolute(base: &PathBuf, path: &str) -> String {
        let path_buf = PathBuf::from(path);
        let is_relative = path.starts_with('.') || !path_buf.is_absolute();
        
        if is_relative {
            let abs_path = base.join(path);
            let path_str = abs_path.to_string_lossy().to_string();
            // 移除开头的 ./
            if path_str.starts_with("./") {
                path_str[2..].to_string()
            } else {
                path_str
            }
        } else {
            path.to_string()
        }
    }
}

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

impl AppConfig {
    /// 从命令行参数创建配置
    pub fn from_cli(args: CliArgs) -> Self {
        Self {
            server: ServerConfig {
                host: args.host,
                port: args.port,
            },
            templates: TemplateConfig {
                dir: args.templates_dir,
                cache_enabled: !args.disable_template_cache,
            },
            static_files: StaticConfig {
                dir: args.static_dir,
                cache_max_age: 86400,
            },
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