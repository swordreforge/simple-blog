# 测试文章 07 - RESTful API 设计原则

RESTful API 是一种 Web API 设计风格，强调资源的统一接口。

## 核心原则

1. **资源导向**：一切皆为资源
2. **统一接口**：使用标准 HTTP 方法
3. **无状态**：每个请求包含所有必要信息
4. **分层系统**：支持代理和负载均衡

## HTTP 方法映射

- GET：获取资源
- POST：创建资源
- PUT：更新资源（全量）
- PATCH：更新资源（部分）
- DELETE：删除资源

## 状态码规范

- 200 OK：请求成功
- 201 Created：资源创建成功
- 400 Bad Request：请求参数错误
- 401 Unauthorized：未授权
- 404 Not Found：资源不存在
- 500 Internal Server Error：服务器错误

## 设计示例

```
GET    /api/articles       # 获取文章列表
POST   /api/articles       # 创建文章
GET    /api/articles/:id   # 获取单篇文章
PUT    /api/articles/:id   # 更新文章
DELETE /api/articles/:id   # 删除文章
```

标签：api, rest, 后端, 架构
分类：技术
摘要：RESTful API 是一种 Web API 设计风格，本文介绍了其核心原则和 HTTP 方法映射。
封面：/img/passage-cover.webp