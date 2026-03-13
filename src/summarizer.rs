use crate::sentence::{filter_sentences, split_sentences};
use crate::similarity::{build_similarity_matrix, normalize_matrix};
use crate::textrank::{select_top_sentences, textrank};
use crate::tokenizer::tokenize;

pub struct Summarizer {
    max_sentences: usize,
}

impl Summarizer {
    pub fn new(max_sentences: usize) -> Self {
        Summarizer { max_sentences }
    }

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

        let tokenized_sentences: Vec<Vec<String>> = filtered_sentences
            .iter()
            .map(|s| tokenize(s))
            .collect();

        let mut similarity_matrix = build_similarity_matrix(&tokenized_sentences);
        normalize_matrix(&mut similarity_matrix);

        let scores = textrank(&similarity_matrix);

        let top_n = self.max_sentences.min(filtered_sentences.len());
        let top_indices = select_top_sentences(&scores, top_n);

        let mut selected_indices: Vec<usize> = top_indices.iter().map(|(i, _)| *i).collect();
        selected_indices.sort();

        let summary: Vec<String> = selected_indices
            .into_iter()
            .map(|i| filtered_sentences[i].clone())
            .collect();

        summary.join("。")
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
}