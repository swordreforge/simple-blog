# RustBlog no-panic 实施指南

## 概述

本文档说明 RustBlog 项目中 `no-panic` 的实施情况和使用指南。

`no-panic` 是一个 Rust crate，可以在编译时验证函数不会发生 panic，从而提高代码的安全性和可靠性。

## 已实施的优化

### 1. 添加依赖

已在 `Cargo.toml` 中添加 `no-panic` 依赖：

```toml
# Panic 检测：编译时验证函数不会 panic
no-panic = "0.1"
```

### 2. 重构生产代码中的 unwrap/expect

已移除以下生产代码中的 panic 风险：

#### embedded.rs:147 - 路径处理
**Before:**
```rust
let relative_path = path_str.strip_prefix(src_dir).unwrap();
```

**After:**
```rust
let relative_path = path_str.strip_prefix(src_dir)
    .ok_or_else(|| format!("Path '{}' should start with '{}'", path_str, src_dir))?;
```

**理由:** 虽然前面已检查 `starts_with(src_dir)`，但使用 `ok_or` 更符合 no-panic 要求。

#### templates/mod.rs:35 - 模板路径处理
**Before:**
```rust
let name = path_str.strip_prefix("templates/")
    .expect("path should start with 'templates/' after check");
```

**After:**
```rust
let name = path_str.strip_prefix("templates/")
    .ok_or("Path should start with 'templates/' after check")?;
```

**理由:** 使用 `ok_or` 替代 `expect`，确保即使条件检查失败也不会 panic。

#### profiling.rs:72 - 火焰图生成
**Before:**
```rust
guard.report().build().unwrap().flamegraph(&mut file)?;
```

**After:**
```rust
let report = guard.report().build()?;
report.flamegraph(&mut file)?;
```

**理由:** 使用 `?` 操作符传播错误，避免 unwrap 导致的 panic。

#### cache/manager.rs:222 - Valkey URL 处理
**Before:**
```rust
let reconnect_task = if valkey_url_owned.is_some() && valkey_backend.is_none() {
    let url = valkey_url_owned.clone().unwrap();
    // ...
}
```

**After:**
```rust
let reconnect_task = if valkey_backend.is_none() {
    if let Some(url) = valkey_url_owned.clone() {
        // ...
    }
}
```

**理由:** 使用 `if let` 模式替代 `is_some()` + `unwrap()`，提高代码可读性和安全性。

#### services/route_type_manager.rs:298 - 模板路径访问
**Before:**
```rust
route.template_path = Some(template_path);
// ...
route.template_path.as_ref().unwrap()
```

**After:**
```rust
route.template_path = Some(template_path.clone());
// ...
template_path
```

**理由:** 使用已存在的变量避免重复访问，避免 unwrap。

#### handlers/api_handlers/dynamic_routes/export.rs:98 - JSON 数组访问
**Before:**
```rust
let routes = match import_obj.get("routes") {
    Some(r) if r.is_array() => r.as_array().unwrap(),
    _ => { /* error */ }
};
```

**After:**
```rust
let routes = match import_obj.get("routes") {
    Some(r) => match r.as_array() {
        Some(arr) => arr,
        None => { /* error: not array */ }
    },
    None => { /* error: missing field */ }
};
```

**理由:** 使用嵌套 `match` 提供更精确的错误信息。

## no-panic 使用指南

### 在函数上使用 #[no_panic] 属性

对于关键函数，可以添加 `#[no_panic]` 属性来确保编译时验证：

```rust
use no_panic::no_panic;

#[no_panic]
pub fn critical_function(x: i32, y: i32) -> i32 {
    // 这个函数必须保证不会 panic
    x + y  // 安全：整数加法不会 panic
}
```

### no-panic 的限制

使用 `#[no_panic]` 的函数必须满足以下条件：

1. **不能使用 unwrap() 或 expect()**
2. **不能使用索引访问（如 arr[i]）** - 应使用 get(i)
3. **不能进行可能导致溢出的算术运算**（如 checked_add）
4. **不能调用可能 panic 的函数**
5. **不能使用 panic! 宏**

### 示例：安全的 no-panic 函数

```rust
use no_panic::no_panic;

#[no_panic]
pub fn safe_add(a: i32, b: i32) -> Option<i32> {
    a.checked_add(b)  // 返回 Option，不会 panic
}

#[no_panic]
pub fn safe_string_length(s: &str) -> usize {
    s.len()  // 获取字符串长度是安全的
}

#[no_panic]
pub fn safe_index_access<T>(arr: &[T], index: usize) -> Option<&T> {
    arr.get(index)  // 使用 get 而不是索引访问
}
```

### 示例：不能使用 no-panic 的函数

```rust
// ❌ 不能使用 no-panic
pub fn unsafe_function(arr: &[i32], index: usize) -> i32 {
    arr[index]  // 可能 panic（索引越界）
}

// ❌ 不能使用 no-panic
pub fn unwrap_function(opt: Option<i32>) -> i32 {
    opt.unwrap()  // 可能 panic（Option 为 None）
}

// ❌ 不能使用 no-panic
pub fn panic_function(x: i32) {
    if x < 0 {
        panic!("x must be non-negative");
    }
}
```

## 编译时验证

在 release 模式下，`no-panic` 会进行编译时验证：

```bash
cargo build --release
```

如果标记为 `#[no_panic]` 的函数可能 panic，编译会失败。

## 测试代码的处理

测试代码不受 `no-panic` 限制，因为：
- 测试本身需要验证边界条件
- `unwrap()` 和 `expect()` 在测试中是合理的
- 测试代码不会被部署到生产环境

## 当前项目中的硬编码正则

以下正则表达式保持不变，因为它们是硬编码的，编译时已知有效：

1. `services/summarize_service.rs` - 14 个 Markdown 处理正则
2. `services/passage_service.rs:13` - HTML 标签移除正则

这些正则表达式使用 `unwrap()` 是安全的，因为：
- 正则表达式是硬编码的字符串
- 在编译时已经验证有效
- 不会在运行时失败

## 性能影响

使用 `no-panic` 的性能影响：

- **编译时间**: 略有增加（额外的编译时检查）
- **运行时性能**: 无影响（`no-panic` 只在编译时检查）
- **二进制大小**: 无影响

## 最佳实践

### 1. 渐进式采用

不需要一次性将所有函数都标记为 `#[no_panic]`。建议：
- 从关键路径的函数开始
- 逐步扩展到其他函数
- 优先处理用户直接调用的 API

### 2. 错误处理策略

使用 `no-panic` 时，推荐以下错误处理模式：

```rust
// ✅ 推荐：返回 Result
pub fn safe_parse(input: &str) -> Result<i32, ParseError> {
    input.parse().map_err(ParseError::InvalidFormat)
}

// ✅ 推荐：返回 Option
pub fn safe_get(arr: &[i32], index: usize) -> Option<i32> {
    arr.get(index).copied()
}

// ❌ 避免：使用 unwrap/expect
pub fn unsafe_parse(input: &str) -> i32 {
    input.parse().unwrap()
}
```

### 3. 结合其他安全措施

`no-panic` 应与其他安全措施结合使用：

- **单元测试**: 验证错误处理逻辑
- **集成测试**: 验证整体行为
- **代码审查**: 人工检查潜在问题
- **Clippy**: 使用 lint 检查常见问题

## 故障排查

### 编译错误：function may panic

如果标记为 `#[no_panic]` 的函数编译失败：

1. 检查函数内是否有 unwrap() 或 expect()
2. 检查是否有索引访问（如 arr[i]）
3. 检查是否有算术运算可能导致溢出
4. 检查调用的其他函数是否可能 panic

### 解决方案

```rust
// 问题：函数可能 panic
#[no_panic]
pub fn problematic(arr: &[i32], index: usize) -> i32 {
    arr[index]  // ❌ 编译错误
}

// 解决方案：使用安全的访问方式
#[no_panic]
pub fn safe(arr: &[i32], index: usize) -> Option<i32> {
    arr.get(index).copied()  // ✅ 编译通过
}
```

## 未来计划

1. **扩展覆盖范围**: 逐步为更多关键函数添加 `#[no_panic]`
2. **文档完善**: 为每个 `#[no_panic]` 函数添加文档说明
3. **测试覆盖**: 确保所有 `#[no_panic]` 函数有充分的测试
4. **性能监控**: 监控使用 `no-panic` 后的性能变化

## 参考资料

- [no-panic crate 文档](https://docs.rs/no-panic/)
- [Rust Error Handling Best Practices](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [The Rustonomicon - Unwinding](https://doc.rust-lang.org/nomicon/unwinding.html)

## 更新日志

- 2026-03-28: 初始版本
  - 添加 `no-panic` 依赖
  - 重构 6 处生产代码中的 unwrap/expect
  - 编写实施指南

## 联系方式

如有问题或建议，请联系：
- 作者: swordreforge <zhujian_20060818@qq.com>
- 项目地址: https://github.com/swordreforge/simple-blog