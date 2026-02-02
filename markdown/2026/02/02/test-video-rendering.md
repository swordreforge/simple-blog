# 视频渲染测试

这是一篇测试文章，用于测试 Markdown 中视频的渲染功能。

## 测试 1: HTML5 Video 标签

<video controls width="640" height="360">
  <source src="https://www.w3schools.com/html/mov_bbb.mp4" type="video/mp4">
  您的浏览器不支持 video 标签。
</video>

## 测试 2: 带自动播放的 Video 标签

<video controls autoplay muted width="640" height="360">
  <source src="https://www.w3schools.com/html/mov_bbb.mp4" type="video/mp4">
  您的浏览器不支持 video 标签。
</video>

## 测试 3: YouTube iframe 嵌入

<iframe width="560" height="315" src="https://www.youtube.com/embed/dQw4w9WgXcQ" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>

## 测试 4: Bilibili iframe 嵌入

<iframe src="//player.bilibili.com/player.html?isOutside=true&bvid=BV1xx411c7mD" scrolling="no" border="0" frameborder="no" framespacing="0" allowfullscreen="true" width="640" height="360"></iframe>

## 测试 5: Vimeo iframe 嵌入

<iframe src="https://player.vimeo.com/video/76979871" width="640" height="360" frameborder="0" allow="autoplay; fullscreen; picture-in-picture" allowfullscreen></iframe>

## 测试 6: 居中对齐的视频

<div style="text-align: center;">
  <video controls width="640" height="360">
    <source src="https://www.w3schools.com/html/mov_bbb.mp4" type="video/mp4">
    您的浏览器不支持 video 标签。
  </video>
</div>

---

以上就是 Markdown 中视频渲染的所有测试场景。

**支持的视频格式：**

1. **HTML5 Video 标签** - 使用原生 HTML5 视频播放器
2. **YouTube iframe** - 嵌入 YouTube 视频
3. **Bilibili iframe** - 嵌入 Bilibili 视频
4. **Vimeo iframe** - 嵌入 Vimeo 视频
5. **其他视频平台的 iframe** - 支持 iframe 嵌入的所有平台