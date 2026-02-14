   - 🔴 高优先级优化

       1. ✅N+1 查询问题 (src/handlers/api_handlers/analytics.rs:257)
         问题: 先查询文章信息再查询统计数据，存在潜在性能问题
         优化: 使用 JOIN 查询一次性获取所有数据，减少数据库往返

       2. ✅代码重复 - 缓存失效逻辑
         问题:
         多个文件中重复的缓存失效代码（passage.rs、categories.rs、tags.rs）
         优化: 创建统一的缓存失效工具函数

       3. ✅缺少测试
         问题: 代码库几乎没有测试文件
         优化: 添加单元测试和集成测试，确保代码质量

       4. ✅API 文档缺失
         问题: API 端点缺少详细文档说明
         优化: 添加 OpenAPI/Swagger 文档和注释说明

       🟡 中优先级优化

       5. ✅COUNT(*) 查询频繁 (src/db/repositories.rs:507)
         问题: 每次分页请求都执行 COUNT 查询，大数据量时影响性能
         优化: 使用计数器表或缓存，通过触发器维护计数

       6. ✅错误处理改进
         问题: 过多使用 unwrap() 和 expect()，如 jwt.rs:165、passage.rs:1055
         优化: 使用 ? 操作符，创建自定义错误类型

       7. ✅Handler 文件过大 (passage.rs 1647 行)
         问题: 单个文件包含太多逻辑，难以维护
         优化: 拆分为 crud.rs、validation.rs、markdown.rs 等模块

       8. ✅日志记录不一致
         问题: 混用 eprintln! 和缺少日志
         优化: 统一使用结构化日志（tracing crate）

       9. ✅限流器内存管理 (src/middleware/ratelimit.rs:95)
         问题: DashMap 存储所有 IP 记录，大量不同 IP 时占用过多内存
         优化: 使用 LRU 缓存替代

       10. ✅Repository Trait 不完善
         问题: Trait 定义不完整，实际使用时依赖具体类型
         优化: 完善接口抽象，引入 Service 层解耦

       🟢 低优先级优化

       11. ✅缓存键设计优化
         问题: 缓存键格式不统一，缺少版本控制
         优化: 使用结构化缓存键，添加版本号支持缓存失效

       12. 类型安全改进 (src/db/models.rs)
         问题: status、visibility 使用字符串而非枚举
         优化: 使用 PassageStatus、PassageVisibility 枚举

       13. 配置验证不足
         问题: 配置解析后没有验证
         优化: 添加配置验证逻辑

       14. 依赖注入改进
         问题: Handler 中直接创建 repository 实例
         优化: 使用依赖注入容器

       15. 批量处理器配置 (src/view_batch.rs:33)
         问题: 批量大小固定，无法自适应负载
         优化: 实现自适应批量大小调整

       预期效果

     ✦ 实施这些优化后预计可获得：
        - 性能提升: 数据库查询速度提升 30-50%
        - 内存使用: 限流器内存占用减少 40-60%
        - 代码质量: 代码重复率降低，可维护性提升
        - 开发效率: 完善的文档和测试提升开发速度
        - 系统稳定性: 更好的错误处理和日志记录
