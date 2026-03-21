//! 摘要服务 - 提供中文文本摘要功能
//!
//! 该服务集成了基于 TextRank 算法的中文文本摘要库，
//! 为博客文章自动生成简洁的摘要。

use rustblog_wip_for_summarize_in_zh_cn::summarize_blog_post;

/// 摘要服务
pub struct SummarizeService;

impl SummarizeService {
    /// 为博客文章生成摘要
    ///
    /// # 参数
    ///
    /// * `content` - 文章内容（纯文本）
    ///
    /// # 返回
    ///
    /// 返回生成的摘要字符串。如果内容为空或过短，返回空字符串。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use crate::services::summarize_service::SummarizeService;
    ///
    /// let content = "这是一篇关于Rust编程的文章...";
    /// let summary = SummarizeService::generate_summary(content);
    /// ```
    pub fn generate_summary(content: &str) -> String {
        // 如果内容为空，返回空字符串
        if content.trim().is_empty() {
            return String::new();
        }

        // 如果内容过短（少于50个字符），直接返回原内容
        if content.len() < 50 {
            return content.to_string();
        }

        // 使用 summarize_blog_post 函数生成摘要
        summarize_blog_post(content)
    }

    /// 从 Markdown 内容中提取纯文本并生成摘要
    ///
    /// # 参数
    ///
    /// * `markdown_content` - Markdown 格式的文章内容
    ///
    /// # 返回
    ///
    /// 返回生成的摘要字符串。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use crate::services::summarize_service::SummarizeService;
    ///
    /// let markdown = "# 标题\n\n这是一篇文章的内容...";
    /// let summary = SummarizeService::generate_summary_from_markdown(markdown);
    /// ```
    pub fn generate_summary_from_markdown(markdown_content: &str) -> String {
        // 从 Markdown 中提取纯文本
        let plain_text = Self::extract_plain_text_from_markdown(markdown_content);

        // 生成摘要
        Self::generate_summary(&plain_text)
    }

    /// 从 Markdown 内容中提取纯文本
    ///
    /// 移除 Markdown 语法标记，只保留文本内容。
    ///
    /// # 参数
    ///
    /// * `markdown_content` - Markdown 格式的文章内容
    ///
    /// # 返回
    ///
    /// 返回提取的纯文本。
    fn extract_plain_text_from_markdown(markdown_content: &str) -> String {
        let mut text = markdown_content.to_string();

        // 移除标题标记 (# ## ### 等)
        text = regex::Regex::new(r"^#+\s+")
            .unwrap()
            .replace_all(&text, "")
            .to_string();

        // 移除粗体标记 (**text** 或 __text__)
        text = regex::Regex::new(r"\*\*([^*]+)\*\*")
            .unwrap()
            .replace_all(&text, "$1")
            .to_string();
        text = regex::Regex::new(r"__([^_]+)__")
            .unwrap()
            .replace_all(&text, "$1")
            .to_string();

        // 移除斜体标记 (*text* 或 _text_)
        text = regex::Regex::new(r"\*([^*]+)\*")
            .unwrap()
            .replace_all(&text, "$1")
            .to_string();
        text = regex::Regex::new(r"_([^_]+)_")
            .unwrap()
            .replace_all(&text, "$1")
            .to_string();

        // 移除代码块 (```code```)
        text = regex::Regex::new(r"```[\s\S]*?```")
            .unwrap()
            .replace_all(&text, "")
            .to_string();

        // 移除行内代码 (`code`)
        text = regex::Regex::new(r"`([^`]+)`")
            .unwrap()
            .replace_all(&text, "$1")
            .to_string();

        // 移除链接 [text](url)
        text = regex::Regex::new(r"\[([^\]]+)\]\([^\)]+\)")
            .unwrap()
            .replace_all(&text, "$1")
            .to_string();

        // 移除图片 ![alt](url)
        text = regex::Regex::new(r"!\[([^\]]*)\]\([^\)]+\)")
            .unwrap()
            .replace_all(&text, "")
            .to_string();

        // 移除引用标记 (> )
        text = regex::Regex::new(r"(?m)^>\s+")
            .unwrap()
            .replace_all(&text, "")
            .to_string();

        // 移除列表标记 (- 或 *)
        text = regex::Regex::new(r"(?m)^[\-\*]\s+")
            .unwrap()
            .replace_all(&text, "")
            .to_string();

        // 移除数字列表标记 (1. 2. 等)
        text = regex::Regex::new(r"(?m)^\d+\.\s+")
            .unwrap()
            .replace_all(&text, "")
            .to_string();

        // 移除水平线 (--- 或 ***)
        text = regex::Regex::new(r"(?m)^[\-\*]{3,}\s*$")
            .unwrap()
            .replace_all(&text, "")
            .to_string();

        // 移除多余空行
        text = regex::Regex::new(r"\n\s*\n\s*\n")
            .unwrap()
            .replace_all(&text, "\n\n")
            .to_string();

        // 移除行首尾空白
        text.trim().to_string()
    }

    /// 为文章内容生成摘要并限制长度
    ///
    /// # 参数
    ///
    /// * `content` - 文章内容
    /// * `max_length` - 摘要最大长度（字符数）
    ///
    /// # 返回
    ///
    /// 返回限制长度的摘要。
    #[allow(dead_code)]
    pub fn generate_summary_with_limit(content: &str, max_length: usize) -> String {
        let summary = Self::generate_summary(content);

        if summary.chars().count() <= max_length {
            summary
        } else {
            // 截断到指定长度并添加省略号
            let truncated: String = summary.chars().take(max_length.saturating_sub(3)).collect();
            format!("{}...", truncated)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_summary() {
        let content = "人工智能是计算机科学的一个分支。它试图了解智能的实质。人工智能包括机器学习、深度学习等技术。这些技术在图像识别、自然语言处理等领域有广泛应用。机器学习是人工智能的核心技术之一。深度学习是机器学习的一种特殊形式。神经网络是深度学习的基础。";
        let summary = SummarizeService::generate_summary(content);

        // 摘要应该不为空
        assert!(!summary.is_empty());

        // 摘要应该比原文短
        assert!(summary.len() < content.len());
    }

    #[test]
    fn test_generate_summary_empty() {
        let content = "";
        let summary = SummarizeService::generate_summary(content);

        assert!(summary.is_empty());
    }

    #[test]
    fn test_generate_summary_short() {
        let content = "短文本";
        let summary = SummarizeService::generate_summary(content);

        // 短文本应该原样返回
        assert_eq!(summary, "短文本");
    }

    #[test]
    fn test_extract_plain_text_from_markdown() {
        let markdown = "# 标题\n\n这是一段**粗体**和*斜体*文本。\n\n```\ncode block\n```";
        let plain_text = SummarizeService::extract_plain_text_from_markdown(markdown);

        // 应该移除 Markdown 标记
        assert!(!plain_text.contains("#"));
        assert!(!plain_text.contains("**"));
        assert!(!plain_text.contains("*"));
        assert!(!plain_text.contains("```"));

        // 应该保留文本内容
        assert!(plain_text.contains("标题"));
        assert!(plain_text.contains("粗体"));
        assert!(plain_text.contains("斜体"));
    }

    #[test]
    fn test_generate_summary_from_markdown() {
        let markdown = "# 人工智能概述\n\n人工智能是计算机科学的一个分支。它试图了解智能的实质。人工智能包括机器学习、深度学习等技术。";
        let summary = SummarizeService::generate_summary_from_markdown(markdown);

        // 摘要应该不为空
        assert!(!summary.is_empty());
    }

    #[test]
    fn test_generate_summary_with_limit() {
        let content = "人工智能是计算机科学的一个分支。它试图了解智能的实质。人工智能包括机器学习、深度学习等技术。这些技术在图像识别、自然语言处理等领域有广泛应用。";
        let max_length = 50;
        let summary = SummarizeService::generate_summary_with_limit(content, max_length);

        // 摘要长度应该不超过限制（使用字符数而不是字节数）
        assert!(summary.chars().count() <= max_length);
    }
}
