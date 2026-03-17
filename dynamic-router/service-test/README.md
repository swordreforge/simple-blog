# 动态路由服务测试

这是一个用于验证动态路由库功能的服务测试平台。

## 功能特性

✅ **动态路由管理**
- 实时添加、删除、查询路由
- 支持多种 HTTP 方法（GET, POST, PUT, DELETE, PATCH）
- 路由验证和类型检查

✅ **Web 管理界面**
- 可视化路由管理
- 实时路由统计
- 一键添加演示路由

✅ **高性能**
- 基于 Rust 和 Actix-Web
- 无锁数据结构优化
- 高效路由匹配

## 快速开始

### 1. 启动服务

```bash
# 在 service-test 目录下
cargo run
```

服务将在 `http://127.0.0.1:8080` 启动。

### 2. 访问管理界面

打开浏览器访问：http://127.0.0.1:8080

### 3. 测试路由功能

#### 通过 Web 界面
1. 点击"添加演示路由"快速添加预设路由
2. 使用表单添加自定义路由
3. 在路由列表中查看、删除或测试路由

#### 通过 API

**列出所有路由**
```bash
curl http://127.0.0.1:8080/admin/routes
```

**添加路由**
```bash
curl -X POST http://127.0.0.1:8080/admin/routes \
  -H "Content-Type: application/json" \
  -d '{
    "path": "/hello",
    "body": "Hello, World!",
    "content_type": "text/plain"
  }'
```

**获取路由详情**
```bash
curl http://127.0.0.1:8080/admin/routes/hello
```

**删除路由**
```bash
curl -X DELETE http://127.0.0.1:8080/admin/routes/hello
```

**访问路由**
```bash
curl http://127.0.0.1:8080/hello
```

## API 端点

### 管理端点

- `GET /admin/routes` - 列出所有路由
- `POST /admin/routes` - 添加新路由
- `GET /admin/routes/{path}` - 获取路由详情
- `DELETE /admin/routes/{path}` - 删除路由

### 演示端点

- `GET /demo/routes` - 获取预设演示路由列表
- `POST /demo/add` - 添加单个演示路由
- `POST /demo/add-all` - 批量添加所有演示路由
- `POST /demo/clear` - 清空所有路由
- `GET /demo/stats` - 获取路由统计信息

### 动态路由

- `GET /{path}` - 访问任意路由
- `POST /{path}` - 访问任意路由
- `PUT /{path}` - 访问任意路由
- `DELETE /{path}` - 访问任意路由
- `PATCH /{path}` - 访问任意路由

## 预设演示路由

| 路由 | 描述 | 支持方法 |
|------|------|----------|
| `/` | 欢迎页面 | GET, POST, PUT, DELETE, PATCH |
| `/api/status` | API 状态接口 | GET, POST, PUT, DELETE, PATCH |
| `/api/user` | 用户信息接口 | GET, POST, PUT, DELETE, PATCH |
| `/api/products` | 产品列表接口 | GET, POST, PUT, DELETE, PATCH |
| `/about` | 关于页面 | GET, POST, PUT, DELETE, PATCH |

## 技术栈

- **语言**: Rust
- **Web 框架**: Actix-Web 4
- **异步运行时**: Tokio
- **序列化**: serde / serde_json
- **日志**: tracing

## 验证的功能

1. ✅ 路由表的增删查操作
2. ✅ 动态路由匹配
3. ✅ 多种 HTTP 方法支持
4. ✅ 路由验证
5. ✅ 实时路由更新
6. ✅ RESTful API 设计
7. ✅ Web 管理界面
8. ✅ 并发安全性

## 注意事项

- 服务默认监听 `127.0.0.1:8080`
- 路由路径必须以 `/` 开头
- 支持的内容类型：text/plain, text/html, application/json, application/xml
- 按 `Ctrl+C` 停止服务

## 扩展开发

如需添加更多功能，可以参考：
- 主项目文档：`../docs/`
- 示例代码：`../examples/`
- API 文档：运行 `cargo doc --open`