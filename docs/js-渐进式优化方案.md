# RustBlog JS 渐进式优化方案

## 概述

本文档描述了 RustBlog 项目的 JavaScript 优化方案，通过分析、合并和压缩技术，解决原生 JS 项目中的全局变量冲突问题，并显著提升性能。

## 问题背景

### 原始问题

1. **全局变量污染**: 多个 JS 文件在全局作用域下共享变量名，极端压缩后变量名被缩短导致冲突
2. **HTTP 请求过多**: 每个页面加载多个 JS 文件，增加网络请求
3. **代码重复**: 共享功能在多个页面重复实现
4. **性能瓶颈**: 没有代码压缩，文件体积大，加载慢

### 项目现状

- **总页面数**: 9 个
- **唯一脚本数**: 28 个
- **共享脚本数**: 12 个
- **页面特定脚本数**: 16 个
- **HTTP 请求数**: 平均每页 6 个脚本

## 优化目标

1. **作用域隔离**: 为每个页面生成独立的、作用域封闭的 JS 文件
2. **减少请求**: 合并多个 JS 文件为单个文件
3. **极致压缩**: 使用 Terser 进行代码压缩和混淆
4. **提升性能**: 减少文件体积和加载时间

## 技术方案

### 1. JS 依赖分析

**工具**: `scripts/analyze-js-dependencies.cjs`

**功能**:
- 扫描所有 HTML 文件，提取 JS 引用
- 分析依赖关系，识别共享代码和页面特定代码
- 生成详细的分析报告

**使用方法**:
```bash
node scripts/analyze-js-dependencies.cjs
```

**输出**:
- `js-analysis/js-dependencies-report.json` - JSON 格式报告
- `js-analysis/js-dependencies-report.md` - Markdown 格式报告

### 2. 合并与压缩

**工具**: `scripts/merge-and-minify-js.cjs`

**功能**:
- 合并每个页面的所有 JS 文件
- 使用 IIFE（立即执行函数）封装，隔离作用域
- 使用 Terser 进行极致压缩
- 生成迁移指南和元数据

**使用方法**:
```bash
node scripts/merge-and-minify-js.cjs
```

**输出**:
- `static/dist/js-merged/*.min.js` - 压缩后的 JS 文件
- `static/dist/js-merged/*.meta.json` - 元数据文件
- `static/dist/js-merged/migration-guide.md` - 迁移指南
- `static/dist/js-merged/summary.json` - 总结报告

### 3. Terser 配置

**极致压缩配置**:
```javascript
{
  compress: {
    dead_code: true,
    drop_console: false,  // 保留 console，便于调试
    conditionals: true,
    evaluate: true,
    booleans: true,
    loops: true,
    unused: true,
    hoist_funs: true,
    join_vars: true,
    side_effects: true,
    reduce_vars: true,
    passes: 3,  // 多次压缩以获得更好的效果
    ecma: 2020
  },
  mangle: {
    toplevel: false,  // 不混淆顶层变量
    eval: true,
    properties: {
      regex: /^_/,  // 混淆以 _ 开头的属性
    }
  },
  format: {
    comments: false,  // 删除所有注释
    ecma: 2020,
    semicolons: true
  }
}
```

## 优化效果

### 整体统计

| 指标 | 原始 | 优化后 | 改善 |
|------|------|--------|------|
| 总大小 | 7,380 KB | 6,852 KB | 7.16% ↓ |
| HTTP 请求数 | 61 | 9 | 85.25% ↓ |
| 平均文件数/页 | 6.8 | 1 | 85.29% ↓ |

### 各页面优化效果

| 页面 | 原始大小 | 压缩后 | 压缩率 | 文件数 |
|------|----------|--------|--------|--------|
| passage.html | 3,293 KB | 3,096 KB | 5.99% | 14 → 1 |
| markdown-editor.html | 3,027 KB | 2,930 KB | 3.20% | 5 → 1 |
| index.html | 172 KB | 142 KB | 17.47% | 8 → 1 |
| friends.html | 127 KB | 109 KB | 13.96% | 3 → 1 |
| collect.html | 189 KB | 156 KB | 17.59% | 10 → 1 |
| about.html | 183 KB | 150 KB | 18.32% | 11 → 1 |
| admin/filemanager.html | 42 KB | 31 KB | 24.66% | 1 → 1 |
| admin/dyn-routing.html | 28 KB | 7 KB | 74.42% | 1 → 1 |
| admin/admin.html | 319 KB | 231 KB | 27.78% | 9 → 1 |

### 共享脚本包

| 包名 | 使用页面 | 包含脚本 | 原始大小 | 压缩后 | 压缩率 |
|------|----------|----------|----------|--------|--------|
| shared-6pages | 6 | keyboard-shortcuts.js, quick-actions-dynamic-routes.js | 68 KB | 51 KB | 24.53% |
| shared-5pages | 5 | modal-animations.js, music-player.js | 23 KB | 17 KB | 26.95% |
| shared-4pages | 4 | floating-text.js, ecc-encrypt.js, login.js, Sakara.js | 86 KB | 77 KB | 9.41% |
| shared-2pages | 2 | katex, mermaid, markdown-preview-modal | 3,046 KB | 2,968 KB | 2.56% |

## 迁移方案

### 方案一：完全迁移（推荐）

**步骤**:
1. 备份原始 HTML 文件
2. 根据迁移指南替换所有 script 标签
3. 测试每个页面的功能
4. 监控性能指标

**优点**:
- 最大化优化效果
- 彻底解决全局变量冲突
- 简化部署流程

**缺点**:
- 需要全面测试
- 一次性改动较大

### 方案二：渐进式迁移

**步骤**:
1. 先迁移高流量页面（如 index.html, passage.html）
2. 观察性能和功能
3. 逐步迁移其他页面
4. 最后迁移管理后台

**优点**:
- 风险可控
- 可以逐步验证效果
- 出现问题容易回滚

**缺点**:
- 迁移周期长
- 需要多次部署

### 方案三：混合方案

**步骤**:
1. 提取高频共享脚本为公共包
2. 页面特定脚本合并
3. 按需加载共享包

**优点**:
- 平衡性能和灵活性
- 可以利用浏览器缓存
- 适合大型项目

**缺点**:
- 实现复杂
- 需要维护多个包

## 具体实施步骤

### 1. 准备阶段

```bash
# 1. 运行分析脚本
node scripts/analyze-js-dependencies.cjs

# 2. 查看分析报告
cat js-analysis/js-dependencies-report.md

# 3. 运行合并压缩脚本
node scripts/merge-and-minify-js.cjs

# 4. 查看生成的文件
ls -lah static/dist/js-merged/
```

### 2. 测试阶段

```bash
# 启动开发服务器
cargo run

# 在浏览器中测试各个页面
# 检查控制台是否有错误
# 验证所有功能正常
```

### 3. 部署阶段

```bash
# 1. 备份原始文件
cp -r templates/ templates-backup/

# 2. 根据迁移指南修改 HTML 文件
# 参考 static/dist/js-merged/migration-guide.md

# 3. 构建生产版本
cargo build --release

# 4. 部署到服务器
./script/deploy.sh
```

## 注意事项

### 1. 调试

- 压缩后的代码难以调试
- 可以使用元数据文件查看源文件映射
- 保留原始文件用于问题排查

### 2. 版本控制

- 为生成的 JS 文件添加版本号
- 使用缓存控制策略
- 便于问题追踪和回滚

### 3. 性能监控

- 监控页面加载时间
- 监控 JS 执行时间
- 监控错误率

### 4. 回滚策略

- 保留原始 HTML 文件
- 保留原始 JS 文件
- 准备快速回滚方案

## 未来优化方向

### 1. 代码分割

- 按路由分割代码
- 按功能分割代码
- 动态导入

### 2. 懒加载

- 延迟加载非关键代码
- 按需加载模块
- 预加载关键资源

### 3. 缓存优化

- 使用 Service Worker
- 实施缓存策略
- 优化缓存失效

### 4. 模块化

- 逐步迁移到 ES 模块
- 使用现代构建工具
- 改善代码组织

## 工具和依赖

### 当前工具

- **Node.js**: 脚本运行环境
- **Terser**: JS 压缩工具
- **glob**: 文件匹配工具

### 未来考虑

- **esbuild**: 更快的构建工具
- **Rollup**: 模块打包工具
- **Webpack**: 功能全面的构建工具

## 总结

通过实施本优化方案，RustBlog 项目的 JavaScript 性能得到了显著提升：

1. **解决了全局变量冲突问题**: 使用 IIFE 封装，每个页面代码完全隔离
2. **减少了 HTTP 请求**: 从平均 6.8 个减少到 1 个，减少 85.29%
3. **提升了加载性能**: 总体压缩率 7.16%，部分页面高达 74.42%
4. **改善了代码质量**: 统一的构建流程，易于维护

该方案采用渐进式优化策略，可以根据实际情况选择不同的迁移方案，确保平稳过渡。

## 附录

### A. 命令参考

```bash
# 分析依赖
node scripts/analyze-js-dependencies.cjs

# 合并压缩
node scripts/merge-and-minify-js.cjs

# 查看报告
cat js-analysis/js-dependencies-report.md
cat static/dist/js-merged/migration-guide.md
cat static/dist/js-merged/summary.json
```

### B. 文件结构

```
rustblog/
├── scripts/
│   ├── analyze-js-dependencies.cjs    # 依赖分析脚本
│   └── merge-and-minify-js.cjs        # 合并压缩脚本
├── js-analysis/
│   ├── js-dependencies-report.json    # 分析报告（JSON）
│   └── js-dependencies-report.md      # 分析报告（Markdown）
├── static/dist/js-merged/
│   ├── *.min.js                       # 压缩后的 JS 文件
│   ├── *.meta.json                    # 元数据文件
│   ├── migration-guide.md             # 迁移指南
│   └── summary.json                   # 总结报告
└── templates/
    └── *.html                         # 原始 HTML 文件
```

### C. 联系方式

如有问题或建议，请联系开发团队。

---

**文档版本**: 1.0
**最后更新**: 2026-03-28
**维护者**: RustBlog 开发团队