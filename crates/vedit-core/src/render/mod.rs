pub mod audio;
pub mod video;
pub mod text;
pub mod compositor;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Formatos de salida de video
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VideoFormat {
    Mp4,
    Mkv,
    Mov,
}

impl std::fmt::Display for VideoFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VideoFormat::Mp4 => write!(f, "mp4"),
            VideoFormat::Mkv => write!(f, "mkv"),
            VideoFormat::Mov => write!(f, "mov"),
        }
    }
}

/// Formatos de salida de solo audio
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioFormat {
    Mp3,
    Wav,
    Aac,
    Flac,
    Ogg,
}

impl std::fmt::Display for AudioFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioFormat::Mp3 => write!(f, "mp3"),
            AudioFormat::Wav => write!(f, "wav"),
            AudioFormat::Aac => write!(f, "aac"),
            AudioFormat::Flac => write!(f, "flac"),
            AudioFormat::Ogg => write!(f, "ogg"),
        }
    }
}

/// Relación de aspecto objetivo
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AspectRatio {
    /// 16:9 panorámica
    Widescreen,
    /// 9:16 vertical (reels/shorts)
    Portrait,
}

impl AspectRatio {
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            AspectRatio::Widescreen => (1920, 1080),
            AspectRatio::Portrait   => (1080, 1920),
        }
    }
}

/// Trabajo de renderizado
#[derive(Debug, Clone)]
pub struct RenderJob {
    pub project_path: PathBuf,
    pub output_path: PathBuf,
    pub audio_only: bool,
    pub video_format: Option<VideoFormat>,
    pub audio_format: Option<AudioFormat>,
    pub aspect: Option<AspectRatio>,
}

/// Resultado del renderizado
#[derive(Debug)]
pub struct RenderOutput {
    pub output_path: PathBuf,
    pub duration_secs: f64,
    pub size_bytes: u64,
}
