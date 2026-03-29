# JS 依赖分析报告

生成时间: 2026-03-28T07:44:29.280Z

## 摘要

- 总页面数: 9
- 唯一脚本数: 28
- 共享脚本数: 12
- 页面特定脚本数: 16

## 共享脚本

| 脚本路径 | 使用页面数 | 使用页面 | 类型 | defer | async |
|---------|----------|---------|------|-------|-------|
| `/js/npm/katex-0.16.27/dist/katex.min.js` | 2 | passage.html, markdown-editor.html | classic | true | false |
| `/js/npm/katex-0.16.27/dist/contrib/auto-render.min.js` | 2 | passage.html, markdown-editor.html | classic | true | false |
| `/js/npm/mermaid@11.12.2/dist/mermaid.min.js` | 2 | passage.html, markdown-editor.html | classic | true | false |
| `/js/modal-animations.js` | 5 | passage.html, index.html, collect.html, about.html, admin/admin.html | classic | true | false |
| `/js/markdown-preview-modal.js?v=2` | 2 | passage.html, collect.html | classic | true | false |
| `/js/keyboard-shortcuts.js` | 6 | passage.html, index.html, friends.html, collect.html, about.html, admin/admin.html | classic | true | false |
| `/js/music-player.js` | 5 | passage.html, index.html, collect.html, about.html, admin/admin.html | classic | true | false |
| `/js/quick-actions-dynamic-routes.js` | 6 | passage.html, markdown-editor.html, index.html, friends.html, collect.html, about.html | classic | true | false |
| `/js/floating-text.js` | 4 | passage.html, index.html, collect.html, about.html | classic | true | false |
| `/js/ecc-encrypt.js` | 4 | index.html, collect.html, about.html, admin/admin.html | classic | true | false |
| `/js/login.js` | 4 | index.html, collect.html, about.html, admin/admin.html | classic | true | false |
| `/js/animations/Sakara.js` | 4 | index.html, friends.html, collect.html, about.html | classic | true | false |

## 页面依赖详情

### passage.html

- 总脚本数: 14
- 共享脚本数: 9
- 页面特定脚本数: 5

**共享脚本:**
- `/js/npm/katex-0.16.27/dist/katex.min.js`
- `/js/npm/katex-0.16.27/dist/contrib/auto-render.min.js`
- `/js/npm/mermaid@11.12.2/dist/mermaid.min.js`
- `/js/modal-animations.js`
- `/js/markdown-preview-modal.js?v=2`
- `/js/keyboard-shortcuts.js`
- `/js/music-player.js`
- `/js/quick-actions-dynamic-routes.js`
- `/js/floating-text.js`

**页面特定脚本:**
- `/js/highlight.min.js`
- `/js/passage-shortcuts.js`
- `/js/passage-focus-mode.js`
- `/js/article-summary.js`
- `/js/virtual-scroll.js`

**所有脚本:**
- `/js/npm/katex-0.16.27/dist/katex.min.js` (type: classic, defer: true, async: false)
- `/js/npm/katex-0.16.27/dist/contrib/auto-render.min.js` (type: classic, defer: true, async: false)
- `/js/npm/mermaid@11.12.2/dist/mermaid.min.js` (type: classic, defer: true, async: false)
- `/js/highlight.min.js` (type: classic, defer: true, async: false)
- `/js/modal-animations.js` (type: classic, defer: true, async: false)
- `/js/markdown-preview-modal.js?v=2` (type: classic, defer: true, async: false)
- `/js/keyboard-shortcuts.js` (type: classic, defer: true, async: false)
- `/js/passage-shortcuts.js` (type: classic, defer: true, async: false)
- `/js/passage-focus-mode.js` (type: classic, defer: true, async: false)
- `/js/article-summary.js` (type: classic, defer: true, async: false)
- `/js/music-player.js` (type: classic, defer: true, async: false)
- `/js/virtual-scroll.js` (type: classic, defer: true, async: false)
- `/js/quick-actions-dynamic-routes.js` (type: classic, defer: true, async: false)
- `/js/floating-text.js` (type: classic, defer: true, async: false)

### markdown-editor.html

- 总脚本数: 5
- 共享脚本数: 4
- 页面特定脚本数: 1

**共享脚本:**
- `/js/npm/katex-0.16.27/dist/katex.min.js`
- `/js/npm/katex-0.16.27/dist/contrib/auto-render.min.js`
- `/js/npm/mermaid@11.12.2/dist/mermaid.min.js`
- `/js/quick-actions-dynamic-routes.js`

**页面特定脚本:**
- `/js/npm/marked@14.1.4/marked.min.js`

**所有脚本:**
- `/js/npm/marked@14.1.4/marked.min.js` (type: classic, defer: false, async: false)
- `/js/npm/katex-0.16.27/dist/katex.min.js` (type: classic, defer: false, async: false)
- `/js/npm/katex-0.16.27/dist/contrib/auto-render.min.js` (type: classic, defer: false, async: false)
- `/js/npm/mermaid@11.12.2/dist/mermaid.min.js` (type: classic, defer: true, async: false)
- `/js/quick-actions-dynamic-routes.js` (type: classic, defer: true, async: false)

### index.html

- 总脚本数: 8
- 共享脚本数: 8
- 页面特定脚本数: 0

**共享脚本:**
- `/js/ecc-encrypt.js`
- `/js/login.js`
- `/js/modal-animations.js`
- `/js/keyboard-shortcuts.js`
- `/js/music-player.js`
- `/js/quick-actions-dynamic-routes.js`
- `/js/floating-text.js`
- `/js/animations/Sakara.js`

**所有脚本:**
- `/js/ecc-encrypt.js` (type: classic, defer: true, async: false)
- `/js/login.js` (type: classic, defer: true, async: false)
- `/js/modal-animations.js` (type: classic, defer: true, async: false)
- `/js/keyboard-shortcuts.js` (type: classic, defer: true, async: false)
- `/js/music-player.js` (type: classic, defer: true, async: false)
- `/js/quick-actions-dynamic-routes.js` (type: classic, defer: true, async: false)
- `/js/floating-text.js` (type: classic, defer: true, async: false)
- `/js/animations/Sakara.js` (type: classic, defer: true, async: false)

### friends.html

- 总脚本数: 3
- 共享脚本数: 3
- 页面特定脚本数: 0

**共享脚本:**
- `/js/keyboard-shortcuts.js`
- `/js/quick-actions-dynamic-routes.js`
- `/js/animations/Sakara.js`

**所有脚本:**
- `/js/keyboard-shortcuts.js` (type: classic, defer: true, async: false)
- `/js/quick-actions-dynamic-routes.js` (type: classic, defer: true, async: false)
- `/js/animations/Sakara.js` (type: classic, defer: true, async: false)

### collect.html

- 总脚本数: 10
- 共享脚本数: 9
- 页面特定脚本数: 1

**共享脚本:**
- `/js/ecc-encrypt.js`
- `/js/login.js`
- `/js/modal-animations.js`
- `/js/markdown-preview-modal.js?v=2`
- `/js/keyboard-shortcuts.js`
- `/js/music-player.js`
- `/js/quick-actions-dynamic-routes.js`
- `/js/floating-text.js`
- `/js/animations/Sakara.js`

**页面特定脚本:**
- `/js/collect-focus-mode.js`

**所有脚本:**
- `/js/ecc-encrypt.js` (type: classic, defer: true, async: false)
- `/js/login.js` (type: classic, defer: true, async: false)
- `/js/modal-animations.js` (type: classic, defer: true, async: false)
- `/js/markdown-preview-modal.js?v=2` (type: classic, defer: true, async: false)
- `/js/keyboard-shortcuts.js` (type: classic, defer: true, async: false)
- `/js/music-player.js` (type: classic, defer: true, async: false)
- `/js/collect-focus-mode.js` (type: classic, defer: true, async: false)
- `/js/quick-actions-dynamic-routes.js` (type: classic, defer: true, async: false)
- `/js/floating-text.js` (type: classic, defer: true, async: false)
- `/js/animations/Sakara.js` (type: classic, defer: true, async: false)

### about.html

- 总脚本数: 11
- 共享脚本数: 8
- 页面特定脚本数: 3

**共享脚本:**
- `/js/ecc-encrypt.js`
- `/js/login.js`
- `/js/modal-animations.js`
- `/js/keyboard-shortcuts.js`
- `/js/music-player.js`
- `/js/quick-actions-dynamic-routes.js`
- `/js/floating-text.js`
- `/js/animations/Sakara.js`

**页面特定脚本:**
- `/js/about-inline-1.js`
- `/js/about-inline-2.js`
- `/js/about-focus-mode.js`

**所有脚本:**
- `/js/about-inline-1.js` (type: module, defer: false, async: false)
- `/js/about-inline-2.js` (type: module, defer: false, async: false)
- `/js/ecc-encrypt.js` (type: classic, defer: true, async: false)
- `/js/login.js` (type: classic, defer: true, async: false)
- `/js/modal-animations.js` (type: classic, defer: true, async: false)
- `/js/about-focus-mode.js` (type: classic, defer: false, async: false)
- `/js/keyboard-shortcuts.js` (type: classic, defer: false, async: false)
- `/js/music-player.js` (type: classic, defer: true, async: false)
- `/js/quick-actions-dynamic-routes.js` (type: classic, defer: true, async: false)
- `/js/floating-text.js` (type: classic, defer: true, async: false)
- `/js/animations/Sakara.js` (type: classic, defer: true, async: false)

### admin/filemanager.html

- 总脚本数: 1
- 共享脚本数: 0
- 页面特定脚本数: 1

**页面特定脚本:**
- `/js/filemanager.js?v=4`

**所有脚本:**
- `/js/filemanager.js?v=4` (type: classic, defer: true, async: false)

### admin/dyn-routing.html

- 总脚本数: 1
- 共享脚本数: 0
- 页面特定脚本数: 1

**页面特定脚本:**
- `/js/dyn-routing.js`

**所有脚本:**
- `/js/dyn-routing.js` (type: classic, defer: false, async: false)

### admin/admin.html

- 总脚本数: 9
- 共享脚本数: 5
- 页面特定脚本数: 4

**共享脚本:**
- `/js/ecc-encrypt.js`
- `/js/login.js`
- `/js/modal-animations.js`
- `/js/keyboard-shortcuts.js`
- `/js/music-player.js`

**页面特定脚本:**
- `/js/admin-inline-1.js`
- `/js/admin-inline-2.js?v=3`
- `/js/markdown-preview-modal.js?v=3`
- `/js/admin-inline-4.js`

**所有脚本:**
- `/js/admin-inline-1.js` (type: module, defer: false, async: false)
- `/js/admin-inline-2.js?v=3` (type: module, defer: false, async: false)
- `/js/ecc-encrypt.js` (type: classic, defer: true, async: false)
- `/js/login.js` (type: classic, defer: true, async: false)
- `/js/modal-animations.js` (type: classic, defer: true, async: false)
- `/js/keyboard-shortcuts.js` (type: classic, defer: true, async: false)
- `/js/markdown-preview-modal.js?v=3` (type: classic, defer: true, async: false)
- `/js/admin-inline-4.js` (type: module, defer: false, async: false)
- `/js/music-player.js` (type: classic, defer: true, async: false)

## 优化建议

### 1. 提取共享脚本为公共模块 (high)

以下脚本被多个页面共享，建议提取为公共模块

**涉及的脚本:**
- `/js/keyboard-shortcuts.js` - 被 6 个页面使用
- `/js/quick-actions-dynamic-routes.js` - 被 6 个页面使用
- `/js/modal-animations.js` - 被 5 个页面使用
- `/js/music-player.js` - 被 5 个页面使用
- `/js/floating-text.js` - 被 4 个页面使用

### 2. 合并页面特定脚本 (medium)

以下页面引用了多个脚本，建议合并为单个文件以减少 HTTP 请求

**涉及的页面:**
- `passage.html` - 14 个脚本
- `markdown-editor.html` - 5 个脚本
- `index.html` - 8 个脚本
- `collect.html` - 10 个脚本
- `about.html` - 11 个脚本
- `admin/admin.html` - 9 个脚本

### 3. 考虑使用 defer/async 优化加载 (low)

以下脚本没有使用 defer 或 async，考虑添加这些属性以优化页面加载性能

**涉及的脚本:**
- `/js/npm/marked@14.1.4/marked.min.js`
- `/js/about-inline-1.js`
- `/js/about-inline-2.js`
- `/js/about-focus-mode.js`
- `/js/dyn-routing.js`
- `/js/admin-inline-1.js`
- `/js/admin-inline-2.js?v=3`
- `/js/admin-inline-4.js`

