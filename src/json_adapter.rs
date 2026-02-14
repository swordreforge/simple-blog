//! JSON 适配层 - 提供 SIMD 优化的 JSON 序列化/反序列化
//!
//! 当 `simd` feature 启用时，使用 simd-json；否则使用 serde_json。
//!
//! 使用方式：
//! ```rust
//! use crate::json_adapter::{to_string, from_str};
//! let data = MyStruct { field: "value" };
//! let json_str = to_string(&data)?;
//! let parsed: MyStruct = from_str(&json_str)?;
//! ```

use serde::{Deserialize, Serialize};

#[cfg(feature = "simd")]
mod simd_impl {
    use super::*;

    /// 序列化 - 使用 SIMD 优化
    #[allow(dead_code)]
    pub fn to_string<T: Serialize>(value: &T) -> Result<String, simd_json::Error> {
        simd_json::serde::to_string(value)
    }

    /// 反序列化 - 使用 SIMD 优化
    ///
    /// # Safety
    ///
    /// 此函数使用 `unsafe` 块包裹 `simd_json::serde::from_str`。
    /// 为什么是安全的：
    /// 1. 我们先创建一个 `String` 的可变副本，拥有完整的所有权
    /// 2. 传递给 `simd_json::serde::from_str` 的 `&mut str` 引用指向我们拥有的内存
    /// 3. 没有其他引用指向这块内存，因此不存在数据竞争
    /// 4. 解析过程中 simd-json 只会修改字符串内容（如反转义），不会越界
    ///
    /// simd-json 标记为 unsafe 的原因是其内部使用了 SIMD 指令进行内存操作，
    /// 而不是因为 API 设计本身有内存安全问题。
    #[allow(dead_code)]
    pub fn from_str<T: for<'de> Deserialize<'de>>(s: &str) -> Result<T, simd_json::Error> {
        let mut buf = s.to_string();
        unsafe {
            simd_json::serde::from_str(&mut buf)
        }
    }

    /// 使用复用缓冲区的反序列化（适用于高频场景）
    ///
    /// # Safety
    /// 参考 `from_str` 的安全性说明。
    #[allow(dead_code)]
    pub fn from_str_with_buf<T: for<'de> Deserialize<'de>>(buf: &mut str) -> Result<T, simd_json::Error> {
        unsafe {
            simd_json::serde::from_str(buf)
        }
    }

    /// 转换为紧凑 JSON 字符串（无空格）
    #[allow(dead_code)]
    pub fn to_vec<T: Serialize>(value: &T) -> Result<Vec<u8>, simd_json::Error> {
        simd_json::serde::to_vec(value)
    }

    /// 从字节数组反序列化
    ///
    /// # Safety
    /// 参考 `from_str` 的安全性说明。
    #[allow(dead_code)]
    pub fn from_slice<T: for<'de> Deserialize<'de>>(s: &[u8]) -> Result<T, simd_json::Error> {
        let mut buf = String::from_utf8_lossy(s).to_string();
        unsafe {
            simd_json::serde::from_str(&mut buf)
        }
    }
}

#[cfg(not(feature = "simd"))]
mod std_impl {
    use super::*;

    /// 序列化 - 使用标准 serde_json
    #[allow(dead_code)]
    pub fn to_string<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
        serde_json::to_string(value)
    }

    /// 反序列化 - 使用标准 serde_json
    #[allow(dead_code)]
    pub fn from_str<'a, T: Deserialize<'a>>(s: &'a str) -> Result<T, serde_json::Error> {
        serde_json::from_str(s)
    }

    /// 使用复用缓冲区的反序列化（与标准实现相同，但保持 API 一致）
    #[allow(dead_code)]
    pub fn from_str_with_buf<T: for<'de> Deserialize<'de>>(buf: &mut str) -> Result<T, serde_json::Error> {
        serde_json::from_str(buf)
    }

    /// 转换为紧凑 JSON 字符串（无空格）
    #[allow(dead_code)]
    pub fn to_vec<T: Serialize>(value: &T) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(value)
    }

    /// 从字节数组反序列化
    #[allow(dead_code)]
    pub fn from_slice<'a, T: Deserialize<'a>>(s: &'a [u8]) -> Result<T, serde_json::Error> {
        serde_json::from_slice(s)
    }
}

// 统一导出
#[cfg(feature = "simd")]
#[allow(unused_imports)]
pub use simd_impl::{to_string, from_str, from_str_with_buf, to_vec, from_slice};

#[cfg(not(feature = "simd"))]
#[allow(unused_imports)]
pub use std_impl::{to_string, from_str, from_str_with_buf, to_vec, from_slice};

/// 获取当前使用的 JSON 后端
#[allow(dead_code)]
pub fn backend_name() -> &'static str {
    if cfg!(feature = "simd") {
        "simd-json"
    } else {
        "serde_json"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestStruct {
        name: String,
        age: u32,
        active: bool,
    }

    #[test]
    fn test_serialize() {
        let data = TestStruct {
            name: "test".to_string(),
            age: 25,
            active: true,
        };
        let result = to_string(&data).unwrap();
        assert!(result.contains("\"name\":\"test\""));
        assert!(result.contains("\"age\":25"));
        assert!(result.contains("\"active\":true"));
    }

    #[test]
    fn test_deserialize() {
        let json_str = r#"{"name":"test","age":25,"active":true}"#;
        let result: TestStruct = from_str(json_str).unwrap();
        assert_eq!(result.name, "test");
        assert_eq!(result.age, 25);
        assert_eq!(result.active, true);
    }

    #[test]
    fn test_roundtrip() {
        let original = TestStruct {
            name: "roundtrip".to_string(),
            age: 30,
            active: false,
        };
        let json_str = to_string(&original).unwrap();
        let parsed: TestStruct = from_str(&json_str).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_backend_name() {
        println!("Using JSON backend: {}", backend_name());
    }
}