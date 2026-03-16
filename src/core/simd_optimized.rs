//! SIMD优化的字符串操作
//!
//! 使用SIMD指令加速字符串比较和路径分割，提升路由匹配性能。
//! 这是一个可选特性，需要启用"simd" feature。

#[cfg(feature = "simd")]
use std::arch::x86_64::*;

/// SIMD优化的字符串比较器
pub struct SimdComparator;

impl SimdComparator {
    /// 比较两个字符串是否相等（使用SIMD优化）
    #[cfg(feature = "simd")]
    #[inline]
    #[target_feature(enable = "avx2")]
    pub unsafe fn equals_simd(a: &str, b: &str) -> bool {
        // 长度快速检查
        if a.len() != b.len() {
            return false;
        }

        let a_bytes = a.as_bytes();
        let b_bytes = b.as_bytes();
        let len = a_bytes.len();

        // 使用AVX2进行32字节块的比较
        let chunks = len / 32;
        let remainder = len % 32;

        for i in 0..chunks {
            let offset = i * 32;
            let a_vec = _mm256_loadu_si256(a_bytes.as_ptr().add(offset) as *const __m256i);
            let b_vec = _mm256_loadu_si256(b_bytes.as_ptr().add(offset) as *const __m256i);
            let cmp = _mm256_cmpeq_epi8(a_vec, b_vec);

            // 检查是否所有字节都相等
            if _mm256_movemask_epi8(cmp) != -1 {
                return false;
            }
        }

        // 处理剩余字节
        if remainder > 0 {
            let offset = chunks * 32;
            for i in 0..remainder {
                if a_bytes[offset + i] != b_bytes[offset + i] {
                    return false;
                }
            }
        }

        true
    }

    /// 比较两个字符串是否相等（安全包装）
    #[cfg(feature = "simd")]
    #[inline]
    pub fn equals(a: &str, b: &str) -> bool {
        if Self::is_avx2_supported() {
            unsafe { Self::equals_simd(a, b) }
        } else {
            a == b
        }
    }

    /// 比较两个字符串是否相等（回退到普通实现）
    #[cfg(not(feature = "simd"))]
    #[inline]
    pub fn equals(a: &str, b: &str) -> bool {
        a == b
    }

    /// 检查字符串是否以指定前缀开头（使用SIMD优化）
    #[cfg(feature = "simd")]
    #[inline]
    #[target_feature(enable = "avx2")]
    pub unsafe fn starts_with_simd(text: &str, prefix: &str) -> bool {
        if prefix.len() > text.len() {
            return false;
        }

        let text_bytes = text.as_bytes();
        let prefix_bytes = prefix.as_bytes();
        let len = prefix_bytes.len();

        // 使用AVX2进行32字节块的比较
        let chunks = len / 32;
        let remainder = len % 32;

        for i in 0..chunks {
            let offset = i * 32;
            let text_vec = _mm256_loadu_si256(text_bytes.as_ptr().add(offset) as *const __m256i);
            let prefix_vec = _mm256_loadu_si256(prefix_bytes.as_ptr().add(offset) as *const __m256i);
            let cmp = _mm256_cmpeq_epi8(text_vec, prefix_vec);

            if _mm256_movemask_epi8(cmp) != -1 {
                return false;
            }
        }

        // 处理剩余字节
        if remainder > 0 {
            let offset = chunks * 32;
            for i in 0..remainder {
                if text_bytes[offset + i] != prefix_bytes[offset + i] {
                    return false;
                }
            }
        }

        true
    }

    /// 检查字符串是否以指定前缀开头（安全包装）
    #[cfg(feature = "simd")]
    #[inline]
    pub fn starts_with(text: &str, prefix: &str) -> bool {
        if Self::is_avx2_supported() {
            unsafe { Self::starts_with_simd(text, prefix) }
        } else {
            text.starts_with(prefix)
        }
    }

    /// 检查字符串是否以指定前缀开头（回退到普通实现）
    #[cfg(not(feature = "simd"))]
    #[inline]
    pub fn starts_with(text: &str, prefix: &str) -> bool {
        text.starts_with(prefix)
    }

    /// 查找最长公共前缀长度（使用SIMD优化）
    #[cfg(feature = "simd")]
    #[inline]
    #[target_feature(enable = "avx2")]
    pub unsafe fn longest_common_prefix_simd(a: &str, b: &str) -> usize {
        let min_len = a.len().min(b.len());
        if min_len == 0 {
            return 0;
        }

        let a_bytes = a.as_bytes();
        let b_bytes = b.as_bytes();

        // 使用AVX2进行32字节块的比较
        let chunks = min_len / 32;
        let remainder = min_len % 32;

        let mut common = 0;

        for i in 0..chunks {
            let offset = i * 32;
            let a_vec = _mm256_loadu_si256(a_bytes.as_ptr().add(offset) as *const __m256i);
            let b_vec = _mm256_loadu_si256(b_bytes.as_ptr().add(offset) as *const __m256i);
            let cmp = _mm256_cmpeq_epi8(a_vec, b_vec);

            // 检查哪些字节相等
            let mask = _mm256_movemask_epi8(cmp);
            if mask == -1 {
                // 所有32字节都相等
                common += 32;
            } else {
                // 找到第一个不匹配的字节
                let trailing_zeros = (!mask as u32).trailing_zeros() as usize;
                return common + trailing_zeros;
            }
        }

        // 处理剩余字节
        for i in 0..remainder {
            let offset = chunks * 32 + i;
            if a_bytes[offset] == b_bytes[offset] {
                common += 1;
            } else {
                break;
            }
        }

        common
    }

    /// 查找最长公共前缀长度（安全包装）
    #[cfg(feature = "simd")]
    #[inline]
    pub fn longest_common_prefix(a: &str, b: &str) -> usize {
        if Self::is_avx2_supported() {
            unsafe { Self::longest_common_prefix_simd(a, b) }
        } else {
            a.chars()
                .zip(b.chars())
                .take_while(|(ca, cb)| ca == cb)
                .count()
        }
    }

    /// 查找最长公共前缀长度（回退到普通实现）
    #[cfg(not(feature = "simd"))]
    #[inline]
    pub fn longest_common_prefix(a: &str, b: &str) -> usize {
        a.chars()
            .zip(b.chars())
            .take_while(|(ca, cb)| ca == cb)
            .count()
    }

    /// 查找字符位置（使用SIMD优化）
    #[cfg(feature = "simd")]
    #[inline]
    #[target_feature(enable = "avx2")]
    pub unsafe fn find_char_simd(text: &str, ch: char) -> Option<usize> {
        let target_byte = ch as u8;
        let bytes = text.as_bytes();
        let len = bytes.len();

        // 创建广播的目标字节向量
        let target_vec = _mm256_set1_epi8(target_byte as i8);

        // 使用AVX2进行32字节块的搜索
        let chunks = len / 32;
        let remainder = len % 32;

        for i in 0..chunks {
            let offset = i * 32;
            let text_vec = _mm256_loadu_si256(bytes.as_ptr().add(offset) as *const __m256i);
            let cmp = _mm256_cmpeq_epi8(text_vec, target_vec);

            // 检查是否有匹配
            let mask = _mm256_movemask_epi8(cmp);
            if mask != 0 {
                // 找到匹配的字节
                let pos = mask.trailing_zeros() as usize;
                return Some(offset + pos);
            }
        }

        // 处理剩余字节
        for i in 0..remainder {
            let offset = chunks * 32 + i;
            if bytes[offset] == target_byte {
                return Some(offset);
            }
        }

        None
    }

    /// 查找字符位置（安全包装）
    #[cfg(feature = "simd")]
    #[inline]
    pub fn find_char(text: &str, ch: char) -> Option<usize> {
        if Self::is_avx2_supported() {
            unsafe { Self::find_char_simd(text, ch) }
        } else {
            text.find(ch)
        }
    }

    /// 查找字符位置（回退到普通实现）
    #[cfg(not(feature = "simd"))]
    #[inline]
    pub fn find_char(text: &str, ch: char) -> Option<usize> {
        text.find(ch)
    }

    /// 检查CPU是否支持AVX2
    #[cfg(feature = "simd")]
    pub fn is_avx2_supported() -> bool {
        is_x86_feature_detected!("avx2")
    }

    /// 检查CPU是否支持AVX2（回退实现）
    #[cfg(not(feature = "simd"))]
    pub fn is_avx2_supported() -> bool {
        false
    }
}

/// SIMD优化的路径分割器
pub struct SimdPathSplitter;

impl SimdPathSplitter {
    /// 快速分割路径（使用SIMD优化）
    #[cfg(feature = "simd")]
    pub fn split_simd(path: &str) -> Vec<&str> {
        if path.is_empty() {
            return Vec::new();
        }

        let bytes = path.as_bytes();
        let len = bytes.len();
        let mut segments = Vec::with_capacity(8);
        let mut start = 0;

        // 跳过开头的 '/'
        if len > 0 && bytes[0] == b'/' {
            start = 1;
        }

        for i in start..len {
            if bytes[i] == b'/' {
                let segment = unsafe { std::str::from_utf8_unchecked(&bytes[start..i]) };
                if !segment.is_empty() {
                    segments.push(segment);
                }
                start = i + 1;
            }
        }

        // 添加最后一个段
        if start < len {
            let segment = unsafe { std::str::from_utf8_unchecked(&bytes[start..]) };
            if !segment.is_empty() {
                segments.push(segment);
            }
        }

        segments
    }

    /// 快速分割路径（回退到普通实现）
    #[cfg(not(feature = "simd"))]
    pub fn split_simd(path: &str) -> Vec<&str> {
        path.split('/')
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// 快速统计路径段数量（使用SIMD优化）
    #[cfg(feature = "simd")]
    pub fn count_segments_simd(path: &str) -> usize {
        if path.is_empty() {
            return 0;
        }

        let bytes = path.as_bytes();
        let len = bytes.len();

        // 使用SIMD计算斜杠数量
        let chunks = len / 32;
        let remainder = len % 32;

        let mut slash_count = 0;

        for i in 0..chunks {
            let offset = i * 32;
            unsafe {
                let vec = _mm256_loadu_si256(bytes.as_ptr().add(offset) as *const __m256i);
                let slash_vec = _mm256_set1_epi8(b'/' as i8);
                let cmp = _mm256_cmpeq_epi8(vec, slash_vec);
                let mask = _mm256_movemask_epi8(cmp);
                slash_count += mask.count_ones() as usize;
            }
        }

        // 处理剩余字节
        for i in 0..remainder {
            let offset = chunks * 32 + i;
            if bytes[offset] == b'/' {
                slash_count += 1;
            }
        }

        // 段数量 = 斜杠数量 + 1（如果路径不为空）
        if len > 0 && bytes[0] != b'/' {
            slash_count + 1
        } else if slash_count > 0 {
            slash_count
        } else {
            0
        }
    }

    /// 快速统计路径段数量（回退到普通实现）
    #[cfg(not(feature = "simd"))]
    pub fn count_segments_simd(path: &str) -> usize {
        path.split('/')
            .filter(|s| !s.is_empty())
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_equals() {
        let a = "hello world";
        let b = "hello world";
        let c = "hello there";

        assert!(SimdComparator::equals(a, b));
        assert!(!SimdComparator::equals(a, c));
    }

    #[test]
    fn test_simd_starts_with() {
        let text = "hello world";
        let prefix = "hello";
        let wrong = "world";

        assert!(SimdComparator::starts_with(text, prefix));
        assert!(!SimdComparator::starts_with(text, wrong));
    }

    #[test]
    fn test_simd_longest_common_prefix() {
        let a = "hello world";
        let b = "hello there";

        let lcp = SimdComparator::longest_common_prefix(a, b);
        assert_eq!(lcp, 6); // "hello "
    }

    #[test]
    fn test_simd_find_char() {
        let text = "hello/world/path";

        let pos = SimdComparator::find_char(text, '/');
        assert_eq!(pos, Some(5));

        let pos = SimdComparator::find_char(text, 'x');
        assert_eq!(pos, None);
    }

    #[test]
    fn test_simd_split() {
        let path = "/users/123/posts";

        let segments = SimdPathSplitter::split_simd(path);
        assert_eq!(segments, vec!["users", "123", "posts"]);
    }

    #[test]
    fn test_simd_count_segments() {
        let path = "/users/123/posts";

        let count = SimdPathSplitter::count_segments_simd(path);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_simd_empty_path() {
        let path = "";

        let segments = SimdPathSplitter::split_simd(path);
        assert!(segments.is_empty());

        let count = SimdPathSplitter::count_segments_simd(path);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_simd_long_string() {
        // 测试超过32字节的字符串
        let long_text = "this is a very long string that exceeds 32 bytes for testing";
        let prefix = "this is a very long string that exceeds 32 bytes";

        assert!(SimdComparator::starts_with(long_text, prefix));
    }

    #[test]
    #[cfg(feature = "simd")]
    fn test_simd_avx2_support() {
        // 测试AVX2支持检测
        let supported = SimdComparator::is_avx2_supported();
        println!("AVX2 supported: {}", supported);
    }
}