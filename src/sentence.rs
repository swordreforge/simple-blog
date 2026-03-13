use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref SENTENCE_SPLIT_REGEX: Regex =
        Regex::new(r"[。！？；\.\?!\n]").expect("Invalid regex for sentence splitting");
}

pub fn split_sentences(text: &str) -> Vec<String> {
    let sentences: Vec<String> = SENTENCE_SPLIT_REGEX
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
            len >= 5 && len <= 200
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