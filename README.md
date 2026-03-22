# RustBlog

一个高性能、低资源占用的现代化博客系统，采用纯 Rust 后端实现，专为轻量级部署环境设计。

## 核心特性

### 🚀 极致性能与低内存占用

- **纯 Rust 后端**：利用 Rust 的内存安全和零成本抽象，确保高性能和稳定性
- **深度编译优化**：经过 LTO (Link-Time Optimization)、codegen-units=1 等多重优化
- **超低内存占用**：
  - 2核2G服务器启动内存占用 **< 25MB**
  - 运行时内存占用 **< 80MB**（标准alloc情况）
  - 适合小型 VPS 和边缘计算设备部署

### 🌐 动态路由系统

内置创新的动态路由引擎，支持：
- 运行时动态添加/修改/删除路由
- 热更新路由配置，无需重启服务
- 多种路由匹配策略优化（Trie、Radix Tree、SIMD 优化等）
- 路由存储持久化（文件/数据库/内存）
- 快捷菜单集成，快速访问常用路由

### 📝 智能文本摘要

集成轻量级中文文本摘要引擎：
- 基于 TextRank 算法的无 AI 纯算法实现
- 支持长文本自动摘要生成
- 语义相似度分析，保留关键信息
- 低资源消耗，无外部依赖

### ⌨️ 前端快捷键绑定

为提升用户体验，提供丰富的快捷键支持：
- 全局快捷键（如 `l` 打开登录面板）
- 页面级快捷键（如文章页面的导航快捷键）
- 可自定义快捷键配置
- 快捷键提示和帮助文档

## 快速开始

### 环境要求

- **Rust**: >= 1.85(需要支持2024edition)
- **操作系统**: Linux / macOS / Windows
- **数据库**: SQLite (默认，内置)
- **缓存**: Valkey/Redis (可选) 或本地缓存

### 构建安装

```bash
# 克隆仓库
git clone https://github.com/yourusername/rustblog.git
cd rustblog

# 构建 Release 版本
cargo build --release

# 运行
./target/release/rustblog
```

### 构建建议

1. **使用最新稳定版 Rust**: 确保使用 Rust 1.85 或更高版本以获得最佳性能
2. **启用编译器优化**: 本项目已配置深度编译优化，无需额外配置
3. **选择合适的内存分配器**（可选）：
   ```bash
   # 使用 jemalloc（推荐用于高并发场景）
   cargo build --release --features jemalloc
   
   # 使用 mimalloc
   cargo build --release --features mimalloc-alloc
   
   # 使用 tcmalloc
   cargo build --release --features tcmalloc-alloc
   ```

## 可用 Features

通过 cargo features 启用不同的功能模块：

| Feature | 说明 | 默认启用 |
|---------|------|---------|
| `default` | 默认配置 | ✅ |
| `valkey` | 启用 Valkey/Redis 缓存支持 | ❌ |
| `profiling` | 启用性能分析工具（flamegraph） | ❌ |
| `simd` | 启用 SIMD 优化的 JSON 解析 | ❌ |
| `jemalloc` | 使用 jemalloc 内存分配器 | ❌ |
| `mimalloc-alloc` | 使用 mimalloc 内存分配器 | ❌ |
| `tcmalloc-alloc` | 使用 tcmalloc 内存分配器 | ❌ |

### 使用示例

```bash
# 启用 Valkey 缓存
cargo build --release --features valkey

# 启用性能分析和 SIMD 优化
cargo build --release --features "profiling,simd"

# 使用 jemalloc 和 Valkey
cargo build --release --features "jemalloc,valkey"
```

## 架构亮点

### 内存管理

- 采用标准 alloc 配置，确保在低资源环境下稳定运行
- 智能缓存策略，自动管理内存使用
- 零拷贝设计，减少不必要的内存分配

### 并发性能

- 基于 Tokio 的异步运行时
- 使用 Actix-Web 高性能 Web 框架
- 无锁数据结构优化并发访问

### 安全性

- 使用 Argon2 进行密码哈希
- ECC (椭圆曲线加密) 用于数据加密
- JWT 身份验证
- SQL 注入防护

### 可扩展性

- 模块化设计，易于扩展功能
- 支持插件式架构
- 灵活的配置系统

## 贡献

欢迎提交 Issue 和 Pull Request！

## 许可证

[MIT License](https://github.com/swordreforge/simple-blog/blob/main/LISENSE)

## 致谢

感谢开源库贡献者和开源项目的支持。
