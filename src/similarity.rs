use std::collections::HashSet;

pub fn cosine_similarity(tokens1: &[String], tokens2: &[String]) -> f64 {
    if tokens1.is_empty() || tokens2.is_empty() {
        return 0.0;
    }

    let set1: HashSet<&String> = tokens1.iter().collect();
    let set2: HashSet<&String> = tokens2.iter().collect();

    let intersection: usize = set1.intersection(&set2).count();

    if intersection == 0 {
        return 0.0;
    }

    intersection as f64 / (set1.len() + set2.len() - intersection) as f64
}

pub fn build_similarity_matrix(tokenized_sentences: &[Vec<String>]) -> Vec<Vec<f64>> {
    let n = tokenized_sentences.len();
    let mut matrix = vec![vec![0.0; n]; n];

    for i in 0..n {
        for j in 0..n {
            if i != j {
                matrix[i][j] = cosine_similarity(&tokenized_sentences[i], &tokenized_sentences[j]);
            }
        }
    }

    matrix
}

pub fn normalize_matrix(matrix: &mut [Vec<f64>]) {
    for row in matrix.iter_mut() {
        let sum: f64 = row.iter().sum();
        if sum > 0.0 {
            for val in row.iter_mut() {
                *val /= sum;
            }
        }
    }
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
}