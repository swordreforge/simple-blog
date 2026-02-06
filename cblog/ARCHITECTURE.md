# RustBlog C语言版本 - 架构文档

## 概述

这是一个使用 C 语言重构的博客系统，采用 **BearSSL + select** 模型，目标是实现极致优化的 HTTPS 服务器。

## 技术架构

### 核心技术栈

```
┌─────────────────────────────────────────────────────────┐
│                     应用层                               │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────┐ │
│  │  路由系统 │  │ HTTP处理 │  │ 模板引擎 │  │  API   │ │
│  └──────────┘  └──────────┘  └──────────┘  └────────┘ │
└─────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────┐
│                     业务层                               │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────┐ │
│  │  文章管理 │  │  用户管理 │  │  评论系统 │  │  音乐  │ │
│  └──────────┘  └──────────┘  └──────────┘  └────────┘ │
└─────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────┐
│                     数据层                               │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────┐ │
│  │  SQLite  │  │  文件系统 │  │  加密模块 │  │  JWT   │ │
│  └──────────┘  └──────────┘  └──────────┘  └────────┘ │
└─────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────┐
│                     网络层                               │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────┐ │
│  │ BearSSL  │  │  select  │  │  Socket  │  │  TCP   │ │
│  └──────────┘  └──────────┘  └──────────┘  └────────┘ │
└─────────────────────────────────────────────────────────┘
```

### 模块说明

#### 1. 服务器核心 (server.c)
- 使用 `select` 系统调用实现单线程事件循环
- 支持 32 个并发连接（可配置）
- 非阻塞 I/O 模型
- 连接超时管理

#### 2. TLS 层 (ssl.c)
- 使用 BearSSL 实现 TLS 1.2
- 支持 RSA 证书
- 自动握手处理
- 最小化代码体积

#### 3. HTTP 协议 (http.c)
- HTTP/1.1 协议解析
- 请求/响应构建
- URL 参数解析
- Base64 编码/解码
- MIME 类型识别

#### 4. 路由系统 (router.c)
- 简单的模式匹配
- 支持通配符路由
- 权限检查中间件
- 静态文件路由

#### 5. 数据库层 (database.c)
- SQLite 精简配置
- 表结构自动创建
- 事务支持
- 预编译语句缓存

#### 6. 模板引擎 (template.c)
- 变量替换
- 内置函数（时间、截断、HTML 转义等）
- 模板包含
- 简单缓存

#### 7. 加密模块 (crypto.c)
- SHA256 哈希
- 密码哈希（简化版 PBKDF2）
- UUID 生成
- 随机数生成
- HMAC-SHA256

#### 8. JWT 认证 (jwt.c)
- JWT Token 生成和验证
- HSA256 签名
- 过期检查
- 角色权限验证

## 数据库设计

### 表结构

#### users（用户表）
```sql
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    email TEXT,
    role TEXT DEFAULT 'user',
    status TEXT DEFAULT 'active',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
```

#### passages（文章表）
```sql
CREATE TABLE passages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    uuid TEXT UNIQUE NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    original_content TEXT,
    summary TEXT,
    author TEXT DEFAULT 'Anonymous',
    tags TEXT DEFAULT '[]',
    category TEXT DEFAULT '未分类',
    status TEXT DEFAULT 'draft',
    file_path TEXT,
    visibility TEXT DEFAULT 'public',
    is_scheduled INTEGER DEFAULT 0,
    published_at INTEGER,
    cover_image TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
```

#### comments（评论表）
```sql
CREATE TABLE comments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL,
    content TEXT NOT NULL,
    passage_uuid TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY(passage_uuid) REFERENCES passages(uuid)
);
```

#### categories（分类表）
```sql
CREATE TABLE categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL,
    description TEXT,
    icon TEXT DEFAULT '📁',
    sort_order INTEGER DEFAULT 0,
    is_enabled INTEGER DEFAULT 1,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
```

#### tags（标签表）
```sql
CREATE TABLE tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL,
    description TEXT,
    color TEXT DEFAULT '#007bff',
    category_id INTEGER DEFAULT 0,
    sort_order INTEGER DEFAULT 0,
    is_enabled INTEGER DEFAULT 1,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
```

#### friend_links（友链表）
```sql
CREATE TABLE friend_links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    nickname TEXT NOT NULL,
    link_url TEXT NOT NULL,
    avatar_url TEXT,
    motto TEXT,
    sort_order INTEGER DEFAULT 0,
    is_enabled INTEGER DEFAULT 1,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
```

#### music（音乐表）
```sql
CREATE TABLE music (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    artist TEXT,
    file_path TEXT NOT NULL,
    file_name TEXT NOT NULL,
    duration TEXT,
    cover_image TEXT,
    created_at INTEGER NOT NULL
);
```

#### settings（设置表）
```sql
CREATE TABLE settings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    key TEXT UNIQUE NOT NULL,
    value TEXT NOT NULL,
    type TEXT DEFAULT 'string',
    description TEXT,
    category TEXT DEFAULT 'general',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
```

## API 接口

### 公开接口

#### 文章相关
- `GET /api/passages` - 获取文章列表
- `GET /api/passage/{id}` - 获取单篇文章
- `GET /api/comments/{passage_uuid}` - 获取文章评论
- `POST /api/comment` - 提交评论

#### 音乐相关
- `GET /api/music` - 获取音乐列表
- `GET /api/music/{id}` - 获取单首音乐

#### 认证相关
- `POST /api/auth/login` - 用户登录

### 管理接口（需要认证）

#### 文章管理
- `GET /api/admin/passages` - 获取文章管理列表
- `POST /api/admin/passage` - 创建文章
- `PUT /api/admin/passage/{id}` - 更新文章
- `DELETE /api/admin/passage/{id}` - 删除文章

#### 用户管理
- `GET /api/admin/users` - 获取用户列表
- `POST /api/admin/user` - 创建用户
- `PUT /api/admin/user/{id}` - 更新用户
- `DELETE /api/admin/user/{id}` - 删除用户

#### 评论管理
- `DELETE /api/admin/comment/{id}` - 删除评论

#### 音乐管理
- `POST /api/admin/music` - 上传音乐
- `DELETE /api/admin/music/{id}` - 删除音乐

#### 系统设置
- `GET /api/admin/settings` - 获取设置
- `PUT /api/admin/settings` - 更新设置

#### 统计数据
- `GET /api/stats` - 获取统计数据

## 性能优化

### 编译优化
- `-Os`: 优化体积
- `-flto`: 链接时优化
- `-fdata-sections -ffunction-sections`: 去除未使用代码
- `--gc-sections`: 链接时垃圾回收
- `--strip-all`: 去除符号表

### 运行时优化
- 单线程 select 事件循环
- 静态内存分配（减少 malloc/free）
- 连接池复用
- 模板渲染缓存
- 数据库连接池（使用 r2d2）

### SQLite 优化
- 禁用不必要的功能（WAL、UTF16 等）
- 减小页面大小（1024 字节）
- 减少缓存大小（2000 页）
- 限制最大 SQL 长度
- 使用预编译语句

## 安全考虑

### TLS
- 使用 BearSSL（经过审计）
- 禁用不安全的密码套件
- 强制 TLS 1.2+
- 使用 RSA 2048 位密钥

### 密码存储
- 使用 PBKDF2 或 Argon2
- 多轮迭代（10000+）
- 随机盐值
- 防止时序攻击

### JWT 认证
- HSA256 签名
- 短期过期时间
- 安全的密钥存储
- 防止重放攻击

### 输入验证
- SQL 注入防护（参数化查询）
- XSS 防护（HTML 转义）
- CSRF Token
- 请求大小限制

## 部署建议

### 系统要求
- Linux (内核 3.0+)
- 512MB RAM
- 10MB 磁盘空间
- root 权限（绑定 443 端口）

### 部署方式
1. 使用 systemd 服务（推荐）
2. 使用 Docker 容器
3. 使用 supervisor

### 反向代理
如果需要使用 Nginx/Caddy 作为反向代理：

```nginx
location / {
    proxy_pass https://127.0.0.1:8443;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
}
```

## 性能指标

预期性能（单核 2GHz CPU）：
- 启动时间: < 100ms
- 内存占用: < 5MB
- 并发连接: 32
- 请求/秒: ~1000（静态）, ~200（动态）
- 二进制大小: 250-500KB

## 开发指南

### 添加新功能
1. 在 `src/handlers/` 中创建处理器
2. 在 `router.c` 中注册路由
3. 在 `database.c` 中添加数据库操作
4. 更新文档

### 调试
- 使用 `make debug` 编译调试版本
- 添加日志输出
- 使用 gdb 或 lldb

### 测试
- 单元测试（待实现）
- 集成测试（待实现）
- 性能测试（待实现）

## 已知限制

1. **单线程**: 使用 select 模型，无法充分利用多核
2. **HTTP/1.1**: 不支持 HTTP/2 和 HTTP/3
3. **无 WebSocket**: 需要额外实现
4. **简化认证**: 密码哈希算法简化，建议生产环境使用 Argon2
5. **JSON 解析**: 简化版，建议使用 cJSON 或类似库

## 未来计划

- [ ] 添加 HTTP/2 支持
- [ ] 实现真正的 Argon2 密码哈希
- [ ] 添加 WebSocket 支持
- [ ] 实现完整的 Markdown 解析器
- [ ] 添加 GeoIP 支持
- [ ] 实现全文搜索
- [ ] 添加单元测试
- [ ] 性能优化和基准测试

## 许可证

MIT License