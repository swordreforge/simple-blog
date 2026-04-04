use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
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
    const MIN_QUALITY: f32 = 20.0; // 设置最小质量，避免过度压缩导致偏绿
    let mut quality_range: RangeInclusive<f32> = MIN_QUALITY..=100.0;
    let mut best_data: Option<Vec<u8>> = None;
    let mut best_diff = f32::INFINITY;

    for _ in 0..max_attempts {
        let mid = (quality_range.start() + quality_range.end()) / 2.0;

        // 根据质量范围调整编码方法，优化色度采样和亮度保留
        // method 0-6: 0=最快，6=最慢但质量最好
        // 更高的 method 值会使用更好的色度采样策略，保留更多颜色信息
        let (method, _description) = match mid {
            q if q >= 70.0 => (6, "best"),      // 高质量：使用最佳方法，保留完整色度信息
            q if q >= 50.0 => (5, "high"),      // 高质量：使用较好的方法
            q if q >= 40.0 => (4, "medium"),    // 中等质量：平衡速度和质量
            q if q >= 30.0 => (3, "balanced"),  // 中低质量：平衡
            _ => (2, "low"),                    // 低质量：但质量不低于 MIN_QUALITY
        };

        let config = LossyConfig::new()
            .with_quality(mid)
            .with_method(method);

        let encoded = EncodeRequest::lossy(&config, rgb, PixelLayout::Rgb8, width, height)
            .encode()
            .context("Failed to encode WebP during quality estimation")?;
        let current_size = encoded.len();
        let diff = (current_size as f32 - target_size as f32).abs();

        // 只在找到更好的结果时才更新，避免不必要的克隆
        if diff < best_diff {
            best_diff = diff;
            best_data = Some(encoded);  // 直接转移所有权，不克隆
        }

        // Early termination if within tolerance (and not exceeding target)
        if current_size <= target_size && (target_size - current_size) <= tolerance_size {
            tracing::info!("Target size reached with tolerance: {} KB (target: {} KB)",
                current_size / 1024, target_size / 1024);
            return Ok(best_data.unwrap());  // 从best_data中取出最佳结果
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

    // 如果无法达到目标大小（因为设置了最小质量限制），返回最佳结果并警告
    if let Some(data) = best_data {
        if data.len() > target_size {
            tracing::warn!(
                "Cannot reach target size {} KB without excessive quality loss. Best result: {} KB (quality >= {})",
                target_size / 1024,
                data.len() / 1024,
                MIN_QUALITY
            );
        }
        return Ok(data);
    }

    Err(anyhow::anyhow!("Failed to estimate quality"))
}

/// 将 RGBA 图像与白色背景合成，直接生成 RGB 像素数据
/// 避免创建中间图像缓冲区，减少内存分配
fn rgba_to_rgb_with_white_bg(rgba: &image::RgbaImage) -> Vec<u8> {
    let (width, height) = (rgba.width(), rgba.height());
    let pixel_count = (width * height) as usize;
    let mut rgb_pixels = Vec::with_capacity(pixel_count * 3);

    for pixel in rgba.pixels() {
        let alpha = pixel[3] as f32 / 255.0;
        let inv_alpha = 1.0 - alpha;
        let r = (pixel[0] as f32 * alpha + 255.0 * inv_alpha) as u8;
        let g = (pixel[1] as f32 * alpha + 255.0 * inv_alpha) as u8;
        let b = (pixel[2] as f32 * alpha + 255.0 * inv_alpha) as u8;
        rgb_pixels.push(r);
        rgb_pixels.push(g);
        rgb_pixels.push(b);
    }

    rgb_pixels
}

pub async fn convert_to_webp(input_path: &Path, output_path: &Path, max_size: usize) -> Result<()> {
    let input_path = input_path.to_path_buf();
    let output_path = output_path.to_path_buf();

    // Use blocking task for image conversion since it's CPU intensive
    tokio::task::block_in_place(|| {
        // 1. Load the image using image library (supports PNG, JPEG, GIF, etc.)
        let img = image::open(&input_path)
            .context(format!("Failed to load image: {:?}", input_path))?;

        let (width, height) = (img.width(), img.height());

        // 2. 根据原图颜色类型处理，避免不必要的内存拷贝
        let rgb_pixels: Vec<u8> = match img.color() {
            // 如果原图已经是RGB格式，直接使用
            image::ColorType::Rgb8 => {
                tracing::debug!("Original image is RGB8, using directly");
                img.to_rgb8().into_raw()
            }
            // 如果是RGBA格式，需要与白色背景合成
            image::ColorType::Rgba8 => {
                tracing::debug!("Original image is RGBA8, converting to RGB with white background");
                let rgba_img = img.to_rgba8();
                rgba_to_rgb_with_white_bg(&rgba_img)
            }
            // 其他格式，转换为RGB
            _ => {
                tracing::debug!("Original image is {:?}, converting to RGB", img.color());
                img.to_rgb8().into_raw()
            }
        };

        // 5. Encode to WebP
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

        // 6. Write WebP data to file
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
        .to_ascii_lowercase()
}

pub fn get_supported_extensions() -> &'static [&'static str] {
    &["jpg", "jpeg", "png", "gif", "webp"]
}

pub fn is_image_file(filename: &str) -> bool {
    let ext = filename
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    get_supported_extensions().contains(&ext.as_str())
}

/// 计算文件的 SHA256 哈希值
/// 使用流式读取分块计算哈希值，避免将整个文件加载到内存
pub fn calculate_hash(file_path: &Path) -> Result<String> {
    use std::io::Read;

    let file = std::fs::File::open(file_path)
        .context(format!("Failed to open file: {:?}", file_path))?;

    let mut reader = std::io::BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192]; // 8KB 缓冲区，平衡内存和性能

    loop {
        let n = reader.read(&mut buffer)
            .context(format!("Failed to read file: {:?}", file_path))?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    let hash = hasher.finalize();
    // 使用 hex::encode 替代 format!，性能更好
    Ok(hex::encode(hash))
}
