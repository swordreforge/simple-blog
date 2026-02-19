use std::env;
use std::path::PathBuf;

fn main() {
    // 获取 Cargo.toml 所在的目录
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    
    // 设置 rust-embed 的工作目录为项目根目录
    // 这样无论从哪个目录运行 cargo build，都能正确嵌入文件
    println!("cargo:rustc-env=RUSTBLOG_ROOT={}", manifest_dir.display());
    
    // 重新运行 build.rs 当以下文件变化时
    println!("cargo:rerun-if-changed=templates");
    println!("cargo:rerun-if-changed=img");
    println!("cargo:rerun-if-changed=music");
}