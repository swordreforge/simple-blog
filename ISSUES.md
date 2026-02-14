# 文章界面的问题(./templates/passage.html,先存档再做修改)

1.现在在归档界面带上#标题访问->跳转文章界面再点击还有更多文章点击加载会出现重复的文章，而直接访问数据正常
   - **修复时间**: 2026-02-13
   - **修复内容**: 修改 `findArticleByTitle` 函数，使用现有分页机制循环加载数据查找文章，而不是单独加载500篇文章，避免与分页逻辑冲突导致重复加载问题。

2.valkey缓存完全正常

```
127.0.0.1:6379> TYPE rustblog:passage:list:all:page:1:limit:10
string
127.0.0.1:6379> GET rustblog:passage:list:all:page:1:limit:10
......省略{\"has_more\":true,\"limit\":10,\"page\":1,\"total\":18,\"total_pages\":2},\"success\":true}"
```

3.点击展开后不能正常地处理一些年份,月，日的处理，它会合并成只有一个年月日的下面展示所有文章
   - **第一次修复 (2026-02-13)**: 修改 `mergeArticlesData` 函数，在重新组织数据时保留原有的文件夹展开状态，避免合并后文件夹状态丢失导致文章显示混乱的问题。
   - **问题未完全解决**: 原来只显示10条（13号2篇+12号8篇），点击加载更多后全显示13号了
   - **第二次修复 (2026-02-13)**: 修改 `organizeArticlesByFolder` 函数，在创建文章对象时保留原始的 `created_at` 属性。这样当 `mergeArticlesData` 提取文章并重新组织时，能正确解析每篇文章的原始日期，避免所有文章被归类到当前日期的问题。











# 完整 Nginx 配置文件
# 适用于 Ubuntu/Debian 等系统，包含你的三层结构及常用优化

user www-data;
worker_processes auto;
pid /run/nginx.pid;
include /etc/nginx/modules-enabled/*.conf;

events {
    worker_connections 1024;        # 每个 worker 最大连接数
    multi_accept on;                # 尽可能接受更多连接
    use epoll;                      # Linux 高性能事件模型
}

http {
    ##
    # 基础设置
    ##
    sendfile on;
    tcp_nopush on;
    tcp_nodelay on;
    keepalive_timeout 65;
    types_hash_max_size 2048;
    server_tokens off;               # 隐藏 Nginx 版本号

    # MIME 类型
    include /etc/nginx/mime.types;
    default_type application/octet-stream;
    
    ##
    # 日志格式与位置
    ##
    access_log /var/log/nginx/access.log combined buffer=32k flush=5s;
    error_log /var/log/nginx/error.log warn;
    
    ##
    # Gzip 压缩（可根据需要启用）
    ##
    gzip on;
    gzip_vary on;
    gzip_proxied any;
    gzip_comp_level 6;
    gzip_types text/plain text/css text/xml application/json application/javascript application/xml+rss application/atom+xml image/svg+xml;
    
    ##
    # Brotli 压缩（需安装 ngx_brotli 模块，若无则注释掉）
    ##
    # brotli on;
    # brotli_comp_level 6;
    # brotli_types text/plain text/css text/xml application/json application/javascript application/xml+rss application/atom+xml image/svg+xml;
}
