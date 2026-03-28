use regex::Regex;
use std::sync::OnceLock;

fn sentence_split_regex() -> &'static Regex {
    static SENTENCE_SPLIT_REGEX: OnceLock<Regex> = OnceLock::new();
    SENTENCE_SPLIT_REGEX.get_or_init(|| {
        Regex::new(r"[。！？；\.\?!\n]").expect("Invalid regex for sentence splitting")
    })
}

pub fn split_sentences(text: &str) -> Vec<String> {
    let sentences: Vec<String> = sentence_split_regex()
        .split(text)
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| {
            let s = s.trim();
            if s.ends_with('，') || s.ends_with(',') {
                s[..s.len() - 1].trim().to_string()
            } else {
                s.to_string()
            }
        })
        .filter(|s| s.len() > 2)
        .collect();

    sentences
}

pub fn filter_sentences(sentences: &[String]) -> Vec<String> {
    sentences
        .iter()
        .filter(|s| {
            let len = s.chars().count();
            (5..=200).contains(&len)
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_sentences() {
        let text = "这是第一句话。这是第二句话！这是第三句话？";
        let sentences = split_sentences(text);
        assert_eq!(sentences.len(), 3);
        assert_eq!(sentences[0], "这是第一句话");
    }

    #[test]
    fn test_filter_sentences() {
        let sentences = vec![
            "太短了".to_string(),
            "这是一个正常的句子，长度适中。".to_string(),
            "这是一个非常非常非常非常非常非常非常非常非常非常非常非常非常非常非常非常非常非常非常非常非常非常非常非常非常非常长的句子。".to_string(),
        ];
        let filtered = filter_sentences(&sentences);
        assert_eq!(filtered.len(), 2);
    }
}