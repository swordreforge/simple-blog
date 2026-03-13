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
    pub fn is_published(&self) -> bool {
        matches!(self, PassageStatus::Published)
    }

    /// 检查是否可见（已发布）
    #[allow(dead_code)]
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
    pub fn is_public(&self) -> bool {
        matches!(self, PassageVisibility::Public)
    }

    /// 检查是否需要认证
    #[allow(dead_code)]
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
    pub fn is_admin(&self) -> bool {
        matches!(self, UserRole::Admin)
    }

    /// 检查是否有编辑权限
    #[allow(dead_code)]
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
    pub id: Option<i64>,
    pub uuid: Option<String>,  // Flake UUID
    pub title: String,
    pub content: String,
    pub original_content: Option<String>,
    pub summary: Option<String>,
    pub summarize: Option<String>,  // 自动生成的摘要
    pub author: String,
    pub tags: String,  // JSON 数组字符串
    pub category: String,
    pub status: PassageStatus,
    pub file_path: Option<String>,
    pub visibility: PassageVisibility,
    pub is_scheduled: bool,
    pub published_at: Option<DateTime<Utc>>,
    pub cover_image: Option<String>,  // 封面图片路径
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
    pub id: Option<i64>,
    pub key: String,
    pub value: String,
    pub r#type: String,
    pub description: Option<String>,
    pub category: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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