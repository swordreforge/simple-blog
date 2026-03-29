use std::env;
use std::path::PathBuf;

fn main() {
    // ── 分配器特征冲突检测 ──────────────────────────────────────────────────
    // 同时启用多个分配器特征会导致每个分配器库均被链接进二进制文件，
    // 造成初始内存开销翻倍（initial-mem ×2）以及潜在的未定义行为。
    // 此处提供最早的检测点；allocator.rs 中的 compile_error! 提供第二道保障。
    let jemalloc = env::var("CARGO_FEATURE_JEMALLOC").is_ok();
    let mimalloc = env::var("CARGO_FEATURE_MIMALLOC_ALLOC").is_ok();
    let tcmalloc = env::var("CARGO_FEATURE_TCMALLOC_ALLOC").is_ok();

    let enabled: Vec<&str> = [
        (jemalloc, "jemalloc"),
        (mimalloc, "mimalloc-alloc"),
        (tcmalloc, "tcmalloc-alloc"),
    ]
    .iter()
    .filter_map(|&(enabled, name)| if enabled { Some(name) } else { None })
    .collect();

    if enabled.len() > 1 {
        panic!(
            "\n\
             ┌─────────────────────────────────────────────────────────────┐\n\
             │              分配器特征冲突（编译错误）                      │\n\
             ├─────────────────────────────────────────────────────────────┤\n\
             │  同时启用了多个分配器特征: [{features}]\n\
             │\n\
             │  问题：每个分配器库均会被链接进二进制文件，\n\
             │        导致初始内存开销翻倍（initial-mem ×2），\n\
             │        并可能引发未定义行为。\n\
             │\n\
             │  解决：在 Cargo 特征中只保留一个分配器特征，\n\
             │        例如：--features jemalloc\n\
             └─────────────────────────────────────────────────────────────┘",
            features = enabled.join(", "),
        );
    }
    // ────────────────────────────────────────────────────────────────────────

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
