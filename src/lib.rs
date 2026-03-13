pub mod sentence;
pub mod similarity;
pub mod summarizer;
pub mod textrank;
pub mod tokenizer;

pub use summarizer::Summarizer;

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
}
