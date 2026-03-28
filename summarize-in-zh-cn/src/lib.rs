//! # 中文文本摘要库
//!
//! 这是一个基于 TextRank 算法的中文文本摘要库，专门为博客文章和长文本设计。
//!
//! ## 特性
//!
//! - **纯算法实现**：基于 TextRank 图排序算法，无需 AI 模型
//! - **中文支持**：使用 jieba-rs 进行中文分词
//! - **MMR 去重**：使用最大边界相关算法减少信息冗余
//! - **位置权重**：重视文章开头和结尾的句子
//! - **自适应长度**：根据文本长度自动调整摘要长度
//!
//! ## 快速开始
//!
//! ```rust
//! use rustblog_wip_for_summarize_in_zh_cn::summarize_blog_post;
//!
//! let text = "这是一篇博客文章...";
//! let summary = summarize_blog_post(text);
//! println!("{}", summary);
//! ```
//!
//! ## 高级用法
//!
//! ```rust
//! use rustblog_wip_for_summarize_in_zh_cn::Summarizer;
//!
//! let text = "这是一篇博客文章...";
//! let summarizer = Summarizer::new(3)
//!     .with_lambda(0.7)
//!     .with_mmr(true)
//!     .with_position_weight(true);
//!
//! let summary = summarizer.summarize(text);
//! ```

pub mod sentence;
pub mod similarity;
pub mod summarizer;
pub mod textrank;
pub mod tokenizer;

pub use summarizer::Summarizer;

/// 为博客文章生成摘要的便捷函数
///
/// 此函数使用预设的优化参数专门为博客文章生成摘要：
/// - 启用 MMR 去重（λ=0.7）
/// - 启用位置权重（首尾句更重要）
/// - 自适应摘要长度
///
/// # 参数
///
/// * `text` - 要摘要的文本内容
///
/// # 返回
///
/// 返回生成的摘要字符串，如果输入为空则返回空字符串
///
/// # 示例
///
/// ```rust
/// use rustblog_wip_for_summarize_in_zh_cn::summarize_blog_post;
///
/// let blog_post = "Rust 是一门系统编程语言...";
/// let summary = summarize_blog_post(blog_post);
/// println!("摘要: {}", summary);
/// ```
pub fn summarize_blog_post(text: &str) -> String {
    Summarizer::default()
        .with_mmr(true)
        .with_position_weight(true)
        .summarize_auto(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_summarization() {
        let text = "人工智能是计算机科学的一个分支。它试图了解智能的实质。人工智能包括机器学习、深度学习等技术。这些技术在图像识别、自然语言处理等领域有广泛应用。";
        let summarizer = Summarizer::new(2);
        let summary = summarizer.summarize(text);
        assert!(!summary.is_empty());
    }

    #[test]
    fn test_summarize_blog_post() {
        let text = "这是一篇博客文章的开头。这里是正文内容。这里还有更多内容。这里是文章的结尾。";
        let summary = summarize_blog_post(text);
        assert!(!summary.is_empty());
    }
}
