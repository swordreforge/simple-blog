use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// 配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

/// 服务器配置（配置文件）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfigFile {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub workers: Option<usize>,
    pub keep_alive: Option<u64>,
    pub keep_alive_timeout: Option<u64>,
    pub client_timeout: Option<u64>,
    pub client_disconnect_timeout: Option<u64>,
    pub max_connections: Option<usize>,
    pub max_connection_rate: Option<usize>,
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

    /// Number of worker threads (default: number of CPU cores)
    #[arg(long)]
    pub workers: Option<usize>,

    /// Keep-alive timeout in seconds (default: 75)
    #[arg(long)]
    pub keep_alive: Option<u64>,

    /// Keep-alive connection timeout in seconds (default: 30)
    #[arg(long)]
    pub keep_alive_timeout: Option<u64>,

    /// Client request timeout in seconds (default: 30)
    #[arg(long)]
    pub client_timeout: Option<u64>,

    /// Client disconnect timeout in seconds (default: 5)
    #[arg(long)]
    pub client_disconnect_timeout: Option<u64>,

    /// Maximum number of concurrent connections (default: 10000)
    #[arg(long)]
    pub max_connections: Option<usize>,

    /// Maximum connection rate per second (default: 256)
    #[arg(long)]
    pub max_connection_rate: Option<usize>,

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

    /// Clear all cache on startup
    #[arg(long)]
    pub clear_cache: bool,

    /// Enable performance profiling (requires --features profiling)
    #[arg(long)]
    pub enable_profiling: bool,

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
            workers: None,
            keep_alive: None,
            keep_alive_timeout: None,
            client_timeout: None,
            client_disconnect_timeout: None,
            max_connections: None,
            max_connection_rate: None,
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
            clear_cache: false,
            enable_profiling: false,
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
            if let Some(workers) = server.workers {
                self.workers = Some(workers);
            }
            if let Some(keep_alive) = server.keep_alive {
                self.keep_alive = Some(keep_alive);
            }
            if let Some(keep_alive_timeout) = server.keep_alive_timeout {
                self.keep_alive_timeout = Some(keep_alive_timeout);
            }
            if let Some(client_timeout) = server.client_timeout {
                self.client_timeout = Some(client_timeout);
            }
            if let Some(client_disconnect_timeout) = server.client_disconnect_timeout {
                self.client_disconnect_timeout = Some(client_disconnect_timeout);
            }
            if let Some(max_connections) = server.max_connections {
                self.max_connections = Some(max_connections);
            }
            if let Some(max_connection_rate) = server.max_connection_rate {
                self.max_connection_rate = Some(max_connection_rate);
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
    fn make_absolute(base: &Path, path: &str) -> String {
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub templates: TemplateConfig,
    pub static_files: StaticConfig,
}

impl AppConfig {
    /// 从命令行参数创建配置
    pub fn from_cli(args: CliArgs) -> Self {
        Self {
            server: ServerConfig {
                host: args.host.clone(),
                port: args.port,
                workers: args.workers,
                keep_alive: args.keep_alive,
                keep_alive_timeout: args.keep_alive_timeout,
                client_timeout: args.client_timeout,
                client_disconnect_timeout: args.client_disconnect_timeout,
                max_connections: args.max_connections,
                max_connection_rate: args.max_connection_rate,
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
    pub workers: Option<usize>,
    pub keep_alive: Option<u64>,
    pub keep_alive_timeout: Option<u64>,
    pub client_timeout: Option<u64>,
    pub client_disconnect_timeout: Option<u64>,
    pub max_connections: Option<usize>,
    pub max_connection_rate: Option<usize>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            workers: None,
            keep_alive: None,
            keep_alive_timeout: None,
            client_timeout: None,
            client_disconnect_timeout: None,
            max_connections: None,
            max_connection_rate: None,
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

/// 配置验证错误
#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum ConfigValidationError {
    #[error("端口 {0} 无效，必须在 1-65535 范围内")]
    InvalidPort(u16),

    #[error("主机地址 '{0}' 无效")]
    InvalidHost(String),

    #[error("Worker 数量 {0} 无效，必须 >= 1")]
    InvalidWorkers(usize),

    #[error("数据库路径 '{0}' 不存在或无法访问")]
    DatabasePathNotFound(String),

    #[error("模板目录 '{0}' 不存在或无法访问")]
    TemplateDirNotFound(String),

    #[error("静态文件目录 '{0}' 不存在或无法访问")]
    StaticDirNotFound(String),

    #[error("GeoIP 数据库 '{0}' 不存在或无法访问")]
    GeoIpDatabaseNotFound(String),

    #[error("TLS 证书 '{0}' 不存在")]
    TlsCertNotFound(String),

    #[error("TLS 私钥 '{0}' 不存在")]
    TlsKeyNotFound(String),

    #[error("启用 TLS 但未提供证书和私钥")]
    TlsMissingCredentials,

    #[error("日志级别 '{0}' 无效，必须是 debug, info, warn, 或 error")]
    InvalidLogLevel(String),

    #[error("JWT 密钥太短，至少需要 32 字符")]
    JwtSecretTooShort,

    #[error("缓存后端 '{0}' 无效，必须是 valkey, local, 或 auto")]
    InvalidCacheBackend(String),

    #[error("启用 Valkey 缓存但未提供连接 URL")]
    ValkeyUrlMissing,

    #[error("缓存 TTL {0} 无效，必须 >= 1")]
    InvalidCacheTtl(u64),

    #[error("配置文件 '{0}' 不存在或无法读取")]
    ConfigFileNotFound(String),

    #[error("配置文件格式错误: {0}")]
    ConfigFileFormatError(String),
}

/// 配置验证结果
#[derive(Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ConfigValidationError>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: ConfigValidationError) {
        self.is_valid = false;
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    pub fn is_valid(&self) -> bool {
        self.is_valid
    }
}

impl CliArgs {
    /// 验证配置
    pub fn validate(&self) -> ValidationResult {
        let mut result = ValidationResult::new();

        // 验证端口 (u16 max is 65535, so only need to check lower bound)
        if self.port < 1 {
            result.add_error(ConfigValidationError::InvalidPort(self.port));
        }

        // 验证主机地址
        if !self.is_valid_host(&self.host) {
            result.add_error(ConfigValidationError::InvalidHost(self.host.clone()));
        }

        // 验证 Worker 数量
        if let Some(workers) = self.workers {
            if workers < 1 {
                result.add_error(ConfigValidationError::InvalidWorkers(workers));
            }
        }

        // 验证数据库路径
        if !Path::new(&self.db_path)
            .parent()
            .is_some_and(|p| p.exists())
        {
            // 检查父目录是否存在
            let parent = Path::new(&self.db_path).parent().unwrap_or(Path::new("."));
            if !parent.exists() {
                result.add_error(ConfigValidationError::DatabasePathNotFound(
                    parent.to_string_lossy().to_string(),
                ));
            }
        }

        // 验证模板目录
        if !Path::new(&self.templates_dir).exists() {
            result.add_warning(format!(
                "模板目录 '{}' 不存在，将使用嵌入的模板",
                self.templates_dir
            ));
        }

        // 验证静态文件目录
        if !Path::new(&self.static_dir).exists() {
            result.add_warning(format!(
                "静态文件目录 '{}' 不存在，将使用嵌入的静态文件",
                self.static_dir
            ));
        }

        // 验证 GeoIP 数据库
        if !Path::new(&self.geoip_db_path).exists() {
            result.add_warning(format!(
                "GeoIP 数据库 '{}' 不存在，地理位置查询将返回 'unknown'",
                self.geoip_db_path
            ));
        }

        // 验证 TLS 配置
        if self.enable_tls {
            if self.tls_cert.is_none() || self.tls_key.is_none() {
                result.add_error(ConfigValidationError::TlsMissingCredentials);
            } else {
                if let Some(ref cert) = self.tls_cert {
                    if !Path::new(cert).exists() {
                        result.add_error(ConfigValidationError::TlsCertNotFound(cert.clone()));
                    }
                }
                if let Some(ref key) = self.tls_key {
                    if !Path::new(key).exists() {
                        result.add_error(ConfigValidationError::TlsKeyNotFound(key.clone()));
                    }
                }
            }
        }

        // 验证日志级别
        if !matches!(self.log_level.as_str(), "debug" | "info" | "warn" | "error") {
            result.add_error(ConfigValidationError::InvalidLogLevel(
                self.log_level.clone(),
            ));
        }

        // 验证 JWT 密钥
        if let Some(ref secret) = self.jwt_secret {
            if secret.len() < 32 {
                result.add_error(ConfigValidationError::JwtSecretTooShort);
            }
        }

        // 验证缓存配置
        if self.enable_cache {
            if !matches!(self.cache_backend.as_str(), "valkey" | "local" | "auto") {
                result.add_error(ConfigValidationError::InvalidCacheBackend(
                    self.cache_backend.clone(),
                ));
            }

            if (self.cache_backend == "valkey"
                || (self.cache_backend == "auto" && self.valkey_url.is_some()))
                && self.valkey_url.is_none()
            {
                result.add_error(ConfigValidationError::ValkeyUrlMissing);
            }

            if self.cache_ttl < 1 {
                result.add_error(ConfigValidationError::InvalidCacheTtl(self.cache_ttl));
            }
        }

        // 验证超时配置
        if let Some(keep_alive) = self.keep_alive {
            if keep_alive < 1 {
                result.add_warning("Keep-alive 超时值过小，建议至少 30 秒".to_string());
            }
        }

        if let Some(keep_alive_timeout) = self.keep_alive_timeout {
            if keep_alive_timeout < 1 {
                result.add_warning("Keep-alive 连接超时值过小，建议至少 30 秒".to_string());
            }
        }

        if let Some(client_timeout) = self.client_timeout {
            if client_timeout < 1 {
                result.add_warning("客户端请求超时值过小，建议至少 30 秒".to_string());
            }
        }

        // 验证最大连接数
        if let Some(max_connections) = self.max_connections {
            if max_connections < 10 {
                result.add_warning("最大连接数过小，建议至少 100".to_string());
            }
        }

        result
    }

    /// 验证主机地址
    fn is_valid_host(&self, host: &str) -> bool {
        if host.is_empty() {
            return false;
        }

        // 检查是否是 "0.0.0.0"
        if host == "0.0.0.0" {
            return true;
        }

        // 检查是否是 "::"
        if host == "::" {
            return true;
        }

        // 检查是否是 localhost
        if host == "localhost" {
            return true;
        }

        // 检查是否是有效的 IPv4 地址
        if host.parse::<std::net::Ipv4Addr>().is_ok() {
            return true;
        }

        // 检查是否是有效的 IPv6 地址
        if host.parse::<std::net::Ipv6Addr>().is_ok() {
            return true;
        }

        // 检查是否是有效的域名
        if host.contains('.') {
            let parts: Vec<&str> = host.split('.').collect();
            if parts.len() >= 2 && parts.iter().all(|p| !p.is_empty()) {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation_valid() {
        // 创建临时目录以通过数据库路径验证
        let temp_dir = std::env::temp_dir().join("rustblog_test");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let db_path = temp_dir.join("test.db").to_string_lossy().to_string();

        let args = CliArgs {
            port: 8080,
            host: "0.0.0.0".to_string(),
            workers: Some(4),
            log_level: "info".to_string(),
            enable_cache: true,
            cache_backend: "local".to_string(),
            cache_ttl: 3600,
            db_path,
            ..Default::default()
        };

        let result = args.validate();
        assert!(result.is_valid());
        assert!(result.errors.is_empty());

        // 清理临时目录
        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_config_validation_invalid_port() {
        let args = CliArgs {
            port: 0,
            ..Default::default()
        };

        let result = args.validate();
        assert!(!result.is_valid());
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_config_validation_invalid_host() {
        let args = CliArgs {
            host: "".to_string(),
            ..Default::default()
        };

        let result = args.validate();
        assert!(!result.is_valid());
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_config_validation_invalid_workers() {
        let args = CliArgs {
            workers: Some(0),
            ..Default::default()
        };

        let result = args.validate();
        assert!(!result.is_valid());
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_config_validation_tls_missing_credentials() {
        let args = CliArgs {
            enable_tls: true,
            tls_cert: None,
            tls_key: None,
            ..Default::default()
        };

        let result = args.validate();
        assert!(!result.is_valid());
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_config_validation_invalid_log_level() {
        let args = CliArgs {
            log_level: "invalid".to_string(),
            ..Default::default()
        };

        let result = args.validate();
        assert!(!result.is_valid());
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_config_validation_jwt_secret_too_short() {
        let args = CliArgs {
            jwt_secret: Some("short".to_string()),
            ..Default::default()
        };

        let result = args.validate();
        assert!(!result.is_valid());
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_config_validation_invalid_cache_backend() {
        let args = CliArgs {
            enable_cache: true,
            cache_backend: "invalid".to_string(),
            ..Default::default()
        };

        let result = args.validate();
        assert!(!result.is_valid());
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_config_validation_valkey_url_missing() {
        let args = CliArgs {
            enable_cache: true,
            cache_backend: "valkey".to_string(),
            valkey_url: None,
            ..Default::default()
        };

        let result = args.validate();
        assert!(!result.is_valid());
        assert!(!result.errors.is_empty());
    }
}
