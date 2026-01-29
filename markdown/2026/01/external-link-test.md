# 外链跳转测试文章

这篇文章用于测试博客系统的外部链接跳转拦截功能。

## 测试场景

### 1. 白名单链接（应该直接跳转）

这些域名在白名单中，应该能够直接跳转，不显示警告：

- [GitHub](https://github.com) - 全球最大的代码托管平台
- [Gitee](https://gitee.com) - 国内优秀的代码托管平台
- [Stack Overflow](https://stackoverflow.com) - 程序员问答社区

访问这些链接时，浏览器应该直接跳转，不会有任何拦截提示。

### 2. 非白名单链接（应该显示警告）

这些域名不在白名单中，应该显示跳转警告提示：

- [百度](https://www.baidu.com) - 中文搜索引擎
- [谷歌](https://www.google.com) - 全球搜索引擎
- [必应](https://www.bing.com) - 微软搜索引擎
- [YouTube](https://www.youtube.com) - 视频分享平台
- [Twitter](https://twitter.com) - 社交媒体平台
- [Facebook](https://www.facebook.com) - 社交网络平台
- [Instagram](https://www.instagram.com) - 图片社交平台
- [LinkedIn](https://www.linkedin.com) - 职业社交平台
- [知乎](https://www.zhihu.com) - 中文问答社区
- [掘金](https://juejin.cn) - 技术社区

访问这些链接时，应该弹出一个警告模态框，提示用户即将离开本站，前往外部链接。

### 3. 同站链接（不应该拦截）

这些链接指向博客内部，不应该触发拦截：

- [首页](/)
- [归档页](/collect)
- [关于页面](/about)
- [管理后台](/admin)

### 4. 相对路径链接（不应该拦截）

- [Markdown 示例](markdown/test.md)
- [测试页面](test.html)

### 5. 其他类型的链接

**邮箱链接：**
- [联系我](mailto:example@example.com)
- [反馈建议](mailto:feedback@rustblog.com)

**电话链接：**
- [致电我们](tel:+861234567890)

**锚点链接：**
- [跳转到测试场景](#测试场景)
- [跳转到白名单链接](#1-白名单链接应该直接跳转)

## 技术细节

### 外链拦截机制

博客系统通过以下机制实现外链拦截：

1. **白名单检查**：系统维护一个域名白名单，白名单中的域名可以直接跳转
2. **JavaScript 拦截**：所有外部链接的点击事件会被 JavaScript 拦截
3. **警告提示**：不在白名单中的链接会显示警告模态框
4. **用户确认**：用户确认后才允许跳转

### 配置说明

外链拦截功能可以在管理后台的设置中进行配置：

- **启用外链警告**：开启/关闭外链跳转警告功能
- **外部链接白名单**：配置允许直接跳转的域名（逗号分隔）
- **警告提示文字**：自定义警告提示的文字内容

### 默认配置

```javascript
{
  "external_link_warning": true,
  "external_link_whitelist": "github.com,gitee.com,stackoverflow.com",
  "external_link_warning_text": "您即将离开本站，前往外部链接"
}
```

## 测试清单

请按照以下清单测试外链拦截功能：

- [ ] 白名单链接能够直接跳转（GitHub、Gitee、Stack Overflow）
- [ ] 非白名单链接显示警告提示（百度、谷歌等）
- [ ] 警告提示中显示正确的目标链接
- [ ] 点击"确认跳转"后能够正确跳转
- [ ] 点击"取消"后不跳转
- [ ] 同站链接不触发拦截
- [ ] 相对路径链接不触发拦截
- [ ] 邮箱链接和电话链接不触发拦截
- [ ] 锚点链接不触发拦截
- [ ] 管理后台能够修改白名单配置
- [ ] 管理后台能够修改警告提示文字
- [ ] 关闭外链警告功能后，所有链接都能直接跳转

## 注意事项

1. **安全性**：外链拦截功能可以防止用户意外点击到恶意链接
2. **用户体验**：对于常用的外部网站（如 GitHub），建议添加到白名单
3. **配置灵活性**：管理员可以根据实际需求调整白名单和警告文字
4. **兼容性**：该功能兼容主流浏览器（Chrome、Firefox、Safari、Edge）

## 相关资源

- [MDN - 外部链接安全最佳实践](https://developer.mozilla.org/zh-CN/docs/Web/Security/External_links)
- [OWASP - 点击劫持防护](https://owasp.org/www-community/attacks/Clickjacking)
- [CSP (Content Security Policy)](https://developer.mozilla.org/zh-CN/docs/Web/HTTP/CSP)

---

**最后更新时间：** 2026-01-29  
**测试版本：** v1.0.0  
**测试状态：** 待测试