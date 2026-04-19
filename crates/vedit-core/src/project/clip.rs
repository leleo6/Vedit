use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─────────────────────────────────────────────────────────────────────────────
// Fade effect
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FadeEffect {
    /// Duración del fade en segundos
    pub duration_secs: f64,
}

// ─────────────────────────────────────────────────────────────────────────────
// AudioClip
// ─────────────────────────────────────────────────────────────────────────────

/// Clip de audio dentro de un track
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioClip {
    pub id: Uuid,
    /// Nombre descriptivo del clip
    pub name: String,
    /// Archivo fuente en disco
    pub source_path: PathBuf,
    /// Inicio en el archivo fuente (segundos)
    pub source_start: f64,
    /// Fin en el archivo fuente (segundos, None = hasta el final)
    pub source_end: Option<f64>,
    /// Posición de inicio en el timeline del proyecto (segundos)
    pub timeline_start: f64,
    /// Volumen individual 0.0–2.0 (1.0 = 100%)
    pub volume: f64,
    /// Repetir N veces (1 = sin loop)
    pub loop_count: u32,
    /// Speed/pitch factor (1.0 = normal)
    pub speed: f64,
    pub fade_in: Option<FadeEffect>,
    pub fade_out: Option<FadeEffect>,
    /// Rangos de silencio en segundos relativos al clip [(start, end)]
    pub mute_ranges: Vec<(f64, f64)>,
}

impl AudioClip {
    pub fn new(name: impl Into<String>, source_path: impl Into<PathBuf>, timeline_start: f64) -> Self {
        let path = source_path.into();
        let path = std::fs::canonicalize(&path).unwrap_or(path);
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            source_path: path,
            source_start: 0.0,
            source_end: None,
            timeline_start,
            volume: 1.0,
            loop_count: 1,
            speed: 1.0,
            fade_in: None,
            fade_out: None,
            mute_ranges: Vec::new(),
        }
    }

    /// Duración neta del clip en el timeline (sin loop)
    pub fn raw_duration(&self) -> f64 {
        match self.source_end {
            Some(end) => (end - self.source_start).max(0.0),
            None => 0.0, // desconocida hasta probar con ffprobe
        }
    }

    /// Duración total considerando loops
    pub fn duration(&self) -> f64 {
        self.raw_duration() * self.loop_count as f64
    }

    pub fn set_fade_in(&mut self, secs: f64) {
        self.fade_in = Some(FadeEffect { duration_secs: secs });
    }

    pub fn set_fade_out(&mut self, secs: f64) {
        self.fade_out = Some(FadeEffect { duration_secs: secs });
    }

    pub fn add_mute_range(&mut self, start: f64, end: f64) {
        self.mute_ranges.push((start, end));
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// VideoClip
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoClip {
    pub id: Uuid,
    pub name: String,
    pub source_path: PathBuf,
    pub source_start: f64,
    pub source_end: Option<f64>,
    pub timeline_start: f64,
    pub volume: f64,
    pub speed: f64,
    pub fade_in: Option<FadeEffect>,
    pub fade_out: Option<FadeEffect>,
    /// Audio de reemplazo (opcional)
    pub replacement_audio: Option<PathBuf>,
}

impl VideoClip {
    pub fn new(name: impl Into<String>, source_path: impl Into<PathBuf>, timeline_start: f64) -> Self {
        let path = source_path.into();
        let path = std::fs::canonicalize(&path).unwrap_or(path);
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            source_path: path,
            source_start: 0.0,
            source_end: None,
            timeline_start,
            volume: 1.0,
            speed: 1.0,
            fade_in: None,
            fade_out: None,
            replacement_audio: None,
        }
    }

    pub fn duration(&self) -> f64 {
        self.source_end
            .map(|e| (e - self.source_start).max(0.0))
            .unwrap_or(0.0)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ImageClip
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageClip {
    pub id: Uuid,
    pub name: String,
    pub source_path: PathBuf,
    pub timeline_start: f64,
    /// Duración de la imagen en el timeline
    pub duration_secs: f64,
    pub fade_in: Option<FadeEffect>,
    pub fade_out: Option<FadeEffect>,
}

impl ImageClip {
    pub fn new(
        name: impl Into<String>,
        source_path: impl Into<PathBuf>,
        timeline_start: f64,
        duration_secs: f64,
    ) -> Self {
        let path = source_path.into();
        let path = std::fs::canonicalize(&path).unwrap_or(path);
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            source_path: path,
            timeline_start,
            duration_secs,
            fade_in: None,
            fade_out: None,
        }
    }
}
