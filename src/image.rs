use anyhow::{Context, Result};
use std::ops::RangeInclusive;
use std::path::Path;
use zenwebp::{EncodeRequest, LossyConfig, PixelLayout};

/// Estimate quality by target file size using binary search
fn estimate_quality_by_target_size(
    rgb: &[u8],
    width: u32,
    height: u32,
    target_size: usize,
    max_attempts: u32,
) -> Result<Vec<u8>> {
    let tolerance = 0.1; // 10% tolerance
    let tolerance_size = (target_size as f32 * tolerance) as usize;
    let mut quality_range: RangeInclusive<f32> = 0.0..=100.0;
    let mut best_data = None;
    let mut best_diff = f32::INFINITY;

    for _ in 0..max_attempts {
        let mid = (quality_range.start() + quality_range.end()) / 2.0;
        let config = LossyConfig::new().with_quality(mid);
        let encoded = EncodeRequest::lossy(&config, rgb, PixelLayout::Rgb8, width, height)
            .encode()
            .context("Failed to encode WebP during quality estimation")?;
        let current_size = encoded.len();
        let diff = (current_size as f32 - target_size as f32).abs();

        if diff < best_diff {
            best_diff = diff;
            best_data = Some(encoded.clone());
        }

        // Early termination if within tolerance (and not exceeding target)
        if current_size <= target_size && (target_size - current_size) <= tolerance_size {
            tracing::info!("Target size reached with tolerance: {} KB (target: {} KB)",
                current_size / 1024, target_size / 1024);
            return Ok(encoded);
        }

        if current_size > target_size {
            quality_range = *quality_range.start()..=mid; // File too large, reduce quality
        } else {
            quality_range = mid..=*quality_range.end(); // File too small, increase quality
        }

        if quality_range.end() - quality_range.start() < 0.5 {
            break;
        }
    }

    best_data.ok_or_else(|| anyhow::anyhow!("Failed to estimate quality"))
}

pub async fn convert_to_webp(input_path: &Path, output_path: &Path, max_size: usize) -> Result<()> {
    let input_path = input_path.to_path_buf();
    let output_path = output_path.to_path_buf();

    // Use blocking task for image conversion since it's CPU intensive
    tokio::task::block_in_place(|| {
        // 1. Load the image using image library (supports PNG, JPEG, GIF, etc.)
        let img = image::open(&input_path)
            .context(format!("Failed to load image: {:?}", input_path))?;

        // 2. Convert to RGB8 format for zenwebp encoding
        // 使用 RGB8 而不是 RGBA8 以避免颜色通道顺序问题
        let rgb_img = img.to_rgb8();
        let (width, height) = (rgb_img.width(), rgb_img.height());

        // 3. Get raw pixel data
        let rgb_pixels = rgb_img.into_raw();

        // 4. Encode to WebP
        let webp_data = if max_size == 0 {
            // No size limit, use high quality (85)
            tracing::info!("No size limit, using quality 85");
            let config = LossyConfig::new().with_quality(85.0);
            EncodeRequest::lossy(&config, &rgb_pixels, PixelLayout::Rgb8, width, height)
                .encode()
                .context("Failed to encode WebP image")?
        } else {
            // Use quality estimation to fit within max_size
            tracing::info!("Target size: {} KB", max_size / 1024);
            estimate_quality_by_target_size(&rgb_pixels, width, height, max_size, 10)
                .context("Failed to estimate quality for target size")?
        };

        // 5. Write WebP data to file
        std::fs::write(&output_path, webp_data)
            .context(format!("Failed to write WebP file: {:?}", output_path))?;

        Ok(())
    })
}

pub fn generate_timestamped_filename(original_name: &str) -> String {
    let timestamp = chrono::Utc::now().timestamp_millis();
    let ext = original_name
        .rsplit('.')
        .next()
        .unwrap_or("webp");
    format!("{}.{}", timestamp, ext)
}

pub fn get_file_extension(filename: &str) -> String {
    filename
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_lowercase()
}

pub fn get_supported_extensions() -> &'static [&'static str] {
    &["jpg", "jpeg", "png", "gif", "webp"]
}

pub fn is_image_file(filename: &str) -> bool {
    let ext = get_file_extension(filename);
    get_supported_extensions().contains(&ext.as_str())
}