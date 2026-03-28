use rust_embed::RustEmbed;
use std::fs;
use std::path::Path;

/// 嵌入的文件系统
/// 包含 templates、img、music 目录
/// 排除 GeoLite2-City.mmdb 文件（可选文件，用户需自行下载）
///
/// 重要：必须在项目根目录（包含 Cargo.toml 的目录）运行 cargo build
/// rust-embed 会在编译时从当前工作目录嵌入文件
#[derive(RustEmbed)]
#[folder = "."]
#[include = "templates/**"]
#[include = "img/**"]
#[include = "music/**"]
#[exclude = "data/GeoLite2-City.mmdb"]
pub struct EmbeddedAssets;

/// 释放嵌入的资源
/// 按照 Go 版本的逻辑：
/// - data、markdown、attachments 目录仅创建，不释放文件
/// - templates 目录下的所有文件（包括 CSS、JS）保持内嵌，不释放
/// - 只有 img 和 music 目录会在启动时释放（如果文件不存在）
pub fn extract_embedded_resources(base_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 释放嵌入的资源...");
    println!("  📁 基础目录: {}", base_dir.display());

    // 测试时的调试代码：提取所有嵌入文件到临时目录以便检查
    #[cfg(debug_assertions)]
    {
        let temp_extract_dir = base_dir.join(".extracted_debug");
        fs::create_dir_all(&temp_extract_dir)?;
        println!(
            "🔍 调试：提取所有嵌入文件到临时目录: {}",
            temp_extract_dir.display()
        );

        let mut extracted_count = 0;
        for path in EmbeddedAssets::iter() {
            let path_str = path.as_ref();
            if let Some(content) = EmbeddedAssets::get(&path) {
                let dst_path = temp_extract_dir.join(path_str);
                if let Some(parent) = dst_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&dst_path, &content.data)?;
                extracted_count += 1;
            }
        }
        println!("🔍 调试：已提取 {} 个文件到临时目录", extracted_count);
    }

    // 测试时的调试代码：打印所有嵌入的文件
    #[cfg(debug_assertions)]
    {
        println!("🔍 调试：所有嵌入的文件:");
        let mut count = 0;
        let iter = EmbeddedAssets::iter();
        let mut first_path = None;

        for path in iter {
            let path_str = path.as_ref();
            if first_path.is_none() {
                first_path = Some(path_str.to_string());
            }
            println!("  - [{}] {}", path_str.len(), path_str);
            count += 1;
        }

        println!("🔍 总共嵌入 {} 个文件/目录", count);
        if let Some(p) = first_path {
            println!("🔍 第一个嵌入文件: {}", p);
        } else {
            println!("🔍 警告: iter() 返回空迭代器！");
        }

        // 尝试直接访问一个已知的文件
        println!("🔍 测试直接访问文件:");
        if let Some(content) = EmbeddedAssets::get("templates/index.html") {
            println!(
                "  ✓ 成功获取 templates/index.html ({} bytes)",
                content.data.len()
            );
        } else {
            println!("  ✗ 无法获取 templates/index.html");
        }

        if let Some(content) = EmbeddedAssets::get("img/avatar.webp") {
            println!(
                "  ✓ 成功获取 img/avatar.webp ({} bytes)",
                content.data.len()
            );
        } else {
            println!("  ✗ 无法获取 img/avatar.webp");
        }
    }

    // 需要创建的目录列表（仅创建，不释放文件）
    let dirs = vec!["attachments", "data", "markdown"];

    // 创建必要的目录
    for dir in &dirs {
        let dir_path = base_dir.join(dir);
        fs::create_dir_all(&dir_path)?;
        println!("  ✓ 创建目录: {}", dir_path.display());
    }

    // 释放 img 目录中的文件
    println!("  📁 处理 img 目录");
    let img_src = "img";
    let img_dst = base_dir.join("img");
    if let Err(e) = extract_dir(img_src, &img_dst) {
        eprintln!("  ⚠️  释放 img 目录失败: {}", e);
    }

    // 释放 music 目录中的文件
    println!("  📁 处理 music 目录");
    let music_src = "music";
    let music_dst = base_dir.join("music");
    if let Err(e) = extract_dir(music_src, &music_dst) {
        eprintln!("  ⚠️  释放 music 目录失败: {}", e);
    }

    println!("✅ 资源释放完成");
    Ok(())
}

/// 从嵌入的文件系统中提取目录
fn extract_dir(src_dir: &str, dst_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // 创建目标目录
    fs::create_dir_all(dst_dir)?;

    let mut extracted_count = 0;
    let mut skipped_count = 0;
    let mut matched_count = 0;

    println!("    🔍 调试: src_dir = '{}'", src_dir);
    println!("    🔍 调试: dst_dir = '{}'", dst_dir.display());

    // 遍历嵌入文件系统中的文件
    for path in EmbeddedAssets::iter() {
        let path_str = path.as_ref();

        // 只处理目标目录下的文件
        if path_str.starts_with(src_dir) {
            matched_count += 1;
            let relative_path = path_str.strip_prefix(src_dir)
                .ok_or_else(|| format!("Path '{}' should start with '{}'", path_str, src_dir))?;
            // 移除可能的前导斜杠
            let relative_path = relative_path.strip_prefix('/').unwrap_or(relative_path);
            let dst_path = dst_dir.join(relative_path);

            println!("    ✓ 匹配: '{}' -> '{}'", path_str, dst_path.display());

            // 检查文件是否已存在
            if dst_path.exists() {
                skipped_count += 1;
                continue;
            }

            // 创建父目录
            if let Some(parent) = dst_path.parent() {
                fs::create_dir_all(parent)?;
            }

            // 提取文件
            if let Some(content) = EmbeddedAssets::get(&path) {
                match std::fs::write(&dst_path, &content.data) {
                    Ok(_) => {
                        extracted_count += 1;
                        println!("    ✓ 提取: {}", dst_path.display());
                    }
                    Err(e) => {
                        eprintln!("    ✗ 无法写入文件 {}: {}", dst_path.display(), e);
                    }
                }
            }
        }
    }

    println!(
        "    📊 统计: 匹配 {} 个文件，提取 {} 个，跳过 {} 个",
        matched_count, extracted_count, skipped_count
    );

    if extracted_count > 0 {
        println!(
            "    - 提取了 {} 个文件，跳过 {} 个已存在的文件",
            extracted_count, skipped_count
        );
    } else {
        println!("    - 所有 {} 个文件已存在，跳过提取", skipped_count);
    }

    Ok(())
}

/// 获取嵌入的文件内容
pub fn get_embedded_file(path: &str) -> Option<Vec<u8>> {
    EmbeddedAssets::get(path).map(|f| f.data.to_vec())
}
