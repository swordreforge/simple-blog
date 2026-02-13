# 测试文章 06 - Docker 容器化入门

Docker 是一个开源的容器化平台，让应用可以在任何环境中一致运行。

## 核心概念

- **镜像 (Image)**：应用的只读模板
- **容器 (Container)**：镜像的运行实例
- **Dockerfile**：构建镜像的脚本
- **Docker Compose**：多容器应用编排

## Dockerfile 示例

```dockerfile
FROM rust:1.75
WORKDIR /app
COPY . .
RUN cargo build --release
CMD ["./target/release/myapp"]
```

## Docker Compose 示例

```yaml
version: '3'
services:
  app:
    build: .
    ports:
      - "8080:8080"
  db:
    image: postgres:15
    environment:
      POSTGRES_PASSWORD: secret
```

标签：docker, 容器化, devops, 运维
分类：技术
摘要：Docker 是一个开源的容器化平台，让应用可以在任何环境中一致运行，本文介绍了 Docker 的核心概念和使用方法。
封面：/img/passage-cover2.webp