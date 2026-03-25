//! 文章服务 - 处理文章相关的业务逻辑

use chrono::Utc;
use once_cell::sync::Lazy;
use pulldown_cmark::{Parser, html};
use regex::Regex;
use std::sync::Arc;

use crate::db::models::Passage;
use crate::db::repositories::PassageRepository;
use crate::error::Result;

static HTML_TAG_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"<[^>]*>").unwrap());

/// 文章服务
#[derive(Clone)]
#[allow(dead_code)]
pub struct PassageService {
    passage_repo: Arc<PassageRepository>,
}

#[allow(dead_code)]
impl PassageService {
    /// 创建新的文章服务
    pub fn new(passage_repo: Arc<PassageRepository>) -> Self {
        Self { passage_repo }
    }

    /// 创建文章
    ///
    /// # 参数
    /// - `title`: 文章标题
    /// - `content`: Markdown 内容
    /// - `author`: 作者
    /// - `tags`: 标签（JSON 数组格式）
    /// - `category`: 分类
    /// - `status`: 状态（published/draft/archived）
    /// - `visibility`: 可见性（public/private/protected）
    /// - `file_path`: Markdown 文件路径
    /// - `cover_image`: 封面图片路径
    ///
    /// # 返回
    /// 返回新创建的文章 ID
    #[allow(clippy::too_many_arguments)]
    pub async fn create_passage(
        &self,
        title: String,
        content: String,
        author: String,
        tags: String,
        category: String,
        status: String,
        visibility: String,
        file_path: Option<String>,
        cover_image: Option<String>,
    ) -> Result<i64> {
        // 转换 Markdown 为 HTML
        let html_content = Self::convert_markdown_to_html(&content);

        // 自动生成摘要
        let summary = Some(Self::extract_summary(&html_content));

        let now = Utc::now();

        // 解析枚举
        let passage_status = crate::db::models::PassageStatus::from_str(&status)
            .unwrap_or(crate::db::models::PassageStatus::Draft);
        let passage_visibility = crate::db::models::PassageVisibility::from_str(&visibility)
            .unwrap_or(crate::db::models::PassageVisibility::Public);

        let passage = Passage {
            id: None,
            uuid: None,
            title,
            content: html_content,
            original_content: Some(content),
            summary,
            summarize: None, // 将在创建时自动生成
            author,
            tags,
            category,
            status: passage_status,
            file_path,
            visibility: passage_visibility,
            is_scheduled: false,
            published_at: None,
            cover_image: cover_image.or_else(|| Some("/img/passage-cover.webp".to_string())),
            created_at: now,
            updated_at: now,
        };

        self.passage_repo
            .create(&passage)
            .await
            .map_err(|e| crate::error::AppError::Database(e.to_string()))
    }

    /// 根据 ID 获取文章
    pub async fn get_passage_by_id(&self, id: i64) -> Result<Passage> {
        self.passage_repo
            .get_by_id(id)
            .await
            .map_err(|e| crate::error::AppError::Database(e.to_string()))
    }

    /// 根据 UUID 获取文章
    pub async fn get_passage_by_uuid(&self, uuid: &str) -> Result<Passage> {
        self.passage_repo
            .get_by_uuid(uuid)
            .await
            .map_err(|e| crate::error::AppError::Database(e.to_string()))
    }

    /// 更新文章
    pub async fn update_passage(&self, passage: Passage) -> Result<()> {
        self.passage_repo
            .update(&passage)
            .await
            .map_err(|e| crate::error::AppError::Database(e.to_string()))
    }

    /// 将 Markdown 转换为 HTML
    fn convert_markdown_to_html(markdown: &str) -> String {
        let parser = Parser::new(markdown);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        html_output
    }

    /// 从 HTML 内容中提取摘要
    fn extract_summary(html_content: &str) -> String {
        // 移除 HTML 标签（使用静态编译的正则，避免每次调用重新编译）
        let text = HTML_TAG_REGEX.replace_all(html_content, "");

        // 移除多余的空白字符
        let text: String = text.split_whitespace().collect::<Vec<&str>>().join(" ");

        // 按字符截取前 200 个字符（支持中文）
        let chars: Vec<char> = text.chars().collect();
        if chars.len() > 200 {
            format!("{}...", chars[..200].iter().collect::<String>())
        } else {
            text
        }
    }
}
