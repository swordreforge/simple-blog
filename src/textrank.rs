const DAMPING_FACTOR: f64 = 0.85;
const MAX_ITERATIONS: usize = 100;
const CONVERGENCE_THRESHOLD: f64 = 1e-5;

/// 使用 TextRank 算法计算句子重要性分数
///
/// # 参数
///
/// * `similarity_matrix` - 已归一化的句子相似度矩阵
///
/// # 返回
///
/// 返回每个句子的 TextRank 分数向量
pub fn textrank(similarity_matrix: &[Vec<f64>]) -> Vec<f64> {
    let n = similarity_matrix.len();
    if n == 0 {
        return vec![];
    }

    let init_score = 1.0 / n as f64;
    let mut scores = vec![init_score; n];
    let mut prev_scores = scores.clone();

    let damping_complement = 1.0 - DAMPING_FACTOR;
    let damping_base = damping_complement / n as f64;

    for _ in 0..MAX_ITERATIONS {
        for i in 0..n {
            let mut sum = 0.0;
            for j in 0..n {
                if i != j && similarity_matrix[j][i] > 0.0 {
                    sum += similarity_matrix[j][i] * prev_scores[j];
                }
            }
            scores[i] = damping_base + DAMPING_FACTOR * sum;
        }

        let max_diff = scores
            .iter()
            .zip(prev_scores.iter())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0_f64, |acc, x| acc.max(x));

        if max_diff < CONVERGENCE_THRESHOLD {
            break;
        }

        prev_scores.copy_from_slice(&scores);
    }

    scores
}

/// 选择得分最高的句子
///
/// # 参数
///
/// * `scores` - 句子分数数组
/// * `top_n` - 要选择的句子数量
///
/// # 返回
///
/// 返回按分数降序排列的 (索引, 分数) 对
pub fn select_top_sentences(scores: &[f64], top_n: usize) -> Vec<(usize, f64)> {
    let mut indexed_scores: Vec<(usize, f64)> = scores
        .iter()
        .enumerate()
        .map(|(i, &score)| (i, score))
        .collect();

    indexed_scores.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    indexed_scores.into_iter().take(top_n).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_textrank() {
        let matrix = vec![
            vec![0.0, 0.5, 0.3],
            vec![0.5, 0.0, 0.4],
            vec![0.3, 0.4, 0.0],
        ];
        let scores = textrank(&matrix);
        assert_eq!(scores.len(), 3);
        assert!(scores.iter().all(|&s| s >= 0.0 && s <= 1.0));
    }

    #[test]
    fn test_select_top_sentences() {
        let scores = vec![0.1, 0.9, 0.5, 0.7];
        let top = select_top_sentences(&scores, 2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].0, 1);
        assert_eq!(top[1].0, 3);
    }

    #[test]
    fn test_empty_matrix() {
        let matrix: Vec<Vec<f64>> = vec![];
        let scores = textrank(&matrix);
        assert!(scores.is_empty());
    }
}