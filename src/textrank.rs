const DAMPING_FACTOR: f64 = 0.85;
const MAX_ITERATIONS: usize = 100;
const CONVERGENCE_THRESHOLD: f64 = 1e-5;

pub fn textrank(similarity_matrix: &[Vec<f64>]) -> Vec<f64> {
    let n = similarity_matrix.len();
    if n == 0 {
        return vec![];
    }

    let mut scores = vec![1.0 / n as f64; n];
    let mut prev_scores = scores.clone();

    for _ in 0..MAX_ITERATIONS {
        for i in 0..n {
            let mut sum = 0.0;
            for j in 0..n {
                if i != j && similarity_matrix[j][i] > 0.0 {
                    sum += similarity_matrix[j][i] * prev_scores[j];
                }
            }
            scores[i] = (1.0 - DAMPING_FACTOR) / n as f64 + DAMPING_FACTOR * sum;
        }

        let max_diff = scores
            .iter()
            .zip(prev_scores.iter())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0_f64, |acc, x| acc.max(x));

        if max_diff < CONVERGENCE_THRESHOLD {
            break;
        }

        prev_scores.clone_from(&scores);
    }

    scores
}

pub fn select_top_sentences(scores: &[f64], top_n: usize) -> Vec<(usize, f64)> {
    let mut indexed_scores: Vec<(usize, f64)> = scores
        .iter()
        .enumerate()
        .map(|(i, &score)| (i, score))
        .collect();

    indexed_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

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