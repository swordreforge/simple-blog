# 待解决问题

## 归档页面时间轴筛选不一致问题

### 问题描述
归档页面（`/collect`）中，时间轴的年份点击功能与标签云的筛选行为不一致。

### 当前行为

| 筛选方式 | 实现方式 | 效果 |
|---------|---------|------|
| **标签云筛选** | 纯前端 CSS 隐藏 (`filterByTag()`) | ✅ 只显示包含该标签的文章 |
| **分类筛选** | 重新调用 API (`/api/passages?category=xxx`) | ✅ 只显示该分类的文章 |
| **年份筛选** | 滚动定位 (`scrollTo`) | ❌ 只滚动到该年份位置，显示所有年份的文章 |

### 问题详情

**时间轴年份点击代码**（`templates/collect.html`）：
```javascript
document.querySelectorAll(".timeline-year").forEach(e => {
    e.addEventListener("click", function() {
        // 只滚动定位，不筛选
        window.scrollTo({top: t.offsetTop-120, behavior:"smooth"});
    });
});
```

**标签云点击代码**（`templates/collect.html`）：
```javascript
function filterByTag(e) {
    document.querySelectorAll(".document-card").forEach(t => {
        // 隐藏不匹配的文章
        n ? t.style.display="block" : (t.style.display="none");
    });
}
```

### 期望行为
时间轴的年份点击应该与标签云的筛选行为一致：
- 点击某个年份后，只显示该年份的文章
- 其他年份的文章应该被隐藏
- 需要添加"全部文章"按钮来恢复显示所有年份

### 相关文件
- `templates/collect.html` - 归档页面模板
- `src/handlers/api_handlers/archive.rs` - 归档 API

### 解决方案建议

#### 方案 1：修改年份点击为筛选模式
```javascript
function filterByYear(year) {
    document.querySelectorAll(".archive-section").forEach(section => {
        if (section.querySelector('h2').textContent.includes(year)) {
            section.style.display = "block";
        } else {
            section.style.display = "none";
        }
    });
}
```

#### 方案 2：添加年份筛选按钮
在时间轴区域添加"全部年份"按钮，点击后恢复显示所有年份。

### 优先级
中等 - 功能性改进，不影响核心功能

---

## 其他潜在问题

### 1. 归档页面无限滚动性能
- 当前每次加载 20 篇文章
- 随着文章数量增加，DOM 节点会越来越多
- 建议：实现虚拟滚动或添加分页控制

### 2. 分类筛选时重新请求 API
- 当前分类筛选会重新请求 `/api/passages?category=xxx&limit=1000`
- 对于大量文章，这可能导致性能问题
- 建议：改用前端筛选模式，与标签云一致

## 手动排查后发现问题及建议

```
http://localhost:8080/api/passages?category=whoami&limit=1000&offset=0
```

````
{
  "data": [
    {
      "author": "管理员",
      "category": "whoami",
      "content": "## how are you?\n",
      "cover_image": "/img/passage-cover2.webp",
      "created_at": "2026-02-12 14:22:17",
      "file_path": "markdown/2026/02/12/testify.md",
      "html_content": null,
      "id": 16,
      "is_scheduled": false,
      "published_at": null,
      "status": "published",
      "summary": "how are you?",
      "tags": "[\"testify\"]",
      "title": "testify",
      "updated_at": "2026-02-12 14:23:44",
      "uuid": "7427718695665455104",
      "visibility": "public"
    },
    {
      "author": "Admin",
      "category": "未分类",
      "content": "## 这是一个代码块测试！！！\n```c/c++\n#include \"bits/stdc++.h\"\nusing namespace std;\nconst int N=10;\nint n;\nbool use[N];\nint s[N];\nvoid dfs(int u){\n\tif(u\u003En){\n\t\tfor(int i=1;i\u003C=n;i++){\n\t\t\tcout\u003C\u003Cs[i]\u003C\u003C\" \";\n\t\t}\n\t\tcout\u003C\u003Cendl;\n\t\treturn;\n\t}\n\tfor(int i=1;i\u003C=n;i++){\n\t\tif(!used[i]){\n\t\t\ts[u]=i,used[i]=true;\n\t\t\tdfs(u+1);\n\t\t\tused[i]=false;s[0]=0\n\t\t}\n\t}\n\t\n\treturn;\n}\nint main(){\n\tcin\u003E\u003En;\n\tdfs(1);\nreturn 0;\n}\n```",
      "cover_image": "/img/passage-cover.webp",
      "created_at": "2026-02-12 00:00:00",
      "file_path": "markdown/2026/02/12/代码块高亮测试.md",
      "html_content": null,
      "id": 15,
      "is_scheduled": false,
      "published_at": null,
      "status": "published",
      "summary": "这是一个代码块测试！！！ #include \"bits/stdc++.h\" using namespace std; const int N=10; int n; bool use[N]; int s[N]; void dfs(int u){ if(u&gt;n){ for(int i=1;i&lt;=n;i++){ cout&lt;&lt;s[i]&lt;&lt;\" \"; } cout&lt;&...",
      "tags": "[]",
      "title": "代码块高亮测试",
      "updated_at": "2026-02-12 14:15:59",
      "uuid": "7427717108989612032",
      "visibility": "public"
    }
  ],
  "pagination": {
    "has_more": false,
    "limit": 1000,
    "page": 1,
    "total": 2,
    "total_pages": 1
  },
  "success": true
}
````

这个后端不支持category=whoami筛选，建议仿照标签云实现前端纯CSS隐藏