# RustBlog 全局模板设置项开发方案

## 一、项目概述

本文档定义了 RustBlog 博客系统的全局模板设置项的完整开发方案，旨在提供灵活、可定制的博客外观和功能配置能力。

## 二、现有设置项分析

### 2.1 当前已实现的设置项

基于 `src/templates/mod.rs` 中的 `TemplateSettings` 结构体，当前已实现的设置项包括：

#### 基础信息
- `name`: 博客名称
- `greting`: 欢迎语
- `year`: 年份
- `foodes`: 博客描述

#### 外观设置
- `background_image`: 背景图片（桌面端）
- `mobile_background_image`: 背景图片（移动端）
- `background_color`: 背景颜色
- `background_size`: 背景尺寸模式
- `background_position`: 背景位置
- `background_repeat`: 背景重复模式
- `background_attachment`: 背景固定方式
- `global_opacity`: 全局透明度
- `blur_amount`: 模糊程度
- `saturate_amount`: 饱和度
- `floating_text_enabled`: 是否启用浮动文字

#### 导航栏设置
- `navbar_glass_color`: 导航栏玻璃效果颜色
- `navbar_text_color`: 导航栏文字颜色

#### 卡片设置
- `card_glass_color`: 卡片玻璃效果颜色

#### 页脚设置
- `footer_glass_color`: 页脚玻璃效果颜色

#### 文章设置
- `article_title`: 是否显示文章标题
- `article_title_prefix`: 文章标题前缀

#### 交互提示
- `switch_notice`: 是否显示切换提示
- `switch_notice_text`: 切换提示文字
- `external_link_warning`: 是否显示外部链接警告
- `external_link_whitelist`: 外部链接白名单
- `external_link_warning_text`: 外部链接警告文字

#### Live2D 设置
- `live2d_enabled`: 是否启用 Live2D
- `live2d_show_on_index`: 首页显示
- `live2d_show_on_passage`: 文章页显示
- `live2d_show_on_collect`: 收藏页显示
- `live2d_show_on_about`: 关于页显示
- `live2d_show_on_admin`: 管理页显示
- `live2d_model_id`: 模型ID
- `live2d_model_path`: 模型路径
- `live2d_cdn_path`: CDN路径
- `live2d_position`: 位置
- `live2d_width`: 宽度
- `live2d_height`: 高度

#### 赞助设置
- `sponsor_enabled`: 是否启用赞助
- `sponsor_title`: 赞助标题
- `sponsor_image`: 赞助图片
- `sponsor_description`: 赞助描述
- `sponsor_button_text`: 赞助按钮文字

#### 全局设置
- `global_avatar`: 全局头像

#### 附件设置
- `attachment_default_visibility`: 默认可见性
- `attachment_max_size`: 最大文件大小
- `attachment_allowed_types`: 允许的文件类型

#### 备案设置
- `beian_enabled`: 是否启用备案信息
- `icp_number`: ICP备案号
- `police_record_code`: 公安备案号
- `police_record_content`: 公安备案内容

## 三、新增设置项方案

### 3.1 主题系统

#### 3.1.1 主题配色方案
```rust
pub struct ThemeSettings {
    // 主题模式
    pub theme_mode: String,  // "light" | "dark" | "auto" | "system"
    
    // 预设主题
    pub preset_theme: String,  // "default" | "ocean" | "forest" | "sunset" | "midnight"
    
    // 自定义配色
    pub primary_color: String,      // 主色调
    pub secondary_color: String,    // 次要色
    pub accent_color: String,       // 强调色
    pub text_color: String,         // 文字颜色
    pub background_color: String,   // 背景色
    pub card_background: String,    // 卡片背景
    pub border_color: String,       // 边框颜色
    
    // 渐变设置
    pub gradient_enabled: bool,
    pub gradient_start: String,
    pub gradient_end: String,
    pub gradient_direction: String,  // "to-right" | "to-bottom" | "to-br" | "diagonal"
    
    // 圆角设置
    pub border_radius_sm: String,   // 小圆角
    pub border_radius_md: String,   // 中圆角
    pub border_radius_lg: String,   // 大圆角
    pub border_radius_full: String, // 完全圆角
    
    // 阴影设置
    pub shadow_enabled: bool,
    pub shadow_color: String,
    pub shadow_light: bool,         // 是否使用浅色阴影
    pub shadow_size: String,        // "sm" | "md" | "lg" | "xl"
}
```

#### 3.1.2 字体系统
```rust
pub struct TypographySettings {
    // 主字体
    pub font_family: String,         // 字体系列
    pub font_size_base: String,      // 基础字号
    pub font_weight: String,         // 字重
    pub line_height: String,         // 行高
    
    // 标题字体
    pub heading_font: String,
    pub heading_weight: String,
    pub heading_line_height: String,
    
    // 代码字体
    pub code_font: String,
    pub code_font_size: String,
    
    // 字体大小系统
    pub font_xs: String,
    pub font_sm: String,
    pub font_md: String,
    pub font_lg: String,
    pub font_xl: String,
    pub font_2xl: String,
    pub font_3xl: String,
    
    // 行高系统
    pub leading_tight: String,
    pub leading_normal: String,
    pub leading_relaxed: String,
    pub leading_loose: String,
}
```

### 3.2 布局系统

#### 3.2.1 整体布局
```rust
pub struct LayoutSettings {
    // 容器设置
    pub container_width: String,    // "sm" | "md" | "lg" | "xl" | "full"
    pub container_padding: String,
    pub max_content_width: String,
    
    // 间距系统
    pub spacing_xs: String,
    pub spacing_sm: String,
    pub spacing_md: String,
    pub spacing_lg: String,
    pub spacing_xl: String,
    pub spacing_2xl: String,
    
    // 边距系统
    pub margin_xs: String,
    pub margin_sm: String,
    pub margin_md: String,
    pub margin_lg: String,
    pub margin_xl: String,
    
    // 导航栏布局
    pub navbar_position: String,    // "fixed" | "sticky" | "static"
    pub navbar_height: String,
    pub navbar_layout: String,      // "left" | "center" | "right" | "split"
    pub navbar_transparent: bool,
    pub navbar_show_logo: bool,
    pub navbar_show_search: bool,
    
    // 侧边栏
    pub sidebar_enabled: bool,
    pub sidebar_position: String,   // "left" | "right"
    pub sidebar_width: String,
    pub sidebar_collapsible: bool,
    
    // 页脚布局
    pub footer_position: String,    // "static" | "sticky" | "fixed"
    pub footer_style: String,       // "simple" | "rich" | "minimal"
}
```

#### 3.2.2 响应式断点
```rust
pub struct ResponsiveSettings {
    // 移动端优先
    pub mobile_first: bool,
    
    // 断点设置
    pub breakpoint_sm: String,      // 默认 640px
    pub breakpoint_md: String,      // 默认 768px
    pub breakpoint_lg: String,      // 默认 1024px
    pub breakpoint_xl: String,      // 默认 1280px
    pub breakpoint_2xl: String,     // 默认 1536px
    
    // 隐藏元素
    pub hide_on_mobile: Vec<String>,
    pub hide_on_tablet: Vec<String>,
    pub hide_on_desktop: Vec<String>,
    
    // 触摸优化
    pub touch_target_size: String,  // 最小触摸目标尺寸
    pub swipe_enabled: bool,
    pub pinch_zoom: bool,
}
```

### 3.3 动画与交互

#### 3.3.1 动画设置
```rust
pub struct AnimationSettings {
    // 全局动画
    pub animations_enabled: bool,
    pub animation_duration: String,  // "fast" | "normal" | "slow"
    pub animation_easing: String,   // "ease" | "ease-in" | "ease-out" | "ease-in-out"
    
    // 过渡效果
    pub transition_enabled: bool,
    pub transition_duration: String,
    
    // 页面加载动画
    pub page_load_animation: String, // "fade" | "slide" | "zoom" | "none"
    pub page_load_duration: String,
    
    // 元素进入动画
    pub scroll_animation: bool,     // 滚动时触发动画
    pub scroll_animation_type: String, // "fade-up" | "fade-in" | "slide-up"
    pub scroll_animation_delay: String,
    
    // 悬停效果
    pub hover_enabled: bool,
    pub hover_scale: bool,
    pub hover_lift: bool,
    pub hover_glow: bool,
    
    // 点击效果
    pub click_ripple: bool,         // 点击波纹效果
    pub click_scale: bool,          // 点击缩放效果
}
```

#### 3.3.2 反馈系统
```rust
pub struct FeedbackSettings {
    // 加载状态
    pub loading_indicator: String,  // "spinner" | "skeleton" | "dots"
    pub loading_text: String,
    
    // 通知系统
    pub toast_position: String,     // "top-right" | "top-left" | "bottom-right" | "bottom-left"
    pub toast_duration: String,
    pub toast_animation: String,
    
    // 错误处理
    pub error_boundary_enabled: bool,
    pub error_page_style: String,   // "minimal" | "illustrated" | "interactive"
    
    // 进度指示
    pub progress_bar_enabled: bool,
    pub progress_bar_position: String,
    
    // 成功状态
    pub success_animation: bool,
}
```

### 3.4 内容展示

#### 3.4.1 文章列表
```rust
pub struct ArticleListSettings {
    // 布局模式
    pub layout_mode: String,        // "grid" | "list" | "masonry" | "card"
    pub columns: String,            // 1 | 2 | 3 | 4 | auto
    
    // 卡片样式
    pub card_style: String,         // "default" | "minimal" | "detailed" | "modern"
    pub card_aspect_ratio: String,  // "auto" | "square" | "portrait" | "landscape"
    
    // 内容显示
    pub show_excerpt: bool,
    pub excerpt_length: i32,
    pub show_date: bool,
    pub show_author: bool,
    pub show_tags: bool,
    pub show_category: bool,
    pub show_views: bool,
    pub show_comments: bool,
    
    // 排序方式
    pub sort_by: String,            // "date" | "views" | "comments" | "custom"
    pub sort_order: String,         // "asc" | "desc"
    
    // 分页设置
    pub pagination_style: String,   // "simple" | "numbered" | "load-more" | "infinite"
    pub per_page: i32,
}
```

#### 3.4.2 文章详情
```rust
pub struct ArticleDetailSettings {
    // 文章布局
    pub content_width: String,      // "narrow" | "normal" | "wide" | "full"
    pub content_alignment: String,  // "left" | "center" | "justify"
    
    // 排版设置
    pub line_height: String,
    pub paragraph_spacing: String,
    pub heading_style: String,      // "default" | "numbered" | "underline"
    
    // 元信息显示
    pub_meta_position: String,      // "top" | "bottom" | "both"
    pub show_reading_time: bool,
    pub show_word_count: bool,
    
    // 目录设置
    pub_toc_enabled: bool,
    pub_toc_position: String,       // "left" | "right" | "floating"
    pub_toc_depth: i32,             // 目录层级深度
    
    // 代码块
    pub_code_theme: String,         // "light" | "dark" | "nord" | "dracula" | "monokai"
    pub code_line_numbers: bool,
    pub code_copy_button: bool,
    
    // 图片处理
    pub image_lightbox: bool,        // 图片灯箱效果
    pub_image_caption: bool,
    pub image_zoom: bool,
    
    // 引用样式
    pub blockquote_style: String,   // "simple" | "bordered" | "modern"
}
```

#### 3.4.3 评论系统
```rust
pub struct CommentSettings {
    // 评论功能
    pub comments_enabled: bool,
    pub guest_comments: bool,       // 是否允许游客评论
    pub_comment_require_approval: bool,
    
    // 评论排序
    pub comment_order: String,      // "newest" | "oldest" | "popular"
    
    // 评论样式
    pub_comment_style: String,      // "nested" | "flat" | "threaded"
    pub_avatar_enabled: bool,
    pub show_timestamp: bool,
    
    // 交互设置
    pub allow_replies: bool,
    pub allow_likes: bool,
    pub max_depth: i32,             // 嵌套深度
    
    // 表情支持
    pub_emoji_enabled: bool,
    pub_emoji_picker: bool,
    
    // Markdown 支持
    pub_markdown_enabled: bool,
    pub_markdown_preview: bool,
}
```

### 3.5 SEO 优化

#### 3.5.1 SEO 设置
```rust
pub struct SEOSettings {
    // 基本 SEO
    pub site_title: String,
    pub site_description: String,
    pub site_keywords: String,
    
    // Open Graph
    pub og_image: String,
    pub og_type: String,
    
    // Twitter Cards
    pub twitter_card: String,       // "summary" | "summary_large_image"
    pub_twitter_image: String,
    
    // 结构化数据
    pub_structured_data_enabled: bool,
    pub_json_ld_type: String,       // "Blog" | "Article" | "Organization"
    
    // Sitemap
    pub_sitemap_enabled: bool,
    pub_sitemap_frequency: String,  // "always" | "hourly" | "daily" | "weekly" | "monthly" | "yearly"
    
    // Robots
    pub_robots_txt_enabled: bool,
    pub_robots_disallow: String,
    
    // Canonical URLs
    pub_canonical_enabled: bool,
    
    // Meta Tags
    pub_meta_author: bool,
    pub_meta_generator: bool,
    pub_meta_theme_color: bool,
}
```

### 3.6 性能优化

#### 3.6.1 性能设置
```rust
pub struct PerformanceSettings {
    // 图片优化
    pub image_lazy_loading: bool,
    pub_image_format: String,       // "webp" | "avif" | "auto"
    pub_image_quality: i32,         // 1-100
    
    // 字体加载
    pub_font_display: String,       // "swap" | "block" | "fallback" | "optional"
    pub_font_preload: bool,
    
    // 代码分割
    pub_code_splitting: bool,
    pub_dynamic_imports: bool,
    
    // 缓存策略
    pub_cache_strategy: String,     // "aggressive" | "balanced" | "minimal"
    pub_cache_duration: String,
    
    // 压缩
    pub_minify_html: bool,
    pub_minify_css: bool,
    pub_minify_js: bool,
    
    // CDN 设置
    pub_cdn_enabled: bool,
    pub_cdn_url: String,
    
    // 预加载
    pub_preload_enabled: bool,
    pub_preconnect_domains: Vec<String>,
}
```

### 3.7 社交媒体

#### 3.7.1 社交设置
```rust
pub struct SocialSettings {
    // 社交链接
    pub_social_github: String,
    pub_social_twitter: String,
    pub_social_facebook: String,
    pub_social_instagram: String,
    pub_social_linkedin: String,
    pub_social_youtube: String,
    pub_social_bilibili: String,
    pub_social_weibo: String,
    pub_social_zhihu: String,
    pub_social_email: String,
    
    // 分享按钮
    pub_share_enabled: bool,
    pub_share_position: String,     // "top" | "bottom" | "floating"
    pub_share_platforms: Vec<String>,
    
    // 关注按钮
    pub_follow_enabled: bool,
    pub_follow_position: String,
    pub follow_style: String,       // "icon" | "text" | "badge"
    
    // RSS Feed
    pub_rss_enabled: bool,
    pub_rss_full_content: bool,
}
```

### 3.8 多语言支持

#### 3.8.1 国际化设置
```rust
pub struct I18nSettings {
    // 语言设置
    pub_default_language: String,   // "zh-CN" | "en-US" | "ja-JP" | "ko-KR"
    pub_available_languages: Vec<String>,
    pub_auto_detect: bool,
    
    // 语言切换器
    pub_language_switcher_enabled: bool,
    pub_language_switcher_style: String, // "dropdown" | "flags" | "text"
    pub_language_switcher_position: String,
    
    // 日期格式
    pub_date_format: String,
    pub_time_format: String,
    pub_timezone: String,
    
    // RTL 支持
    pub_rtl_support: bool,
}
```

### 3.9 无障碍访问

#### 3.9.1 无障碍设置
```rust
pub struct AccessibilitySettings {
    // 屏幕阅读器
    pub_sr_text_enabled: bool,
    pub_skip_links_enabled: bool,
    
    // 键盘导航
    pub_keyboard_navigation: bool,
    pub_focus_visible: bool,
    pub_shortcuts_enabled: bool,
    
    // 对比度
    pub_high_contrast_mode: bool,
    pub_color_blind_mode: String,  // "protanopia" | "deuteranopia" | "tritanopia" | "none"
    
    // 字体大小
    pub_font_scaling: bool,
    pub_max_font_scale: String,
    
    // 动画控制
    pub_prefers_reduced_motion: bool,
    pub_pause_animations: bool,
}
```

### 3.10 安全与隐私

#### 3.10.1 安全设置
```rust
pub struct SecuritySettings {
    // 内容安全策略
    pub_csp_enabled: bool,
    pub_csp_mode: String,          // "strict" | "balanced" | "relaxed"
    
    // Cookie 设置
    pub_cookie_secure: bool,
    pub_cookie_same_site: String,  // "Strict" | "Lax" | "None"
    
    // HTTPS
    pub_force_https: bool,
    pub_hsts_enabled: bool,
    
    // 隐私
    pub_privacy_policy_enabled: bool,
    pub_cookie_consent_enabled: bool,
    pub_analytics_enabled: bool,
    
    // XSS 防护
    pub_xss_protection_enabled: bool,
    pub_content_type_nosniff: bool,
}
```

## 四、技术实现方案

### 4.1 数据库结构设计

#### 4.1.1 设置表
```sql
CREATE TABLE settings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    category VARCHAR(50) NOT NULL,      -- 设置分类
    key VARCHAR(100) NOT NULL,          -- 设置键名
    value TEXT,                          -- 设置值（JSON字符串）
    type VARCHAR(20) NOT NULL,           -- 数据类型：string, number, boolean, object, array
    is_public BOOLEAN DEFAULT FALSE,     -- 是否公开（前端可见）
    description TEXT,                    -- 设置描述
    default_value TEXT,                  -- 默认值
    validation_rules TEXT,               -- 验证规则（JSON）
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(category, key)
);

CREATE INDEX idx_settings_category ON settings(category);
CREATE INDEX idx_settings_public ON settings(is_public);
```

#### 4.1.2 设置分类表
```sql
CREATE TABLE setting_categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug VARCHAR(50) UNIQUE NOT NULL,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    icon VARCHAR(50),
    order_index INTEGER DEFAULT 0,
    parent_id INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (parent_id) REFERENCES setting_categories(id)
);
```

#### 4.1.3 设置预设表
```sql
CREATE TABLE setting_presets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(100) NOT NULL,
    slug VARCHAR(50) UNIQUE NOT NULL,
    description TEXT,
    thumbnail VARCHAR(255),
    category_id INTEGER,
    config JSON NOT NULL,               -- 预设配置
    is_default BOOLEAN DEFAULT FALSE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    created_by INTEGER,
    FOREIGN KEY (category_id) REFERENCES setting_categories(id),
    FOREIGN KEY (created_by) REFERENCES users(id)
);
```

### 4.2 API 接口设计

#### 4.2.1 设置管理 API

**获取所有设置**
```
GET /api/settings
Response: {
  "success": true,
  "data": {
    "appearance": { ... },
    "typography": { ... },
    "layout": { ... },
    ...
  }
}
```

**获取单个分类设置**
```
GET /api/settings/:category
Response: {
  "success": true,
  "data": { ... }
}
```

**更新设置**
```
PATCH /api/settings/:category
Body: {
  "field1": "value1",
  "field2": "value2"
}
Response: {
  "success": true,
  "message": "设置更新成功"
}
```

**重置设置**
```
POST /api/settings/:category/reset
Body: {
  "fields": ["field1", "field2"]  // 可选，不传则重置整个分类
}
Response: {
  "success": true,
  "data": { ... }
}
```

**应用预设**
```
POST /api/settings/presets/:id/apply
Response: {
  "success": true,
  "message": "预设应用成功"
}
```

**保存自定义预设**
```
POST /api/settings/presets
Body: {
  "name": "My Custom Theme",
  "description": "...",
  "category_id": 1,
  "config": { ... }
}
Response: {
  "success": true,
  "preset_id": 123
}
```

#### 4.2.2 实时预览 API

**获取实时预览数据**
```
GET /api/settings/preview
Query: ?changes={...}
Response: {
  "success": true,
  "preview_css": "...",
  "preview_js": "..."
}
```

### 4.3 前端架构设计

#### 4.3.1 设置模块结构
```
src/
├── js/
│   ├── settings/
│   │   ├── index.js                    # 主入口
│   │   ├── store.js                    # 状态管理
│   │   ├── api.js                      # API 调用
│   │   ├── validators.js               # 验证器
│   │   ├── categories/
│   │   │   ├── appearance.js
│   │   │   ├── typography.js
│   │   │   ├── layout.js
│   │   │   ├── animation.js
│   │   │   ├── article.js
│   │   │   ├── comment.js
│   │   │   ├── seo.js
│   │   │   ├── performance.js
│   │   │   ├── social.js
│   │   │   ├── i18n.js
│   │   │   ├── accessibility.js
│   │   │   └── security.js
│   │   ├── components/
│   │   │   ├── SettingCard.js          # 设置卡片
│   │   │   ├── SettingGroup.js         # 设置组
│   │   │   ├── PresetGallery.js        # 预设画廊
│   │   │   ├── LivePreview.js          # 实时预览
│   │   │   ├── ColorPicker.js          # 颜色选择器
│   │   │   ├── FontSelector.js         # 字体选择器
│   │   │   ├── ThemeEditor.js          # 主题编辑器
│   │   │   └── ImportExport.js         # 导入导出
│   │   └── utils/
│   │       ├── color.js                # 颜色工具
│   │       ├── typography.js           # 字体工具
│   │       ├── css.js                  # CSS 生成
│   │       └── storage.js              # 本地存储
```

#### 4.3.2 CSS 变量系统
```css
:root {
  /* 主题色彩 */
  --color-primary: #007bff;
  --color-secondary: #6c757d;
  --color-accent: #28a745;
  --color-background: #ffffff;
  --color-surface: #f8f9fa;
  --color-border: #dee2e6;
  
  /* 渐变 */
  --gradient-start: #667eea;
  --gradient-end: #764ba2;
  --gradient-direction: 135deg;
  
  /* 间距 */
  --spacing-xs: 0.25rem;
  --spacing-sm: 0.5rem;
  --spacing-md: 1rem;
  --spacing-lg: 1.5rem;
  --spacing-xl: 2rem;
  --spacing-2xl: 3rem;
  
  /* 圆角 */
  --radius-sm: 0.25rem;
  --radius-md: 0.5rem;
  --radius-lg: 1rem;
  --radius-xl: 1.5rem;
  --radius-full: 9999px;
  
  /* 阴影 */
  --shadow-sm: 0 1px 2px rgba(0,0,0,0.05);
  --shadow-md: 0 4px 6px rgba(0,0,0,0.07);
  --shadow-lg: 0 10px 15px rgba(0,0,0,0.1);
  --shadow-xl: 0 20px 25px rgba(0,0,0,0.15);
  
  /* 字体 */
  --font-family-base: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  --font-family-heading: Georgia, serif;
  --font-family-code: "JetBrains Mono", monospace;
  
  /* 字号 */
  --font-size-xs: 0.75rem;
  --font-size-sm: 0.875rem;
  --font-size-md: 1rem;
  --font-size-lg: 1.125rem;
  --font-size-xl: 1.25rem;
  --font-size-2xl: 1.5rem;
  --font-size-3xl: 1.875rem;
  
  /* 动画 */
  --transition-fast: 150ms ease;
  --transition-normal: 200ms ease;
  --transition-slow: 300ms ease;
  --ease-in-out: cubic-bezier(0.4, 0, 0.2, 1);
}
```

#### 4.3.3 动态 CSS 生成
```javascript
// css-generator.js
export class CSSGenerator {
  static generateThemeCSS(settings) {
    let css = ':root {\n';
    
    // 颜色变量
    css += `  --color-primary: ${settings.primary_color};\n`;
    css += `  --color-secondary: ${settings.secondary_color};\n`;
    css += `  --color-accent: ${settings.accent_color};\n`;
    css += `  --color-background: ${settings.background_color};\n`;
    
    // 渐变变量
    if (settings.gradient_enabled) {
      css += `  --gradient-start: ${settings.gradient_start};\n`;
      css += `  --gradient-end: ${settings.gradient_end};\n`;
      css += `  --gradient-direction: ${settings.gradient_direction};\n`;
    }
    
    // 圆角变量
    css += `  --radius-sm: ${settings.border_radius_sm};\n`;
    css += `  --radius-md: ${settings.border_radius_md};\n`;
    css += `  --radius-lg: ${settings.border_radius_lg};\n`;
    
    css += '}\n';
    
    return css;
  }
  
  static generateTypographyCSS(settings) {
    let css = ':root {\n';
    
    // 字体家族
    css += `  --font-family-base: ${settings.font_family};\n`;
    css += `  --font-family-heading: ${settings.heading_font};\n`;
    css += `  --font-family-code: ${settings.code_font};\n`;
    
    // 字号
    css += `  --font-size-base: ${settings.font_size_base};\n`;
    css += `  --font-size-xs: ${settings.font_xs};\n`;
    css += `  --font-size-sm: ${settings.font_sm};\n`;
    css += `  --font-size-lg: ${settings.font_lg};\n`;
    css += `  --font-size-xl: ${settings.font_xl};\n`;
    
    css += '}\n';
    
    return css;
  }
  
  static generateLayoutCSS(settings) {
    let css = ':root {\n';
    
    // 间距
    css += `  --spacing-xs: ${settings.spacing_xs};\n`;
    css += `  --spacing-sm: ${settings.spacing_sm};\n`;
    css += `  --spacing-md: ${settings.spacing_md};\n`;
    css += `  --spacing-lg: ${settings.spacing_lg};\n`;
    css += `  --spacing-xl: ${settings.spacing_xl};\n`;
    
    css += '}\n';
    
    return css;
  }
}
```

### 4.4 后端实现

#### 4.4.1 Rust 结构体定义
```rust
// 在 src/templates/mod.rs 中添加

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ThemeSettings {
    // 主题模式
    pub theme_mode: String,
    pub preset_theme: String,
    
    // 自定义配色
    pub primary_color: String,
    pub secondary_color: String,
    pub accent_color: String,
    pub text_color: String,
    pub background_color: String,
    pub card_background: String,
    pub border_color: String,
    
    // 渐变设置
    pub gradient_enabled: bool,
    pub gradient_start: String,
    pub gradient_end: String,
    pub gradient_direction: String,
    
    // 圆角设置
    pub border_radius_sm: String,
    pub border_radius_md: String,
    pub border_radius_lg: String,
    pub border_radius_full: String,
    
    // 阴影设置
    pub shadow_enabled: bool,
    pub shadow_color: String,
    pub shadow_light: bool,
    pub shadow_size: String,
}

impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            theme_mode: "auto".to_string(),
            preset_theme: "default".to_string(),
            primary_color: "#007bff".to_string(),
            secondary_color: "#6c757d".to_string(),
            accent_color: "#28a745".to_string(),
            text_color: "#333333".to_string(),
            background_color: "#ffffff".to_string(),
            card_background: "#f8f9fa".to_string(),
            border_color: "#dee2e6".to_string(),
            gradient_enabled: true,
            gradient_start: "#667eea".to_string(),
            gradient_end: "#764ba2".to_string(),
            gradient_direction: "135deg".to_string(),
            border_radius_sm: "0.25rem".to_string(),
            border_radius_md: "0.5rem".to_string(),
            border_radius_lg: "1rem".to_string(),
            border_radius_full: "9999px".to_string(),
            shadow_enabled: true,
            shadow_color: "rgba(0,0,0,0.1)".to_string(),
            shadow_light: true,
            shadow_size: "md".to_string(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TypographySettings {
    // 主字体
    pub font_family: String,
    pub font_size_base: String,
    pub font_weight: String,
    pub line_height: String,
    
    // 标题字体
    pub heading_font: String,
    pub heading_weight: String,
    pub heading_line_height: String,
    
    // 代码字体
    pub code_font: String,
    pub code_font_size: String,
    
    // 字体大小系统
    pub font_xs: String,
    pub font_sm: String,
    pub font_md: String,
    pub font_lg: String,
    pub font_xl: String,
    pub font_2xl: String,
    pub font_3xl: String,
    
    // 行高系统
    pub leading_tight: String,
    pub leading_normal: String,
    pub leading_relaxed: String,
    pub leading_loose: String,
}

impl Default for TypographySettings {
    fn default() -> Self {
        Self {
            font_family: "-apple-system, BlinkMacSystemFont, \"Segoe UI\", Roboto, sans-serif".to_string(),
            font_size_base: "1rem".to_string(),
            font_weight: "400".to_string(),
            line_height: "1.5".to_string(),
            heading_font: "Georgia, serif".to_string(),
            heading_weight: "700".to_string(),
            heading_line_height: "1.2".to_string(),
            code_font: "\"JetBrains Mono\", monospace".to_string(),
            code_font_size: "0.875rem".to_string(),
            font_xs: "0.75rem".to_string(),
            font_sm: "0.875rem".to_string(),
            font_md: "1rem".to_string(),
            font_lg: "1.125rem".to_string(),
            font_xl: "1.25rem".to_string(),
            font_2xl: "1.5rem".to_string(),
            font_3xl: "1.875rem".to_string(),
            leading_tight: "1.25".to_string(),
            leading_normal: "1.5".to_string(),
            leading_relaxed: "1.75".to_string(),
            leading_loose: "2".to_string(),
        }
    }
}

// 更新 TemplateSettings 结构体
pub struct TemplateSettings {
    // ... 原有字段 ...
    
    // 新增设置项
    pub theme: ThemeSettings,
    pub typography: TypographySettings,
    pub layout: LayoutSettings,
    pub animation: AnimationSettings,
    pub feedback: FeedbackSettings,
    pub article_list: ArticleListSettings,
    pub article_detail: ArticleDetailSettings,
    pub comment: CommentSettings,
    pub seo: SEOSettings,
    pub performance: PerformanceSettings,
    pub social: SocialSettings,
    pub i18n: I18nSettings,
    pub accessibility: AccessibilitySettings,
    pub security: SecuritySettings,
}
```

#### 4.4.2 API 路由处理
```rust
// src/handlers/api_handlers/settings.rs

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

/// 获取所有设置
pub async fn get_all_settings(pool: web::Data<SqlitePool>) -> impl Responder {
    match sqlx::query!(
        "SELECT category, key, value, type FROM settings WHERE is_public = true ORDER BY category, key"
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(rows) => {
            let mut settings = serde_json::Map::new();
            
            for row in rows {
                let category = settings.entry(row.category).or_insert_with(serde_json::Map::new);
                
                let value = match row.type.as_str() {
                    "boolean" => serde_json::from_str::<bool>(&row.value).unwrap_or(false),
                    "number" => serde_json::from_str::<serde_json::Number>(&row.value).unwrap_or(serde_json::Number::from(0)),
                    "object" => serde_json::from_str::<serde_json::Value>(&row.value).unwrap_or(serde_json::json!({})),
                    "array" => serde_json::from_str::<serde_json::Value>(&row.value).unwrap_or(serde_json::json!([])),
                    _ => serde_json::Value::String(row.value),
                };
                
                category.insert(row.key, value);
            }
            
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "data": settings
            }))
        }
        Err(e) => {
            eprintln!("Failed to fetch settings: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "获取设置失败"
            }))
        }
    }
}

/// 更新设置
pub async fn update_settings(
    category: web::Path<String>,
    updates: web::Json<serde_json::Map<String, serde_json::Value>>,
    pool: web::Data<SqlitePool>,
) -> impl Responder {
    let category = category.into_inner();
    
    // 验证设置
    if let Err(e) = validate_settings(&category, &updates) {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "message": e.to_string()
        }));
    }
    
    // 更新到数据库
    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to begin transaction: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "数据库错误"
            }));
        }
    };
    
    for (key, value) in updates.iter() {
        let value_str = match value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Object(obj) => serde_json::to_string(obj).unwrap_or_default(),
            serde_json::Value::Array(arr) => serde_json::to_string(arr).unwrap_or_default(),
            _ => continue,
        };
        
        let value_type = match value {
            serde_json::Value::Bool(_) => "boolean",
            serde_json::Value::Number(_) => "number",
            serde_json::Value::Object(_) => "object",
            serde_json::Value::Array(_) => "array",
            _ => "string",
        };
        
        if let Err(e) = sqlx::query!(
            "INSERT INTO settings (category, key, value, type, is_public) 
             VALUES (?, ?, ?, ?, true)
             ON CONFLICT(category, key) DO UPDATE SET 
             value = excluded.value, type = excluded.type, updated_at = CURRENT_TIMESTAMP",
            category, key, value_str, value_type
        )
        .execute(&mut *tx)
        .await
        {
            eprintln!("Failed to update setting {}:{}: {}", category, key, e);
        }
    }
    
    if let Err(e) = tx.commit().await {
        eprintln!("Failed to commit transaction: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "message": "保存失败"
        }));
    }
    
    // 清除缓存
    if let Err(e) = clear_settings_cache(&category).await {
        eprintln!("Failed to clear cache: {}", e);
    }
    
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "设置更新成功"
    }))
}

/// 重置设置
pub async fn reset_settings(
    category: web::Path<String>,
    fields: web::Query<ResetFieldsQuery>,
    pool: web::Data<SqlitePool>,
) -> impl Responder {
    let category = category.into_inner();
    
    if let Err(e) = sqlx::query!(
        "UPDATE settings 
         SET value = default_value, updated_at = CURRENT_TIMESTAMP 
         WHERE category = ?",
        category
    )
    .execute(pool.get_ref())
    .await
    {
        eprintln!("Failed to reset settings: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "message": "重置失败"
        }));
    }
    
    // 清除缓存
    if let Err(e) = clear_settings_cache(&category).await {
        eprintln!("Failed to clear cache: {}", e);
    }
    
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "设置已重置"
    }))
}

#[derive(Debug, Deserialize)]
struct ResetFieldsQuery {
    fields: Option<String>,
}

fn validate_settings(category: &str, updates: &serde_json::Map<String, serde_json::Value>) -> Result<(), String> {
    match category {
        "theme" => validate_theme_settings(updates),
        "typography" => validate_typography_settings(updates),
        "layout" => validate_layout_settings(updates),
        "animation" => validate_animation_settings(updates),
        _ => Ok(()),
    }
}

fn validate_theme_settings(updates: &serde_json::Map<String, serde_json::Value>) -> Result<(), String> {
    // 验证颜色格式
    if let Some(serde_json::Value::String(color)) = updates.get("primary_color") {
        if !is_valid_color(color) {
            return Err("无效的颜色格式".to_string());
        }
    }
    
    // 验证圆角
    if let Some(serde_json::Value::String(radius)) = updates.get("border_radius_md") {
        if !is_valid_radius(radius) {
            return Err("无效的圆角值".to_string());
        }
    }
    
    Ok(())
}

fn validate_typography_settings(updates: &serde_json::Map<String, serde_json::Value>) -> Result<(), String> {
    // 验证字体大小
    if let Some(serde_json::Value::String(size)) = updates.get("font_size_base") {
        if !is_valid_font_size(size) {
            return Err("无效的字体大小".to_string());
        }
    }
    
    Ok(())
}

fn validate_layout_settings(updates: &serde_json::Map<String, serde_json::Value>) -> Result<(), String> {
    // 验证间距
    if let Some(serde_json::Value::String(spacing)) = updates.get("spacing_md") {
        if !is_valid_spacing(spacing) {
            return Err("无效的间距值".to_string());
        }
    }
    
    Ok(())
}

fn validate_animation_settings(updates: &serde_json::Map<String, serde_json::Value>) -> Result<(), String> {
    // 验证动画时长
    if let Some(serde_json::Value::String(duration)) = updates.get("transition_duration") {
        if !is_valid_duration(duration) {
            return Err("无效的动画时长".to_string());
        }
    }
    
    Ok(())
}

fn is_valid_color(color: &str) -> bool {
    // 验证十六进制颜色、RGB、RGBA、HSL等格式
    let re = regex::Regex::new(r"^#([0-9a-fA-F]{3}|[0-9a-fA-F]{6})$|^rgba?\([^)]+\)|^hsla?\([^)]+\)$").unwrap();
    re.is_match(color)
}

fn is_valid_radius(radius: &str) -> bool {
    // 验证圆角值（可以是 px, rem, % 等）
    let re = regex::Regex::new(r"^\d+(\.\d+)?(px|rem|%|em|vw|vh|ch)$").unwrap();
    re.is_match(radius)
}

fn is_valid_font_size(size: &str) -> bool {
    // 验证字体大小
    let re = regex::Regex::new(r"^\d+(\.\d+)?(px|rem|em|vw|vh|%)$").unwrap();
    re.is_match(size)
}

fn is_valid_spacing(spacing: &str) -> bool {
    // 验证间距值
    let re = regex::Regex::new(r"^\d+(\.\d+)?(px|rem|em|vw|vh|%)$").unwrap();
    re.is_match(spacing)
}

fn is_valid_duration(duration: &str) -> bool {
    // 验证动画时长
    let re = regex::Regex::new(r"^\d+(\.\d+)?(ms|s)$").unwrap();
    re.is_match(duration)
}

async fn clear_settings_cache(category: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 清除 Valkey 缓存
    use crate::cache::manager::CacheManager;
    
    let cache_key = format!("settings:{}", category);
    CacheManager::delete(&cache_key).await?;
    
    Ok(())
}
```

## 五、前端管理界面设计

### 5.1 设置页面布局

```
管理员后台 → 系统设置
├── 基础设置
│   ├── 站点信息
│   ├── 联系方式
│   └── 备案信息
├── 外观设置
│   ├── 主题配色
│   ├── 字体设置
│   ├── 布局配置
│   ├── 动画效果
│   └── 背景设置
├── 内容设置
│   ├── 文章列表
│   ├── 文章详情
│   ├── 评论系统
│   └── SEO 优化
├── 功能设置
│   ├── Live2D
│   ├── 音乐播放器
│   ├── 社交媒体
│   └── 多语言
├── 性能设置
│   ├── 图片优化
│   ├── 缓存策略
│   └── CDN 配置
├── 安全设置
│   ├── 内容安全
│   ├── 隐私设置
│   └── 访问控制
└── 无障碍设置
    ├── 屏幕阅读器
    ├── 键盘导航
    └── 对比度增强
```

### 5.2 设置组件设计

#### 5.2.1 主题编辑器组件
```javascript
// ThemeEditor.js
class ThemeEditor {
  constructor(container, settings) {
    this.container = container;
    this.settings = settings;
    this.init();
  }
  
  init() {
    this.render();
    this.bindEvents();
    this.initLivePreview();
  }
  
  render() {
    this.container.innerHTML = `
      <div class="theme-editor">
        <div class="theme-editor-sidebar">
          <div class="theme-presets">
            <h3>预设主题</h3>
            <div class="preset-grid">
              ${this.renderPresets()}
            </div>
          </div>
          
          <div class="theme-colors">
            <h3>配色方案</h3>
            ${this.renderColorPickers()}
          </div>
          
          <div class="theme-typography">
            <h3>字体设置</h3>
            ${this.renderFontSelectors()}
          </div>
        </div>
        
        <div class="theme-editor-main">
          <div class="theme-preview">
            <div class="preview-header">实时预览</div>
            <div class="preview-content">
              <iframe id="previewFrame"></iframe>
            </div>
          </div>
          
          <div class="theme-advanced">
            <h3>高级设置</h3>
            ${this.renderAdvancedSettings()}
          </div>
        </div>
      </div>
    `;
  }
  
  renderPresets() {
    const presets = [
      { id: 'default', name: '默认主题', preview: '#007bff' },
      { id: 'ocean', name: '海洋主题', preview: '#00bcd4' },
      { id: 'forest', name: '森林主题', preview: '#4caf50' },
      { id: 'sunset', name: '日落主题', preview: '#ff5722' },
      { id: 'midnight', name: '午夜主题', preview: '#673ab7' },
    ];
    
    return presets.map(p => `
      <div class="preset-item" data-preset="${p.id}">
        <div class="preset-preview" style="background: ${p.preview}"></div>
        <div class="preset-name">${p.name}</div>
      </div>
    `).join('');
  }
  
  renderColorPickers() {
    const colors = [
      { key: 'primary_color', label: '主色调', default: '#007bff' },
      { key: 'secondary_color', label: '次要色', default: '#6c757d' },
      { key: 'accent_color', label: '强调色', default: '#28a745' },
      { key: 'background_color', label: '背景色', default: '#ffffff' },
      { key: 'text_color', label: '文字色', default: '#333333' },
    ];
    
    return colors.map(c => `
      <div class="color-picker-item">
        <label>${c.label}</label>
        <input type="color" 
               class="color-input" 
               data-key="${c.key}" 
               value="${this.settings[c.key] || c.default}">
        <input type="text" 
               class="color-text" 
               value="${this.settings[c.key] || c.default}">
      </div>
    `).join('');
  }
  
  renderFontSelectors() {
    const fonts = [
      { key: 'font_family', label: '主字体', options: [
        { value: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif', label: '系统默认' },
        { value: '"Helvetica Neue", Helvetica, Arial, sans-serif', label: 'Helvetica' },
        { value: 'Georgia, serif', label: 'Georgia' },
        { value: '"Times New Roman", Times, serif', label: 'Times New Roman' },
        { value: '"Courier New", Courier, monospace', label: 'Courier New' },
      ]},
    ];
    
    return fonts.map(f => `
      <div class="font-selector-item">
        <label>${f.label}</label>
        <select class="font-select" data-key="${f.key}">
          ${f.options.map(o => `
            <option value="${o.value}" ${this.settings[f.key] === o.value ? 'selected' : ''}>
              ${o.label}
            </option>
          `).join('')}
        </select>
      </div>
    `).join('');
  }
  
  renderAdvancedSettings() {
    return `
      <div class="advanced-settings">
        <div class="setting-group">
          <h4>渐变效果</h4>
          <div class="setting-item">
            <label>
              <input type="checkbox" class="toggle-checkbox" data-key="gradient_enabled" ${this.settings.gradient_enabled ? 'checked' : ''}>
              启用渐变背景
            </label>
          </div>
          <div class="setting-item">
            <label>起始颜色</label>
            <input type="color" class="color-input" data-key="gradient_start" value="${this.settings.gradient_start}">
          </div>
          <div class="setting-item">
            <label>结束颜色</label>
            <input type="color" class="color-input" data-key="gradient_end" value="${this.settings.gradient_end}">
          </div>
          <div class="setting-item">
            <label>渐变方向</label>
            <select class="select-input" data-key="gradient_direction">
              ${['to-right', 'to-bottom', 'to-br', 'diagonal'].map(dir => `
                <option value="${dir}" ${this.settings.gradient_direction === dir ? 'selected' : ''}>
                  ${this.getGradientLabel(dir)}
                </option>
              `).join('')}
            </select>
          </div>
        </div>
        
        <div class="setting-group">
          <h4>圆角设置</h4>
          <div class="setting-item">
            <label>小圆角</label>
            <input type="range" class="range-input" data-key="border_radius_sm" min="0" max="20" value="${this.extractNumber(this.settings.border_radius_sm)}">
          </div>
          <div class="setting-item">
            <label>中圆角</label>
            <input type="range" class="range-input" data-key="border_radius_md" min="0" max="30" value="${this.extractNumber(this.settings.border_radius_md)}">
          </div>
          <div class="setting-item">
            <label>大圆角</label>
            <input type="range" class="range-input" data-key="border_radius_lg" min="0" max="50" value="${this.extractNumber(this.settings.border_radius_lg)}">
          </div>
        </div>
        
        <div class="setting-group">
          <h4>阴影设置</h4>
          <div class="setting-item">
            <label>
              <input type="checkbox" class="toggle-checkbox" data-key="shadow_enabled" ${this.settings.shadow_enabled ? 'checked' : ''}>
              启用阴影效果
            </label>
          </div>
          <div class="setting-item">
            <label>阴影颜色</label>
            <input type="color" class="color-input" data-key="shadow_color" value="${this.settings.shadow_color}">
          </div>
          <div class="setting-item">
            <label>阴影大小</label>
            <select class="select-input" data-key="shadow_size">
              ${['sm', 'md', 'lg', 'xl'].map(size => `
                <option value="${size}" ${this.settings.shadow_size === size ? 'selected' : ''}>
                  ${this.getShadowLabel(size)}
                </option>
              `).join('')}
            </select>
          </div>
        </div>
      </div>
    `;
  }
  
  bindEvents() {
    // 预设主题点击
    this.container.querySelectorAll('.preset-item').forEach(item => {
      item.addEventListener('click', () => {
        const presetId = item.dataset.preset;
        this.applyPreset(presetId);
      });
    });
    
    // 颜色选择器
    this.container.querySelectorAll('.color-input').forEach(input => {
      input.addEventListener('input', (e) => {
        const key = e.target.dataset.key;
        const value = e.target.value;
        this.updateSetting(key, value);
        this.updatePreview();
      });
    });
    
    // 字体选择器
    this.container.querySelectorAll('.font-select').forEach(select => {
      select.addEventListener('change', (e) => {
        const key = e.target.dataset.key;
        const value = e.target.value;
        this.updateSetting(key, value);
        this.updatePreview();
      });
    });
    
    // 开关切换
    this.container.querySelectorAll('.toggle-checkbox').forEach(checkbox => {
      checkbox.addEventListener('change', (e) => {
        const key = e.target.dataset.key;
        const value = e.target.checked;
        this.updateSetting(key, value);
        this.updatePreview();
      });
    });
    
    // 滑块
    this.container.querySelectorAll('.range-input').forEach(input => {
      input.addEventListener('input', (e) => {
        const key = e.target.dataset.key;
        const value = e.target.value + 'px';
        this.updateSetting(key, value);
        this.updatePreview();
      });
    });
    
    // 下拉选择
    this.container.querySelectorAll('.select-input').forEach(select => {
      select.addEventListener('change', (e) => {
        const key = e.target.dataset.key;
        const value = e.target.value;
        this.updateSetting(key, value);
        this.updatePreview();
      });
    });
  }
  
  initLivePreview() {
    const previewFrame = document.getElementById('previewFrame');
    
    // 加载预览页面
    previewFrame.src = '/preview';
    
    // 监听预览页面的消息
    window.addEventListener('message', (e) => {
      if (e.data.type === 'preview-ready') {
        this.updatePreview();
      }
    });
  }
  
  updatePreview() {
    const previewFrame = document.getElementById('previewFrame');
    const css = CSSGenerator.generateThemeCSS(this.settings);
    
    previewFrame.contentWindow.postMessage({
      type: 'update-theme',
      css: css
    }, '*');
  }
  
  updateSetting(key, value) {
    this.settings[key] = value;
    
    // 防抖保存
    if (this.saveTimeout) {
      clearTimeout(this.saveTimeout);
    }
    
    this.saveTimeout = setTimeout(() => {
      this.saveSettings();
    }, 1000);
  }
  
  async saveSettings() {
    try {
      const response = await fetch('/api/settings/theme', {
        method: 'PATCH',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': this.getAuthHeader()
        },
        body: JSON.stringify(this.settings)
      });
      
      const result = await response.json();
      
      if (result.success) {
        showToast('设置保存成功', 'success');
      } else {
        showToast(result.message, 'error');
      }
    } catch (error) {
      console.error('保存设置失败:', error);
      showToast('保存设置失败', 'error');
    }
  }
  
  async applyPreset(presetId) {
    try {
      const response = await fetch(`/api/settings/presets/${presetId}`, {
        headers: {
          'Authorization': this.getAuthHeader()
        }
      });
      
      const result = await response.json();
      
      if (result.success) {
        this.settings = { ...this.settings, ...result.data.config };
        this.render();
        this.bindEvents();
        this.updatePreview();
        showToast('预设应用成功', 'success');
      } else {
        showToast(result.message, 'error');
      }
    } catch (error) {
      console.error('应用预设失败:', error);
      showToast('应用预设失败', 'error');
    }
  }
  
  getGradientLabel(direction) {
    const labels = {
      'to-right': '向右',
      'to-bottom': '向下',
      'to-br': '右下',
      'diagonal': '对角'
    };
    return labels[direction] || direction;
  }
  
  getShadowLabel(size) {
    const labels = {
      'sm': '小',
      'md': '中',
      'lg': '大',
      'xl': '超大'
    };
    return labels[size] || size;
  }
  
  extractNumber(value) {
    const match = value.match(/(\d+(\.\d+)?)/);
    return match ? parseFloat(match[1]) : 0;
  }
  
  getAuthHeader() {
    const token = this.getCookie('auth_token');
    return `Bearer ${token}`;
  }
  
  getCookie(name) {
    const value = `; ${document.cookie}`;
    const parts = value.split(`; ${name}=`);
    if (parts.length === 2) return parts.pop().split(';').shift();
    return '';
  }
}
```

### 5.3 实时预览系统

```javascript
// LivePreview.js
class LivePreview {
  constructor() {
    this.settings = {};
    this.debounceTimer = null;
    this.init();
  }
  
  init() {
    this.loadSettings();
    this.setupMessageListener();
  }
  
  async loadSettings() {
    try {
      const response = await fetch('/api/settings');
      const result = await response.json();
      
      if (result.success) {
        this.settings = result.data;
        this.applySettings();
      }
    } catch (error) {
      console.error('加载设置失败:', error);
    }
  }
  
  setupMessageListener() {
    window.addEventListener('message', (e) => {
      if (e.data.type === 'update-theme') {
        this.applyThemeCSS(e.data.css);
      } else if (e.data.type === 'update-typography') {
        this.applyTypographyCSS(e.data.css);
      } else if (e.data.type === 'update-layout') {
        this.applyLayoutCSS(e.data.css);
      }
    });
    
    // 通知父窗口预览已就绪
    window.parent.postMessage({ type: 'preview-ready' }, '*');
  }
  
  applySettings() {
    const themeCSS = CSSGenerator.generateThemeCSS(this.settings.theme || {});
    const typographyCSS = CSSGenerator.generateTypographyCSS(this.settings.typography || {});
    const layoutCSS = CSSGenerator.generateLayoutCSS(this.settings.layout || {});
    
    // 应用CSS
    this.applyCSS(themeCSS + typographyCSS + layoutCSS);
    
    // 应用暗色模式
    this.applyDarkMode(this.settings.theme?.theme_mode);
  }
  
  applyCSS(css) {
    let styleElement = document.getElementById('dynamic-styles');
    
    if (!styleElement) {
      styleElement = document.createElement('style');
      styleElement.id = 'dynamic-styles';
      document.head.appendChild(styleElement);
    }
    
    styleElement.textContent = css;
  }
  
  applyThemeCSS(css) {
    let styleElement = document.getElementById('theme-styles');
    
    if (!styleElement) {
      styleElement = document.createElement('style');
      styleElement.id = 'theme-styles';
      document.head.appendChild(styleElement);
    }
    
    styleElement.textContent = css;
  }
  
  applyTypographyCSS(css) {
    let styleElement = document.getElementById('typography-styles');
    
    if (!styleElement) {
      styleElement = document.createElement('style');
      styleElement.id = 'typography-styles';
      document.head.appendChild(styleElement);
    }
    
    styleElement.textContent = css;
  }
  
  applyLayoutCSS(css) {
    let styleElement = document.getElementById('layout-styles');
    
    if (!styleElement) {
      styleElement = document.createElement('style');
      styleElement.id = 'layout-styles';
      document.head.appendChild(styleElement);
    }
    
    styleElement.textContent = css;
  }
  
  applyDarkMode(mode) {
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    
    let isDark = mode === 'dark' || (mode === 'auto' && prefersDark);
    
    if (isDark) {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
  }
}
```

## 六、实施路线图

### 阶段一：核心框架（第1-2周）
- [ ] 设计并创建数据库表结构
- [ ] 实现后端数据模型和 API 接口
- [ ] 创建前端设置模块基础架构
- [ ] 实现设置加载和保存功能

### 阶段二：主题系统（第3-4周）
- [ ] 实现主题配色设置
- [ ] 实现字体系统
- [ ] 创建主题编辑器组件
- [ ] 实现预设主题管理
- [ ] 开发实时预览功能

### 阶段三：布局与内容（第5-6周）
- [ ] 实现布局配置
- [ ] 优化文章列表样式
- [ ] 改进文章详情页
- [ ] 增强评论系统
- [ ] 实现 SEO 设置

### 阶段四：高级功能（第7-8周）
- [ ] 实现动画效果配置
- [ ] 添加反馈系统
- [ ] 开发社交媒体集成
- [ ] 实现多语言支持
- [ ] 添加无障碍功能

### 阶段五：性能与安全（第9-10周）
- [ ] 实现性能优化设置
- [ ] 添加安全配置
- [ ] 优化缓存策略
- [ ] 性能测试和优化
- [ ] 安全审计和加固

### 阶段六：测试与优化（第11-12周）
- [ ] 全面功能测试
- [ ] 性能测试和优化
- [ ] 跨浏览器测试
- [ ] 响应式测试
- [ ] 用户体验优化

## 七、技术栈总结

### 后端
- **语言**: Rust
- **框架**: Actix-web
- **数据库**: SQLite (通过 sqlx)
- **缓存**: Valkey (Redis 兼容)

### 前端
- **核心**: 原生 JavaScript (ES6+)
- **样式**: 原生 CSS (CSS Variables, Grid, Flexbox)
- **图标**: SVG (无外部依赖)
- **构建**: 无需构建工具（可选 Vite 优化）

### 工具
- **代码格式化**: Prettier
- **代码检查**: Clippy
- **版本控制**: Git

## 八、注意事项

### 8.1 性能考虑
- 使用 CSS 变量实现主题切换，减少重绘
- 实现设置缓存机制，减少数据库查询
- 使用防抖技术优化保存操作
- 延迟加载非关键设置

### 8.2 安全考虑
- 对所有用户输入进行验证和清理
- 防止 CSS 注入攻击
- 限制上传文件大小和类型
- 实现权限控制

### 8.3 兼容性
- 确保跨浏览器兼容性
- 提供优雅降级方案
- 支持移动端和桌面端
- 考虑无障碍访问需求

### 8.4 可扩展性
- 模块化设计，易于扩展新功能
- 插件化架构，支持第三方扩展
- 清晰的 API 设计，便于集成
- 完善的文档和注释

## 九、预期效果

### 9.1 用户体验
- 直观的可视化设置界面
- 实时预览，所见即所得
- 丰富的预设主题
- 灵活的自定义选项

### 9.2 开发体验
- 清晰的代码结构
- 完善的类型定义
- 模块化的组件设计
- 详细的文档说明

### 9.3 性能指标
- 页面加载时间 < 2s
- 首次内容绘制 < 1s
- 交互响应时间 < 100ms
- Lighthouse 评分 > 90

## 十、后续迭代方向

### 10.1 AI 辅助
- AI 主题生成
- 智能配色推荐
- 自动化 SEO 优化

### 10.2 高级功能
- 自定义组件库
- 插件市场
- 主题市场

### 10.3 社区功能
- 主题分享
- 用户贡献
- 评分和评论

---

**文档版本**: 1.0  
**创建日期**: 2026-02-17  
**最后更新**: 2026-02-17  
**维护者**: RustBlog Team