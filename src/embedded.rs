use rust_embed::RustEmbed;
use std::fs;
use std::io::Write;
use std::path::Path;

/// 嵌入的文件系统
/// 包含 templates、img、music 目录
/// 排除 GeoLite2-City.mmdb 文件（可选文件，用户需自行下载）
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
pub fn extract_embedded_resources() -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 释放嵌入的资源...");

    // 需要创建的目录列表（仅创建，不释放文件）
    let dirs = vec![
        "attachments",
        "data",
        "markdown",
    ];

    // 创建必要的目录
    for dir in &dirs {
        fs::create_dir_all(dir)?;
        println!("  ✓ 创建目录: {}", dir);
    }

    // 释放 img 目录中的文件
    println!("  📁 处理 img 目录");
    if let Err(e) = extract_dir("img", "img") {
        eprintln!("  ⚠️  释放 img 目录失败: {}", e);
    }

    // 释放 music 目录中的文件
    println!("  📁 处理 music 目录");
    if let Err(e) = extract_dir("music", "music") {
        eprintln!("  ⚠️  释放 music 目录失败: {}", e);
    }

    println!("✅ 资源释放完成");
    Ok(())
}

/// 从嵌入的文件系统中提取目录
fn extract_dir(src_dir: &str, dst_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 创建目标目录
    fs::create_dir_all(dst_dir)?;

    let mut extracted_count = 0;
    let mut skipped_count = 0;

    // 遍历嵌入文件系统中的文件
    for path in EmbeddedAssets::iter() {
        let path_str = path.as_ref();
        
        // 只处理目标目录下的文件
        if path_str.starts_with(src_dir) {
            let relative_path = path_str.strip_prefix(src_dir).unwrap();
            // 移除可能的前导斜杠
            let relative_path = relative_path.strip_prefix('/').unwrap_or(relative_path);
            let dst_path = Path::new(dst_dir).join(relative_path);

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
                    Ok(_) => extracted_count += 1,
                    Err(e) => {
                        eprintln!("    - 无法写入文件 {}: {}", dst_path.display(), e);
                    }
                }
            }
        }
    }

    if extracted_count > 0 {
        println!("    - 提取了 {} 个文件，跳过 {} 个已存在的文件", extracted_count, skipped_count);
    } else {
        println!("    - 所有 {} 个文件已存在，跳过提取", skipped_count);
    }

    Ok(())
}

/// 检查嵌入的文件是否存在
pub fn has_embedded_file(path: &str) -> bool {
    EmbeddedAssets::get(path).is_some()
}

/// 获取嵌入的文件内容
pub fn get_embedded_file(path: &str) -> Option<Vec<u8>> {
    EmbeddedAssets::get(path).map(|f| f.data.to_vec())
}