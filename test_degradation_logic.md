# 缓存降级逻辑测试说明

## 改进内容

### 1. 增加失败计数器：连续失败 N 次才降级
- **配置**: `consecutive_failures_threshold = 3`
- **行为**: 只有连续失败 3 次才会触发降级
- **恢复**: 成功一次就重置计数器

### 2. 区分错误类型：只有超时/连接错误才降级
- **新增错误类型**: `TimeoutError` (超时错误)
- **严重错误**: `ConnectionError` 和 `TimeoutError`
- **降级阈值**: 严重错误连续 2 次就降级 (`critical_error_threshold = 2`)
- **普通错误**: 连续 3 次才降级

### 3. 滑动窗口：时间窗口内有 M% 失败率才降级
- **配置**:
  - `enable_sliding_window = false` (默认禁用)
  - `sliding_window_seconds = 60` (60秒窗口)
  - `sliding_window_failure_rate = 50.0` (50%失败率)
- **行为**: 统计最近60秒内的操作，如果失败率≥50%则降级
- **内存限制**: 最多记录1000次操作

## 测试场景

### 场景1: 偶发失败不应降级
```
操作1: 成功 ✅
操作2: 失败 ❌ (连续1次)
操作3: 成功 ✅ (计数器重置)
操作4: 失败 ❌ (连续1次)
操作5: 成功 ✅ (计数器重置)
```
**预期**: 不会降级

### 场景2: 连续失败3次触发降级
```
操作1: 失败 ❌ (连续1次)
操作2: 失败 ❌ (连续2次)
操作3: 失败 ❌ (连续3次) -> 触发降级
```
**预期**: 触发降级，使用本地缓存

### 场景3: 严重错误快速降级
```
操作1: 超时 ❌ (严重错误, 连续1次)
操作2: 超时 ❌ (严重错误, 连续2次) -> 触发降级
```
**预期**: 连续2次严重错误就降级

### 场景4: 滑动窗口降级 (需启用 enable_sliding_window)
```
时间0: 操作1 失败 ❌
时间5: 操作2 成功 ✅
时间10: 操作3 失败 ❌
时间15: 操作4 失败 ❌
时间20: 操作5 失败 ❌
```
**结果**: 5次操作中4次失败，失败率80% > 50%，触发降级

## 日志输出

### 正常降级日志
```
⚠️  Valkey 连续失败 3 次，触发降级
```

### 严重错误降级日志
```
⚠️  Valkey 主缓存失败 (严重错误, 连续 2 次): Valkey operation timed out after 5s, 触发降级
```

### 滑动窗口降级日志
```
⚠️  Valkey 滑动窗口失败率 80.0% (4/5), 触发降级
```

### 未达到阈值的警告日志
```
⚠️  Valkey 主缓存失败 (严重错误, 连续 1/2 次): Valkey operation timed out after 5s
```

## 如何启用滑动窗口

在 `DegradationConfig::default()` 中修改：
```rust
pub fn default() -> Self {
    Self {
        consecutive_failures_threshold: 3,
        critical_error_threshold: 2,
        enable_sliding_window: true,  // 改为 true
        sliding_window_seconds: 60,
        sliding_window_failure_rate: 50.0,
    }
}
```

## 代码位置

- 错误类型定义: `src/cache/backend.rs:26-33`
- 降级配置: `src/cache/manager.rs:17-32`
- 降级逻辑: `src/cache/manager.rs:229-246` (get方法)
- 超时错误处理: `src/cache/valkey.rs:93-97`