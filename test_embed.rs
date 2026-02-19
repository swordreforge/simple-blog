use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "."]
#[include = "templates/**"]
pub struct TestEmbed;

fn main() {
    println!("=== 测试嵌入文件 ===");
    println!("\n所有嵌入的文件:");
    let mut count = 0;
    for path in TestEmbed::iter() {
        println!("  - {}", path.as_ref());
        count += 1;
    }
    println!("\n总共: {} 个文件", count);

    // 检查 index.html
    println!("\n检查 index.html:");
    if let Some(content) = TestEmbed::get("templates/index.html") {
        println!("  ✓ 找到了 index.html ({} bytes)", content.data.len());
    } else {
        println!("  ✗ 没有找到 index.html");
    }
}