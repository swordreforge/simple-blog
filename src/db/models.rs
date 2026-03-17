use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::fmt;

/// 文章状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PassageStatus {
    /// 已发布
    Published,
    /// 草稿
    Draft,
    /// 已归档
    Archived,
    /// 已删除（软删除）
    Deleted,
}

impl fmt::Display for PassageStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PassageStatus::Published => write!(f, "published"),
            PassageStatus::Draft => write!(f, "draft"),
            PassageStatus::Archived => write!(f, "archived"),
            PassageStatus::Deleted => write!(f, "deleted"),
        }
    }
}

impl AsRef<str> for PassageStatus {
    fn as_ref(&self) -> &str {
        match self {
            PassageStatus::Published => "published",
            PassageStatus::Draft => "draft",
            PassageStatus::Archived => "archived",
            PassageStatus::Deleted => "deleted",
        }
    }
}

impl PassageStatus {
    /// 从字符串解析状态
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "published" => Some(PassageStatus::Published),
            "draft" => Some(PassageStatus::Draft),
            "archived" => Some(PassageStatus::Archived),
            "deleted" => Some(PassageStatus::Deleted),
            _ => None,
        }
    }

    /// 检查是否为已发布状态
    #[inline]
    pub fn is_published(&self) -> bool {
        matches!(self, PassageStatus::Published)
    }

    /// 检查是否可见（已发布）
    #[allow(dead_code)]
    #[inline]
    pub fn is_visible(&self) -> bool {
        matches!(self, PassageStatus::Published)
    }
}

// 为数据库实现 ToSql 和 FromSql
impl rusqlite::types::ToSql for PassageStatus {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        self.as_ref().to_sql()
    }
}

impl rusqlite::types::FromSql for PassageStatus {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let s: String = rusqlite::types::FromSql::column_result(value)?;
        Self::from_str(&s)
            .ok_or_else(|| rusqlite::types::FromSqlError::InvalidType)
    }
}

/// 文章可见性枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PassageVisibility {
    /// 公开
    Public,
    /// 私有
    Private,
    /// 受密码保护
    Protected,
}

impl fmt::Display for PassageVisibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PassageVisibility::Public => write!(f, "public"),
            PassageVisibility::Private => write!(f, "private"),
            PassageVisibility::Protected => write!(f, "protected"),
        }
    }
}

impl AsRef<str> for PassageVisibility {
    fn as_ref(&self) -> &str {
        match self {
            PassageVisibility::Public => "public",
            PassageVisibility::Private => "private",
            PassageVisibility::Protected => "protected",
        }
    }
}

impl PassageVisibility {
    /// 从字符串解析可见性
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "public" => Some(PassageVisibility::Public),
            "private" => Some(PassageVisibility::Private),
            "protected" => Some(PassageVisibility::Protected),
            _ => None,
        }
    }

    /// 检查是否为公开可见
    #[inline]
    pub fn is_public(&self) -> bool {
        matches!(self, PassageVisibility::Public)
    }

    /// 检查是否需要认证
    #[allow(dead_code)]
    #[inline]
    pub fn requires_auth(&self) -> bool {
        matches!(self, PassageVisibility::Private | PassageVisibility::Protected)
    }
}

// 为数据库实现 ToSql 和 FromSql
impl rusqlite::types::ToSql for PassageVisibility {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        self.as_ref().to_sql()
    }
}

impl rusqlite::types::FromSql for PassageVisibility {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let s: String = rusqlite::types::FromSql::column_result(value)?;
        Self::from_str(&s)
            .ok_or_else(|| rusqlite::types::FromSqlError::InvalidType)
    }
}

/// 用户角色枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    /// 管理员
    Admin,
    /// 编辑
    Editor,
    /// 订阅者
    Subscriber,
}

impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserRole::Admin => write!(f, "admin"),
            UserRole::Editor => write!(f, "editor"),
            UserRole::Subscriber => write!(f, "subscriber"),
        }
    }
}

impl AsRef<str> for UserRole {
    fn as_ref(&self) -> &str {
        match self {
            UserRole::Admin => "admin",
            UserRole::Editor => "editor",
            UserRole::Subscriber => "subscriber",
        }
    }
}

impl UserRole {
    /// 从字符串解析角色
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "admin" => Some(UserRole::Admin),
            "editor" => Some(UserRole::Editor),
            "subscriber" => Some(UserRole::Subscriber),
            _ => None,
        }
    }

    /// 检查是否为管理员
    #[allow(dead_code)]
    #[inline]
    pub fn is_admin(&self) -> bool {
        matches!(self, UserRole::Admin)
    }

    /// 检查是否有编辑权限
    #[allow(dead_code)]
    #[inline]
    pub fn can_edit(&self) -> bool {
        matches!(self, UserRole::Admin | UserRole::Editor)
    }
}

// 为数据库实现 ToSql 和 FromSql
impl rusqlite::types::ToSql for UserRole {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        self.as_ref().to_sql()
    }
}

impl rusqlite::types::FromSql for UserRole {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let s: String = rusqlite::types::FromSql::column_result(value)?;
        Self::from_str(&s)
            .ok_or_else(|| rusqlite::types::FromSqlError::InvalidType)
    }
}

/// 用户状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    /// 活跃
    Active,
    /// 已禁用
    Disabled,
    /// 已删除
    Deleted,
}

impl fmt::Display for UserStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserStatus::Active => write!(f, "active"),
            UserStatus::Disabled => write!(f, "disabled"),
            UserStatus::Deleted => write!(f, "deleted"),
        }
    }
}

impl AsRef<str> for UserStatus {
    fn as_ref(&self) -> &str {
        match self {
            UserStatus::Active => "active",
            UserStatus::Disabled => "disabled",
            UserStatus::Deleted => "deleted",
        }
    }
}

impl UserStatus {
    /// 从字符串解析状态
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "active" => Some(UserStatus::Active),
            "disabled" => Some(UserStatus::Disabled),
            "deleted" => Some(UserStatus::Deleted),
            _ => None,
        }
    }

    /// 检查是否活跃
    #[allow(dead_code)]
    #[inline]
    pub fn is_active(&self) -> bool {
        matches!(self, UserStatus::Active)
    }
}

// 为数据库实现 ToSql 和 FromSql
impl rusqlite::types::ToSql for UserStatus {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        self.as_ref().to_sql()
    }
}

impl rusqlite::types::FromSql for UserStatus {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let s: String = rusqlite::types::FromSql::column_result(value)?;
        Self::from_str(&s)
            .ok_or_else(|| rusqlite::types::FromSqlError::InvalidType)
    }
}

/// 文章模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Passage {
    // 热点字段：经常访问的，放在前面以提高缓存友好性
    pub id: Option<i64>,
    pub title: String,
    pub status: PassageStatus,
    pub visibility: PassageVisibility,
    pub is_scheduled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // 冷字段：较少访问的，放在后面
    pub uuid: Option<String>,  // Flake UUID
    pub content: String,
    pub original_content: Option<String>,
    pub summary: Option<String>,
    pub summarize: Option<String>,  // 自动生成的摘要
    pub author: String,
    pub tags: String,  // JSON 数组字符串
    pub category: String,
    pub file_path: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub cover_image: Option<String>,  // 封面图片路径
}

impl Passage {
    /// 创建新的文章实例
    #[allow(dead_code)]
    pub fn new(title: String, content: String) -> Self {
        Self {
            id: None,
            uuid: None,
            title,
            content,
            original_content: None,
            summary: None,
            summarize: None,
            author: "Anonymous".to_string(),
            tags: "[]".to_string(),
            category: "未分类".to_string(),
            status: PassageStatus::Draft,
            file_path: None,
            visibility: PassageVisibility::Public,
            is_scheduled: false,
            published_at: None,
            cover_image: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// 检查文章是否可见
    #[allow(dead_code)]
    pub fn is_visible(&self) -> bool {
        self.status.is_published() && self.visibility.is_public()
    }

    /// 检查文章是否需要认证
    #[allow(dead_code)]
    pub fn requires_auth(&self) -> bool {
        !self.status.is_published() || self.visibility.requires_auth()
    }
}

/// 用户模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Option<i64>,
    pub username: String,
    pub password: String,
    pub email: String,
    pub role: UserRole,
    pub status: UserStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    /// 创建新的用户实例
    #[allow(dead_code)]
    pub fn new(username: String, password: String, email: String) -> Self {
        Self {
            id: None,
            username,
            password,
            email,
            role: UserRole::Subscriber,
            status: UserStatus::Active,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// 检查用户是否活跃
    #[allow(dead_code)]
    pub fn is_active(&self) -> bool {
        self.status.is_active()
    }

    /// 检查用户是否为管理员
    #[allow(dead_code)]
    pub fn is_admin(&self) -> bool {
        self.role.is_admin()
    }

    /// 检查用户是否有编辑权限
    #[allow(dead_code)]
    pub fn can_edit(&self) -> bool {
        self.role.can_edit()
    }
}

/// 评论模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: Option<i64>,
    pub username: String,
    pub content: String,
    pub passage_uuid: String,  // 使用 uuid 而不是 passage_id
    pub created_at: DateTime<Utc>,
}

/// 设置模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    // 热点字段：经常访问的，放在前面以提高缓存友好性
    pub id: Option<i64>,
    pub key: String,
    pub value: String,
    pub category: String,
    pub updated_at: DateTime<Utc>,

    // 冷字段：较少访问的，放在后面
    pub r#type: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// 关于页面主卡片模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AboutMainCard {
    pub id: Option<i64>,
    pub title: String,
    pub icon: String,
    pub layout_type: String,
    pub custom_css: String,
    pub sort_order: i32,
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 关于页面次卡片模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AboutSubCard {
    pub id: Option<i64>,
    pub main_card_id: i64,
    pub title: String,
    pub description: String,
    pub icon: String,
    pub link_url: String,
    pub layout_type: String,
    pub custom_css: String,
    pub sort_order: i32,
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 分类模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: Option<i64>,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub sort_order: i32,
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 标签模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: Option<i64>,
    pub name: String,
    pub description: String,
    pub color: String,
    pub category_id: i64,
    pub sort_order: i32,
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 附件模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: Option<i64>,
    pub file_name: String,
    pub stored_name: String,
    pub file_path: String,
    pub file_type: String,
    pub content_type: String,
    pub file_size: i64,
    pub passage_uuid: Option<String>,  // 使用 uuid 而不是 passage_id
    pub visibility: PassageVisibility,
    pub show_in_passage: bool,
    pub uploaded_at: DateTime<Utc>,
}

/// 音乐轨道模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicTrack {
    pub id: Option<i64>,
    pub title: String,
    pub artist: String,
    pub file_path: String,
    pub file_name: String,
    pub duration: String,
    pub cover_image: String,
    pub created_at: DateTime<Utc>,
}

/// 友链模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendLink {
    pub id: Option<i64>,
    pub nickname: String,
    pub link_url: String,
    pub avatar_url: String,
    pub motto: String,
    pub sort_order: i32,
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 归档统计数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveStats {
    pub year: String,
    pub month: String,
    pub count: i32,
}

/// 标签统计数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagStats {
    pub id: i64,
    pub name: String,
    pub count: i32,
}

// ==================== 动态路由相关模型 ====================

/// 路由类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RouteType {
    /// 内存路由
    Memory,
    /// 文件路由
    File,
    /// 数据库路由
    Database,
}

impl fmt::Display for RouteType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RouteType::Memory => write!(f, "memory"),
            RouteType::File => write!(f, "file"),
            RouteType::Database => write!(f, "database"),
        }
    }
}

impl AsRef<str> for RouteType {
    fn as_ref(&self) -> &str {
        match self {
            RouteType::Memory => "memory",
            RouteType::File => "file",
            RouteType::Database => "database",
        }
    }
}

impl RouteType {
    /// 从字符串解析路由类型
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "memory" => Some(RouteType::Memory),
            "file" => Some(RouteType::File),
            "database" => Some(RouteType::Database),
            _ => None,
        }
    }
}

// 为数据库实现 ToSql 和 FromSql
impl rusqlite::types::ToSql for RouteType {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        self.as_ref().to_sql()
    }
}

impl rusqlite::types::FromSql for RouteType {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let s: String = rusqlite::types::FromSql::column_result(value)?;
        Self::from_str(&s)
            .ok_or_else(|| rusqlite::types::FromSqlError::InvalidType)
    }
}

/// 处理器类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HandlerType {
    /// 重定向处理器
    Redirect,
    /// 静态内容处理器
    Static,
    /// 模板渲染处理器
    Template,
    /// 代理处理器
    Proxy,
    /// 自定义处理器
    Custom,
}

impl fmt::Display for HandlerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HandlerType::Redirect => write!(f, "redirect"),
            HandlerType::Static => write!(f, "static"),
            HandlerType::Template => write!(f, "template"),
            HandlerType::Proxy => write!(f, "proxy"),
            HandlerType::Custom => write!(f, "custom"),
        }
    }
}

impl AsRef<str> for HandlerType {
    fn as_ref(&self) -> &str {
        match self {
            HandlerType::Redirect => "redirect",
            HandlerType::Static => "static",
            HandlerType::Template => "template",
            HandlerType::Proxy => "proxy",
            HandlerType::Custom => "custom",
        }
    }
}

impl HandlerType {
    /// 从字符串解析处理器类型
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "redirect" => Some(HandlerType::Redirect),
            "static" => Some(HandlerType::Static),
            "template" => Some(HandlerType::Template),
            "proxy" => Some(HandlerType::Proxy),
            "custom" => Some(HandlerType::Custom),
            _ => None,
        }
    }
}

// 为数据库实现 ToSql 和 FromSql
impl rusqlite::types::ToSql for HandlerType {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        self.as_ref().to_sql()
    }
}

impl rusqlite::types::FromSql for HandlerType {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let s: String = rusqlite::types::FromSql::column_result(value)?;
        Self::from_str(&s)
            .ok_or_else(|| rusqlite::types::FromSqlError::InvalidType)
    }
}

/// 动态路由模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicRoute {
    /// 路由ID
    pub id: Option<i64>,
    /// 路由名称（管理员可见，便于识别和管理）
    pub route_name: Option<String>,
    /// 路由类型
    pub route_type: RouteType,
    /// 路由路径（支持Ant风格通配符，如 /api/**、/user/*）
    pub path: String,
    /// 处理器类型（redirect、static、proxy、custom）
    pub handler_type: HandlerType,
    /// 处理器配置 (JSON)
    pub handler_config: serde_json::Value,
    /// 内容来源（database 或 file），仅对 static 类型有意义
    pub content_source: Option<String>,
    /// 纯文本内容或文件路径
    pub content_template: Option<String>,
    /// Content-Type 提示（可选）
    pub content_type_hint: Option<String>,
    /// 是否启用
    pub enabled: bool,
    /// 优先级 (数字越小优先级越高，与Java设计一致)
    pub priority: i32,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 创建者
    pub created_by: Option<String>,
    /// 扩展元数据 (JSON)
    pub metadata: Option<serde_json::Value>,
}

impl DynamicRoute {
    /// 创建新的路由实例
    pub fn new(
        route_type: RouteType,
        path: String,
        handler_type: HandlerType,
        handler_config: serde_json::Value,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            route_name: None,
            route_type,
            path,
            handler_type,
            handler_config,
            content_source: None,
            content_template: None,
            content_type_hint: None,
            enabled: true,
            priority: 0,
            created_at: now,
            updated_at: now,
            created_by: None,
            metadata: None,
        }
    }

    /// 检查路由是否可用
    pub fn is_available(&self) -> bool {
        self.enabled
    }
}

/// 动态路由操作日志模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicRouteLog {
    pub id: Option<i64>,
    pub route_id: Option<i64>,
    pub action: String,
    pub old_config: Option<String>,
    pub new_config: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// 动态路由统计模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicRouteStats {
    pub id: Option<i64>,
    pub route_id: i64,
    pub access_count: i64,
    pub last_accessed_at: Option<DateTime<Utc>>,
    pub total_response_time_ms: i64,
    pub avg_response_time_ms: f64,
    pub error_count: i64,
    pub updated_at: DateTime<Utc>,
}

/// 创建路由请求
#[derive(Debug, Deserialize)]
pub struct CreateRouteRequest {
    pub route_name: Option<String>,
    pub route_type: Option<RouteType>,
    pub path: String,
    pub handler_type: HandlerType,
    pub handler_config: serde_json::Value,
    pub content_source: Option<String>,
    pub content_template: Option<String>,
    pub content_type_hint: Option<String>,
    pub enabled: Option<bool>,
    pub priority: Option<i32>,
    pub metadata: Option<serde_json::Value>,
}

/// 更新路由请求
#[derive(Debug, Deserialize)]
pub struct UpdateRouteRequest {
    pub route_name: Option<String>,
    pub route_type: Option<RouteType>,
    pub path: Option<String>,
    pub handler_type: Option<HandlerType>,
    pub handler_config: Option<serde_json::Value>,
    pub content_source: Option<String>,
    pub content_template: Option<String>,
    pub content_type_hint: Option<String>,
    pub enabled: Option<bool>,
    pub priority: Option<i32>,
    pub metadata: Option<serde_json::Value>,
}

/// 路由列表查询参数
#[derive(Debug, Deserialize)]
pub struct ListRoutesQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub route_type: Option<RouteType>,
    pub enabled: Option<bool>,
}