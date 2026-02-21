use anyhow::{Context, Result};
use rand::Rng;
use std::path::{Path, PathBuf};
use tokio::process::Command;

pub async fn convert_to_webp(input_path: &Path, output_path: &Path) -> Result<()> {
    let output = Command::new("ffmpeg")
        .args([
            "-i",
            input_path.to_str().unwrap(),
            "-c:v",
            "libwebp",
            "-quality",
            "85",
            "-y",
            output_path.to_str().unwrap(),
        ])
        .output()
        .await
        .context("Failed to execute ffmpeg")?;

    if !output.status.success() {
        anyhow::bail!(
            "FFmpeg conversion failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
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

pub fn is_webp_file(filename: &str) -> bool {
    get_file_extension(filename) == "webp"
}

pub fn get_filename_without_ext(filename: &str) -> String {
    let last_dot = filename.rfind('.');
    match last_dot {
        Some(dot) => filename[..dot].to_string(),
        None => filename.to_string(),
    }
}

pub async fn download_image(url: &str, output_path: &Path) -> Result<()> {
    let response = reqwest::get(url).await?;

    if !response.status().is_success() {
        anyhow::bail!("HTTP error: {}", response.status());
    }

    let bytes = response.bytes().await?;

    if let Some(parent) = output_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    tokio::fs::write(output_path, bytes).await?;

    Ok(())
}

pub fn get_supported_extensions() -> &'static [&'static str] {
    &["jpg", "jpeg", "png", "gif", "webp"]
}

pub fn is_image_file(filename: &str) -> bool {
    let ext = get_file_extension(filename);
    get_supported_extensions().contains(&ext.as_str())
}