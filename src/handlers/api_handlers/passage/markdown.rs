use moka::sync::Cache;
use once_cell::sync::Lazy;
use pulldown_cmark::{Options, Parser, html};
use std::fs;
use std::path::Path;
use std::time::Duration;

/// 将 Markdown 转换为 HTML（带缓存，使用 moka 无锁缓存）
pub fn convert_markdown_to_html(markdown: &str) -> String {
    // 使用内容哈希作为缓存键
    let mut hasher = md5::Md5::default();
    md5::Digest::update(&mut hasher, markdown.as_bytes());
    let content_hash = format!("{:x}", md5::Digest::finalize(hasher));

    // 静态缓存：使用 moka::sync::Cache（无锁、高性能、内置 LRU 和 TTL）
    static RENDER_CACHE: Lazy<Cache<String, String>> = Lazy::new(|| {
        Cache::builder()
            .max_capacity(1000)
            .time_to_live(Duration::from_secs(3600)) // 1小时 TTL
            .build()
    });

    // 无锁获取缓存
    if let Some(cached_html) = RENDER_CACHE.get(&content_hash) {
        return cached_html;
    }

    // 缓存未命中，执行渲染
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(markdown, options);
    // HTML 通常比 Markdown 大 1.5-2 倍，预分配容量避免重分配
    let mut html_output = String::with_capacity(markdown.len() * 2);
    html::push_html(&mut html_output, parser);

    // 无锁插入缓存
    RENDER_CACHE.insert(content_hash, html_output.clone());

    html_output
}

/// 更新 Markdown 文件
pub fn update_markdown_file(file_path: &str, content: &str) -> Result<(), String> {
    // 创建目录
    if let Some(parent) = Path::new(file_path).parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }

    // 写入文件
    fs::write(file_path, content).map_err(|e| format!("写入文件失败: {}", e))?;

    Ok(())
}

/// 更新 Markdown 文件名（如果标题改变）
pub fn update_markdown_file_name(old_path: &str, new_title: &str, content: &str) -> String {
    // 构建新文件路径
    if let Some(parent) = Path::new(old_path).parent() {
        let new_path = parent.join(format!("{}.md", new_title));

        // 删除旧文件
        let _ = fs::remove_file(old_path);

        // 创建新文件
        if let Some(new_path_str) = new_path.to_str() {
            if let Err(e) = update_markdown_file(new_path_str, content) {
                eprintln!("更新文件名失败: {}", e);
                return old_path.to_string();
            }
        } else {
            eprintln!("无法将新路径转换为字符串");
            return old_path.to_string();
        }

        new_path
            .to_str()
            .map(|s| s.to_string())
            .unwrap_or_else(|| old_path.to_string())
    } else {
        old_path.to_string()
    }
}
