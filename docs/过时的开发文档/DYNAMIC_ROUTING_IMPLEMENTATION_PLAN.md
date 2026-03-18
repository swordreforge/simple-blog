# 动态路由管理功能实施计划书

## 一、功能概述

### 1.1 功能描述

为RustBlog添加动态路由管理功能，允许管理员通过管理界面动态创建、编辑、删除路由，支持多种路由类型（内存路由、文件路由、数据库路由），无需重启服务即可生效。

### 1.2 核心特性

- ✅ 管理员鉴权保护
- ✅ 三种路由类型支持
  - **内存路由**：存储在内存中，重启后失效（适合临时路由）
  - **文件路由**：持久化到JSON/YAML文件（适合配置管理）
  - **数据库路由**：持久化到SQLite数据库（适合生产环境）
- ✅ RESTful CRUD API
- ✅ 路由测试和预览功能
- ✅ 路由冲突检测
- ✅ 操作日志记录

### 1.3 技术栈

- **后端框架**：Actix-Web 4.12.1
- **数据库**：SQLite + r2d2连接池
- **认证**：JWT Token + Cookie
- **前端**：HTML + JavaScript（原生）
- **序列化**：serde_json

---

## 二、架构设计

### 2.1 系统架构图

```
┌─────────────────────────────────────────────────────────────────┐
│                         管理员浏览器                              │
│                    /admin/dyn-routing                            │
└────────────────────────┬────────────────────────────────────────┘
                         │ HTTP请求
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Actix-Web 应用层                              │
│  ┌────────────────────────────────────────────────────────┐    │
│  │  认证中间件 (JWT验证)                                   │    │
│  │  - 检查Cookie中的auth_token                            │    │
│  │  - 验证Token有效性                                     │    │
│  │  - 提取用户信息和角色                                   │    │
│  └────────────────────────────────────────────────────────┘    │
│                          │                                       │
│  ┌────────────────────────────────────────────────────────┐    │
│  │  权限检查                                              │    │
│  │  - 验证角色为admin                                     │    │
│  │  - 非管理员返回403                                     │    │
│  └────────────────────────────────────────────────────────┘    │
│                          │                                       │
│  ┌──────────────────────┐  ┌──────────────────────┐           │
│  │  页面路由             │  │  API路由              │           │
│  │  GET /admin/         │  │  /api/admin/          │           │
│  │  dyn-routing         │  │  dynamic-routes/*     │           │
│  └──────────┬───────────┘  └──────────┬───────────┘           │
│             │                         │                        │
│  ┌──────────▼─────────────────────────▼─────────────────────┐ │
│  │            路由处理器层 (Handlers)                         │ │
│  │  - render_dyn_routing_page()      // 页面渲染             │ │
│  │  - list_routes()                  // 列表查询             │ │
│  │  - create_route()                 // 创建路由             │ │
│  │  - update_route()                 // 更新路由             │ │
│  │  - delete_route()                 // 删除路由             │ │
│  │  - test_route()                   // 路由测试             │ │
│  └──────────────────────────────────────────────────────────┘ │
└─────────────────────────────┬─────────────────────────────────┘
                              │
┌─────────────────────────────▼─────────────────────────────────┐
│                      业务逻辑层 (Services)                      │
│  ┌────────────────────────────────────────────────────────┐   │
│  │  DynamicRouteService                                   │   │
│  │  - validate_route_config()     // 配置验证              │   │
│  │  - check_route_conflicts()     // 冲突检测              │   │
│  │  - get_route_statistics()      // 统计信息              │   │
│  └────────────────────────────────────────────────────────┘   │
└─────────────────────────────┬─────────────────────────────────┘
                              │
┌─────────────────────────────▼─────────────────────────────────┐
│                      数据访问层 (Repositories)                  │
│  ┌────────────────────────────────────────────────────────┐   │
│  │  DynamicRouteRepository                                │   │
│  │  - create()      // 创建路由                           │   │
│  │  - get_by_id()   // 根据ID查询                          │   │
│  │  - get_by_path() // 根据路径查询                        │   │
│  │  - list()        // 列表查询                            │   │
│  │  - update()      // 更新路由                            │   │
│  │  - delete()      // 删除路由                            │   │
│  └────────────────────────────────────────────────────────┘   │
└─────────────────────────────┬─────────────────────────────────┘
                              │
          ┌───────────────────┼───────────────────┐
          │                   │                   │
┌─────────▼──────┐  ┌────────▼────────┐  ┌───────▼────────┐
│   SQLite数据库  │  │   JSON文件       │  │   内存存储      │
│  (持久化存储)   │  │  (文件存储)      │  │  (临时存储)    │
│  - routes表    │  │  - routes.json   │  │  - HashMap     │
│  - logs表      │  │  - routes.yaml   │  │  - Arc<RwLock> │
└────────────────┘  └─────────────────┘  └────────────────┘
```

### 2.2 路由类型设计

#### 2.2.1 内存路由 (Memory Route)

**特点：**
- 存储在应用程序内存中
- 重启后自动丢失
- 适合临时重定向、测试场景
- 性能最优（无IO开销）

**配置示例：**
```json
{
  "route_type": "memory",
  "path": "/temporary-redirect",
  "handler": {
    "type": "redirect",
    "target": "/new-location",
    "status_code": 302
  },
  "enabled": true,
  "created_at": "2026-03-17T10:00:00Z"
}
```

#### 2.2.2 文件路由 (File Route)

**特点：**
- 持久化到JSON/YAML文件
- 支持版本控制（Git）
- 适合配置管理、批量导入导出
- 支持热重载

**配置示例：**
```json
{
  "route_type": "file",
  "path": "/api/v1/users",
  "handler": {
    "type": "proxy",
    "target": "http://backend-service:8080/users",
    "timeout": 5000
  },
  "enabled": true,
  "metadata": {
    "description": "代理到后端服务",
    "tags": ["proxy", "api"]
  }
}
```

**文件存储格式：**
```json
// data/routes.json
{
  "version": "1.0",
  "routes": [
    {
      "id": "file_001",
      "path": "/old-url",
      "handler": { "type": "redirect", "target": "/new-url", "status_code": 301 },
      "enabled": true
    }
  ]
}
```

#### 2.2.3 数据库路由 (Database Route)

**特点：**
- 持久化到SQLite数据库
- 支持事务操作
- 支持复杂查询和统计
- 适合生产环境

**数据库表结构：**
```sql
CREATE TABLE dynamic_routes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    route_type TEXT NOT NULL,  -- 'memory', 'file', 'database'
    path TEXT NOT NULL UNIQUE,
    handler_type TEXT NOT NULL,  -- 'redirect', 'static', 'template', 'proxy', 'custom'
    handler_config TEXT NOT NULL,  -- JSON配置
    enabled BOOLEAN DEFAULT 1,
    priority INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_by TEXT,
    metadata TEXT  -- JSON扩展字段
);

CREATE INDEX idx_dynamic_routes_path ON dynamic_routes(path);
CREATE INDEX idx_dynamic_routes_type ON dynamic_routes(route_type);
CREATE INDEX idx_dynamic_routes_enabled ON dynamic_routes(enabled);
```

### 2.3 处理器类型 (Handler Types)

#### 2.3.1 重定向处理器 (Redirect Handler)

```json
{
  "type": "redirect",
  "target": "https://example.com/new-page",
  "status_code": 301,
  "preserve_query": true
}
```

#### 2.3.2 静态内容处理器 (Static Handler)

```json
{
  "type": "static",
  "content": "Hello, World!",
  "content_type": "text/plain; charset=utf-8",
  "headers": {
    "Cache-Control": "public, max-age=3600",
    "X-Custom-Header": "value"
  }
}
```

#### 2.3.3 模板渲染处理器 (Template Handler)

```json
{
  "type": "template",
  "template_name": "custom_page.html",
  "context": {
    "title": "自定义页面",
    "content": "页面内容"
  }
}
```

#### 2.3.4 代理处理器 (Proxy Handler)

```json
{
  "type": "proxy",
  "target": "http://backend:8080/api",
  "timeout": 5000,
  "strip_prefix": false
}
```

#### 2.3.5 自定义处理器 (Custom Handler)

```json
{
  "type": "custom",
  "script": "lua",
  "source": "function handle(req) return {status=200, body='OK'} end"
}
```

---

## 三、数据库设计

### 3.1 dynamic_routes 表

```sql
CREATE TABLE IF NOT EXISTS dynamic_routes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    route_type TEXT NOT NULL CHECK(route_type IN ('memory', 'file', 'database')),
    path TEXT NOT NULL UNIQUE,
    handler_type TEXT NOT NULL CHECK(handler_type IN ('redirect', 'static', 'template', 'proxy', 'custom')),
    handler_config TEXT NOT NULL,  -- JSON配置
    enabled BOOLEAN DEFAULT 1,
    priority INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_by TEXT,
    metadata TEXT  -- JSON扩展字段
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_dynamic_routes_path ON dynamic_routes(path);
CREATE INDEX IF NOT EXISTS idx_dynamic_routes_type ON dynamic_routes(route_type);
CREATE INDEX IF NOT EXISTS idx_dynamic_routes_enabled ON dynamic_routes(enabled);
CREATE INDEX IF NOT EXISTS idx_dynamic_routes_priority ON dynamic_routes(priority DESC);

-- 触发器：自动更新时间戳
CREATE TRIGGER IF NOT EXISTS update_dynamic_routes_timestamp
AFTER UPDATE ON dynamic_routes
FOR EACH ROW
BEGIN
    UPDATE dynamic_routes SET updated_at = datetime('now') WHERE id = NEW.id;
END;
```

### 3.2 dynamic_route_logs 表（操作日志）

```sql
CREATE TABLE IF NOT EXISTS dynamic_route_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    route_id INTEGER,
    action TEXT NOT NULL,  -- 'create', 'update', 'delete', 'enable', 'disable'
    old_config TEXT,
    new_config TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_by TEXT,
    ip_address TEXT,
    user_agent TEXT,
    FOREIGN KEY (route_id) REFERENCES dynamic_routes(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_dynamic_route_logs_route_id ON dynamic_route_logs(route_id);
CREATE INDEX IF NOT EXISTS idx_dynamic_route_logs_action ON dynamic_route_logs(action);
CREATE INDEX IF NOT EXISTS idx_dynamic_route_logs_created_at ON dynamic_route_logs(created_at DESC);
```

### 3.3 动态路由统计表

```sql
CREATE TABLE IF NOT EXISTS dynamic_route_stats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    route_id INTEGER NOT NULL,
    access_count INTEGER DEFAULT 0,
    last_accessed_at TEXT,
    total_response_time_ms INTEGER DEFAULT 0,
    avg_response_time_ms REAL DEFAULT 0,
    error_count INTEGER DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (route_id) REFERENCES dynamic_routes(id) ON DELETE CASCADE,
    UNIQUE(route_id)
);

CREATE INDEX IF NOT EXISTS idx_dynamic_route_stats_route_id ON dynamic_route_stats(route_id);
```

---

## 四、API设计

### 4.1 API路由列表

| 方法 | 路径 | 描述 | 鉴权 |
|------|------|------|------|
| GET | `/api/admin/dynamic-routes` | 获取路由列表 | ✅ Admin |
| GET | `/api/admin/dynamic-routes/:id` | 获取路由详情 | ✅ Admin |
| POST | `/api/admin/dynamic-routes` | 创建路由 | ✅ Admin |
| PUT | `/api/admin/dynamic-routes/:id` | 更新路由 | ✅ Admin |
| PATCH | `/api/admin/dynamic-routes/:id` | 部分更新路由 | ✅ Admin |
| DELETE | `/api/admin/dynamic-routes/:id` | 删除路由 | ✅ Admin |
| POST | `/api/admin/dynamic-routes/:id/enable` | 启用路由 | ✅ Admin |
| POST | `/api/admin/dynamic-routes/:id/disable` | 禁用路由 | ✅ Admin |
| POST | `/api/admin/dynamic-routes/test` | 测试路由 | ✅ Admin |
| GET | `/api/admin/dynamic-routes/:id/stats` | 获取路由统计 | ✅ Admin |
| POST | `/api/admin/dynamic-routes/batch` | 批量操作 | ✅ Admin |
| GET | `/api/admin/dynamic-routes/export` | 导出路由配置 | ✅ Admin |
| POST | `/api/admin/dynamic-routes/import` | 导入路由配置 | ✅ Admin |
| GET | `/api/admin/dynamic-routes/logs` | 获取操作日志 | ✅ Admin |

### 4.2 API详细说明

#### 4.2.1 获取路由列表

**请求：**
```
GET /api/admin/dynamic-routes?page=1&limit=20&route_type=database&enabled=true
```

**响应：**
```json
{
  "success": true,
  "data": {
    "routes": [
      {
        "id": 1,
        "route_type": "database",
        "path": "/old-url",
        "handler_type": "redirect",
        "handler_config": {
          "type": "redirect",
          "target": "/new-url",
          "status_code": 301
        },
        "enabled": true,
        "priority": 0,
        "created_at": "2026-03-17T10:00:00Z",
        "updated_at": "2026-03-17T10:00:00Z",
        "created_by": "admin",
        "stats": {
          "access_count": 1250,
          "avg_response_time_ms": 0.5
        }
      }
    ],
    "total": 45,
    "page": 1,
    "limit": 20
  }
}
```

#### 4.2.2 创建路由

**请求：**
```json
POST /api/admin/dynamic-routes
{
  "route_type": "database",
  "path": "/custom-page",
  "handler_type": "template",
  "handler_config": {
    "type": "template",
    "template_name": "custom_page.html",
    "context": {
      "title": "自定义页面",
      "content": "页面内容"
    }
  },
  "enabled": true,
  "priority": 10
}
```

**响应：**
```json
{
  "success": true,
  "message": "路由创建成功",
  "data": {
    "id": 2,
    "route_type": "database",
    "path": "/custom-page",
    ...
  }
}
```

#### 4.2.3 更新路由

**请求：**
```json
PUT /api/admin/dynamic-routes/2
{
  "route_type": "database",
  "path": "/custom-page",
  "handler_type": "template",
  "handler_config": {
    "type": "template",
    "template_name": "updated_page.html",
    "context": {
      "title": "更新的标题",
      "content": "更新的内容"
    }
  },
  "enabled": true,
  "priority": 10
}
```

#### 4.2.4 删除路由

**请求：**
```
DELETE /api/admin/dynamic-routes/2
```

**响应：**
```json
{
  "success": true,
  "message": "路由删除成功"
}
```

#### 4.2.5 测试路由

**请求：**
```json
POST /api/admin/dynamic-routes/test
{
  "route_type": "database",
  "path": "/test-path",
  "handler_type": "redirect",
  "handler_config": {
    "type": "redirect",
    "target": "/target",
    "status_code": 302
  }
}
```

**响应：**
```json
{
  "success": true,
  "message": "路由测试成功",
  "data": {
    "match": true,
    "conflict": false,
    "response_preview": {
      "status_code": 302,
      "headers": {
        "Location": "/target"
      }
    }
  }
}
```

#### 4.2.6 导出路由

**请求：**
```
GET /api/admin/dynamic-routes/export?format=json
```

**响应：**
```json
{
  "success": true,
  "data": {
    "version": "1.0",
    "exported_at": "2026-03-17T10:00:00Z",
    "routes": [
      {
        "route_type": "database",
        "path": "/route1",
        "handler_type": "redirect",
        "handler_config": {...}
      }
    ]
  }
}
```

---

## 五、前端界面设计

### 5.1 页面路由

```
GET /admin/dyn-routing
```

**功能：**
- 渲染动态路由管理页面
- 需要管理员权限
- 加载所有路由数据

### 5.2 界面布局

```
┌─────────────────────────────────────────────────────────────┐
│  动态路由管理                               [搜索框] [刷新]   │
├─────────────────────────────────────────────────────────────┤
│  [添加路由] [批量导入] [批量导出] [批量删除]                  │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────┬───────────┬────────────┬───────────┬────────────┐ │
│  │ ID  │ 路径      │ 类型       │ 处理器     │ 操作       │ │
│  ├─────┼───────────┼────────────┼───────────┼────────────┤ │
│  │ 1   │ /old-url  │ database   │ redirect  │ [编辑]     │ │
│  │     │           │            │           │ [删除]     │ │
│  ├─────┼───────────┼────────────┼───────────┼────────────┤ │
│  │ 2   │ /custom   │ memory     │ template  │ [编辑]     │ │
│  │     │           │            │           │ [删除]     │ │
│  ├─────┼───────────┼────────────┼───────────┼────────────┤ │
│  │ 3   │ /api/v1   │ file       │ proxy     │ [编辑]     │ │
│  │     │           │            │           │ [删除]     │ │
│  └─────┴───────────┴────────────┴───────────┴────────────┘ │
│                                                             │
│  共 45 条路由    [上一页] 1 / 3 [下一页]                    │
└─────────────────────────────────────────────────────────────┘
```

### 5.3 添加/编辑路由对话框

```
┌─────────────────────────────────────────────────────────────┐
│  添加路由                                            [X]     │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  路由类型: [▼database ▼]                                    │
│  (可选：memory, file, database)                             │
│                                                             │
│  路径: [/custom-path                    ]                  │
│  [✔] 检查冲突                                               │
│                                                             │
│  处理器类型: [▼redirect ▼]                                  │
│  (可选：redirect, static, template, proxy, custom)          │
│                                                             │
│  处理器配置 (JSON):                                          │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ {                                                     │  │
│  │   "type": "redirect",                                 │  │
│  │   "target": "/new-location",                          │  │
│  │   "status_code": 301                                  │  │
│  │ }                                                     │  │
│  └──────────────────────────────────────────────────────┘  │
│  [格式化] [验证] [预设模板 ▼]                               │
│                                                             │
│  优先级: [0            ]                                    │
│                                                             │
│  [✔] 立即启用                                               │
│                                                             │
│  [取消] [测试] [保存]                                       │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 5.4 路由统计面板

```
┌─────────────────────────────────────────────────────────────┐
│  路由统计                                                    │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  总路由数:     45                                            │
│  启用中:       42 (93.3%)                                    │
│  禁用:         3 (6.7%)                                      │
│                                                             │
│  按类型分布:                                                 │
│  ├─ Database:  30 (66.7%)  ■■■■■■                          │
│  ├─ Memory:    10 (22.2%)  ■■■                             │
│  └─ File:      5 (11.1%)   ■                                │
│                                                             │
│  按处理器分布:                                               │
│  ├─ Redirect:  25 (55.6%)  ■■■■■                           │
│  ├─ Template:  10 (22.2%)  ■■■                             │
│  ├─ Proxy:     5 (11.1%)   ■                                │
│  ├─ Static:    3 (6.7%)    ■                               │
│  └─ Custom:    2 (4.4%)    ■                               │
│                                                             │
│  最近7天访问:  12,345                                        │
│  平均响应时间: 0.8ms                                         │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## 六、核心代码结构

### 6.1 后端代码结构

```
src/
├── routes/
│   ├── mod.rs                              # 路由配置入口
│   ├── api_routes.rs                       # 添加动态路由API
│   └── page_routes.rs                      # 添加页面路由
├── handlers/
│   ├── page_handlers.rs                    # 添加页面处理器
│   └── api_handlers/
│       ├── mod.rs                          # 导出动态路由处理器
│       └── dynamic_routes/
│           ├── mod.rs                      # 模块入口
│           ├── list.rs                     # 列表处理器
│           ├── create.rs                   # 创建处理器
│           ├── update.rs                   # 更新处理器
│           ├── delete.rs                   # 删除处理器
│           ├── test.rs                     # 测试处理器
│           ├── stats.rs                    # 统计处理器
│           └── batch.rs                    # 批量操作处理器
├── db/
│   ├── repositories.rs                     # 添加DynamicRouteRepository
│   └── models.rs                           # 添加DynamicRoute模型
├── services/
│   └── dynamic_route_service.rs            # 业务逻辑层
├── middleware/
│   └── auth.rs                             # 已有认证中间件
└── app_state.rs                            # 添加路由表字段
```

### 6.2 前端代码结构

```
templates/
└── admin/
    ├── dyn-routing.html                    # 主页面
    └── dyn-routing-modal.html              # 模态框（可选）

templates/js/
└── dyn-routing.js                          # 页面逻辑
```

---

## 七、实施步骤

### 阶段1：数据库和模型（第1天）

**任务清单：**
- [ ] 创建数据库迁移脚本
- [ ] 实现DynamicRoute模型
- [ ] 实现DynamicRouteRepository
- [ ] 实现操作日志Repository
- [ ] 实现统计Repository

**代码示例：**

```rust
// src/db/models.rs
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DynamicRoute {
    pub id: Option<i64>,
    pub route_type: String,  // 'memory', 'file', 'database'
    pub path: String,
    pub handler_type: String,
    pub handler_config: serde_json::Value,
    pub enabled: bool,
    pub priority: i32,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub created_by: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

// src/db/repositories.rs
impl PassageRepository {
    pub fn dynamic_route_repository(&self) -> DynamicRouteRepository {
        DynamicRouteRepository::new(self.pool.clone())
    }
}

#[derive(Clone)]
pub struct DynamicRouteRepository {
    pool: Arc<Pool<SqliteConnectionManager>>,
}

impl DynamicRouteRepository {
    pub fn new(pool: Arc<Pool<SqliteConnectionManager>>) -> Self {
        Self { pool }
    }
    
    pub async fn create(&self, route: &DynamicRoute) -> Result<i64> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO dynamic_routes (route_type, path, handler_type, handler_config, enabled, priority, created_by, metadata)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                &route.route_type,
                &route.path,
                &route.handler_type,
                &route.handler_config.to_string(),
                &route.enabled,
                &route.priority,
                &route.created_by,
                &route.metadata.map(|v| v.to_string()),
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }
    
    pub async fn get_by_id(&self, id: i64) -> Result<DynamicRoute> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, route_type, path, handler_type, handler_config, enabled, priority, created_at, updated_at, created_by, metadata
             FROM dynamic_routes WHERE id = ?"
        )?;
        
        stmt.query_row(params![id], |row| {
            Ok(DynamicRoute {
                id: Some(row.get(0)?),
                route_type: row.get(1)?,
                path: row.get(2)?,
                handler_type: row.get(3)?,
                handler_config: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
                enabled: row.get(5)?,
                priority: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                created_by: row.get(9)?,
                metadata: row.get::<_, Option<String>>(10)?.and_then(|s| serde_json::from_str(&s).ok()),
            })
        }).map_err(|e| e.into())
    }
    
    pub async fn list(&self, offset: i64, limit: i64) -> Result<(Vec<DynamicRoute>, i64)> {
        let conn = self.pool.get()?;
        
        // 获取总数
        let total: i64 = conn.query_row("SELECT COUNT(*) FROM dynamic_routes", [], |row| row.get(0))?;
        
        // 获取列表
        let mut stmt = conn.prepare(
            "SELECT id, route_type, path, handler_type, handler_config, enabled, priority, created_at, updated_at, created_by, metadata
             FROM dynamic_routes ORDER BY priority DESC, id ASC LIMIT ? OFFSET ?"
        )?;
        
        let routes = stmt.query_map(params![limit, offset], |row| {
            Ok(DynamicRoute {
                id: Some(row.get(0)?),
                route_type: row.get(1)?,
                path: row.get(2)?,
                handler_type: row.get(3)?,
                handler_config: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
                enabled: row.get(5)?,
                priority: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                created_by: row.get(9)?,
                metadata: row.get::<_, Option<String>>(10)?.and_then(|s| serde_json::from_str(&s).ok()),
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        
        Ok((routes, total))
    }
    
    pub async fn update(&self, id: i64, route: &DynamicRoute) -> Result<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE dynamic_routes SET route_type=?, path=?, handler_type=?, handler_config=?, enabled=?, priority=?, metadata=?
             WHERE id=?",
            params![
                &route.route_type,
                &route.path,
                &route.handler_type,
                &route.handler_config.to_string(),
                &route.enabled,
                &route.priority,
                &route.metadata.map(|v| v.to_string()),
                id,
            ],
        )?;
        Ok(())
    }
    
    pub async fn delete(&self, id: i64) -> Result<()> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM dynamic_routes WHERE id=?", params![id])?;
        Ok(())
    }
    
    pub async fn get_by_path(&self, path: &str) -> Result<Option<DynamicRoute>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, route_type, path, handler_type, handler_config, enabled, priority, created_at, updated_at, created_by, metadata
             FROM dynamic_routes WHERE path = ?"
        )?;
        
        let result = stmt.query_row(params![path], |row| {
            Ok(DynamicRoute {
                id: Some(row.get(0)?),
                route_type: row.get(1)?,
                path: row.get(2)?,
                handler_type: row.get(3)?,
                handler_config: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
                enabled: row.get(5)?,
                priority: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                created_by: row.get(9)?,
                metadata: row.get::<_, Option<String>>(10)?.and_then(|s| serde_json::from_str(&s).ok()),
            })
        });
        
        match result {
            Ok(route) => Ok(Some(route)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
```

### 阶段2：API处理器（第2-3天）

**任务清单：**
- [ ] 实现列表查询API
- [ ] 实现创建API
- [ ] 实现更新API
- [ ] 实现删除API
- [ ] 实现测试API
- [ ] 实现统计API
- [ ] 实现批量操作API
- [ ] 实现导入导出API

**代码示例：**

```rust
// src/handlers/api_handlers/dynamic_routes/mod.rs
pub mod list;
pub mod create;
pub mod update;
pub mod delete;
pub mod test;
pub mod stats;
pub mod batch;
pub mod export;

pub use list::*;
pub use create::*;
pub use update::*;
pub use delete::*;
pub use test::*;
pub use stats::*;
pub use batch::*;
pub use export::*;

// src/handlers/api_handlers/dynamic_routes/list.rs
use actix_web::{web, HttpResponse};
use crate::app_state::AppState;
use crate::middleware::auth::check_admin_auth;

pub async fn list_routes(
    req: actix_web::HttpRequest,
    query: web::Query<ListQuery>,
    state: web::Data<AppState>,
) -> HttpResponse {
    // 权限检查
    if check_admin_auth(&req).is_none() {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "success": false,
            "message": "需要管理员权限"
        }));
    }
    
    // 查询参数
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);
    let offset = (page - 1) * limit;
    
    // 从Repository获取数据
    let repo = state.repository().dynamic_route_repository();
    match repo.list(offset, limit).await {
        Ok((routes, total)) => {
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "data": {
                    "routes": routes,
                    "total": total,
                    "page": page,
                    "limit": limit
                }
            }))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": format!("查询失败: {}", e)
            }))
        }
    }
}

#[derive(serde::Deserialize)]
pub struct ListQuery {
    page: Option<i64>,
    limit: Option<i64>,
    route_type: Option<String>,
    enabled: Option<bool>,
}

// src/handlers/api_handlers/dynamic_routes/create.rs
pub async fn create_route(
    req: actix_web::HttpRequest,
    route: web::Json<CreateRouteRequest>,
    state: web::Data<AppState>,
) -> HttpResponse {
    // 权限检查
    let admin_info = match check_admin_auth(&req) {
        Some(info) => info,
        None => return HttpResponse::Forbidden().json(serde_json::json!({
            "success": false,
            "message": "需要管理员权限"
        })),
    };
    
    // 验证路由配置
    if let Err(e) = validate_route_config(&route) {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "message": format!("配置验证失败: {}", e)
        }));
    }
    
    // 检查路径冲突
    let repo = state.repository().dynamic_route_repository();
    if let Ok(Some(_)) = repo.get_by_path(&route.path).await {
        return HttpResponse::Conflict().json(serde_json::json!({
            "success": false,
            "message": "路径已存在"
        }));
    }
    
    // 创建路由
    let dynamic_route = crate::db::models::DynamicRoute {
        id: None,
        route_type: route.route_type.clone(),
        path: route.path.clone(),
        handler_type: route.handler_type.clone(),
        handler_config: route.handler_config.clone(),
        enabled: route.enabled.unwrap_or(true),
        priority: route.priority.unwrap_or(0),
        created_at: None,
        updated_at: None,
        created_by: Some(admin_info.1),
        metadata: route.metadata.clone(),
    };
    
    match repo.create(&dynamic_route).await {
        Ok(id) => {
            // 记录操作日志
            log_route_operation(&repo, id, "create", None, &dynamic_route, &admin_info.1);
            
            HttpResponse::Created().json(serde_json::json!({
                "success": true,
                "message": "路由创建成功",
                "data": {
                    "id": id,
                    "path": dynamic_route.path
                }
            }))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": format!("创建失败: {}", e)
            }))
        }
    }
}

#[derive(serde::Deserialize)]
pub struct CreateRouteRequest {
    route_type: String,
    path: String,
    handler_type: String,
    handler_config: serde_json::Value,
    enabled: Option<bool>,
    priority: Option<i32>,
    metadata: Option<serde_json::Value>,
}

fn validate_route_config(route: &CreateRouteRequest) -> Result<(), String> {
    // 验证路由类型
    if !["memory", "file", "database"].contains(&route.route_type.as_str()) {
        return Err("无效的路由类型".to_string());
    }
    
    // 验证处理器类型
    if !["redirect", "static", "template", "proxy", "custom"].contains(&route.handler_type.as_str()) {
        return Err("无效的处理器类型".to_string());
    }
    
    // 验证路径格式
    if !route.path.starts_with('/') {
        return Err("路径必须以 / 开头".to_string());
    }
    
    // 验证处理器配置
    if route.handler_config.is_null() || !route.handler_config.is_object() {
        return Err("处理器配置必须是有效的JSON对象".to_string());
    }
    
    Ok(())
}

fn log_route_operation(
    repo: &crate::db::repositories::DynamicRouteRepository,
    route_id: i64,
    action: &str,
    old_config: Option<&crate::db::models::DynamicRoute>,
    new_config: &crate::db::models::DynamicRoute,
    username: &str,
) {
    // TODO: 实现操作日志记录
}
```

### 阶段3：页面路由和处理器（第4天）

**任务清单：**
- [ ] 添加页面路由配置
- [ ] 实现页面渲染处理器
- [ ] 创建HTML模板
- [ ] 实现JavaScript逻辑

**代码示例：**

```rust
// src/routes/page_routes.rs
pub fn configure_page_routes(cfg: &mut web::ServiceConfig) {
    // 现有路由...
    
    // 添加动态路由管理页面
    cfg.service(
        web::resource("/admin/dyn-routing")
            .route(web::get().to(page_handlers::render_dyn_routing_page))
    );
}

// src/handlers/page_handlers.rs
use crate::middleware::auth::check_admin_auth;

pub async fn render_dyn_routing_page(
    req: actix_web::HttpRequest,
    state: web::Data<AppState>,
) -> HttpResponse {
    // 权限检查
    if check_admin_auth(&req).is_none() {
        return HttpResponse::Forbidden().body("需要管理员权限");
    }
    
    // 获取路由统计
    let repo = state.repository().dynamic_route_repository();
    let (routes, total) = match repo.list(0, 1000).await {
        Ok(data) => data,
        Err(_) => return HttpResponse::InternalServerError().body("加载失败"),
    };
    
    // 计算统计信息
    let enabled_count = routes.iter().filter(|r| r.enabled).count();
    let disabled_count = routes.len() - enabled_count;
    
    let mut context = tera::Context::new();
    context.insert("title", "动态路由管理");
    context.insert("total_routes", &total);
    context.insert("enabled_count", &enabled_count);
    context.insert("disabled_count", &disabled_count);
    context.insert("routes", &routes);
    
    // 渲染模板
    match crate::templates::TERA.render("admin/dyn-routing.html", &context) {
        Ok(html) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html),
        Err(e) => {
            eprintln!("模板渲染错误: {}", e);
            HttpResponse::InternalServerError().body("模板渲染失败")
        }
    }
}
```

### 阶段4：前端界面（第5-6天）

**任务清单：**
- [ ] 创建HTML模板
- [ ] 实现JavaScript逻辑
- [ ] 实现路由列表展示
- [ ] 实现添加/编辑对话框
- [ ] 实现路由测试功能
- [ ] 实现批量操作

**代码示例：**

```html
<!-- templates/admin/dyn-routing.html -->
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ title }} - 动态路由管理</title>
    <link rel="stylesheet" href="/css/admin.css">
    <style>
        .container {
            max-width: 1400px;
            margin: 0 auto;
            padding: 20px;
        }
        
        .header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 20px;
        }
        
        .stats-panel {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 15px;
            margin-bottom: 20px;
        }
        
        .stat-card {
            background: rgba(255, 255, 255, 0.1);
            padding: 15px;
            border-radius: 8px;
        }
        
        .stat-value {
            font-size: 24px;
            font-weight: bold;
        }
        
        .toolbar {
            display: flex;
            gap: 10px;
            margin-bottom: 20px;
        }
        
        .table-container {
            overflow-x: auto;
        }
        
        table {
            width: 100%;
            border-collapse: collapse;
        }
        
        th, td {
            padding: 12px;
            text-align: left;
            border-bottom: 1px solid rgba(255, 255, 255, 0.1);
        }
        
        .route-type {
            display: inline-block;
            padding: 2px 8px;
            border-radius: 4px;
            font-size: 12px;
        }
        
        .route-type.database { background: #4CAF50; }
        .route-type.memory { background: #2196F3; }
        .route-type.file { background: #FF9800; }
        
        .handler-type {
            display: inline-block;
            padding: 2px 8px;
            border-radius: 4px;
            font-size: 12px;
            background: rgba(255, 255, 255, 0.2);
        }
        
        .action-buttons {
            display: flex;
            gap: 5px;
        }
        
        .modal {
            display: none;
            position: fixed;
            top: 0;
            left: 0;
            width: 100%;
            height: 100%;
            background: rgba(0, 0, 0, 0.7);
            z-index: 1000;
        }
        
        .modal.active {
            display: flex;
            justify-content: center;
            align-items: center;
        }
        
        .modal-content {
            background: #1a1a1a;
            padding: 30px;
            border-radius: 8px;
            width: 90%;
            max-width: 600px;
            max-height: 90vh;
            overflow-y: auto;
        }
        
        .form-group {
            margin-bottom: 20px;
        }
        
        .form-group label {
            display: block;
            margin-bottom: 5px;
            font-weight: bold;
        }
        
        .form-group input,
        .form-group select,
        .form-group textarea {
            width: 100%;
            padding: 10px;
            border: 1px solid rgba(255, 255, 255, 0.2);
            border-radius: 4px;
            background: rgba(255, 255, 255, 0.05);
            color: white;
            font-family: inherit;
        }
        
        .form-group textarea {
            min-height: 150px;
            font-family: monospace;
        }
        
        .modal-actions {
            display: flex;
            gap: 10px;
            justify-content: flex-end;
            margin-top: 20px;
        }
        
        .btn {
            padding: 10px 20px;
            border: none;
            border-radius: 4px;
            cursor: pointer;
            font-weight: bold;
        }
        
        .btn-primary {
            background: #4CAF50;
            color: white;
        }
        
        .btn-secondary {
            background: rgba(255, 255, 255, 0.2);
            color: white;
        }
        
        .btn-danger {
            background: #f44336;
            color: white;
        }
        
        .enabled-badge {
            color: #4CAF50;
        }
        
        .disabled-badge {
            color: #f44336;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>动态路由管理</h1>
            <div>
                <button class="btn btn-secondary" onclick="loadRoutes()">刷新</button>
            </div>
        </div>
        
        <div class="stats-panel">
            <div class="stat-card">
                <div class="stat-value">{{ total_routes }}</div>
                <div>总路由数</div>
            </div>
            <div class="stat-card">
                <div class="stat-value enabled-badge">{{ enabled_count }}</div>
                <div>启用中</div>
            </div>
            <div class="stat-card">
                <div class="stat-value disabled-badge">{{ disabled_count }}</div>
                <div>已禁用</div>
            </div>
        </div>
        
        <div class="toolbar">
            <button class="btn btn-primary" onclick="showCreateModal()">添加路由</button>
            <button class="btn btn-secondary" onclick="exportRoutes()">批量导出</button>
            <button class="btn btn-secondary" onclick="document.getElementById('importInput').click()">批量导入</button>
            <input type="file" id="importInput" accept=".json" style="display: none" onchange="importRoutes(event)">
        </div>
        
        <div class="table-container">
            <table>
                <thead>
                    <tr>
                        <th>ID</th>
                        <th>路径</th>
                        <th>路由类型</th>
                        <th>处理器类型</th>
                        <th>优先级</th>
                        <th>状态</th>
                        <th>创建时间</th>
                        <th>操作</th>
                    </tr>
                </thead>
                <tbody id="routesTableBody">
                    {% for route in routes %}
                    <tr data-id="{{ route.id }}">
                        <td>{{ route.id }}</td>
                        <td><code>{{ route.path }}</code></td>
                        <td><span class="route-type {{ route.route_type }}">{{ route.route_type }}</span></td>
                        <td><span class="handler-type">{{ route.handler_type }}</span></td>
                        <td>{{ route.priority }}</td>
                        <td>
                            {% if route.enabled %}
                            <span class="enabled-badge">✓ 启用</span>
                            {% else %}
                            <span class="disabled-badge">✗ 禁用</span>
                            {% endif %}
                        </td>
                        <td>{{ route.created_at | default(value="-") }}</td>
                        <td>
                            <div class="action-buttons">
                                <button class="btn btn-secondary" onclick="editRoute({{ route.id }})">编辑</button>
                                <button class="btn btn-danger" onclick="deleteRoute({{ route.id }})">删除</button>
                                {% if route.enabled %}
                                <button class="btn btn-secondary" onclick="toggleRoute({{ route.id }}, false)">禁用</button>
                                {% else %}
                                <button class="btn btn-primary" onclick="toggleRoute({{ route.id }}, true)">启用</button>
                                {% endif %}
                            </div>
                        </td>
                    </tr>
                    {% endfor %}
                </tbody>
            </table>
        </div>
    </div>
    
    <!-- 添加/编辑路由模态框 -->
    <div class="modal" id="routeModal">
        <div class="modal-content">
            <h2 id="modalTitle">添加路由</h2>
            <form id="routeForm">
                <input type="hidden" id="routeId">
                
                <div class="form-group">
                    <label for="routeType">路由类型 *</label>
                    <select id="routeType" required>
                        <option value="database">Database (数据库)</option>
                        <option value="memory">Memory (内存)</option>
                        <option value="file">File (文件)</option>
                    </select>
                </div>
                
                <div class="form-group">
                    <label for="routePath">路径 *</label>
                    <input type="text" id="routePath" placeholder="/custom-path" required>
                </div>
                
                <div class="form-group">
                    <label for="handlerType">处理器类型 *</label>
                    <select id="handlerType" required>
                        <option value="redirect">Redirect (重定向)</option>
                        <option value="static">Static (静态内容)</option>
                        <option value="template">Template (模板渲染)</option>
                        <option value="proxy">Proxy (代理)</option>
                        <option value="custom">Custom (自定义)</option>
                    </select>
                </div>
                
                <div class="form-group">
                    <label for="handlerConfig">处理器配置 (JSON) *</label>
                    <textarea id="handlerConfig" placeholder='{"type": "redirect", "target": "/new-location", "status_code": 301}' required></textarea>
                </div>
                
                <div class="form-group">
                    <label>
                        <input type="checkbox" id="routeEnabled" checked>
                        立即启用
                    </label>
                </div>
                
                <div class="form-group">
                    <label for="routePriority">优先级</label>
                    <input type="number" id="routePriority" value="0" min="0">
                </div>
                
                <div class="modal-actions">
                    <button type="button" class="btn btn-secondary" onclick="closeModal()">取消</button>
                    <button type="button" class="btn btn-secondary" onclick="testRoute()">测试</button>
                    <button type="submit" class="btn btn-primary">保存</button>
                </div>
            </form>
        </div>
    </div>
    
    <script src="/js/dyn-routing.js"></script>
</body>
</html>
```

```javascript
// templates/js/dyn-routing.js
let currentRouteId = null;

// 加载路由列表
async function loadRoutes() {
    try {
        const response = await fetch('/api/admin/dynamic-routes');
        const result = await response.json();
        
        if (result.success) {
            const tbody = document.getElementById('routesTableBody');
            tbody.innerHTML = result.data.routes.map(route => `
                <tr data-id="${route.id}">
                    <td>${route.id}</td>
                    <td><code>${escapeHtml(route.path)}</code></td>
                    <td><span class="route-type ${route.route_type}">${route.route_type}</span></td>
                    <td><span class="handler-type">${route.handler_type}</span></td>
                    <td>${route.priority}</td>
                    <td>
                        ${route.enabled 
                            ? '<span class="enabled-badge">✓ 启用</span>' 
                            : '<span class="disabled-badge">✗ 禁用</span>'}
                    </td>
                    <td>${route.created_at || '-'}</td>
                    <td>
                        <div class="action-buttons">
                            <button class="btn btn-secondary" onclick="editRoute(${route.id})">编辑</button>
                            <button class="btn btn-danger" onclick="deleteRoute(${route.id})">删除</button>
                            ${route.enabled 
                                ? `<button class="btn btn-secondary" onclick="toggleRoute(${route.id}, false)">禁用</button>`
                                : `<button class="btn btn-primary" onclick="toggleRoute(${route.id}, true)">启用</button>`}
                        </div>
                    </td>
                </tr>
            `).join('');
            
            // 更新统计
            document.querySelector('.stat-card:nth-child(1) .stat-value').textContent = result.data.total;
            const enabledCount = result.data.routes.filter(r => r.enabled).length;
            const disabledCount = result.data.routes.length - enabledCount;
            document.querySelector('.stat-card:nth-child(2) .stat-value').textContent = enabledCount;
            document.querySelector('.stat-card:nth-child(3) .stat-value').textContent = disabledCount;
        }
    } catch (error) {
        console.error('加载路由失败:', error);
        alert('加载路由失败: ' + error.message);
    }
}

// 显示创建模态框
function showCreateModal() {
    currentRouteId = null;
    document.getElementById('modalTitle').textContent = '添加路由';
    document.getElementById('routeId').value = '';
    document.getElementById('routeType').value = 'database';
    document.getElementById('routePath').value = '';
    document.getElementById('handlerType').value = 'redirect';
    document.getElementById('handlerConfig').value = JSON.stringify({
        type: 'redirect',
        target: '/new-location',
        status_code: 301
    }, null, 2);
    document.getElementById('routeEnabled').checked = true;
    document.getElementById('routePriority').value = 0;
    document.getElementById('routeModal').classList.add('active');
}

// 编辑路由
async function editRoute(id) {
    try {
        const response = await fetch(`/api/admin/dynamic-routes/${id}`);
        const result = await response.json();
        
        if (result.success) {
            const route = result.data;
            currentRouteId = id;
            document.getElementById('modalTitle').textContent = '编辑路由';
            document.getElementById('routeId').value = route.id;
            document.getElementById('routeType').value = route.route_type;
            document.getElementById('routePath').value = route.path;
            document.getElementById('handlerType').value = route.handler_type;
            document.getElementById('handlerConfig').value = JSON.stringify(route.handler_config, null, 2);
            document.getElementById('routeEnabled').checked = route.enabled;
            document.getElementById('routePriority').value = route.priority;
            document.getElementById('routeModal').classList.add('active');
        }
    } catch (error) {
        console.error('加载路由详情失败:', error);
        alert('加载路由详情失败: ' + error.message);
    }
}

// 关闭模态框
function closeModal() {
    document.getElementById('routeModal').classList.remove('active');
}

// 保存路由
async function saveRoute(event) {
    event.preventDefault();
    
    const routeData = {
        route_type: document.getElementById('routeType').value,
        path: document.getElementById('routePath').value,
        handler_type: document.getElementById('handlerType').value,
        handler_config: JSON.parse(document.getElementById('handlerConfig').value),
        enabled: document.getElementById('routeEnabled').checked,
        priority: parseInt(document.getElementById('routePriority').value) || 0
    };
    
    try {
        const url = currentRouteId 
            ? `/api/admin/dynamic-routes/${currentRouteId}`
            : '/api/admin/dynamic-routes';
        
        const method = currentRouteId ? 'PUT' : 'POST';
        
        const response = await fetch(url, {
            method: method,
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(routeData)
        });
        
        const result = await response.json();
        
        if (result.success) {
            alert(result.message);
            closeModal();
            loadRoutes();
        } else {
            alert('保存失败: ' + result.message);
        }
    } catch (error) {
        console.error('保存路由失败:', error);
        alert('保存路由失败: ' + error.message);
    }
}

// 删除路由
async function deleteRoute(id) {
    if (!confirm('确定要删除这个路由吗？')) {
        return;
    }
    
    try {
        const response = await fetch(`/api/admin/dynamic-routes/${id}`, {
            method: 'DELETE'
        });
        
        const result = await response.json();
        
        if (result.success) {
            alert(result.message);
            loadRoutes();
        } else {
            alert('删除失败: ' + result.message);
        }
    } catch (error) {
        console.error('删除路由失败:', error);
        alert('删除路由失败: ' + error.message);
    }
}

// 切换路由状态
async function toggleRoute(id, enabled) {
    const action = enabled ? 'enable' : 'disable';
    
    try {
        const response = await fetch(`/api/admin/dynamic-routes/${id}/${action}`, {
            method: 'POST'
        });
        
        const result = await response.json();
        
        if (result.success) {
            alert(result.message);
            loadRoutes();
        } else {
            alert('操作失败: ' + result.message);
        }
    } catch (error) {
        console.error('切换状态失败:', error);
        alert('切换状态失败: ' + error.message);
    }
}

// 测试路由
async function testRoute() {
    const routeData = {
        route_type: document.getElementById('routeType').value,
        path: document.getElementById('routePath').value,
        handler_type: document.getElementById('handlerType').value,
        handler_config: JSON.parse(document.getElementById('handlerConfig').value)
    };
    
    try {
        const response = await fetch('/api/admin/dynamic-routes/test', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(routeData)
        });
        
        const result = await response.json();
        
        if (result.success) {
            alert('路由测试成功!\n' + JSON.stringify(result.data, null, 2));
        } else {
            alert('路由测试失败: ' + result.message);
        }
    } catch (error) {
        console.error('测试路由失败:', error);
        alert('测试路由失败: ' + error.message);
    }
}

// 导出路由
async function exportRoutes() {
    try {
        const response = await fetch('/api/admin/dynamic-routes/export');
        const result = await response.json();
        
        if (result.success) {
            const blob = new Blob([JSON.stringify(result.data, null, 2)], { type: 'application/json' });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = `routes-export-${new Date().toISOString().split('T')[0]}.json`;
            a.click();
            URL.revokeObjectURL(url);
        }
    } catch (error) {
        console.error('导出失败:', error);
        alert('导出失败: ' + error.message);
    }
}

// 导入路由
async function importRoutes(event) {
    const file = event.target.files[0];
    if (!file) return;
    
    try {
        const text = await file.text();
        const data = JSON.parse(text);
        
        const response = await fetch('/api/admin/dynamic-routes/import', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(data)
        });
        
        const result = await response.json();
        
        if (result.success) {
            alert(`成功导入 ${result.data.imported} 条路由`);
            loadRoutes();
        } else {
            alert('导入失败: ' + result.message);
        }
    } catch (error) {
        console.error('导入失败:', error);
        alert('导入失败: ' + error.message);
    }
    
    event.target.value = '';
}

// HTML转义
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// 事件监听
document.getElementById('routeForm').addEventListener('submit', saveRoute);

// 页面加载时初始化
document.addEventListener('DOMContentLoaded', loadRoutes);
```

### 阶段5：API路由配置（第7天）

**任务清单：**
- [ ] 在api_routes.rs中添加所有动态路由API
- [ ] 在mod.rs中导出处理器模块

**代码示例：**

```rust
// src/routes/api_routes.rs
// 在文件末尾添加动态路由API
cfg.service(
    web::resource("/api/admin/dynamic-routes")
        .route(web::get().to(api_handlers::dynamic_routes::list_routes))
        .route(web::post().to(api_handlers::dynamic_routes::create_route))
).service(
    web::resource("/api/admin/dynamic-routes/{id}")
        .route(web::get().to(api_handlers::dynamic_routes::get_route))
        .route(web::put().to(api_handlers::dynamic_routes::update_route))
        .route(web::patch().to(api_handlers::dynamic_routes::patch_route))
        .route(web::delete().to(api_handlers::dynamic_routes::delete_route))
).service(
    web::resource("/api/admin/dynamic-routes/{id}/enable")
        .route(web::post().to(api_handlers::dynamic_routes::enable_route))
).service(
    web::resource("/api/admin/dynamic-routes/{id}/disable")
        .route(web::post().to(api_handlers::dynamic_routes::disable_route))
).service(
    web::resource("/api/admin/dynamic-routes/test")
        .route(web::post().to(api_handlers::dynamic_routes::test_route))
).service(
    web::resource("/api/admin/dynamic-routes/{id}/stats")
        .route(web::get().to(api_handlers::dynamic_routes::get_route_stats))
).service(
    web::resource("/api/admin/dynamic-routes/batch")
        .route(web::post().to(api_handlers::dynamic_routes::batch_operations))
).service(
    web::resource("/api/admin/dynamic-routes/export")
        .route(web::get().to(api_handlers::dynamic_routes::export_routes))
).service(
    web::resource("/api/admin/dynamic-routes/import")
        .route(web::post().to(api_handlers::dynamic_routes::import_routes))
).service(
    web::resource("/api/admin/dynamic-routes/logs")
        .route(web::get().to(api_handlers::dynamic_routes::get_logs))
);
```

---

## 八、测试计划

### 8.1 单元测试

```rust
// tests/unit/dynamic_routes_test.rs
use rustblog::db::repositories::DynamicRouteRepository;
use rustblog::db::models::DynamicRoute;

#[tokio::test]
async fn test_create_route() {
    let pool = create_test_pool().await;
    let repo = DynamicRouteRepository::new(pool);
    
    let route = DynamicRoute {
        id: None,
        route_type: "database".to_string(),
        path: "/test-route".to_string(),
        handler_type: "redirect".to_string(),
        handler_config: serde_json::json!({
            "type": "redirect",
            "target": "/target",
            "status_code": 301
        }),
        enabled: true,
        priority: 0,
        created_at: None,
        updated_at: None,
        created_by: Some("test".to_string()),
        metadata: None,
    };
    
    let id = repo.create(&route).await.unwrap();
    assert!(id > 0);
}

#[tokio::test]
async fn test_get_by_path() {
    let pool = create_test_pool().await;
    let repo = DynamicRouteRepository::new(pool);
    
    // 创建测试路由
    let route = create_test_route("/test-path");
    repo.create(&route).await.unwrap();
    
    // 查询路由
    let found = repo.get_by_path("/test-path").await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().path, "/test-path");
}

#[tokio::test]
async fn test_path_conflict() {
    let pool = create_test_pool().await;
    let repo = DynamicRouteRepository::new(pool);
    
    let route1 = create_test_route("/conflict-path");
    repo.create(&route1).await.unwrap();
    
    let route2 = create_test_route("/conflict-path");
    let result = repo.create(&route2).await;
    
    assert!(result.is_err());
}
```

### 8.2 集成测试

```rust
// tests/integration/dynamic_routes_integration_test.rs
use actix_web::{test, web, App};
use rustblog::app_state::create_test_state;

#[actix_web::test]
async fn test_create_route_api() {
    let state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(rustblog::routes::configure_routes)
    ).await;
    
    // 创建请求
    let payload = serde_json::json!({
        "route_type": "database",
        "path": "/integration-test",
        "handler_type": "redirect",
        "handler_config": {
            "type": "redirect",
            "target": "/target",
            "status_code": 301
        }
    });
    
    let req = test::TestRequest::post()
        .uri("/api/admin/dynamic-routes")
        .set_json(&payload)
        .insert_header(("Cookie", "auth_token=test_admin_token"))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
}

#[actix_web::test]
async fn test_list_routes_api() {
    let state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(rustblog::routes::configure_routes)
    ).await;
    
    let req = test::TestRequest::get()
        .uri("/api/admin/dynamic-routes?page=1&limit=20")
        .insert_header(("Cookie", "auth_token=test_admin_token"))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["success"].as_bool().unwrap());
}
```

---

## 九、部署计划

### 9.1 数据库迁移

```bash
# 创建迁移脚本
cat > migrations/001_create_dynamic_routes.sql << 'EOF'
-- 创建动态路由表
CREATE TABLE IF NOT EXISTS dynamic_routes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    route_type TEXT NOT NULL CHECK(route_type IN ('memory', 'file', 'database')),
    path TEXT NOT NULL UNIQUE,
    handler_type TEXT NOT NULL CHECK(handler_type IN ('redirect', 'static', 'template', 'proxy', 'custom')),
    handler_config TEXT NOT NULL,
    enabled BOOLEAN DEFAULT 1,
    priority INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_by TEXT,
    metadata TEXT
);

CREATE INDEX IF NOT EXISTS idx_dynamic_routes_path ON dynamic_routes(path);
CREATE INDEX IF NOT EXISTS idx_dynamic_routes_type ON dynamic_routes(route_type);
CREATE INDEX IF NOT EXISTS idx_dynamic_routes_enabled ON dynamic_routes(enabled);

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
    ip_address TEXT,
    user_agent TEXT,
    FOREIGN KEY (route_id) REFERENCES dynamic_routes(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_dynamic_route_logs_route_id ON dynamic_route_logs(route_id);

-- 创建统计表
CREATE TABLE IF NOT EXISTS dynamic_route_stats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    route_id INTEGER NOT NULL,
    access_count INTEGER DEFAULT 0,
    last_accessed_at TEXT,
    total_response_time_ms INTEGER DEFAULT 0,
    avg_response_time_ms REAL DEFAULT 0,
    error_count INTEGER DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (route_id) REFERENCES dynamic_routes(id) ON DELETE CASCADE,
    UNIQUE(route_id)
);

CREATE INDEX IF NOT EXISTS idx_dynamic_route_stats_route_id ON dynamic_route_stats(route_id);
EOF

# 执行迁移
sqlite3 data/rustblog.db < migrations/001_create_dynamic_routes.sql
```

### 9.2 编译和部署

```bash
# 编译
cargo build --release

# 备份现有数据库
cp data/rustblog.db data/rustblog.db.backup.$(date +%Y%m%d_%H%M%S)

# 部署
systemctl stop rustblog
cp target/release/rustblog /usr/local/bin/
systemctl start rustblog

# 验证
curl -H "Cookie: auth_token=YOUR_TOKEN" http://localhost:8080/api/admin/dynamic-routes
```

---

## 十、时间估算

| 阶段 | 任务 | 预计时间 |
|------|------|----------|
| 阶段1 | 数据库和模型 | 1天 |
| 阶段2 | API处理器 | 2-3天 |
| 阶段3 | 页面路由和处理器 | 1天 |
| 阶段4 | 前端界面 | 2-3天 |
| 阶段5 | API路由配置 | 1天 |
| 阶段6 | 测试 | 2天 |
| 阶段7 | 文档和部署 | 1天 |
| **总计** | | **11-12天** |

---

## 十一、验收标准

### 11.1 功能验收

- [ ] 可以通过 `/admin/dyn-routing` 访问管理页面
- [ ] 管理员可以创建、编辑、删除路由
- [ ] 支持三种路由类型（内存、文件、数据库）
- [ ] 支持五种处理器类型（重定向、静态、模板、代理、自定义）
- [ ] 路由测试功能正常
- [ ] 路由冲突检测正常
- [ ] 批量导入导出功能正常
- [ ] 操作日志记录正常

### 11.2 性能验收

- [ ] 路由列表查询 < 100ms
- [ ] 路由创建 < 50ms
- [ ] 路由测试 < 50ms
- [ ] 支持1000+路由

### 11.3 安全验收

- [ ] 非管理员无法访问
- [ ] 输入验证完整
- [ ] SQL注入防护
- [ ] XSS防护

---

## 十二、后续优化

### 12.1 功能扩展

- 路由分组管理
- 路由版本控制
- 路由A/B测试
- 实时路由监控
- 路由性能分析

### 12.2 性能优化

- 路由缓存优化
- 批量操作优化
- 查询优化
- 连接池优化

### 12.3 用户体验优化

- 路由可视化编辑器
- 拖拽式路由配置
- 路由模板库
- 智能路由建议

---

## 十三、参考资料

- [Actix-Web官方文档](https://actix.rs/)
- [SQLite文档](https://www.sqlite.org/docs.html)
- [Serde文档](https://serde.rs/)
- [dynamic-router文档](../dynamic-router/docs/)
- [RustBlog现有代码](../src/)





核心功能
   - 动态路由管理：支持管理员通过界面创建、编辑、删除路由
   - 三种路由类型：内存路由（临时）、文件路由（持久化）、数据库路由（生产环境）
   - 五种处理器类型：重定向、静态内容、模板渲染、代理、自定义

  架构设计
   - 完整的三层架构：处理器层 → 业务逻辑层 → 数据访问层
   - 详细的数据库表结构设计（dynamic_routes、dynamic_route_logs、dynamic_route_stats）
   - RESTful API设计（13个API端点）

  实施计划（7个阶段，总计11-12天）
   1. 阶段1：数据库和模型（1天）
   2. 阶段2：API处理器（2-3天）
   3. 阶段3：页面路由和处理器（1天）
   4. 阶段4：前端界面（2-3天）
   5. 阶段5：API路由配置（1天）
   6. 阶段6：测试（2天）
   7. 阶段7：文档和部署（1天）

  关键特性
   - ✅ 管理员鉴权保护
   - ✅ 路由冲突检测
   - ✅ 操作日志记录
   - ✅ 路由统计功能
   - ✅ 批量导入导出
   - ✅ 路由测试和预览