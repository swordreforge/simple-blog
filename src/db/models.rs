use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// 文章模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Passage {
    pub id: Option<i64>,
    pub uuid: Option<String>,  // Flake UUID
    pub title: String,
    pub content: String,
    pub original_content: Option<String>,
    pub summary: Option<String>,
    pub author: String,
    pub tags: String,  // JSON 数组字符串
    pub category: String,
    pub status: String,
    pub file_path: Option<String>,
    pub visibility: String,
    pub is_scheduled: bool,
    pub published_at: Option<DateTime<Utc>>,
    pub cover_image: Option<String>,  // 封面图片路径
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 用户模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Option<i64>,
    pub username: String,
    pub password: String,
    pub email: String,
    pub role: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 访客模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Visitor {
    pub id: Option<i64>,
    pub ip: String,
    pub user_agent: Option<String>,
    pub visit_date: String,
    pub created_at: DateTime<Utc>,
}

/// 文章阅读记录模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleView {
    pub id: Option<i64>,
    pub passage_uuid: String,  // 使用 uuid 而不是 passage_id
    pub ip: String,
    pub user_agent: Option<String>,
    pub country: String,
    pub city: String,
    pub region: String,
    pub view_date: String,
    pub view_time: DateTime<Utc>,
    pub duration: i32,
    pub created_at: DateTime<Utc>,
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
    pub visibility: String,
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