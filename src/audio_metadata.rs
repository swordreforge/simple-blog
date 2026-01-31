use std::path::Path;

/// 音频元数据
#[derive(Debug, Clone)]
pub struct AudioMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
}

/// 音频格式枚举
#[derive(Debug, Clone, PartialEq)]
pub enum AudioFormat {
    Mp3,
    Flac,
    Ogg,
    Wav,
    Unknown,
}

impl AudioFormat {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "mp3" => AudioFormat::Mp3,
            "flac" => AudioFormat::Flac,
            "ogg" => AudioFormat::Ogg,
            "wav" => AudioFormat::Wav,
            _ => AudioFormat::Unknown,
        }
    }
}

/// 提取音频元数据
pub fn extract_metadata(file_path: &str) -> Result<AudioMetadata, String> {
    let path = Path::new(file_path);
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let format = AudioFormat::from_extension(extension);

    match format {
        AudioFormat::Mp3 => extract_mp3_metadata(file_path),
        AudioFormat::Flac => extract_flac_metadata(file_path),
        AudioFormat::Ogg => extract_ogg_metadata(file_path),
        AudioFormat::Wav => extract_wav_metadata(file_path),
        AudioFormat::Unknown => {
            // 尝试自动检测
            if let Ok(metadata) = extract_mp3_metadata(file_path) {
                return Ok(metadata);
            }
            if let Ok(metadata) = extract_flac_metadata(file_path) {
                return Ok(metadata);
            }
            Ok(AudioMetadata {
                title: None,
                artist: None,
            })
        }
    }
}

/// 提取 MP3 元数据
fn extract_mp3_metadata(file_path: &str) -> Result<AudioMetadata, String> {
    let tag = id3::Tag::read_from_path(file_path)
        .map_err(|e| format!("无法读取 MP3 文件: {}", e))?;

    Ok(AudioMetadata {
        title: tag.title().map(|s| s.to_string()),
        artist: tag.artist().map(|s| s.to_string()),
    })
}

/// 提取 FLAC 元数据
fn extract_flac_metadata(file_path: &str) -> Result<AudioMetadata, String> {
    let tag = metaflac::Tag::read_from_path(file_path)
        .map_err(|e| format!("无法读取 FLAC 文件: {}", e))?;

    let comments = match tag.vorbis_comments() {
        Some(c) => c,
        None => {
            return Ok(AudioMetadata {
                title: None,
                artist: None,
            });
        }
    };

    Ok(AudioMetadata {
        title: comments.get("TITLE")
            .and_then(|v: &Vec<String>| v.first())
            .map(|s: &String| s.to_string()),
        artist: comments.get("ARTIST")
            .and_then(|v: &Vec<String>| v.first())
            .map(|s: &String| s.to_string()),
    })
}

/// 提取 OGG 元数据
fn extract_ogg_metadata(file_path: &str) -> Result<AudioMetadata, String> {
    // OGG 可以尝试用 FLAC 库读取
    extract_flac_metadata(file_path)
}

/// 提取 WAV 元数据
fn extract_wav_metadata(file_path: &str) -> Result<AudioMetadata, String> {
    // WAV 文件通常没有 ID3 标签，尝试读取
    if let Ok(tag) = id3::Tag::read_from_path(file_path) {
        return Ok(AudioMetadata {
            title: tag.title().map(|s| s.to_string()),
            artist: tag.artist().map(|s| s.to_string()),
        });
    }

    Ok(AudioMetadata {
        title: None,
        artist: None,
    })
}

/// 使用文件名作为回退
pub fn fallback_metadata(filename: &str) -> AudioMetadata {
    let title = filename
        .trim_end_matches(".mp3")
        .trim_end_matches(".flac")
        .trim_end_matches(".ogg")
        .trim_end_matches(".wav")
        .to_string();

    AudioMetadata {
        title: Some(title),
        artist: Some("未知艺术家".to_string()),
    }
}