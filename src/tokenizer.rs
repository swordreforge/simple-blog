use jieba_rs::Jieba;
use lazy_static::lazy_static;
use std::collections::HashSet;

lazy_static! {
    static ref JIEBA: Jieba = Jieba::new();
    static ref STOPWORDS: HashSet<String> = {
        let stopwords = include_str!("../data/hit_stopwords.txt");
        stopwords
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
            .map(|line| line.trim().to_string())
            .collect()
    };
}

pub fn tokenize(text: &str) -> Vec<String> {
    let words = JIEBA.cut(text, false);
    words
        .into_iter()
        .map(|word| word.to_string())
        .filter(|word| {
            !STOPWORDS.contains(word) && word.len() > 1 && !word.chars().all(|c| c.is_ascii_digit())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let text = "这是一个测试文本，用于测试分词功能。";
        let tokens = tokenize(text);
        assert!(!tokens.is_empty());
        assert!(!tokens.contains(&"是".to_string()));
        assert!(!tokens.contains(&"的".to_string()));
    }
}