# Dynamic Router 集成到 RustBlog 开发计划

## 项目概述

将 `dynamic-router` (基于 Actix-Web 的动态路由管理库) 集成到 RustBlog 项目中，为用户提供运行时动态添加、删除和管理路由的能力。

### 目标
- 用户可以通过管理界面动态添加自定义路由
- 支持多种路由类型（重定向、静态内容、模板渲染、自定义处理器）
- 路由配置持久化到数据库
- 实时生效，无需重启服务
- 保持与现有路由系统的兼容性

---

## 一、技术架构分析

### 1.1 Dynamic Router 核心特性

基于代码分析，`dynamic-router` 库提供以下核心功能：

```rust
// 核心模块
pub mod core;        // 核心数据结构和抽象
pub mod storage;     // 持久化存储接口和实现
pub mod actix;       // Actix-Web 集成

// 主要类型
pub use core::{ArcRouteEntry, RouteEntry, RouteTable, SimpleRoute};
pub use core::cache_optimized::CacheOptimizedRouteTable;
pub use core::dynamic_route_table::DynamicRouteTable;
pub use storage::{FileStorage, MemoryStorage, RouteStorage};
```

**特性：**
- 线程安全的路由表 (`RouteTable` 使用 `RwLock<HashMap>`)
- 支持多种持久化方式（内存、文件、数据库）
- 动态分片和负载均衡
- 缓存优化版本
- Actix-Web 深度集成

### 1.2 RustBlog 现有架构

```rust
// 当前路由系统
src/routes/
├── mod.rs           // 路由配置入口
├── api_routes.rs    // API 路由
├── page_routes.rs   // 页面路由
└── static_routes.rs // 静态文件路由

// 数据库
- SQLite (rusqlite + r2d2)
- 现有 Repository 模式

// 状态管理
- AppState (依赖注入容器)
- Arc<Repository>
- Arc<AppCache>
```

---

## 二、集成方案设计

### 2.1 整体架构

```
┌─────────────────────────────────────────────────────────┐
│                     RustBlog 应用                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐      │
│  │ 现有路由     │  │ 动态路由表   │  │ 管理界面     │      │
│  │ (静态编译)   │  │ (运行时)     │  │ (新增)       │      │
│  └─────────────┘  └─────────────┘  └─────────────┘      │
│         ↓                ↓                ↓              │
│  ┌─────────────────────────────────────────────────┐    │
│  │              路由分发器 (Middleware)              │    │
│  │  - 静态路由优先                                    │    │
│  │  - 动态路由兜底                                    │    │
│  └─────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
                            ↓
                  ┌─────────────────┐
                  │   SQLite 数据库  │
                  │ - routes 表     │
                  │ - 路由持久化     │
                  └─────────────────┘
```

### 2.2 路由匹配策略

1. **优先级**：静态路由 > 动态路由
2. **匹配流程**：
   ```
   请求 → 静态路由匹配 → 找到？→ 处理
          ↓ 未找到
          动态路由表匹配 → 找到？→ 处理
          ↓ 未找到
          404 处理
   ```

### 2.3 数据库设计

新增 `dynamic_routes` 表：

```sql
CREATE TABLE dynamic_routes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL UNIQUE,
    route_type TEXT NOT NULL, -- 'redirect', 'static', 'template', 'custom'
    config TEXT NOT NULL,     -- JSON 配置
    status TEXT NOT NULL,     -- 'active', 'disabled', 'archived'
    priority INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    created_by TEXT,          -- 创建者
    metadata TEXT             -- 扩展元数据 (JSON)
);

CREATE INDEX idx_dynamic_routes_path ON dynamic_routes(path);
CREATE INDEX idx_dynamic_routes_status ON dynamic_routes(status);
```

**配置字段示例：**

```json
// 重定向类型
{
  "redirect_url": "https://example.com",
  "redirect_type": "301",  // 301/302
  "preserve_query": true
}

// 静态内容类型
{
  "content": "Hello World",
  "content_type": "text/plain",
  "headers": {
    "Cache-Control": "public, max-age=3600"
  }
}

// 模板渲染类型
{
  "template_name": "custom_page.html",
  "context": {
    "title": "自定义页面",
    "content": "页面内容"
  }
}
```

---

## 三、分阶段开发计划

### 阶段 1：基础集成（第 1-2 周）

**目标**：将 dynamic-router 库集成到项目，建立基础架构

#### 任务清单

1. **依赖配置**
   - [ ] 在 `Cargo.toml` 中添加 `dynamic-route-actix` 为本地依赖
   - [ ] 解决版本冲突（actix-web 版本统一）
   - [ ] 配置必要的 features（database, sqlite）

2. **数据库集成**
   - [ ] 创建 `dynamic_routes` 表迁移脚本
   - [ ] 实现 `RouteStorage` trait 的 SQLite 实现
   - [ ] 创建 `DynamicRouteRepository`

3. **核心集成**
   - [ ] 在 `AppState` 中添加 `RouteTable` 实例
   - [ ] 实现启动时从数据库加载路由
   - [ ] 实现路由变更时的持久化

4. **路由分发**
   - [ ] 创建动态路由分发器 handler
   - [ ] 配置路由优先级（静态 > 动态）
   - [ ] 实现动态路由的中间件

**验收标准：**
- ✅ 编译通过，无版本冲突
- ✅ 启动时能从数据库加载路由
- ✅ 动态路由能正常响应请求

---

### 阶段 2：管理界面（第 3-4 周）

**目标**：提供用户友好的管理界面

#### 任务清单

1. **后端 API**
   - [ ] 实现路由 CRUD API
     - `GET /api/admin/routes` - 列表路由
     - `POST /api/admin/routes` - 创建路由
     - `PUT /api/admin/routes/:id` - 更新路由
     - `DELETE /api/admin/routes/:id` - 删除路由
     - `PATCH /api/admin/routes/:id/status` - 启用/禁用
   - [ ] 实现路由验证逻辑
   - [ ] 实现路由测试功能（预览）

2. **前端界面**
   - [ ] 创建路由管理页面模板
   - [ ] 实现路由列表展示（表格）
   - [ ] 实现路由添加/编辑表单
   - [ ] 实现路由类型选择器
   - [ ] 实现路由测试功能

3. **用户体验优化**
   - [ ] 实现路由冲突检测
   - [ ] 添加使用示例和帮助文档
   - [ ] 实现批量操作（启用/禁用/删除）
   - [ ] 添加操作日志记录

**验收标准：**
- ✅ 用户可以通过界面添加、编辑、删除路由
- ✅ 路由实时生效
- ✅ 提供清晰的使用说明

---

### 阶段 3：高级功能（第 5-6 周）

**目标**：实现高级路由功能

#### 任务清单

1. **路由类型扩展**
   - [ ] 实现重定向路由（301/302）
   - [ ] 实现静态内容路由
   - [ ] 实现模板渲染路由
   - [ ] 实现自定义处理器路由（脚本化）

2. **高级特性**
   - [ ] 实现路由优先级
   - [ ] 实现路由条件匹配（HTTP 方法、Header 等）
   - [ ] 实现路由统计（访问次数、响应时间）
   - [ ] 实现路由导入/导出（JSON/YAML）

3. **性能优化**
   - [ ] 实现路由缓存
   - [ ] 优化路由匹配性能
   - [ ] 实现路由表热更新

**验收标准：**
- ✅ 支持多种路由类型
- ✅ 提供路由统计信息
- ✅ 性能满足生产环境要求

---

### 阶段 4：测试与优化（第 7 周）

**目标**：全面测试和性能优化

#### 任务清单

1. **单元测试**
   - [ ] 测试路由增删改查
   - [ ] 测试路由匹配逻辑
   - [ ] 测试持久化功能
   - [ ] 测试并发安全

2. **集成测试**
   - [ ] 测试与现有路由系统的兼容性
   - [ ] 测试管理界面功能
   - [ ] 测试路由优先级
   - [ ] 测试边界情况

3. **性能测试**
   - [ ] 基准测试（路由查找性能）
   - [ ] 负载测试（高并发场景）
   - [ ] 内存占用测试
   - [ ] 优化热点路径

4. **文档完善**
   - [ ] 编写用户文档
   - [ ] 编写开发者文档
   - [ ] 编写 API 文档
   - [ ] 添加示例代码

**验收标准：**
- ✅ 测试覆盖率 > 80%
- ✅ 性能满足预期（< 1ms 路由查找）
- ✅ 文档完整清晰

---

## 四、技术难点与解决方案

### 4.1 依赖版本冲突

**问题**：dynamic-router 和 rustblog 使用不同版本的 actix-web

**解决方案**：
1. 统一 actix-web 版本到 4.12.1
2. 检查 transitive dependencies
3. 使用 `[patch]` 或 `[replace]` 解决冲突
4. 考虑将 dynamic-router 改为 workspace member

**预期工作量**：1-2 天

### 4.2 线程安全与并发

**问题**：RouteTable 在多 worker 环境下的线程安全

**解决方案**：
1. 使用 `Arc<RouteTable>` 在 worker 间共享
2. dynamic-router 内部已使用 `RwLock`，无需额外处理
3. 写操作（增删路由）需要加锁
4. 读操作（路由匹配）支持并发

**预期工作量**：已由库解决，需验证

### 4.3 路由优先级处理

**问题**：静态路由和动态路由的优先级管理

**解决方案**：
1. 静态路由优先，动态路由作为 fallback
2. 在中间件中实现优先级逻辑
3. 提供 `priority` 字段控制动态路由间的优先级
4. 支持路由覆盖（相同路径，高优先级覆盖低优先级）

**预期工作量**：2-3 天

### 4.4 持久化一致性

**问题**：RouteTable 和数据库的同步问题

**解决方案**：
1. 实现事务性更新（先数据库，后内存）
2. 使用 `write-ahead log` 确保一致性
3. 实现定期全量同步（每 5 分钟）
4. 提供手动同步接口

**预期工作量**：3-4 天

### 4.5 性能优化

**问题**：动态路由查找可能成为性能瓶颈

**解决方案**：
1. 使用 `CacheOptimizedRouteTable`（LRU 缓存）
2. 预编译正则表达式
3. 使用 `Trie` 树优化路径匹配
4. 实现路由表分片（按路径前缀）

**预期工作量**：2-3 天

### 4.6 路由冲突检测

**问题**：用户添加的路由可能与现有路由冲突

**解决方案**：
1. 实现静态路由和动态路由的冲突检测
2. 提供冲突警告和建议
3. 支持路由预览（不保存，测试匹配）
4. 使用正则表达式匹配检测

**预期工作量**：2 天

---

## 五、风险评估与应对

### 5.1 技术风险

| 风险 | 影响 | 概率 | 应对措施 |
|------|------|------|----------|
| 依赖版本冲突 | 高 | 中 | 提前检查，使用 workspace |
| 性能不达标 | 高 | 低 | 使用缓存优化，性能测试 |
| 线程安全问题 | 高 | 低 | 代码审查，并发测试 |
| 数据库迁移失败 | 中 | 中 | 备份数据，提供回滚脚本 |

### 5.2 业务风险

| 风险 | 影响 | 概率 | 应对措施 |
|------|------|------|----------|
| 用户不理解概念 | 中 | 高 | 提供示例和文档 |
| 滥用动态路由 | 中 | 中 | 限制路由数量，提供监控 |
| 安全问题（如开放重定向） | 高 | 低 | 输入验证，安全审计 |

---

## 六、适配计划

### 6.1 代码适配清单

#### dynamic-router 库适配

1. **版本统一**
   ```toml
   [dependencies]
   actix-web = "4.12.1"  # 与 rustblog 统一
   ```

2. **Feature 调整**
   ```toml
   [dependencies.dynamic-route-actix]
   path = "../dynamic-router"
   features = ["sqlite", "database"]
   ```

#### rustblog 项目适配

1. **新增模块**
   ```
   src/
   ├── routes/
   │   └── dynamic_routes.rs  # 动态路由配置
   ├── handlers/
   │   └── dynamic_route_handlers.rs  # 动态路由处理器
   ├── db/
   │   └── repositories/
   │       └── dynamic_route_repository.rs
   └── models/
       └── dynamic_route.rs
   ```

2. **修改文件**
   - `src/routes/mod.rs` - 添加动态路由配置
   - `src/main.rs` - 初始化 RouteTable
   - `src/app_state.rs` - 添加 RouteTable 字段

3. **新增模板**
   ```
   templates/admin/
   ├── dynamic_routes.html  # 路由管理页面
   └── dynamic_route_edit.html  # 路由编辑页面
   ```

### 6.2 数据库迁移脚本

```sql
-- 创建动态路由表
CREATE TABLE IF NOT EXISTS dynamic_routes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL UNIQUE,
    route_type TEXT NOT NULL CHECK(route_type IN ('redirect', 'static', 'template', 'custom')),
    config TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active' CHECK(status IN ('active', 'disabled', 'archived')),
    priority INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_by TEXT,
    metadata TEXT
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_dynamic_routes_path ON dynamic_routes(path);
CREATE INDEX IF NOT EXISTS idx_dynamic_routes_status ON dynamic_routes(status);
CREATE INDEX IF NOT EXISTS idx_dynamic_routes_priority ON dynamic_routes(priority DESC);

-- 创建触发器（自动更新 updated_at）
CREATE TRIGGER IF NOT EXISTS update_dynamic_routes_timestamp
AFTER UPDATE ON dynamic_routes
FOR EACH ROW
BEGIN
    UPDATE dynamic_routes SET updated_at = datetime('now') WHERE id = NEW.id;
END;

-- 创建操作日志表
CREATE TABLE IF NOT EXISTS dynamic_route_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    route_id INTEGER,
    action TEXT NOT NULL,
    old_config TEXT,
    new_config TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_by TEXT,
    FOREIGN KEY (route_id) REFERENCES dynamic_routes(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_dynamic_route_logs_route_id ON dynamic_route_logs(route_id);
```

---

## 七、时间估算

| 阶段 | 任务 | 预计时间 | 依赖 |
|------|------|----------|------|
| 阶段 1 | 基础集成 | 2 周 | 无 |
| 阶段 2 | 管理界面 | 2 周 | 阶段 1 |
| 阶段 3 | 高级功能 | 2 周 | 阶段 2 |
| 阶段 4 | 测试优化 | 1 周 | 阶段 3 |
| **总计** | | **7 周** | |

---

## 八、验收标准

### 8.1 功能验收

- [ ] 用户可以通过界面添加路由
- [ ] 支持至少 3 种路由类型（重定向、静态、模板）
- [ ] 路由实时生效，无需重启
- [ ] 路由配置持久化到数据库
- [ ] 提供路由冲突检测
- [ ] 提供路由统计信息

### 8.2 性能验收

- [ ] 路由查找延迟 < 1ms (P99)
- [ ] 支持 1000+ 动态路由
- [ ] 并发访问不降低性能 > 10%
- [ ] 内存占用 < 100MB (1000 路由)

### 8.3 安全验收

- [ ] 输入验证（防止注入）
- [ ] 权限控制（仅管理员可操作）
- [ ] 防止开放重定向
- [ ] 操作日志记录

### 8.4 兼容性验收

- [ ] 不影响现有静态路由
- [ ] 向后兼容（旧版本数据库可迁移）
- [ ] 不增加启动时间 > 1s

---

## 九、后续优化方向

1. **性能优化**
   - 使用 SIMD 优化字符串匹配
   - 实现路由表分片
   - 优化缓存策略

2. **功能扩展**
   - 支持通配符路由
   - 支持路由组管理
   - 支持路由版本控制
   - 支持路由 A/B 测试

3. **开发体验**
   - 提供路由 DSLe
   - 提供可视化路由编辑器
   - 提供路由测试工具

4. **运维支持**
   - 路由监控告警
   - 路由备份恢复
   - 路由导入导出

---

## 十、参考资料

- [dynamic-router 文档](../dynamic-router/docs/)
- [Actix-Web 官方文档](https://actix.rs/)
- [RustBlog 架构文档](./ARCHITECTURE_PAGINATION.md)
- [RustBlog 缓存文档](./CACHE_CONCURRENCY_GUIDE.md)