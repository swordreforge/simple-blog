# 数据库连接池优化说明

## 优化内容

本次优化主要针对 SQLite 数据库连接池，以支持更好的读并发性能。

### 1. 连接池配置优化

#### 优化前
```rust
let pool = Pool::builder()
    .max_size(15)
    .min_idle(Some(5))
    .build(manager)?;
```

#### 优化后
```rust
const DB_MAX_CONNECTIONS: u32 = 50;  // 最大连接数
const DB_MIN_IDLE: u32 = 10;         // 最小空闲连接数
const DB_CONNECTION_TIMEOUT: u64 = 30;  // 连接超时（秒）
const DB_IDLE_TIMEOUT: u64 = 600;   // 空闲连接超时（秒，10分钟）
const DB_MAX_LIFETIME: u64 = 1800;  // 连接最大生命周期（秒，30分钟）

let pool = Pool::builder()
    .max_size(DB_MAX_CONNECTIONS)
    .min_idle(Some(DB_MIN_IDLE))
    .connection_timeout(std::time::Duration::from_secs(DB_CONNECTION_TIMEOUT))
    .idle_timeout(Some(std::time::Duration::from_secs(DB_IDLE_TIMEOUT)))
    .max_lifetime(Some(std::time::Duration::from_secs(DB_MAX_LIFETIME)))
    .test_on_check_out(true)  // 获取连接时测试连接是否有效
    .build(manager)?;
```

### 2. SQLite WAL 模式启用

启用了 Write-Ahead Logging (WAL) 模式，显著提升读并发性能：

```sql
PRAGMA journal_mode = WAL;
PRAGMA wal_autocheckpoint = 1000;
PRAGMA synchronous = NORMAL;
PRAGMA cache_size = -64000;  -- 64MB 缓存
PRAGMA temp_store = MEMORY;  -- 临时表使用内存
PRAGMA mmap_size = 268435456;  -- 256MB 内存映射
```

**WAL 模式的优势：**
- 读写操作不会互相阻塞
- 多个读操作可以并发执行
- 减少磁盘 I/O
- 提高整体并发性能

### 3. 连接池监控

新增了连接池状态监控功能：

#### 获取连接池状态
```
GET /api/db/pool-status
```

响应示例：
```json
{
  "success": true,
  "data": {
    "max_connections": 50,
    "min_idle": 10,
    "current_connections": 15,
    "idle_connections": 8,
    "active_connections": 7,
    "connection_utilization": 14.0
  }
}
```

#### 数据库健康检查
```
GET /api/db/health
```

响应示例：
```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "pool_status": {
      "max_connections": 50,
      "min_idle": 10,
      "current_connections": 15,
      "idle_connections": 8,
      "active_connections": 7,
      "connection_utilization": 14.0
    },
    "database_size": 1048576,
    "wal_enabled": true
  }
}
```

## 配置参数说明

| 参数 | 值 | 说明 |
|------|-----|------|
| `DB_MAX_CONNECTIONS` | 50 | 连接池最大连接数，可根据服务器内存调整 |
| `DB_MIN_IDLE` | 10 | 最小空闲连接数，保证随时有可用连接 |
| `DB_CONNECTION_TIMEOUT` | 30秒 | 获取连接的超时时间 |
| `DB_IDLE_TIMEOUT` | 600秒 | 空闲连接的超时时间，避免连接长时间闲置 |
| `DB_MAX_LIFETIME` | 1800秒 | 连接的最大生命周期，定期重建连接 |
| `test_on_check_out` | true | 获取连接时测试连接有效性 |

## 性能提升

### 优化前
- 最大连接数：15
- 最小空闲连接：5
- 不支持 WAL 模式
- 读写操作可能互相阻塞

### 优化后
- 最大连接数：50（提升 233%）
- 最小空闲连接：10（提升 100%）
- 启用 WAL 模式
- 支持并发读写
- 连接自动健康检查

### 预期性能提升
- **读并发性能**：提升 200-300%
- **写操作响应时间**：减少 30-50%
- **并发请求处理能力**：提升 150-200%

## 监控建议

1. **定期检查连接池状态**
   ```bash
   curl http://localhost:8080/api/db/pool-status
   ```

2. **监控连接利用率**
   - 如果 `connection_utilization` 持续超过 80%，考虑增加 `DB_MAX_CONNECTIONS`
   - 如果 `connection_utilization` 持续低于 20%，可以减少 `DB_MIN_IDLE`

3. **关注数据库大小**
   ```bash
   curl http://localhost:8080/api/db/health
   ```
   - 定期清理无用数据
   - 考虑数据库归档策略

4. **WAL 文件管理**
   - WAL 文件会随着写操作增长
   - 定期检查 `data/blog.db-wal` 和 `data/blog.db-shm` 文件大小
   - 如果过大，可以手动触发 checkpoint：
     ```sql
     PRAGMA wal_checkpoint(TRUNCATE);
     ```

## 注意事项

1. **内存使用**
   - 增加连接数会增加内存使用
   - 每个连接大约占用 1-2MB 内存
   - 50 个连接约需 50-100MB 内存

2. **WAL 模式限制**
   - WAL 模式不支持某些 SQLite 功能（如某些备份方法）
   - 在网络文件系统上使用可能有问题
   - 需要适当的文件权限

3. **并发写入**
   - 虽然 WAL 模式支持并发读取，但写入仍然是串行的
   - 高并发写入时，考虑使用队列或批处理

## 调优建议

根据实际负载情况调整参数：

### 低负载场景（个人博客，日均访问 < 1000）
```rust
const DB_MAX_CONNECTIONS: u32 = 20;
const DB_MIN_IDLE: u32 = 5;
```

### 中等负载场景（日均访问 1000-10000）
```rust
const DB_MAX_CONNECTIONS: u32 = 50;
const DB_MIN_IDLE: u32 = 10;
```

### 高负载场景（日均访问 > 10000）
```rust
const DB_MAX_CONNECTIONS: u32 = 100;
const DB_MIN_IDLE: u32 = 20;
```

## 故障排查

### 问题：连接池耗尽
**症状：** 请求超时，日志中出现 "No connection available"

**解决方案：**
1. 检查连接池状态：`GET /api/db/pool-status`
2. 增加最大连接数
3. 检查是否有连接泄漏（未正确释放连接）

### 问题：响应速度慢
**症状：** 数据库查询响应时间长

**解决方案：**
1. 检查数据库索引是否合理
2. 检查 WAL 文件大小
3. 增加缓存大小：`PRAGMA cache_size`
4. 考虑使用查询缓存

### 问题：数据库文件损坏
**症状：** 查询失败，日志中出现 "database disk image is malformed"

**解决方案：**
1. 从备份恢复
2. 使用 SQLite 的 dump 功能导出数据
3. 重新初始化数据库

## 相关文件

- `/src/db/init.rs` - 数据库初始化和连接池配置
- `/src/handlers/api_handlers/db_stats.rs` - 数据库统计 API
- `/src/routes/api_routes.rs` - API 路由配置
- `/data/blog.db` - 数据库文件
- `/data/blog.db-wal` - WAL 文件
- `/data/blog.db-shm` - 共享内存文件