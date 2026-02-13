# 测试文章 02 - WebAssembly 入门

WebAssembly (Wasm) 是一种可以在现代 Web 浏览器中运行的新型代码格式。

## 什么是 WebAssembly？

WebAssembly 是一种二进制指令格式，旨在为 Web 提供高性能的执行环境。

## 使用场景

- 图像/视频处理
- 游戏引擎
- 科学计算

## 示例代码

```rust
#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

标签：wasm, webassembly, 前端, rust
分类：技术
摘要：WebAssembly 是一种可以在现代 Web 浏览器中运行的新型代码格式，本文介绍其基础概念和使用场景。
封面：/img/passage-cover2.webp