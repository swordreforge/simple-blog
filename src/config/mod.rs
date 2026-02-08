use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use clap::Parser;

/// 配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    #[serde(default)]
    pub server: Option<ServerConfigFile>,
    #[serde(default)]
    pub database: Option<DatabaseConfigFile>,
    #[serde(default)]
    pub templates: Option<TemplateConfigFile>,
    #[serde(default)]
    pub static_files: Option<StaticConfigFile>,
    #[serde(default)]
    pub geoip: Option<GeoIpConfigFile>,
    #[serde(default)]
    pub tls: Option<TlsConfigFile>,
    #[serde(default)]
    pub logging: Option<LoggingConfigFile>,
    #[serde(default)]
    pub jwt: Option<JwtConfigFile>,
    #[serde(default)]
    pub cache: Option<CacheConfigFile>,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            server: None,
            database: None,
            templates: None,
            static_files: None,
            geoip: None,
            tls: None,
            logging: None,
            jwt: None,
            cache: None,
        }
    }
}

/// 服务器配置（配置文件）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfigFile {
    pub host: Option<String>,
    pub port: Option<u16>,
}

/// 数据库配置（配置文件）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfigFile {
    pub path: Option<String>,
}

/// 模板配置（配置文件）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfigFile {
    pub dir: Option<String>,
    pub cache_enabled: Option<bool>,
}

/// 静态文件配置（配置文件）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticConfigFile {
    pub dir: Option<String>,
    pub cache_max_age: Option<u32>,
}

/// GeoIP 配置（配置文件）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoIpConfigFile {
    pub database_path: Option<String>,
}

/// TLS 配置（配置文件）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfigFile {
    pub enabled: Option<bool>,
    pub cert: Option<String>,
    pub key: Option<String>,
}

/// 日志配置（配置文件）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfigFile {
    pub level: Option<String>,
}

/// JWT配置（配置文件）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfigFile {
    pub secret: Option<String>,
}

/// 缓存配置（配置文件）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfigFile {
    pub enabled: Option<bool>,
    pub backend: Option<String>,
    pub valkey_url: Option<String>,
    pub valkey_pool_size: Option<u32>,
    pub ttl_seconds: Option<u64>,
    pub fallback_to_local: Option<bool>,
}

/// 命令行参数配置
#[derive(Parser, Debug, Clone)]
#[command(name = "rustblog")]
#[command(about = "A simple blog system written in Rust", long_about = None)]
#[command(version = "1.1.4")]
#[command(arg_required_else_help = false)]
pub struct CliArgs {
    /// 配置文件路径 (TOML 格式)
    #[arg(short = 'c', long)]
    pub config: Option<String>,

    /// Port to listen on
    #[arg(short = 'p', long, default_value = "8080")]
    pub port: u16,

    /// Host to bind to
    #[arg(long, default_value = "0.0.0.0")]
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

    /// JWT secret key
    #[arg(long)]
    pub jwt_secret: Option<String>,

    /// Enable cache
    #[arg(long)]
    pub enable_cache: bool,

    /// Cache backend (valkey, local, auto)
    #[arg(long, default_value = "auto")]
    pub cache_backend: String,

    /// Valkey connection URL
    #[arg(long)]
    pub valkey_url: Option<String>,

    /// Cache TTL in seconds
    #[arg(long, default_value = "3600")]
    pub cache_ttl: u64,

    /// Fallback to local cache if Valkey fails
    #[arg(long, default_value = "true")]
    pub cache_fallback: bool,

    /// 基础目录（可执行文件所在目录，自动计算）
    #[arg(skip)]
    pub base_dir: PathBuf,
}

impl Default for CliArgs {
    fn default() -> Self {
        Self {
            config: None,
            port: 8080,
            host: "0.0.0.0".to_string(),
            db_path: "./data/blog.db".to_string(),
            templates_dir: "templates".to_string(),
            static_dir: "static".to_string(),
            log_level: "info".to_string(),
            enable_tls: false,
            tls_cert: None,
            tls_key: None,
            geoip_db_path: "./data/GeoLite2-City.mmdb".to_string(),
            disable_template_cache: false,
            jwt_secret: None,
            enable_cache: false,
            cache_backend: "auto".to_string(),
            valkey_url: None,
            cache_ttl: 3600,
            cache_fallback: true,
            base_dir: PathBuf::from("."),
        }
    }
}

impl CliArgs {
    /// 从配置文件加载配置
    pub fn load_from_config_file(path: &str) -> Result<ConfigFile, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: ConfigFile = toml::from_str(&content)?;
        Ok(config)
    }

    /// 合并配置文件和命令行参数（命令行参数优先）
    pub fn merge_with_config(&mut self, config: ConfigFile) {
        // 服务器配置
        if let Some(server) = config.server {
            if let Some(host) = server.host {
                self.host = host;
            }
            if let Some(port) = server.port {
                self.port = port;
            }
        }

        // 数据库配置
        if let Some(database) = config.database {
            if let Some(path) = database.path {
                self.db_path = path;
            }
        }

        // 模板配置
        if let Some(templates) = config.templates {
            if let Some(dir) = templates.dir {
                self.templates_dir = dir;
            }
            if let Some(cache_enabled) = templates.cache_enabled {
                self.disable_template_cache = !cache_enabled;
            }
        }

        // 静态文件配置
        if let Some(static_files) = config.static_files {
            if let Some(dir) = static_files.dir {
                self.static_dir = dir;
            }
        }

        // GeoIP 配置
        if let Some(geoip) = config.geoip {
            if let Some(database_path) = geoip.database_path {
                self.geoip_db_path = database_path;
            }
        }

        // TLS 配置
        if let Some(tls) = config.tls {
            if let Some(enabled) = tls.enabled {
                self.enable_tls = enabled;
            }
            if let Some(cert) = tls.cert {
                self.tls_cert = Some(cert);
            }
            if let Some(key) = tls.key {
                self.tls_key = Some(key);
            }
        }

        // 日志配置
        if let Some(logging) = config.logging {
            if let Some(level) = logging.level {
                self.log_level = level;
            }
        }

        // JWT 配置
        if let Some(jwt) = config.jwt {
            if let Some(secret) = jwt.secret {
                self.jwt_secret = Some(secret);
            }
        }

        // 缓存配置
        if let Some(cache) = config.cache {
            if let Some(enabled) = cache.enabled {
                self.enable_cache = enabled;
            }
            if let Some(backend) = cache.backend {
                self.cache_backend = backend;
            }
            if let Some(valkey_url) = cache.valkey_url {
                self.valkey_url = Some(valkey_url);
            }
            if let Some(ttl) = cache.ttl_seconds {
                self.cache_ttl = ttl;
            }
            if let Some(fallback) = cache.fallback_to_local {
                self.cache_fallback = fallback;
            }
        }
    }

    /// 将相对路径转换为绝对路径
    pub fn resolve_paths(&mut self) {
        // 获取基础目录
        self.base_dir = if let Ok(exe_path) = std::env::current_exe() {
            let exe_dir = exe_path.parent().unwrap_or_else(|| Path::new("."));
            let exe_dir = exe_dir.to_path_buf();

            // 检查可执行文件所在目录是否包含 Cargo.toml
            let has_cargo_toml = exe_dir.join("Cargo.toml").exists();

            if has_cargo_toml {
                // 如果可执行文件所在目录有 Cargo.toml，说明是开发环境
                println!("🔍 检测到开发环境 (Cargo.toml 存在)");
                exe_dir
            } else {
                // 如果没有 Cargo.toml，说明是生产环境（静态编译的部署）
                println!("🔍 检测到生产环境，使用可执行文件所在目录作为基准");
                exe_dir
            }
        } else {
            // 无法获取可执行文件路径，使用当前工作目录
            println!("⚠️  无法获取可执行文件路径，使用当前工作目录");
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"))
        };

        println!("📁 基础目录: {}", self.base_dir.display());

        // 数据库路径
        self.db_path = Self::make_absolute(&self.base_dir, &self.db_path);

        // 模板目录
        self.templates_dir = Self::make_absolute(&self.base_dir, &self.templates_dir);

        // 静态文件目录
        self.static_dir = Self::make_absolute(&self.base_dir, &self.static_dir);

        // GeoIP 数据库路径
        self.geoip_db_path = Self::make_absolute(&self.base_dir, &self.geoip_db_path);

        // TLS 证书和密钥
        if let Some(ref mut cert) = self.tls_cert {
            *cert = Self::make_absolute(&self.base_dir, cert.as_str());
        }
        if let Some(ref mut key) = self.tls_key {
            *key = Self::make_absolute(&self.base_dir, key.as_str());
        }
    }

    /// 获取基础目录
    pub fn get_base_dir(&self) -> &PathBuf {
        &self.base_dir
    }

    /// 将路径转换为绝对路径
    fn make_absolute(base: &PathBuf, path: &str) -> String {
        let path_buf = PathBuf::from(path);
        let is_relative = path.starts_with('.') || !path_buf.is_absolute();

        if is_relative {
            let abs_path = base.join(path);
            // 规范化路径，移除 ./ 和 ..
            let canonical = if abs_path.exists() {
                abs_path.canonicalize().unwrap_or(abs_path)
            } else {
                // 对于不存在的路径，手动规范化
                std::path::absolute(&abs_path).unwrap_or(abs_path)
            };
            canonical.to_string_lossy().to_string()
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
            host: "0.0.0.0".to_string(),
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