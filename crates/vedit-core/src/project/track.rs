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
    /// Orden de superposición (solo para tracks de imagen)
    pub layer_order: usize,
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
            layer_order: 0,
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
        let image_end = self.image_clips.iter()
            .map(|c| c.timeline_start + c.duration())
            .fold(0.0_f64, f64::max);
        audio_end.max(video_end).max(image_end)
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

    // ── Image clips ────────────────────────────────────────────────────────
    pub fn add_image_clip(&mut self, clip: ImageClip) -> Uuid {
        let id = clip.id;
        self.image_clips.push(clip);
        id
    }

    pub fn remove_image_clip(&mut self, id: Uuid) -> bool {
        let before = self.image_clips.len();
        self.image_clips.retain(|c| c.id != id);
        self.image_clips.len() < before
    }

    pub fn image_clip_mut(&mut self, id: Uuid) -> Option<&mut ImageClip> {
        self.image_clips.iter_mut().find(|c| c.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_kind_display() {
        assert_eq!(TrackKind::Audio.to_string(), "audio");
        assert_eq!(TrackKind::Video.to_string(), "video");
        assert_eq!(TrackKind::Image.to_string(), "image");
    }

    #[test]
    fn test_track_creation() {
        let track = Track::new(TrackKind::Video, "My Video Track");
        assert_eq!(track.name, "My Video Track");
        assert_eq!(track.kind, TrackKind::Video);
        assert_eq!(track.volume, 1.0);
        assert_eq!(track.muted, false);
        assert_eq!(track.audio_clips.len(), 0);
        assert_eq!(track.video_clips.len(), 0);
        assert_eq!(track.image_clips.len(), 0);
    }

    #[test]
    fn test_track_set_volume() {
        let mut track = Track::new(TrackKind::Audio, "Audio Track");
        
        // Normal case
        track.set_volume(0.5);
        assert_eq!(track.volume, 0.5);
        
        // Edge case: max limit
        track.set_volume(2.5);
        assert_eq!(track.volume, 2.0);
        
        // Edge case: min limit
        track.set_volume(-1.0);
        assert_eq!(track.volume, 0.0);
    }

    #[test]
    fn test_track_rename_and_mute() {
        let mut track = Track::new(TrackKind::Audio, "Audio");
        
        track.rename("New Name");
        assert_eq!(track.name, "New Name");
        
        assert_eq!(track.muted, false);
        track.mute();
        assert_eq!(track.muted, true);
        track.unmute();
        assert_eq!(track.muted, false);
    }

    #[test]
    fn test_track_duration_secs() {
        let mut track = Track::new(TrackKind::Video, "Mixed Track");
        
        assert_eq!(track.duration_secs(), 0.0);
        
        // Add video clip ending at 10.0
        let mut v_clip = VideoClip::new("video", "v.mp4", 0.0);
        v_clip.source_end = Some(10.0);
        track.add_video_clip(v_clip);
        assert_eq!(track.duration_secs(), 10.0);
        
        // Add audio clip ending at 15.0
        let mut a_clip = AudioClip::new("audio", "a.wav", 5.0);
        a_clip.source_end = Some(10.0); // duration = 10.0
        track.add_audio_clip(a_clip); // ends at 5.0 + 10.0 = 15.0
        assert_eq!(track.duration_secs(), 15.0);
        
        // Add image clip ending at 8.0
        let i_clip = ImageClip::new("img", "img.png", 3.0, 5.0); // ends at 8.0
        track.add_image_clip(i_clip);
        assert_eq!(track.duration_secs(), 15.0); // max is still 15.0
    }

    #[test]
    fn test_track_clip_management() {
        let mut track = Track::new(TrackKind::Video, "Video Track");
        
        let clip = VideoClip::new("vid", "v.mp4", 0.0);
        let id = track.add_video_clip(clip.clone());
        
        assert_eq!(track.video_clips.len(), 1);
        
        // Mutate clip
        let mut_clip = track.video_clip_mut(id).unwrap();
        mut_clip.name = "Renamed".to_string();
        assert_eq!(track.video_clips[0].name, "Renamed");
        
        // Remove clip
        assert!(track.remove_video_clip(id));
        assert_eq!(track.video_clips.len(), 0);
        
        // Remove non-existent clip
        assert!(!track.remove_video_clip(Uuid::new_v4()));
    }
}
