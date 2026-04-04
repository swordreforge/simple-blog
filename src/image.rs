use anyhow::{Context, Result};
use std::path::Path;
use zenwebp::{EncodeRequest, LossyConfig, PixelLayout};

pub async fn convert_to_webp(input_path: &Path, output_path: &Path) -> Result<()> {
    let input_path = input_path.to_path_buf();
    let output_path = output_path.to_path_buf();

    // Use blocking task for image conversion since it's CPU intensive
    tokio::task::block_in_place(|| {
        // 1. Load the image using image library (supports PNG, JPEG, GIF, etc.)
        let img = image::open(&input_path)
            .context(format!("Failed to load image: {:?}", input_path))?;

        // 2. Convert to RGBA8 format for zenwebp encoding
        let rgba_img = img.to_rgba8();
        let (width, height) = (rgba_img.width(), rgba_img.height());

        // 3. Get raw pixel data
        let rgba_pixels = rgba_img.into_raw();

        // 4. Configure zenwebp lossy encoder with quality (0-100)
        let config = LossyConfig::new().with_quality(85.0);

        // 5. Encode to WebP
        let webp_data = EncodeRequest::lossy(
            &config,
            &rgba_pixels,
            PixelLayout::Rgba8,
            width,
            height,
        )
        .encode()
        .context("Failed to encode WebP image")?;

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
        .to_lowercase()
}

pub fn get_supported_extensions() -> &'static [&'static str] {
    &["jpg", "jpeg", "png", "gif", "webp"]
}

pub fn is_image_file(filename: &str) -> bool {
    let ext = get_file_extension(filename);
    get_supported_extensions().contains(&ext.as_str())
}