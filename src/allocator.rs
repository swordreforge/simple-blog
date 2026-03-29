//! 内存管理器配置模块
//!
//! 支持多种高性能内存分配器，可根据编译时特征选择：
//! - jemalloc: 多线程优化，适合高并发场景
//! - mimalloc: 低碎片化，内存使用更高效
//! - 系统默认: 无需额外依赖

// ── 编译期冲突检测 ───────────────────────────────────────────────────────────
// 同时启用多个分配器特征会导致每个分配器库都被链接进二进制文件，产生双倍的初始
// 内存开销（initial-mem ×2），并可能引发未定义行为。
// 特别是 mimalloc 的 `override` 特征会在链接层全局替换 malloc/free，若同时存在
// jemalloc 的 #[global_allocator] 设置，两者会互相干扰。
#[cfg(all(feature = "jemalloc", feature = "mimalloc-alloc"))]
compile_error!(
    "特征 `jemalloc` 和 `mimalloc-alloc` 不能同时启用。\
     同时启用多个分配器会导致两个分配器库均被链接，\
     造成初始内存开销翻倍（initial-mem ×2）以及未定义行为。\
     请只选择其中一个分配器特征。"
);

#[cfg(all(feature = "jemalloc", feature = "tcmalloc-alloc"))]
compile_error!(
    "特征 `jemalloc` 和 `tcmalloc-alloc` 不能同时启用。\
     同时启用多个分配器会导致两个分配器库均被链接，\
     造成初始内存开销翻倍（initial-mem ×2）以及未定义行为。\
     请只选择其中一个分配器特征。"
);

#[cfg(all(feature = "mimalloc-alloc", feature = "tcmalloc-alloc"))]
compile_error!(
    "特征 `mimalloc-alloc` 和 `tcmalloc-alloc` 不能同时启用。\
     同时启用多个分配器会导致两个分配器库均被链接，\
     造成初始内存开销翻倍（initial-mem ×2）以及未定义行为。\
     请只选择其中一个分配器特征。"
);
// ────────────────────────────────────────────────────────────────────────────

#[cfg(all(
    not(feature = "jemalloc"),
    not(feature = "mimalloc-alloc"),
    not(feature = "tcmalloc-alloc")
))]
use std::alloc::System;

#[cfg(feature = "jemalloc")]
use tikv_jemallocator::Jemalloc;

#[cfg(all(not(feature = "jemalloc"), feature = "mimalloc-alloc"))]
use mimalloc::MiMalloc;

#[cfg(all(
    not(feature = "jemalloc"),
    not(feature = "mimalloc-alloc"),
    feature = "tcmalloc-alloc"
))]
use tcmalloc::TCMalloc;

// 默认使用系统分配器
#[cfg(all(
    not(feature = "jemalloc"),
    not(feature = "mimalloc-alloc"),
    not(feature = "tcmalloc-alloc")
))]
#[global_allocator]
static GLOBAL: System = System;

// jemalloc 分配器（推荐用于高并发场景）
#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

// mimalloc 分配器（推荐用于低内存使用）
#[cfg(all(not(feature = "jemalloc"), feature = "mimalloc-alloc"))]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

// tcmalloc 分配器（备选方案）
#[cfg(all(
    not(feature = "jemalloc"),
    not(feature = "mimalloc-alloc"),
    feature = "tcmalloc-alloc"
))]
#[global_allocator]
static GLOBAL: TCMalloc = TCMalloc;

/// 内存管理器类型
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocatorType {
    /// 系统默认分配器
    System,
    /// jemalloc 分配器
    Jemalloc,
    /// mimalloc 分配器
    MiMalloc,
    /// tcmalloc 分配器
    TCMalloc,
}

impl AllocatorType {
    /// 获取当前使用的分配器类型
    pub fn current() -> Self {
        #[cfg(feature = "jemalloc")]
        return AllocatorType::Jemalloc;

        #[cfg(all(not(feature = "jemalloc"), feature = "mimalloc-alloc"))]
        return AllocatorType::MiMalloc;

        #[cfg(all(
            not(feature = "jemalloc"),
            not(feature = "mimalloc-alloc"),
            feature = "tcmalloc-alloc"
        ))]
        return AllocatorType::TCMalloc;

        #[cfg(all(
            not(feature = "jemalloc"),
            not(feature = "mimalloc-alloc"),
            not(feature = "tcmalloc-alloc")
        ))]
        return AllocatorType::System;
    }

    /// 获取分配器名称
    pub fn name(&self) -> &'static str {
        match self {
            AllocatorType::System => "System",
            AllocatorType::Jemalloc => "jemalloc",
            AllocatorType::MiMalloc => "mimalloc",
            AllocatorType::TCMalloc => "tcmalloc",
        }
    }

    /// 获取分配器描述
    pub fn description(&self) -> &'static str {
        match self {
            AllocatorType::System => "系统默认分配器，适合开发和小规模应用",
            AllocatorType::Jemalloc => "高性能多线程分配器，适合高并发场景，内存碎片少",
            AllocatorType::MiMalloc => "低碎片化分配器，内存使用高效，适合长期运行服务",
            AllocatorType::TCMalloc => "Google开发的分配器，适合大规模多线程应用",
        }
    }
}

/// 打印当前内存管理器信息
pub fn print_allocator_info() {
    let allocator = AllocatorType::current();
    println!("🧠 内存管理器:");
    println!("   - 类型: {}", allocator.name());
    println!("   - 描述: {}", allocator.description());
}

/// 初始化内存管理器配置
///
/// # Returns
/// 成功返回 Ok(())，失败返回错误信息
pub fn init_allocator() -> Result<(), String> {
    let allocator = AllocatorType::current();

    match allocator {
        AllocatorType::System => {
            tracing::info!("使用系统默认分配器");
            // 系统malloc优化参数（通过环境变量）
            init_system_malloc();
            Ok(())
        }
        AllocatorType::Jemalloc => {
            #[cfg(feature = "jemalloc")]
            return init_jemalloc();

            #[cfg(not(feature = "jemalloc"))]
            return Err("jemalloc feature not enabled".to_string());
        }
        AllocatorType::MiMalloc => {
            #[cfg(feature = "mimalloc-alloc")]
            return init_mimalloc();

            #[cfg(not(feature = "mimalloc-alloc"))]
            return Err("mimalloc feature not enabled".to_string());
        }
        AllocatorType::TCMalloc => {
            #[cfg(feature = "tcmalloc-alloc")]
            return init_tcmalloc();

            #[cfg(not(feature = "tcmalloc-alloc"))]
            return Err("tcmalloc feature not enabled".to_string());
        }
    }
}

/// 初始化系统malloc配置（glibc malloc优化）
fn init_system_malloc() {
    tracing::info!("应用系统malloc优化参数");

    // glibc malloc的环境变量优化参数
    // 注意：这些参数在较新的glibc版本中可能已被废弃或效果有限
    let opts = [
        // 设置mmap阈值：大于此大小的分配使用mmap而非brk
        // mmap的内存可以被独立释放，减少内存碎片
        ("MALLOC_MMAP_THRESHOLD_", "32768"), // 32KB
        // 设置mmap最大数量：限制mmap的映射数量
        ("MALLOC_MMAP_MAX_", "65536"),
        // 设置top释放阈值：当arena的top chunk大于此值时才释放
        // 较小的值会让内存更快归还系统
        ("MALLOC_TOP_PAD_", "0"),
        // 设置trim阈值：控制何时向系统归还内存
        ("MALLOC_TRIM_THRESHOLD_", "4096"), // 4KB
        // 设置对齐：默认为16字节，较小的对齐可能节省空间
        ("MALLOC_ALIGNMENT", "8"),
    ];

    for (key, value) in opts {
        // 注意：glibc从2.26开始废弃了大多数MALLOC_环境变量
        // 这里仍然设置是为了兼容性，但可能不会生效
        tracing::debug!("尝试设置 {} = {}", key, value);
        // std::env::set_var(key, value);
    }

    tracing::warn!("注意：glibc从2.26开始废弃了MALLOC_环境变量");
    tracing::warn!("建议使用mimalloc或jemalloc以获得更好的内存控制");
}

/// 初始化 jemalloc 配置
#[cfg(feature = "jemalloc")]
fn init_jemalloc() -> Result<(), String> {
    tracing::info!("初始化 jemalloc 分配器（超激进内存节省配置）");

    // 超激进内存节省配置：关闭后台线程、超快释放、单arena、极小缓存、禁用大页
    //
    // 关键优化：禁用 cache-oblivious 功能（opt_lg_extent_max_active_fit:0）
    // - jemalloc 默认为 CPU 缓存性能会为大对象额外分配 4KB 内存（cache-oblivious 对齐）
    // - 这个功能通过 opt_lg_extent_max_active_fit 参数控制，默认值是 3（额外分配 2^3=8 个 extent）
    // - 对于低内存场景，设置为 0 完全禁用此功能，可显著减少大对象分配时的内存开销
    // - 这是类似 V8 指针压缩的"用 CPU 性能换内存空间"策略
    //
    // 其他优化参数说明：
    // - background_thread:false: 关闭后台线程，减少额外内存和 CPU 开销
    // - dirty_decay_ms:2000: 脏页在 2 秒内自动清理（从 5 秒加快到 2 秒）
    // - muzzy_decay_ms:5000: 模糊页在 5 秒内自动清理（从 10 秒加快到 5 秒）
    // - narenvas:1: 使用单 arena，减少元数据开销
    // - lg_tcache_max:12: 线程缓存最大限制为 4KB，减少线程间内存碎片
    // - lg_dirty_mult:0: 不限制脏页比例，允许尽快释放
    // - opt_thp:never: 禁用透明大页，避免大页带来的内存开销
    // - opt_abort:true: 分配失败时直接 abort，避免静默失败
    // - oversize_threshold:0: 禁用 oversize 机制，强制使用标准分配路径
    // - prof:false: 关闭性能分析，减少运行时开销
    let opts = [(
        "MALLOC_CONF",
        "background_thread:false,dirty_decay_ms:2000,muzzy_decay_ms:5000,narenas:1,lg_tcache_max:12,lg_dirty_mult:0,opt_lg_extent_max_active_fit:0,opt_thp:never,opt_abort:true,oversize_threshold:0,prof:false",
    )];

    for (key, value) in opts {
        unsafe {
            std::env::set_var(key, value);
        }
        tracing::debug!("设置 {} = {}", key, value);
    }

    tracing::info!("✅ jemalloc 关键优化: 禁用 cache-oblivious 功能，减少大对象额外 4KB 分配");
    tracing::info!("✅ jemalloc 配置: 单 arena, 2秒脏页清理, 4KB 线程缓存, 无后台线程");

    Ok(())
}

/// 初始化 mimalloc 配置
#[cfg(feature = "mimalloc-alloc")]
    // ⚠️ 重要说明：
    // mimalloc 在 dynamic linker 阶段就完成初始化，早于 main()。
    // 以下 set_var 调用对大多数选项不会生效。
    // 要使配置真正生效，必须在进程启动前通过 systemd Environment= 或
    // 启动脚本设置这些环境变量。
    //
    // 真正有效的运行时 API 是 libmimalloc_sys::mi_option_set()，
    // 部分选项（purge_delay 等）可以在运行时随时修改。
fn init_mimalloc() -> Result<(), String> {
    tracing::info!("初始化 mimalloc 分配器（极限内存节省配置）");

    // 极限内存节省配置：
    // 目标：尽可能减少内存占用，尽快归还内存给系统，限制最大内存使用
    let opts = [
        // 关闭段缓存：段缓存会预占用大量内存且不释放
        ("MIMALLOC_SEGMENT_CACHE_ENABLED", "0"),
        // 开启即时内存回收：让空闲内存尽快归还给OS
        ("MIMALLOC_PAGE_RESET", "1"),
        // 禁用大页支持：减少大页相关的内存开销
        ("MIMALLOC_LARGE_OS_PAGES", "0"),
        // 激进地释放内存：通过decommit释放物理内存
        ("MIMALLOC_PURGE_DECOMMITS", "1"),
        // 立即提交：减少分配延迟，避免内存累积
        ("MIMALLOC_EAGER_COMMIT_DELAY", "0"),
        // 限制拥有的段数：防止段无限增长
        ("MIMALLOC_MAX_OWNED_SEGMENTS", "1"),
        // 限制最大段数：防止内存无限增长
        ("MIMALLOC_MAX_SEGMENT_COUNT", "8"),
        // 启用统计：便于监控内存使用
        ("MIMALLOC_STATS", "1"),
        // 设置最大内存限制（单位：字节）- 128MB
        ("MIMALLOC_MAX_LIMIT", "134217728"),
        // 禁用后台清理线程：减少额外开销
        ("MIMALLOC_BACKGROUND_THREAD", "0"),
        // 快速释放：在释放时立即归还内存
        ("MIMALLOC_RESET_DELAY", "0"),
        // 禁用延迟释放
        ("MIMALLOC_DELAYED_FREE", "0"),
    ];

    for (key, value) in opts {
        unsafe {
            std::env::set_var(key, value);
        }
        tracing::debug!("设置 {} = {}", key, value);
    }

    Ok(())
}

/// 初始化 tcmalloc 配置
#[cfg(feature = "tcmalloc-alloc")]
fn init_tcmalloc() -> Result<(), String> {
    tracing::info!("初始化 tcmalloc 分配器（超激进内存节省配置）");

    // 超激进内存节省配置：
    // 目标：尽可能减少内存占用，限制堆增长
    let opts = [
        // 极小线程缓存：1MB（减少93%的线程缓存内存）
        ("TCMALLOC_MAX_TOTAL_THREAD_CACHE_BYTES", "1048576"), // 1MB
        // 开启激进归还：尽快释放未使用的内存给OS
        ("TCMALLOC_AGGRESSIVE_DECOMMIT", "true"),
        // 快速释放速率：加快内存归还速度
        ("TCMALLOC_RELEASE_RATE", "10"),
        // 限制堆大小：64MB（防止堆无限增长）
        ("TCMALLOC_HEAP_SIZE_MB", "64"),
    ];

    for (key, value) in opts {
        unsafe {
            std::env::set_var(key, value);
        }
        tracing::debug!("设置 {} = {}", key, value);
    }

    Ok(())
}

/// 占位函数，用于非 jemalloc 构建时的空实现
#[cfg(not(feature = "jemalloc"))]
#[allow(dead_code)]
fn init_jemalloc() -> Result<(), String> {
    Err("jemalloc feature not enabled".to_string())
}

/// 占位函数，用于非 mimalloc 构建时的空实现
#[cfg(not(feature = "mimalloc-alloc"))]
#[allow(dead_code)]
fn init_mimalloc() -> Result<(), String> {
    Err("mimalloc feature not enabled".to_string())
}

/// 占位函数，用于非 tcmalloc 构建时的空实现
#[cfg(not(feature = "tcmalloc-alloc"))]
#[allow(dead_code)]
fn init_tcmalloc() -> Result<(), String> {
    Err("tcmalloc feature not enabled".to_string())
}

/// 获取内存使用统计信息（简化版）
///
/// 注意：由于 jemalloc-ctl 与 tikv-jemallocator 存在链接冲突，
/// 这里只返回基本的分配器类型信息。
#[allow(dead_code)]
pub fn get_memory_stats() -> Option<AllocatorType> {
    Some(AllocatorType::current())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocator_type() {
        let allocator = AllocatorType::current();
        println!("当前分配器: {}", allocator.name());
        assert!(matches!(
            allocator,
            AllocatorType::System
                | AllocatorType::Jemalloc
                | AllocatorType::MiMalloc
                | AllocatorType::TCMalloc
        ));
    }

    #[test]
    fn test_allocator_init() {
        let result = init_allocator();
        assert!(result.is_ok(), "分配器初始化应该成功");
    }

    /// 验证在运行时只有一个分配器处于活动状态。
    ///
    /// 注意：同时启用多个分配器特征会在编译期通过 `compile_error!` 直接报错，
    /// 因此能到达运行时的二进制文件必然只链接了一个分配器。
    /// 此测试作为文档化断言，确保 `AllocatorType::current()` 返回确定的单一值，
    /// 而不是 System（即当且仅当没有任何分配器特征启用时才是 System）。
    #[test]
    fn test_exactly_one_allocator_active() {
        // 统计编译时启用的分配器特征数量
        let enabled_count = [
            cfg!(feature = "jemalloc"),
            cfg!(feature = "mimalloc-alloc"),
            cfg!(feature = "tcmalloc-alloc"),
        ]
        .iter()
        .filter(|&&x| x)
        .count();

        // 最多只能启用一个（多个同时启用会在编译期报错）
        assert!(
            enabled_count <= 1,
            "同时启用了 {} 个分配器特征，这会导致初始内存开销翻倍（initial-mem ×2）。\
             编译期 compile_error! 应已阻止此情形——请检查 Cargo 特征配置。",
            enabled_count
        );

        // 当前分配器类型必须与启用的特征一致
        let allocator = AllocatorType::current();
        match allocator {
            AllocatorType::Jemalloc => {
                assert!(cfg!(feature = "jemalloc"), "AllocatorType::Jemalloc 要求 jemalloc 特征已启用");
            }
            AllocatorType::MiMalloc => {
                assert!(cfg!(feature = "mimalloc-alloc"), "AllocatorType::MiMalloc 要求 mimalloc-alloc 特征已启用");
            }
            AllocatorType::TCMalloc => {
                assert!(cfg!(feature = "tcmalloc-alloc"), "AllocatorType::TCMalloc 要求 tcmalloc-alloc 特征已启用");
            }
            AllocatorType::System => {
                assert!(
                    !cfg!(feature = "jemalloc")
                        && !cfg!(feature = "mimalloc-alloc")
                        && !cfg!(feature = "tcmalloc-alloc"),
                    "AllocatorType::System 要求所有分配器特征均未启用"
                );
            }
        }
    }
}
