# RustBlog 游标分页架构设计

## 概述

RustBlog 采用游标分页（Cursor Pagination）机制，支持无限文章数量的快速查询和渲染，避免了传统 OFFSET 分页的性能瓶颈。

## 后端设计

### 1. 数据库设计

#### 表结构
```sql
CREATE TABLE passages (
    id INTEGER PRIMARY KEY,              -- 自增主键，理论上支持无限文章
    uuid TEXT UNIQUE NOT NULL,            -- Snowflake UUID，用于分布式场景
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    original_content TEXT,
    summary TEXT,
    author TEXT DEFAULT '管理员',
    tags TEXT DEFAULT '[]',
    category TEXT DEFAULT '未分类',
    status TEXT DEFAULT 'published',
    file_path TEXT,
    visibility TEXT DEFAULT 'public',
    is_scheduled INTEGER DEFAULT 0,
    published_at DATETIME,
    cover_image TEXT DEFAULT '/img/passage-cover.webp',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

#### 索引优化
```sql
-- 复合索引，优化分页查询
CREATE INDEX idx_passages_status_created ON passages(status, created_at DESC);

-- 单列索引
CREATE INDEX idx_passages_created_at ON passages(created_at);
CREATE INDEX idx_passages_status ON passages(status);
```

#### SQLite 性能优化
```rust
// 启用 WAL 模式以支持更好的并发读写
PRAGMA journal_mode = WAL;

// 增加 WAL 文件大小限制
PRAGMA wal_autocheckpoint = 1000;

// 平衡性能和数据安全
PRAGMA synchronous = NORMAL;

// 64MB 缓存
PRAGMA cache_size = -64000;

// 临时表使用内存
PRAGMA temp_store = MEMORY;

// 256MB 内存映射
PRAGMA mmap_size = 268435456;
```

### 2. 游标分页实现

#### 游标格式
```
created_at|id
```

- **created_at**: ISO 8601 格式的时间戳，如 `2026-02-02 00:00:00+00:00`
- **id**: 文章的自增 ID
- **分隔符**: `|` 避免与时间戳中的 `:` 冲突

#### 查询逻辑
```sql
-- 第一页：没有游标
SELECT * FROM passages 
WHERE status = 'published' 
ORDER BY created_at DESC, id DESC 
LIMIT ?;

-- 后续页：使用游标
SELECT * FROM passages 
WHERE status = 'published' 
  AND (created_at < ? OR (created_at = ? AND id < ?))
ORDER BY created_at DESC, id DESC 
LIMIT ?;
```

#### 性能特点
- **时间复杂度**: O(1) - 使用索引扫描，无需计算 OFFSET
- **空间复杂度**: O(1) - 每次只查询固定数量的记录
- **并发性能**: WAL 模式支持读写并发

### 3. API 设计

#### 请求参数
```
GET /api/passages?limit=10
GET /api/passages?cursor=2026-02-02%2000%3A00%3A00%2B00%3A00%7C5&limit=10
```

#### 响应格式
```json
{
  "success": true,
  "data": [...],
  "pagination": {
    "has_more": true,
    "next_cursor": "2026-02-02 00:00:00+00:00|5",
    "limit": 10
  }
}
```

#### 限制
- 单次请求最多 1000 篇文章
- 游标格式必须符合规范
- 支持日期筛选（年/月/日）

## 前端设计

### 1. 分页策略

#### 配置参数
```javascript
const pageSize = 10;              // 每页加载 10 篇文章
const useCursorPagination = true; // 启用游标分页
let currentCursor = null;         // 当前游标
let hasMoreArticles = true;       // 是否还有更多文章
```

#### 数据加载流程
```javascript
// 第一次请求：没有游标
GET /api/passages?limit=10

// 后续请求：使用游标
GET /api/passages?cursor=2026-02-02%2000%3A00%3A00%2B00%3A00%7C5&limit=10
```

### 2. 数据结构

#### 文章组织
```javascript
articlesData = {
  folders: [
    {
      name: "2026年",
      id: "year-2026",
      open: false,
      subfolders: {
        "2026-02": {
          name: "02月",
          id: "2026-02",
          open: false,
          subfolders: {
            "2026-02-02": {
              name: "02日",
              id: "2026-02-02",
              open: false,
              files: [...]
            }
          }
        }
      }
    }
  ]
};
```

#### 去重机制
```javascript
function mergeArticlesData(existingData, newData) {
  // 使用 Map 确保文章 ID 唯一
  const articleMap = new Map();
  existingArticles.forEach(article => {
    articleMap.set(article.id, article);
  });
  newArticles.forEach(article => {
    articleMap.set(article.id, article);
  });
  
  // 重新组织所有去重后的文章
  const allArticles = Array.from(articleMap.values());
  return organizeArticlesByFolder(allArticles);
}
```

### 3. 渲染优化

#### 增量渲染
```javascript
// 只渲染新增的文件夹和文件
function renderFileTreeIncremental(newData) {
  // 简化实现：重新渲染整个树
  renderFileTree();
}
```

#### 无限滚动
```javascript
// 滚动到底部 300px 时加载更多
fileTree.addEventListener('scroll', () => {
  if (scrollTop + clientHeight >= scrollHeight - 300 
      && hasMoreArticles 
      && !isLoadingArticles) {
    fetchArticlesData();
  }
});
```

## 性能分析

### 1. 数据库性能

| 指标 | 传统分页 (OFFSET) | 游标分页 |
|------|------------------|----------|
| 时间复杂度 | O(n) | O(1) |
| 深页性能 | 差 | 优秀 |
| 缓存友好 | 差 | 优秀 |
| 并发性能 | 一般 | 优秀 |

### 2. 前端性能

| 指标 | 当前实现 | 优化建议 |
|------|----------|----------|
| 初始加载 | 10 篇文章 | ✅ 优秀 |
| 内存占用 | 随文章数增长 | ⚠️ 需优化 |
| DOM 节点数 | 随文章数增长 | ⚠️ 需优化 |
| 渲染性能 | 良好 | ✅ 优秀 |

### 3. 扩展性

#### 理论限制
- **文章数量**: 9,223,372,036,854,775,807 (INTEGER 最大值)
- **数据库大小**: 281TB (SQLite 单个文件限制)
- **并发连接**: 20 (连接池配置)

#### 实际限制
- **前端内存**: 建议不超过 10,000 篇文章
- **文件夹数量**: 建议不超过 1,000 个年份
- **文件大小**: 建议定期清理旧文章

## 优化建议

### 1. 数据库优化
- [ ] 定期执行 `VACUUM` 回收空间
- [ ] 定期执行 `ANALYZE` 更新统计信息
- [ ] 考虑分区表（按年份分区）
- [ ] 增加全文搜索索引

### 2. 前端优化
- [ ] 实现虚拟滚动（Virtual Scrolling）
- [ ] 按年/月筛选功能
- [ ] 懒加载折叠的文件夹
- [ ] 使用 Web Worker 处理数据

### 3. 缓存优化
- [ ] 增加文章列表缓存
- [ ] 实现智能预加载
- [ ] 使用 Service Worker 离线缓存

### 4. 监控告警
- [ ] 监控数据库文件大小
- [ ] 监控查询性能
- [ ] 监控前端内存占用
- [ ] 设置自动归档机制

## 最佳实践

### 1. 文章上传
- 每篇文章不超过 10MB
- 定期清理草稿
- 使用标签和分类组织文章

### 2. 数据维护
- 每月执行一次 `VACUUM`
- 每季度归档旧文章
- 定期备份数据库

### 3. 性能监控
- 监控慢查询
- 监控缓存命中率
- 监控前端加载时间

## 总结

RustBlog 的游标分页架构设计能够很好地支持无限文章的场景：

✅ **优势**
- 查询性能稳定，不受数据量影响
- 支持并发读写
- 前端按需加载，用户体验好
- 代码简洁，易于维护

⚠️ **注意事项**
- 前端内存占用随文章数增长
- 需要定期维护数据库
- 建议实施归档策略

🎯 **适用场景**
- 中小型博客（< 10,000 篇文章）
- 需要快速查询的场景
- 高并发读写的场景