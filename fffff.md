存在的一致性问题：
   1. 无批量失效机制：修改标签/分类时，需要手动删除受影响的文章缓存，但代
      码中未发现标签/分类修改时的缓存清理逻辑
   2. 缓存穿透风险：如果数据在 TTL 期间被修改，缓存仍返回旧数据
   3. 分布式环境下：多个实例之间的缓存同步依赖 Redis
      的原子性，但当前没有使用发布/订阅或 key space notifications 机制

✦ 改进建议：
  如果需要强一致性，可以考虑：
   - 使用 Cache-Aside Pattern + Write-Through：写操作时同步更新缓存
   - 引入 Write-Behind：写操作时异步删除相关缓存
   - 使用 Redis 的 Pub/Sub 或 Key Space 
     Notifications：监听数据变化并自动失效缓存
   - 缩短 TTL 时间（如改为 30-60 秒）来平衡一致性与性
