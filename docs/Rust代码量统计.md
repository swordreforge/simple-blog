# Rust 代码量统计

**统计日期**: 2026年3月22日
**项目**: rustblog

## 总体概览

| 指标 | 数值 |
|------|------|
| 总代码行数 | 58,582 行 |
| 总文件数 | 168 个 .rs 文件 |

## 按模块分布

### 1. 主项目 (src/)

| 指标 | 数值 |
|------|------|
| 文件数 | 93 个 |
| 代码行数 | 34,328 行 |
| 占比 | 58.6% |

**最大的文件** (前 5 名):

| 文件路径 | 行数 |
|----------|------|
| `src/db/repositories.rs` | 2,978 行 |
| `src/db/init.rs` | 2,268 行 |
| `src/templates/mod.rs` | 1,406 行 |
| `src/handlers/api_handlers/passage/crud.rs` | 1,339 行 |
| `src/services/route_storage.rs` | 948 行 |

### 2. 动态路由模块 (dynamic-router/)

| 指标 | 数值 |
|------|------|
| 源码文件数 | 34 个 |
| 源码行数 | 15,821 行 |
| 测试文件数 | 15 个 |
| 测试行数 | 4,751 行 |
| 示例文件数 | 6 个 |
| 示例行数 | 924 行 |
| 基准测试文件数 | 2 个 |
| 基准测试行数 | 798 行 |
| 总占比 | 38.2% (含测试和示例) |

**最大的源文件** (前 5 名):

| 文件路径 | 行数 |
|----------|------|
| `dynamic-router/src/storage/database_storage.rs` | 983 行 |
| `dynamic-router/src/core/route_table.rs` | 978 行 |
| `dynamic-router/src/core/object_pool.rs` | 801 行 |
| `dynamic-router/src/core/bytes_optimized.rs` | 778 行 |
| `dynamic-router/src/core/dynamic_sharding.rs` | 728 行 |

### 3. 中文摘要模块 (summarize-in-zh-cn/)

| 指标 | 数值 |
|------|------|
| 文件数 | 7 个 |
| 代码行数 | 987 行 |
| 占比 | 1.7% |

**文件列表**:

| 文件路径 | 行数 |
|----------|------|
| `summarize-in-zh-cn/src/summarizer.rs` | 335 行 |
| `summarize-in-zh-cn/src/similarity.rs` | 269 行 |
| `summarize-in-zh-cn/src/textrank.rs` | 106 行 |
| `summarize-in-zh-cn/src/lib.rs` | 94 行 |
| `summarize-in-zh-cn/src/main.rs` | 77 行 |
| `summarize-in-zh-cn/src/sentence.rs` | 62 行 |
| `summarize-in-zh-cn/src/tokenizer.rs` | 44 行 |

## 代码规模分析

### 模块占比

```
主项目          ████████████████████████████████████████████████████████████ 58.6%
动态路由        ████████████████████████████████████████████████           38.2%
中文摘要        ███                                                        1.7%
```

### 复杂度分析

**高复杂度文件** (>1000 行):

1. `src/db/repositories.rs` (2,978 行) - 数据库操作
2. `src/db/init.rs` (2,268 行) - 数据库初始化
3. `src/templates/mod.rs` (1,406 行) - 模板管理
4. `src/handlers/api_handlers/passage/crud.rs` (1,339 行) - 文章 CRUD 操作

**中等复杂度文件** (500-1000 行):

- `src/services/route_storage.rs` (948 行) - 路由存储服务
- `src/services/dynamic_route_service.rs` (778 行) - 动态路由服务
- `src/cache/manager.rs` (761 行) - 缓存管理
- `src/services/route_type_manager.rs` (754 行) - 路由类型管理
- `src/db/models.rs` (754 行) - 数据库模型
- `src/handlers/api_handlers/attachments.rs` (668 行) - 附件处理
- `src/handlers/api_handlers/dynamic_routes/update.rs` (669 行) - 动态路由更新
- `src/handlers/api_handlers/filemanager.rs` (582 行) - 文件管理
- `src/handlers/api_handlers/about.rs` (579 行) - 关于页面处理
- `src/lock_monitor.rs` (555 行) - 锁监控
- `src/handlers/api_handlers/settings.rs` (555 行) - 设置处理
- `src/handlers/api_handlers/music.rs` (507 行) - 音乐处理
- `src/routes/api_routes.rs` (499 行) - API 路由
- `src/main.rs` (491 行) - 主入口

## 测试覆盖率

- **动态路由测试**: 15 个测试文件，4,751 行测试代码
- **动态路由示例**: 6 个示例文件，924 行示例代码
- **动态路由基准测试**: 2 个基准测试文件，798 行基准测试代码

## 总结

项目整体代码规模较大，核心模块集中在：

1. **数据库操作** - `src/db/` 模块包含数据库初始化、模型定义和仓储操作
2. **动态路由** - `dynamic-router/` 是一个独立的动态路由系统，包含多种优化策略
3. **业务处理** - `src/handlers/` 和 `src/services/` 包含博客的主要业务逻辑
4. **模板管理** - `src/templates/` 负责页面模板的渲染和管理

建议关注高复杂度文件的重构和模块拆分，以提高代码的可维护性。