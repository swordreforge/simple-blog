# 测试文章 04 - 数据库设计最佳实践

良好的数据库设计是系统性能和可维护性的基础。

## 数据库范式

- **第一范式 (1NF)**：确保每列都是原子的
- **第二范式 (2NF)**：满足 1NF 且非主属性完全依赖于主键
- **第三范式 (3NF)**：满足 2NF 且非主属性不传递依赖于主键

## 索引优化

```sql
CREATE INDEX idx_user_email ON users(email);
CREATE INDEX idx_post_created_at ON posts(created_at DESC);
```

## 查询优化技巧

1. 避免使用 SELECT *
2. 合理使用索引
3. 使用 LIMIT 限制结果集
4. 避免在 WHERE 子句中使用函数

标签：数据库, sql, 优化, 后端
分类：技术
摘要：良好的数据库设计是系统性能和可维护性的基础，本文介绍了数据库范式和索引优化技巧。
封面：/img/passage-cover2.webp