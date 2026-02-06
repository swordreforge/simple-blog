# RustBlog C语言重构版本

## 概述

这是一个使用 C 语言重构的博客系统，采用 **BearSSL + select** 模型，目标是实现极致优化的 HTTPS 服务器。

### 技术栈

- **TLS**: BearSSL (极小体积的 TLS 1.2 实现)
- **I/O模型**: select (单线程事件循环)
- **数据库**: SQLite (精简配置)
- **加密**: SHA256, Argon2, JWT
- **模板**: 自定义微型模板引擎
- **Markdown**: 简化版 Markdown 解析器

### 目标

- **二进制大小**: 250-500KB
- **内存占用**: < 5MB
- **并发连接**: 32 (可配置)
- **启动时间**: < 100ms

## 功能特性

### 已实现
- ✅ HTTPS 服务器 (BearSSL + select)
- ✅ SQLite 数据库支持
- ✅ 用户认证 (JWT + 密码哈希)
- ✅ 文章管理 (Markdown 渲染)
- ✅ 模板引擎
- ✅ 静态文件服务

### 计划中
- 🔄 音乐播放器
- 🔄 评论系统
- 🔄 友链管理
- 🔄 附件管理
- 🔄 GeoIP 定位

## 项目结构

```
cblog/
├── src/
│   ├── include/          # 头文件
│   ├── main.c            # 主入口
│   ├── server.c          # 服务器核心
│   ├── ssl.c             # BearSSL 封装
│   ├── http.c            # HTTP 协议处理
│   ├── router.c          # 路由系统
│   ├── database.c        # 数据库封装
│   ├── template.c        # 模板引擎
│   ├── crypto.c          # 加密功能
│   ├── jwt.c             # JWT 实现
│   ├── handlers/         # 请求处理器
│   └── utils/            # 工具函数
├── templates/            # HTML 模板
├── static/               # 静态文件
├── data/                 # 数据和证书
├── Makefile              # 构建系统
└── README.md             # 本文件
```

## 快速开始

### 依赖

- GCC 或 Clang
- Make
- OpenSSL (仅用于生成证书)

### 构建

```bash
# 1. 克隆 BearSSL
make bearssl

# 2. 生成证书
make certs

# 3. 编译
make

# 4. 初始化数据库
make init-db

# 5. 运行
make run
```

### 配置

编辑 `config.json`:

```json
{
  "server": {
    "host": "0.0.0.0",
    "port": 443,
    "max_connections": 32
  },
  "database": {
    "path": "data/blog.db"
  },
  "tls": {
    "cert_path": "data/cert.der",
    "key_path": "data/key.der"
  }
}
```

## API 端点

### 公开端点
- `GET /` - 首页
- `GET /passage/{id}` - 文章详情
- `GET /api/passages` - 文章列表
- `POST /api/auth/login` - 用户登录

### 管理端点 (需要认证)
- `GET /api/admin/passages` - 文章管理
- `POST /api/admin/passage` - 创建文章
- `PUT /api/admin/passage/{id}` - 更新文章
- `DELETE /api/admin/passage/{id}` - 删除文章

## 开发

### 调试模式

```bash
make debug
```

### 大小分析

```bash
make size
```

### 清理

```bash
make clean      # 清理构建文件
make distclean  # 深度清理
```

## 性能优化

### 编译优化
- `-Os`: 优化体积
- `-flto`: 链接时优化
- `-fdata-sections -ffunction-sections`: 去除未使用的代码
- `--gc-sections`: 链接时垃圾回收
- `--strip-all`: 去除符号表

### 运行时优化
- 单线程 select 事件循环
- 静态内存分配
- 连接池复用
- 模板缓存

## 与 Rust 版本对比

| 特性 | Rust 版本 | C 版本 |
|------|-----------|--------|
| 二进制大小 | ~2MB | ~300KB |
| 内存占用 | ~20MB | ~5MB |
| 启动时间 | ~200ms | ~50ms |
| 并发模型 | async/await | select |
| TLS | rustls | BearSSL |
| 功能完整性 | 100% | ~70% |

## 许可证

MIT License

## 贡献

欢迎提交 Issue 和 Pull Request！