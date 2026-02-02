# Zig Blog

基于 Zig 语言编写的高性能博客系统，从 Rust 版本移植而来。

## 特性

- **高性能**: 使用 Zig 语言编写，零成本抽象
- **轻量级**: 无外部依赖，单文件部署
- **安全**: 类型安全，内存安全
- **SQLite 数据库**: 轻量级数据库，连接池支持
- **JWT 认证**: 安全的用户认证机制
- **Markdown 支持**: 完整的 Markdown 渲染
- **RESTful API**: 现代化的 API 设计
- **模板引擎**: 简单高效的模板系统

## 项目结构

```
zig-blog/
├── src/
│   ├── main.zig           # 主入口
│   ├── config.zig         # 配置管理
│   ├── database.zig       # 数据库操作
│   ├── repository.zig     # 数据仓库层
│   ├── auth.zig           # 认证服务
│   ├── http_server.zig    # HTTP 服务器
│   ├── markdown.zig       # Markdown 转换
│   ├── uuid.zig           # UUID 生成
│   ├── template.zig       # 模板引擎
│   └── models.zig         # 数据模型
├── build.zig              # 构建配置
├── build.zig.zon          # 依赖配置
└── README.md              # 说明文档
```

## 快速开始

### 1. 安装依赖

```bash
cd zig-blog
zig fetch
```

### 2. 构建项目

```bash
zig build
```

### 3. 运行服务器

```bash
zig build run
```

服务器将在 `http://localhost:8080` 启动。

### 4. 运行测试

```bash
zig build test
```

## 配置

配置在 `src/config.zig` 中定义：

- `server.host`: 服务器地址 (默认: 0.0.0.0)
- `server.port`: 服务器端口 (默认: 8080)
- `database.path`: 数据库路径 (默认: data/blog.db)
- `database.max_connections`: 最大连接数 (默认: 20)
- `jwt.secret`: JWT 密钥
- `jwt.expire_hours`: Token 过期时间 (默认: 24小时)

## API 端点

### 认证
- `POST /api/login` - 用户登录
- `POST /api/register` - 用户注册
- `GET /api/check` - 检查登录状态

### 文章
- `GET /api/passage/list` - 获取文章列表
- `GET /api/passage/:uuid` - 获取单篇文章
- `POST /api/passage` - 创建文章
- `PUT /api/passage/:uuid` - 更新文章
- `DELETE /api/passage/:uuid` - 删除文章

### 评论
- `GET /api/comments` - 获取评论列表
- `POST /api/comments` - 创建评论
- `DELETE /api/comments/:id` - 删除评论

### 页面
- `GET /` - 主页
- `GET /passage` - 文章列表页
- `GET /passage/:uuid` - 文章详情页
- `GET /collect` - 归档页
- `GET /about` - 关于页
- `GET /admin` - 管理后台

## 默认账户

- 用户名: `admin`
- 密码: `admin123`

**注意**: 生产环境请修改默认密码！

## 数据库表

### users
用户信息表

### passages
文章表

### comments
评论表

### settings
设置表

### attachments
附件表

### article_views
文章阅读统计表

## 开发

### 添加新的 API 端点

1. 在 `src/http_server.zig` 中添加路由
2. 实现处理函数
3. 在 `src/repository.zig` 中添加数据访问方法（如果需要）

### 数据库迁移

修改 `src/database.zig` 中的 `initTables` 函数来添加或修改表结构。

## 与 Rust 版本的对比

| 特性 | Rust 版本 | Zig 版本 |
|------|-----------|----------|
| Web 框架 | Actix-web | httpz |
| 数据库 | SQLite + r2d2 | SQLite + 自定义连接池 |
| 模板引擎 | Tera | 自定义模板引擎 |
| 认证 | JWT + jsonwebtoken | JWT (简化实现) |
| 密码哈希 | Argon2 | 简化实现 |
| Markdown | pulldown-cmark | 自定义解析器 |
| 资源嵌入 | rust-embed | 待实现 |

## 贡献

欢迎提交 Issue 和 Pull Request！

## 许可证

MIT License

## 致谢

- 原始 Rust 版本的设计和架构
- Zig 社区的优秀库和工具