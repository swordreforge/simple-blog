//! 字节优化模块
//!
//! 使用 `Bytes` 类型高效处理二进制数据
//! 提供零拷贝操作和高效的内存管理

use bytes::{BufMut, Bytes, BytesMut};

/// 优化的字节缓冲区
///
/// 使用 `Bytes` 类型提供零拷贝的字节操作
#[derive(Debug, Clone)]
pub struct OptimizedBytes {
    inner: Bytes,
}

impl OptimizedBytes {
    /// 从字节切片创建
    pub fn from_slice(slice: &[u8]) -> Self {
        Self {
            inner: Bytes::copy_from_slice(slice),
        }
    }

    /// 从 Vec<u8> 创建
    pub fn from_vec(vec: Vec<u8>) -> Self {
        Self {
            inner: Bytes::from(vec),
        }
    }

    /// 从静态字节切片创建（零拷贝）
    pub fn from_static(slice: &'static [u8]) -> Self {
        Self {
            inner: Bytes::from_static(slice),
        }
    }

    /// 获取字节切片引用
    pub fn as_slice(&self) -> &[u8] {
        self.inner.as_ref()
    }

    /// 获取长度
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// 分割字节
    pub fn split_at(&self, mid: usize) -> (OptimizedBytes, OptimizedBytes) {
        let left = self.inner.slice(0..mid);
        let right = self.inner.slice(mid..);
        (
            OptimizedBytes { inner: left },
            OptimizedBytes { inner: right },
        )
    }

    /// 切片
    pub fn slice(&self, range: std::ops::Range<usize>) -> OptimizedBytes {
        OptimizedBytes {
            inner: self.inner.slice(range),
        }
    }

    /// 转换为 Bytes
    pub fn into_bytes(self) -> Bytes {
        self.inner
    }

    /// 转换为 Vec<u8>
    pub fn into_vec(self) -> Vec<u8> {
        self.inner.to_vec()
    }

    /// 检查是否包含模式
    pub fn contains(&self, pattern: &[u8]) -> bool {
        self.windows(pattern.len()).any(|w| w == pattern)
    }

    /// 查找模式
    pub fn find(&self, pattern: &[u8]) -> Option<usize> {
        self.windows(pattern.len())
            .position(|w| w == pattern)
    }

    /// 替换模式
    pub fn replace(&self, from: &[u8], to: &[u8]) -> OptimizedBytes {
        let mut result = BytesMut::with_capacity(self.len());
        let mut start = 0;

        while start < self.len() {
            if let Some(pos) = self[start..].windows(from.len()).position(|w| w == from) {
                let actual_pos = start + pos;
                result.extend_from_slice(&self.as_slice()[start..actual_pos]);
                result.extend_from_slice(to);
                start = actual_pos + from.len();
            } else {
                result.extend_from_slice(&self.as_slice()[start..]);
                break;
            }
        }

        OptimizedBytes {
            inner: result.freeze(),
        }
    }

    /// 转换为字符串（如果有效）
    pub fn to_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(self.as_slice())
    }
}

impl AsRef<[u8]> for OptimizedBytes {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl std::ops::Deref for OptimizedBytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl From<Vec<u8>> for OptimizedBytes {
    fn from(vec: Vec<u8>) -> Self {
        Self::from_vec(vec)
    }
}

impl From<&[u8]> for OptimizedBytes {
    fn from(slice: &[u8]) -> Self {
        Self::from_slice(slice)
    }
}

impl From<Bytes> for OptimizedBytes {
    fn from(bytes: Bytes) -> Self {
        Self { inner: bytes }
    }
}

impl From<OptimizedBytes> for Bytes {
    fn from(optimized: OptimizedBytes) -> Self {
        optimized.inner
    }
}

/// 字节池
///
/// 用于复用字节缓冲区，减少内存分配
#[derive(Debug)]
pub struct BytesPool {
    pool: Vec<BytesMut>,
    max_size: usize,
    chunk_size: usize,
}

impl BytesPool {
    /// 创建新的字节池
    pub fn new(chunk_size: usize, max_size: usize) -> Self {
        Self {
            pool: Vec::new(),
            max_size,
            chunk_size,
        }
    }

    /// 获取字节缓冲区
    pub fn get(&mut self) -> BytesMut {
        self.pool.pop().unwrap_or_else(|| BytesMut::with_capacity(self.chunk_size))
    }

    /// 归还字节缓冲区
    pub fn put(&mut self, mut buf: BytesMut) {
        if self.pool.len() < self.max_size {
            buf.clear();
            self.pool.push(buf);
        }
    }

    /// 获取指定大小的字节缓冲区
    pub fn get_with_capacity(&mut self, capacity: usize) -> BytesMut {
        let buf = self.get();
        if buf.capacity() < capacity {
            BytesMut::with_capacity(capacity)
        } else {
            buf
        }
    }

    /// 清空池
    pub fn clear(&mut self) {
        self.pool.clear();
    }

    /// 获取池中缓冲区数量
    pub fn len(&self) -> usize {
        self.pool.len()
    }

    /// 检查池是否为空
    pub fn is_empty(&self) -> bool {
        self.pool.is_empty()
    }
}

impl Default for BytesPool {
    fn default() -> Self {
        Self::new(4096, 16)
    }
}

/// 字节构建器
///
/// 用于高效构建字节缓冲区
#[derive(Debug)]
pub struct BytesBuilder {
    buf: BytesMut,
}

impl BytesBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            buf: BytesMut::new(),
        }
    }

    /// 创建带有初始容量的构建器
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: BytesMut::with_capacity(capacity),
        }
    }

    /// 追加字节
    pub fn extend(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }

    /// 追加另一个 BytesBuilder
    pub fn extend_from_builder(&mut self, other: &BytesBuilder) {
        self.buf.extend_from_slice(other.as_slice());
    }

    /// 追加 OptimizedBytes
    pub fn extend_from_optimized(&mut self, other: &OptimizedBytes) {
        self.buf.extend_from_slice(other.as_slice());
    }

    /// 写入 u8
    pub fn write_u8(&mut self, val: u8) {
        self.buf.put_u8(val);
    }

    /// 写入 u16（大端序）
    pub fn write_u16_be(&mut self, val: u16) {
        self.buf.extend_from_slice(&val.to_be_bytes());
    }

    /// 写入 u16（小端序）
    pub fn write_u16_le(&mut self, val: u16) {
        self.buf.extend_from_slice(&val.to_le_bytes());
    }

    /// 写入 u32（大端序）
    pub fn write_u32_be(&mut self, val: u32) {
        self.buf.extend_from_slice(&val.to_be_bytes());
    }

    /// 写入 u32（小端序）
    pub fn write_u32_le(&mut self, val: u32) {
        self.buf.extend_from_slice(&val.to_le_bytes());
    }

    /// 写入字符串
    pub fn write_str(&mut self, s: &str) {
        self.buf.extend_from_slice(s.as_bytes());
    }

    /// 获取当前长度
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// 获取容量
    pub fn capacity(&self) -> usize {
        self.buf.capacity()
    }

    /// 获取字节切片
    pub fn as_slice(&self) -> &[u8] {
        self.buf.as_ref()
    }

    /// 构建最终的字节
    pub fn build(self) -> OptimizedBytes {
        OptimizedBytes {
            inner: self.buf.freeze(),
        }
    }

    /// 清空构建器
    pub fn clear(&mut self) {
        self.buf.clear();
    }

    /// 预留空间
    pub fn reserve(&mut self, additional: usize) {
        self.buf.reserve(additional);
    }
}

impl Default for BytesBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AsRef<[u8]> for BytesBuilder {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

/// 字节切片视图
///
/// 提供对字节的零拷贝视图
#[derive(Debug, Clone)]
pub struct BytesView<'a> {
    inner: &'a [u8],
}

impl<'a> BytesView<'a> {
    /// 从字节切片创建视图
    pub fn new(slice: &'a [u8]) -> Self {
        Self { inner: slice }
    }

    /// 从 OptimizedBytes 创建视图
    pub fn from_optimized(bytes: &'a OptimizedBytes) -> Self {
        Self {
            inner: bytes.as_slice(),
        }
    }

    /// 获取字节切片
    pub fn as_slice(&self) -> &'a [u8] {
        self.inner
    }

    /// 获取长度
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// 切片
    pub fn slice(&self, range: std::ops::Range<usize>) -> BytesView<'a> {
        BytesView {
            inner: &self.inner[range],
        }
    }

    /// 分割
    pub fn split_at(&self, mid: usize) -> (BytesView<'a>, BytesView<'a>) {
        let (left, right) = self.inner.split_at(mid);
        (BytesView::new(left), BytesView::new(right))
    }

    /// 查找模式
    pub fn find(&self, pattern: &[u8]) -> Option<usize> {
        self.inner.windows(pattern.len()).position(|w| w == pattern)
    }

    /// 检查是否包含模式
    pub fn contains(&self, pattern: &[u8]) -> bool {
        self.find(pattern).is_some()
    }

    /// 迭代字节
    pub fn iter(&self) -> std::slice::Iter<'a, u8> {
        self.inner.iter()
    }

    /// 转换为字符串（如果有效）
    pub fn to_str(&self) -> Result<&'a str, std::str::Utf8Error> {
        std::str::from_utf8(self.inner)
    }
}

impl<'a> AsRef<[u8]> for BytesView<'a> {
    fn as_ref(&self) -> &[u8] {
        self.inner
    }
}

impl<'a> std::ops::Deref for BytesView<'a> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

/// 字节分割器
///
/// 高效地分割字节缓冲区
#[derive(Debug, Clone)]
pub struct BytesSplitter<'a> {
    bytes: &'a [u8],
    delimiter: &'a [u8],
    pos: usize,
}

impl<'a> BytesSplitter<'a> {
    /// 创建新的分割器
    pub fn new(bytes: &'a [u8], delimiter: &'a [u8]) -> Self {
        Self {
            bytes,
            delimiter,
            pos: 0,
        }
    }

    /// 获取下一个片段
    pub fn next_view(&mut self) -> Option<BytesView<'a>> {
        if self.pos >= self.bytes.len() {
            return None;
        }

        if let Some(delimiter_pos) = self.bytes[self.pos..]
            .windows(self.delimiter.len())
            .position(|w| w == self.delimiter)
        {
            let end = self.pos + delimiter_pos;
            let view = BytesView::new(&self.bytes[self.pos..end]);
            self.pos = end + self.delimiter.len();
            Some(view)
        } else {
            let view = BytesView::new(&self.bytes[self.pos..]);
            self.pos = self.bytes.len();
            Some(view)
        }
    }

    /// 收集所有片段
    pub fn collect_all(mut self) -> Vec<BytesView<'a>> {
        let mut result = Vec::new();
        while let Some(view) = self.next_view() {
            result.push(view);
        }
        result
    }
}

/// 字节比较器
///
/// 高效比较字节缓冲区
pub struct BytesComparator;

impl BytesComparator {
    /// 比较两个字节缓冲区
    pub fn compare(left: &[u8], right: &[u8]) -> std::cmp::Ordering {
        left.cmp(right)
    }

    /// 检查字节缓冲区是否相等
    pub fn equals(left: &[u8], right: &[u8]) -> bool {
        left == right
    }

    /// 检查字节缓冲区是否以模式开头
    pub fn starts_with(bytes: &[u8], pattern: &[u8]) -> bool {
        bytes.starts_with(pattern)
    }

    /// 检查字节缓冲区是否以模式结尾
    pub fn ends_with(bytes: &[u8], pattern: &[u8]) -> bool {
        bytes.ends_with(pattern)
    }
}

/// 字节转换器
///
/// 在不同字节表示之间转换
pub struct BytesConverter;

impl BytesConverter {
    /// 转换为十六进制字符串
    pub fn to_hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }

    /// 从十六进制字符串转换
    pub fn from_hex(hex: &str) -> Result<Vec<u8>, String> {
        if !hex.len().is_multiple_of(2) {
            return Err("Hex string must have even length".to_string());
        }

        (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|e| e.to_string()))
            .collect()
    }

    /// 转换为 Base64
    pub fn to_base64(bytes: &[u8]) -> String {
        use base64::prelude::*;
        BASE64_STANDARD.encode(bytes)
    }

    /// 从 Base64 转换
    pub fn from_base64(base64: &str) -> Result<Vec<u8>, String> {
        use base64::prelude::*;
        BASE64_STANDARD
            .decode(base64)
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimized_bytes_from_slice() {
        let bytes = OptimizedBytes::from_slice(b"hello");
        assert_eq!(bytes.as_slice(), b"hello");
        assert_eq!(bytes.len(), 5);
    }

    #[test]
    fn test_optimized_bytes_from_static() {
        let bytes = OptimizedBytes::from_static(b"static");
        assert_eq!(bytes.as_slice(), b"static");
    }

    #[test]
    fn test_optimized_bytes_split_at() {
        let bytes = OptimizedBytes::from_slice(b"hello world");
        let (left, right) = bytes.split_at(5);
        assert_eq!(left.as_slice(), b"hello");
        assert_eq!(right.as_slice(), b" world");
    }

    #[test]
    fn test_optimized_bytes_slice() {
        let bytes = OptimizedBytes::from_slice(b"hello world");
        let sliced = bytes.slice(0..5);
        assert_eq!(sliced.as_slice(), b"hello");
    }

    #[test]
    fn test_optimized_bytes_contains() {
        let bytes = OptimizedBytes::from_slice(b"hello world");
        assert!(bytes.contains(b"world"));
        assert!(!bytes.contains(b"rust"));
    }

    #[test]
    fn test_optimized_bytes_find() {
        let bytes = OptimizedBytes::from_slice(b"hello world");
        assert_eq!(bytes.find(b"world"), Some(6));
        assert_eq!(bytes.find(b"rust"), None);
    }

    #[test]
    fn test_optimized_bytes_replace() {
        let bytes = OptimizedBytes::from_slice(b"hello world");
        let replaced = bytes.replace(b"world", b"rust");
        assert_eq!(replaced.as_slice(), b"hello rust");
    }

    #[test]
    fn test_optimized_bytes_to_str() {
        let bytes = OptimizedBytes::from_slice(b"hello");
        assert_eq!(bytes.to_str().unwrap(), "hello");
    }

    #[test]
    fn test_bytes_pool() {
        let mut pool = BytesPool::new(1024, 4);

        let buf1 = pool.get();
        assert_eq!(buf1.capacity(), 1024);

        pool.put(buf1);

        let buf2 = pool.get();
        assert_eq!(buf2.capacity(), 1024);
        assert_eq!(pool.len(), 0);
    }

    #[test]
    fn test_bytes_pool_get_with_capacity() {
        let mut pool = BytesPool::new(1024, 4);

        let buf = pool.get_with_capacity(2048);
        assert!(buf.capacity() >= 2048);
    }

    #[test]
    fn test_bytes_builder() {
        let mut builder = BytesBuilder::new();
        builder.extend(b"hello");
        builder.extend(b" ");
        builder.extend(b"world");

        let bytes = builder.build();
        assert_eq!(bytes.as_slice(), b"hello world");
    }

    #[test]
    fn test_bytes_builder_with_capacity() {
        let builder = BytesBuilder::with_capacity(100);
        assert!(builder.capacity() >= 100);
    }

    #[test]
    fn test_bytes_builder_write_u8() {
        let mut builder = BytesBuilder::new();
        builder.write_u8(0x42);
        builder.write_u8(0x43);

        let bytes = builder.build();
        assert_eq!(bytes.as_slice(), &[0x42, 0x43]);
    }

    #[test]
    fn test_bytes_builder_write_u16_be() {
        let mut builder = BytesBuilder::new();
        builder.write_u16_be(0x1234);

        let bytes = builder.build();
        assert_eq!(bytes.as_slice(), &[0x12, 0x34]);
    }

    #[test]
    fn test_bytes_builder_write_str() {
        let mut builder = BytesBuilder::new();
        builder.write_str("hello");

        let bytes = builder.build();
        assert_eq!(bytes.as_slice(), b"hello");
    }

    #[test]
    fn test_bytes_view() {
        let bytes = OptimizedBytes::from_slice(b"hello world");
        let view = BytesView::from_optimized(&bytes);

        assert_eq!(view.as_slice(), b"hello world");
        assert_eq!(view.len(), 11);
    }

    #[test]
    fn test_bytes_view_slice() {
        let bytes = OptimizedBytes::from_slice(b"hello world");
        let view = BytesView::from_optimized(&bytes);
        let sliced = view.slice(0..5);

        assert_eq!(sliced.as_slice(), b"hello");
    }

    #[test]
    fn test_bytes_view_find() {
        let bytes = OptimizedBytes::from_slice(b"hello world");
        let view = BytesView::from_optimized(&bytes);

        assert_eq!(view.find(b"world"), Some(6));
    }

    #[test]
    fn test_bytes_splitter() {
        let bytes = b"a,b,c,d";
        let mut splitter = BytesSplitter::new(bytes, b",");

        assert_eq!(splitter.next().unwrap().as_slice(), b"a");
        assert_eq!(splitter.next().unwrap().as_slice(), b"b");
        assert_eq!(splitter.next().unwrap().as_slice(), b"c");
        assert_eq!(splitter.next().unwrap().as_slice(), b"d");
        assert!(splitter.next().is_none());
    }

    #[test]
    fn test_bytes_splitter_collect_all() {
        let bytes = b"a,b,c";
        let splitter = BytesSplitter::new(bytes, b",");
        let segments = splitter.collect_all();

        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0].as_slice(), b"a");
        assert_eq!(segments[1].as_slice(), b"b");
        assert_eq!(segments[2].as_slice(), b"c");
    }

    #[test]
    fn test_bytes_comparator_compare() {
        let left = b"abc";
        let right = b"abd";

        assert_eq!(BytesComparator::compare(left, right), std::cmp::Ordering::Less);
    }

    #[test]
    fn test_bytes_comparator_starts_with() {
        let bytes = b"hello world";

        assert!(BytesComparator::starts_with(bytes, b"hello"));
        assert!(!BytesComparator::starts_with(bytes, b"world"));
    }

    #[test]
    fn test_bytes_comparator_ends_with() {
        let bytes = b"hello world";

        assert!(BytesComparator::ends_with(bytes, b"world"));
        assert!(!BytesComparator::ends_with(bytes, b"hello"));
    }

    #[test]
    fn test_bytes_converter_to_hex() {
        let bytes = b"abc";
        let hex = BytesConverter::to_hex(bytes);

        assert_eq!(hex, "616263");
    }

    #[test]
    fn test_bytes_converter_from_hex() {
        let hex = "616263";
        let bytes = BytesConverter::from_hex(hex).unwrap();

        assert_eq!(bytes, b"abc");
    }

    #[test]
    fn test_bytes_converter_to_base64() {
        let bytes = b"abc";
        let base64 = BytesConverter::to_base64(bytes);

        assert_eq!(base64, "YWJj");
    }

    #[test]
    fn test_bytes_converter_from_base64() {
        let base64 = "YWJj";
        let bytes = BytesConverter::from_base64(base64).unwrap();

        assert_eq!(bytes, b"abc");
    }
}