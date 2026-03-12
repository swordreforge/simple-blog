# 缓存并发安全使用指南

## 概述

本指南展示了如何在博客系统中使用并发安全防护机制，防止缓存击穿、穿透、雪崩等问题。

## 1. 缓存击穿防护（互斥锁）

### 问题
热点文章缓存过期时，大量并发请求同时穿透到数据库。

### 解决方案
使用 `SafeCacheBackend` 的 `get_or_load` 方法，内置互斥锁保护。

```rust
use rustblog::cache::{SafeCacheBackend, CacheLock};

// 创建安全的缓存后端
let safe_cache = SafeCacheBackend::new(cache_backend, true); // 启用分布式锁

// 使用 get_or_load 模式
let result = safe_cache.get_or_load(
    &cache_key,
    || async {
        // 从数据库加载文章
        passage_repo.get_by_id(id).await
            .map_err(|e| CacheError::Unknown(e.to_string()))
            .map(|p| serde_json::to_string(&p).ok())
    },
    Duration::from_secs(3600),
).await?;
```

## 2. 缓存穿透防护（缓存空值）

### 问题
查询不存在的文章时，每次都打到数据库。

### 解决方案
自动缓存空值，防止重复穿透。

```rust
// SafeCacheBackend 自动处理空值缓存
let result = safe_cache.get_safe(&cache_key).await;

// 如果 result 为 None，可能是：
// 1. 缓存中没有数据
// 2. 缓存中有空值标记（说明数据不存在）
// 3. 数据不存在且未缓存空值

// 检查是否命中空值缓存
if let Some(value) = safe_cache.inner().get(&cache_key).await {
    if is_null_value(&value) {
        // 数据不存在，直接返回 404
        return Ok(HttpResponse::NotFound().finish());
    }
}
```

## 3. 缓存雪崩防护（过期时间打散）

### 问题
大量缓存同时过期，集体穿透到数据库。

### 解决方案
自动在 TTL 基础上增加随机抖动（默认 10%）。

```rust
// SafeCacheBackend 自动处理 TTL 抖动
safe_cache.set_safe(
    &cache_key,
    Some(&json_value),
    Duration::from_secs(3600), // 基础 TTL 1 小时
).await?;

// 实际 TTL 会在 3240 ~ 3960 秒之间随机
```

## 4. 缓存删除失败重试

### 问题
删除缓存失败后，导致脏数据残留。

### 解决方案
使用 `RetryCacheBackend` 自动重试删除操作。

```rust
use rustblog::cache::RetryCacheBackend;

// 创建带重试机制的缓存后端
let retry_cache = RetryCacheBackend::new(cache_backend, true); // 启用重试

// 正常删除，失败会自动重试
retry_cache.delete(&cache_key).await?;

// 批量删除
retry_cache.delete_many(&keys).await?;

// 模式删除
retry_cache.delete_pattern("passage:*").await?;

// 查看重试队列状态
let queue_size = retry_cache.get_queue_size().await;
println!("当前重试队列长度: {}", queue_size);
```

## 5. 延迟双删（保证双写一致性）

### 问题
更新数据库后删除缓存，并发读请求可能将旧数据写回缓存。

### 解决方案
在更新数据库后，先删除缓存，延迟一段时间后再次删除。

```rust
async fn update_passage_with_double_delete(
    id: i32,
    update_data: UpdatePassage,
    cache: &CacheManager,
) -> Result<HttpResponse> {
    // 1. 更新数据库
    passage_repo.update(id, &update_data).await?;

    let cache_key = format!("passage:{}", id);

    // 2. 第一次删除缓存
    let _ = cache.delete(&cache_key).await;

    // 3. 延迟 500ms 后第二次删除
    tokio::spawn({
        let cache = cache.clone();
        let cache_key = cache_key.clone();
        async move {
            tokio::time::sleep(Duration::from_millis(500)).await;
            let _ = cache.delete(&cache_key).await;
            tracing::debug!("延迟双删完成: {}", cache_key);
        }
    });

    Ok(HttpResponse::Ok().finish())
}
```

## 6. 完整示例：文章缓存使用

```rust
use actix_web::{web, HttpResponse};
use rustblog::cache::{SafeCacheBackend, RetryCacheBackend, is_null_value};
use std::time::Duration;

/// 获取文章详情（带完整并发防护）
pub async fn get_passage(
    id: web::Path<i32>,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let cache_key = format!("passage:{}", id);

    // 使用 SafeCacheBackend + RetryCacheBackend 的组合
    let safe_cache = SafeCacheBackend::new(
        RetryCacheBackend::new(state.cache.manager().unwrap().clone(), true),
        true
    );

    // 使用 get_or_load 模式，防止缓存击穿
    let result = safe_cache.get_or_load(
        &cache_key,
        || async {
            // 从数据库加载
            passage_repo.get_by_id(id)
                .await
                .map_err(|e| CacheError::Unknown(e.to_string()))
                .and_then(|p| {
                    if p.status.is_published() && p.visibility.is_public() {
                        // 仅缓存公开文章
                        serde_json::to_string(&p)
                            .map_err(|e| CacheError::Unknown(e.to_string()))
                            .map(Some)
                    } else {
                        Ok(None) // 不缓存非公开文章
                    }
                })
        },
        Duration::from_secs(3600), // TTL 1 小时，自动抖动
    ).await?;

    match result {
        Some(json_str) => {
            // 检查是否为空值标记
            if is_null_value(&json_str) {
                return Ok(HttpResponse::NotFound().finish());
            }

            let passage: PassageResponse = serde_json::from_str(&json_str)?;
            Ok(HttpResponse::Ok()
                .insert_header(("Cache-Control", "public, max-age=300"))
                .insert_header(("X-Cache", "HIT"))
                .json(passage))
        }
        None => {
            // 数据不存在，返回 404
            Ok(HttpResponse::NotFound().finish())
        }
    }
}

/// 更新文章（带延迟双删）
pub async fn update_passage(
    id: web::Path<i32>,
    update_data: web::Json<UpdatePassage>,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    // 1. 更新数据库
    passage_repo.update(*id, &update_data).await?;

    let cache_key = format!("passage:{}", id);

    // 2. 第一次删除缓存（带重试）
    let _ = state.cache.manager().unwrap().delete(&cache_key).await;

    // 3. 删除相关缓存（如文章列表）
    let _ = state.cache.manager().unwrap().delete_pattern("passage:list:*").await;

    // 4. 延迟双删
    tokio::spawn({
        let cache = state.cache.manager().unwrap().clone();
        let cache_key = cache_key.clone();
        async move {
            tokio::time::sleep(Duration::from_millis(500)).await;
            let _ = cache.delete(&cache_key).await;
            let _ = cache.delete_pattern("passage:list:*").await;
        }
    });

    Ok(HttpResponse::Ok().json(serde_json::json!({"success": true})))
}
```

## 7. 配置建议

### RetryConfig
```rust
let retry_config = RetryConfig {
    max_retries: 3,              // 最多重试 3 次
    retry_interval: 1,           // 初始重试间隔 1 秒
    max_queue_size: 1000,        // 队列最大长度 1000
    backoff_multiplier: 2.0,     // 指数退避倍数 2.0
};
```

### TTL 抖动
```rust
// 基础 TTL 1 小时，10% 抖动
// 实际 TTL 范围：3240 ~ 3960 秒（54 ~ 66 分钟）
let ttl = Duration::from_secs(3600);
let jittered = jitter_ttl(ttl.as_secs(), 10);
```

## 8. 监控指标

建议监控以下指标：

1. **缓存命中率**：监控缓存命中情况
2. **重试队列长度**：`retry_cache.get_queue_size().await`
3. **缓存锁竞争**：记录锁获取失败次数
4. **空值缓存命中率**：监控穿透防护效果
5. **删除失败率**：监控删除操作的失败情况

## 9. 注意事项

1. **延迟双删的延迟时间**：建议 300-500ms，根据业务调整
2. **重试队列大小**：避免队列过大导致内存问题
3. **空值缓存 TTL**：建议较短（1-5分钟），避免占用过多空间
4. **TTL 抖动百分比**：建议 5-15%，避免失效时间过于集中
5. **分布式锁**：仅在真正需要时启用，避免性能开销

## 10. 性能对比

| 防护机制 | 性能影响 | 内存影响 | 推荐场景 |
|---------|---------|---------|---------|
| 缓存击穿防护 | 低 | 低 | 热点数据 |
| 缓存穿透防护 | 低 | 低 | 存在不存在的查询 |
| 缓存雪崩防护 | 无 | 无 | 所有场景 |
| 删除失败重试 | 低 | 中 | 关键数据一致性 |
| 延迟双删 | 低 | 无 | 写操作频繁 |