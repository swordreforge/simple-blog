/// 性能分析模块
/// 
/// 使用 pprof-rs 进行性能分析和火焰图生成
/// 
/// 使用方式:
/// 1. 启用 profiling feature: cargo build --release --features profiling
/// 2. 运行程序: ./rustblog --enable-profiling
/// 3. 访问应用程序触发性能数据收集
/// 4. 按 Ctrl+C 停止程序，自动生成火焰图

#[cfg(feature = "profiling")]
use pprof::{ProfilerGuard, ProfilerGuardBuilder};

use std::path::PathBuf;

#[cfg(feature = "profiling")]
pub struct ProfilingManager {
    guard: Option<ProfilerGuard<'static>>,
    enabled: bool,
    output_dir: PathBuf,
}

#[cfg(feature = "profiling")]
#[allow(dead_code)]
impl ProfilingManager {
    /// 创建性能分析管理器
    pub fn new(output_dir: PathBuf) -> Self {
        Self {
            guard: None,
            enabled: false,
            output_dir,
        }
    }

    /// 启用性能分析
    pub fn enable(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.enabled {
            return Ok(());
        }

        // 创建输出目录
        std::fs::create_dir_all(&self.output_dir)?;

        // 创建 ProfilerGuard
        self.guard = Some(
            ProfilerGuardBuilder::default()
                .frequency(1000) // 采样频率：1000 Hz
                .blocklist(&["libc", "libgcc", "pthread"])
                .build()?
        );

        self.enabled = true;
        println!("🔥 性能分析已启用，采样频率: 1000 Hz");
        println!("📊 火焰图将保存到: {}", self.output_dir.display());

        Ok(())
    }

    /// 禁用性能分析并生成报告
    pub fn disable_and_generate_report(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.enabled {
            return Ok(());
        }

        // 停止性能分析
        if let Some(guard) = self.guard.take() {
            self.enabled = false;

            // 生成火焰图
            let report_path = self.output_dir.join("flamegraph.svg");
            let mut file = std::fs::File::create(&report_path)?;
            guard.report().build().unwrap().flamegraph(&mut file)?;
            println!("🔥 火焰图已生成: {}", report_path.display());
        }

        Ok(())
    }
}

#[cfg(feature = "profiling")]
impl Drop for ProfilingManager {
    fn drop(&mut self) {
        if self.enabled {
            let _ = self.disable_and_generate_report();
        }
    }
}

#[cfg(not(feature = "profiling"))]
pub struct ProfilingManager;

#[cfg(not(feature = "profiling"))]
#[allow(dead_code)]
impl ProfilingManager {
    pub fn new(_output_dir: PathBuf) -> Self {
        println!("⚠️  性能分析功能未启用，请使用 --features profiling 编译");
        Self
    }

    pub fn enable(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Err("性能分析功能未启用".into())
    }

    pub fn disable_and_generate_report(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}