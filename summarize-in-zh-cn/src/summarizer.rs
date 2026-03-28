use crate::sentence::{filter_sentences, split_sentences};
use crate::similarity::{build_similarity_matrix, mmr_selection, normalize_matrix};
use crate::textrank::textrank;
use crate::tokenizer::tokenize;

/// 文本摘要器
///
/// 使用 TextRank 算法生成文本摘要，支持 MMR 去重和位置权重。
///
/// # 示例
///
/// ```rust
/// use rustblog_wip_for_summarize_in_zh_cn::Summarizer;
///
/// // 创建默认摘要器（5句话）
/// let summarizer = Summarizer::new(5);
///
/// // 自定义配置
/// let summarizer = Summarizer::new(3)
///     .with_lambda(0.7)
///     .with_mmr(true)
///     .with_position_weight(true);
///
/// let summary = summarizer.summarize("你的文本内容...");
/// ```
pub struct Summarizer {
    max_sentences: usize,
    lambda: f64,
    use_mmr: bool,
    use_position_weight: bool,
}

impl Summarizer {
    /// 创建一个新的摘要器
    ///
    /// # 参数
    ///
    /// * `max_sentences` - 摘要的最大句子数
    ///
    /// # 示例
    ///
    /// ```rust
    /// use rustblog_wip_for_summarize_in_zh_cn::Summarizer;
    ///
    /// let summarizer = Summarizer::new(3);
    /// ```
    pub fn new(max_sentences: usize) -> Self {
        Summarizer {
            max_sentences,
            lambda: 0.7,
            use_mmr: true,
            use_position_weight: true,
        }
    }

    /// 设置 MMR 算法的平衡参数 λ
    ///
    /// λ 用于平衡相关性和冗余度：
    /// - λ 接近 1.0：更重视句子重要性（TextRank 分数）
    /// - λ 接近 0.0：更重视去重
    /// - 推荐：0.5-0.7
    ///
    /// # 参数
    ///
    /// * `lambda` - 平衡参数，必须在 0.0 到 1.0 之间
    ///
    /// # Panics
    ///
    /// 如果 lambda 不在 [0.0, 1.0] 范围内会 panic
    ///
    /// # 示例
    ///
    /// ```rust
    /// use rustblog_wip_for_summarize_in_zh_cn::Summarizer;
    ///
    /// let summarizer = Summarizer::new(5).with_lambda(0.7);
    /// ```
    pub fn with_lambda(mut self, lambda: f64) -> Self {
        assert!((0.0..=1.0).contains(&lambda), "lambda must be between 0.0 and 1.0");
        self.lambda = lambda;
        self
    }

    /// 启用或禁用 MMR 去重
    ///
    /// MMR (Maximal Marginal Relevance) 算法用于减少摘要中的信息冗余，
    /// 确保选出的句子既重要又各不相同。
    ///
    /// # 参数
    ///
    /// * `use_mmr` - 是否启用 MMR 去重
    ///
    /// # 示例
    ///
    /// ```rust
    /// use rustblog_wip_for_summarize_in_zh_cn::Summarizer;
    ///
    /// let summarizer = Summarizer::new(5).with_mmr(true);
    /// ```
    pub fn with_mmr(mut self, use_mmr: bool) -> Self {
        self.use_mmr = use_mmr;
        self
    }

    /// 启用或禁用位置权重
    ///
    /// 位置权重会给予文章开头和结尾的句子更高的权重：
    /// - 前 2 句权重 × 1.3
    /// - 后 2 句权重 × 1.2
    ///
    /// 这对博客文章等结构化文本特别有用，因为开头通常介绍主题，
    /// 结尾通常总结要点。
    ///
    /// # 参数
    ///
    /// * `use_position_weight` - 是否启用位置权重
    ///
    /// # 示例
    ///
    /// ```rust
    /// use rustblog_wip_for_summarize_in_zh_cn::Summarizer;
    ///
    /// let summarizer = Summarizer::new(5).with_position_weight(true);
    /// ```
    pub fn with_position_weight(mut self, use_position_weight: bool) -> Self {
        self.use_position_weight = use_position_weight;
        self
    }

    fn apply_position_weight(&self, scores: &mut [f64], total: usize) {
        if !self.use_position_weight || total < 3 {
            return;
        }

        let first_count = 2.min(total);
        let last_count = 2.min(total);

        for score in scores.iter_mut().take(first_count) {
            *score *= 1.3;
        }

        for score in scores.iter_mut().skip(total.saturating_sub(last_count)) {
            *score *= 1.2;
        }
    }

    /// 生成文本摘要
///
    /// 使用 TextRank 算法提取文本中最重要的句子，生成摘要。
    /// 摘要长度由创建时指定的 `max_sentences` 决定。
    ///
    /// # 参数
    ///
    /// * `text` - 要摘要的文本内容
    ///
    /// # 返回
    ///
    /// 返回生成的摘要字符串，句子之间用"。"连接。
    /// 如果输入为空或无法生成摘要，返回空字符串。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use rustblog_wip_for_summarize_in_zh_cn::Summarizer;
    ///
    /// let summarizer = Summarizer::new(3);
    /// let text = "这是一篇长文本...";
    /// let summary = summarizer.summarize(text);
    /// println!("{}", summary);
    /// ```
    pub fn summarize(&self, text: &str) -> String {
        if text.trim().is_empty() {
            return String::new();
        }

        let sentences = split_sentences(text);
        if sentences.is_empty() {
            return String::new();
        }

        let filtered_sentences = filter_sentences(&sentences);
        if filtered_sentences.is_empty() {
            return String::new();
        }

        let total = filtered_sentences.len();

        let tokenized_sentences: Vec<Vec<String>> = filtered_sentences
            .iter()
            .map(|s| tokenize(s))
            .collect();

        let mut similarity_matrix = build_similarity_matrix(&tokenized_sentences);
        normalize_matrix(&mut similarity_matrix);

        let mut scores = textrank(&similarity_matrix);

        self.apply_position_weight(&mut scores, total);

        let top_n = self.max_sentences.min(total);
        let selected_indices = if self.use_mmr {
            mmr_selection(&scores, &similarity_matrix, top_n, self.lambda)
        } else {
            let mut indexed_scores: Vec<(usize, f64)> = scores
                .iter()
                .enumerate()
                .map(|(i, &score)| (i, score))
                .collect();

            indexed_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            let mut indices: Vec<usize> = indexed_scores
                .into_iter()
                .take(top_n)
                .map(|(i, _)| i)
                .collect();
            indices.sort();
            indices
        };

        let summary: Vec<String> = selected_indices
            .into_iter()
            .map(|i| filtered_sentences[i].clone())
            .collect();

        summary.join("。")
    }

    /// 生成自适应长度的文本摘要
    ///
    /// 根据文本长度自动调整摘要长度：
    /// - 0-5 句：全部保留
    /// - 6-15 句：提取 3 句
    /// - 16-30 句：提取 5 句
    /// - 30+ 句：提取 7 句
    ///
    /// # 参数
    ///
    /// * `text` - 要摘要的文本内容
    ///
    /// # 返回
    ///
    /// 返回生成的摘要字符串。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use rustblog_wip_for_summarize_in_zh_cn::Summarizer;
    ///
    /// let summarizer = Summarizer::new(5);
    /// let text = "这是一篇很长的博客文章...";
    /// let summary = summarizer.summarize_auto(text);
    /// println!("{}", summary);
    /// ```
    pub fn summarize_auto(&self, text: &str) -> String {
        if text.trim().is_empty() {
            return String::new();
        }

        let sentences = split_sentences(text);
        let total = sentences.len();

        let adaptive_max = match total {
            0..=5 => total,
            6..=15 => 3,
            16..=30 => 5,
            _ => 7,
        };

        let adaptive_summarizer = Summarizer {
            max_sentences: adaptive_max,
            lambda: self.lambda,
            use_mmr: self.use_mmr,
            use_position_weight: self.use_position_weight,
        };

        adaptive_summarizer.summarize(text)
    }
}

impl Default for Summarizer {
    fn default() -> Self {
        Summarizer::new(5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summarize_empty() {
        let summarizer = Summarizer::new(3);
        let summary = summarizer.summarize("");
        assert!(summary.is_empty());
    }

    #[test]
    fn test_summarize_simple() {
        let text = "人工智能是计算机科学的一个分支。它试图了解智能的实质。人工智能包括机器学习、深度学习等技术。这些技术在图像识别、自然语言处理等领域有广泛应用。";
        let summarizer = Summarizer::new(2);
        let summary = summarizer.summarize(text);
        assert!(!summary.is_empty());
    }

    #[test]
    fn test_summarizer_with_custom_lambda() {
        let summarizer = Summarizer::new(2).with_lambda(0.5).with_mmr(true);
        let text = "Rust是一门系统编程语言。Rust以高性能著称。Rust内存安全。Rust并发性强。";
        let summary = summarizer.summarize(text);
        assert!(!summary.is_empty());
    }

    #[test]
    fn test_summarizer_without_mmr() {
        let summarizer = Summarizer::new(2).with_mmr(false);
        let text = "Rust是一门系统编程语言。Rust以高性能著称。Rust内存安全。Rust并发性强。";
        let summary = summarizer.summarize(text);
        assert!(!summary.is_empty());
    }

    #[test]
    fn test_summarize_auto() {
        let summarizer = Summarizer::new(5);
        let text = "第一句很重要。第二句是内容。第三句继续。第四句还在。第五句继续写。第六句很重要。第七句结尾。第八句总结。第九句结束。第十句完成。";
        let summary = summarizer.summarize_auto(text);
        assert!(!summary.is_empty());
    }

    #[test]
    fn test_position_weight() {
        let summarizer = Summarizer::new(2).with_position_weight(true);
        let text = "这是文章的开头句。中间有一些内容。还有更多内容。继续写内容。这是文章的结尾句。";
        let summary = summarizer.summarize(text);
        assert!(!summary.is_empty());
    }
}