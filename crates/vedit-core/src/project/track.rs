use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::project::clip::{AudioClip, VideoClip, ImageClip};

/// Tipo de track
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrackKind {
    Audio,
    Video,
    Image,
}

impl std::fmt::Display for TrackKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrackKind::Audio => write!(f, "audio"),
            TrackKind::Video => write!(f, "video"),
            TrackKind::Image => write!(f, "image"),
        }
    }
}

/// Track de audio/video/imagen dentro del proyecto
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: Uuid,
    pub name: String,
    pub kind: TrackKind,
    /// Volumen base 0.0–2.0 (1.0 = 100%)
    pub volume: f64,
    pub muted: bool,
    /// Orden en el timeline (menor = primero)
    pub order: usize,
    pub audio_clips: Vec<AudioClip>,
    pub video_clips: Vec<VideoClip>,
    pub image_clips: Vec<ImageClip>,
}

impl Track {
    pub fn new(kind: TrackKind, name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            kind,
            volume: 1.0,
            muted: false,
            order: 0,
            audio_clips: Vec::new(),
            video_clips: Vec::new(),
            image_clips: Vec::new(),
        }
    }

    /// Duración total del track (fin del último clip)
    pub fn duration_secs(&self) -> f64 {
        let audio_end = self.audio_clips.iter()
            .map(|c| c.timeline_start + c.duration())
            .fold(0.0_f64, f64::max);
        let video_end = self.video_clips.iter()
            .map(|c| c.timeline_start + c.duration())
            .fold(0.0_f64, f64::max);
        audio_end.max(video_end)
    }

    pub fn mute(&mut self) { self.muted = true; }
    pub fn unmute(&mut self) { self.muted = false; }

    pub fn set_volume(&mut self, vol: f64) {
        self.volume = vol.max(0.0).min(2.0);
    }

    pub fn rename(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    // ── Audio clips ────────────────────────────────────────────────────────
    pub fn add_audio_clip(&mut self, clip: AudioClip) -> Uuid {
        let id = clip.id;
        self.audio_clips.push(clip);
        id
    }

    pub fn remove_audio_clip(&mut self, id: Uuid) -> bool {
        let before = self.audio_clips.len();
        self.audio_clips.retain(|c| c.id != id);
        self.audio_clips.len() < before
    }

    pub fn audio_clip_mut(&mut self, id: Uuid) -> Option<&mut AudioClip> {
        self.audio_clips.iter_mut().find(|c| c.id == id)
    }

    // ── Video clips ────────────────────────────────────────────────────────
    pub fn add_video_clip(&mut self, clip: VideoClip) -> Uuid {
        let id = clip.id;
        self.video_clips.push(clip);
        id
    }

    pub fn remove_video_clip(&mut self, id: Uuid) -> bool {
        let before = self.video_clips.len();
        self.video_clips.retain(|c| c.id != id);
        self.video_clips.len() < before
    }

    pub fn video_clip_mut(&mut self, id: Uuid) -> Option<&mut VideoClip> {
        self.video_clips.iter_mut().find(|c| c.id == id)
    }
}
