use std::collections::HashSet;

/// 计算两个词向量之间的余弦相似度（使用 Jaccard 相似度）
///
/// # 参数
///
/// * `tokens1` - 第一个句子的词向量
/// * `tokens2` - 第二个句子的词向量
///
/// # 返回
///
/// 返回相似度值，范围 [0.0, 1.0]，值越大表示越相似
///
/// # 示例
///
/// ```rust
/// use rustblog_wip_for_summarize_in_zh_cn::similarity::cosine_similarity;
///
/// let tokens1 = vec!["苹果".to_string(), "香蕉".to_string()];
/// let tokens2 = vec!["苹果".to_string(), "橙子".to_string()];
/// let sim = cosine_similarity(&tokens1, &tokens2);
/// ```
pub fn cosine_similarity(tokens1: &[String], tokens2: &[String]) -> f64 {
    if tokens1.is_empty() || tokens2.is_empty() {
        return 0.0;
    }

    // 对于短句子，直接使用线性查找可能更快
    if tokens1.len() < 10 && tokens2.len() < 10 {
        let mut intersection = 0usize;
        for token1 in tokens1 {
            if tokens2.contains(token1) {
                intersection += 1;
            }
        }
        if intersection == 0 {
            return 0.0;
        }
        return intersection as f64 / (tokens1.len() + tokens2.len() - intersection) as f64;
    }

    // 对于长句子，使用 HashSet
    let set1: HashSet<&String> = tokens1.iter().collect();
    let set2: HashSet<&String> = tokens2.iter().collect();

    let intersection: usize = set1.intersection(&set2).count();

    if intersection == 0 {
        return 0.0;
    }

    intersection as f64 / (set1.len() + set2.len() - intersection) as f64
}

/// 构建句子相似度矩阵
///
/// # 参数
///
/// * `tokenized_sentences` - 已分词的句子列表
///
/// # 返回
///
/// 返回 n×n 的相似度矩阵，matrix\[i\]\[j\] 表示句子 i 和 j 的相似度
///
/// # 示例
///
/// ```rust
/// use rustblog_wip_for_summarize_in_zh_cn::similarity::build_similarity_matrix;
///
/// let sentences = vec![
///     vec!["苹果".to_string(), "香蕉".to_string()],
///     vec!["苹果".to_string(), "橙子".to_string()],
/// ];
/// let matrix = build_similarity_matrix(&sentences);
/// ```
pub fn build_similarity_matrix(tokenized_sentences: &[Vec<String>]) -> Vec<Vec<f64>> {
    let n = tokenized_sentences.len();
    if n == 0 {
        return vec![];
    }

    // 预分配容量
    let mut matrix: Vec<Vec<f64>> = Vec::with_capacity(n);

    for i in 0..n {
        let mut row = Vec::with_capacity(n);
        for j in 0..n {
            if i != j {
                row.push(cosine_similarity(&tokenized_sentences[i], &tokenized_sentences[j]));
            } else {
                row.push(0.0);
            }
        }
        matrix.push(row);
    }

    matrix
}

/// 归一化相似度矩阵
///
/// 将矩阵的每一行归一化，使每行的和为 1。
/// 这使得矩阵可以用于 TextRank 算法的转移概率计算。
///
/// # 参数
///
/// * `matrix` - 要归一化的相似度矩阵
///
/// # 示例
///
/// ```rust
/// use rustblog_wip_for_summarize_in_zh_cn::similarity::normalize_matrix;
///
/// let mut matrix = vec![vec![0.5, 0.3], vec![0.3, 0.5]];
/// normalize_matrix(&mut matrix);
/// ```
pub fn normalize_matrix(matrix: &mut [Vec<f64>]) {
    for row in matrix.iter_mut() {
        let sum: f64 = row.iter().sum();
        if sum > 0.0 {
            let inv_sum = 1.0 / sum;
            for val in row.iter_mut() {
                *val *= inv_sum;
            }
        }
    }
}

/// 使用 MMR 算法选择去重后的句子
///
/// MMR (Maximal Marginal Relevance) 算法在保证句子重要性的同时，
/// 减少已选句子之间的冗余。
///
/// 算法公式：
/// ```text
/// MMR = λ * relevance - (1-λ) * max_redundancy
/// ```
///
/// # 参数
///
/// * `scores` - 句子的 TextRank 分数
/// * `similarity_matrix` - 句子相似度矩阵
/// * `top_n` - 要选择的句子数量
/// * `lambda` - 平衡参数（0.0-1.0），接近 1 更重视重要性，接近 0 更重视去重
///
/// # 返回
///
/// 返回选中的句子索引列表，按原文顺序排序
///
/// # 示例
///
/// ```rust
/// use rustblog_wip_for_summarize_in_zh_cn::similarity::mmr_selection;
///
/// let scores = vec![0.9, 0.8, 0.7, 0.6];
/// let matrix = vec![
///     vec![0.0, 0.8, 0.2, 0.1],
///     vec![0.8, 0.0, 0.1, 0.1],
///     vec![0.2, 0.1, 0.0, 0.7],
///     vec![0.1, 0.1, 0.7, 0.0],
/// ];
/// let selected = mmr_selection(&scores, &matrix, 2, 0.7);
/// ```
pub fn mmr_selection(
    scores: &[f64],
    similarity_matrix: &[Vec<f64>],
    top_n: usize,
    lambda: f64,
) -> Vec<usize> {
    let n = scores.len();
    if n == 0 || top_n == 0 {
        return vec![];
    }

    let mut selected = Vec::with_capacity(top_n);
    let mut remaining: Vec<usize> = Vec::with_capacity(n);
    remaining.extend(0..n);

    let one_minus_lambda = 1.0 - lambda;

    for _ in 0..top_n.min(n) {
        let mut best_idx = 0;
        let mut best_score = f64::NEG_INFINITY;

        for &idx in &remaining {
            let relevance = scores[idx];

            let redundancy = if selected.is_empty() {
                0.0
            } else {
                selected
                    .iter()
                    .map(|&sel_idx| similarity_matrix[idx][sel_idx])
                    .fold(f64::NEG_INFINITY, |a, b| a.max(b))
            };

            let mmr_score = lambda * relevance - one_minus_lambda * redundancy;

            if mmr_score > best_score {
                best_score = mmr_score;
                best_idx = idx;
            }
        }

        selected.push(best_idx);
        // 使用 swap_remove 替代 retain，时间复杂度 O(n) -> O(1)
        if let Some(pos) = remaining.iter().position(|&x| x == best_idx) {
            remaining.swap_remove(pos);
        }
    }

    selected.sort();
    selected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let tokens1 = vec!["苹果".to_string(), "香蕉".to_string()];
        let tokens2 = vec!["苹果".to_string(), "橙子".to_string()];
        let sim = cosine_similarity(&tokens1, &tokens2);
        assert!(sim > 0.0);
        assert!(sim <= 1.0);
    }

    #[test]
    fn test_cosine_similarity_empty() {
        let tokens1: Vec<String> = vec![];
        let tokens2 = vec!["苹果".to_string()];
        let sim = cosine_similarity(&tokens1, &tokens2);
        assert_eq!(sim, 0.0);
    }

    #[test]
    fn test_build_similarity_matrix() {
        let sentences = vec![
            vec!["苹果".to_string(), "香蕉".to_string()],
            vec!["苹果".to_string(), "橙子".to_string()],
        ];
        let matrix = build_similarity_matrix(&sentences);
        assert_eq!(matrix.len(), 2);
        assert_eq!(matrix[0].len(), 2);
        assert_eq!(matrix[0][0], 0.0);
        assert!(matrix[0][1] > 0.0);
    }

    #[test]
    fn test_mmr_selection() {
        let scores = vec![0.9, 0.8, 0.7, 0.6];
        let similarity_matrix = vec![
            vec![0.0, 0.8, 0.2, 0.1],
            vec![0.8, 0.0, 0.1, 0.1],
            vec![0.2, 0.1, 0.0, 0.7],
            vec![0.1, 0.1, 0.7, 0.0],
        ];
        let selected = mmr_selection(&scores, &similarity_matrix, 2, 0.7);
        assert_eq!(selected.len(), 2);
    }

    #[test]
    fn test_mmr_selection_empty() {
        let scores: Vec<f64> = vec![];
        let matrix: Vec<Vec<f64>> = vec![];
        let selected = mmr_selection(&scores, &matrix, 2, 0.7);
        assert!(selected.is_empty());
    }
}