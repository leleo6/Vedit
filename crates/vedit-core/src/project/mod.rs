pub mod track;
pub mod clip;
pub mod io;

use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow::Result;
use crate::project::track::{Track, TrackKind};

/// Metadatos del proyecto (nombre, resolución, fps, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub name: String,
    pub fps: f64,
    pub width: u32,
    pub height: u32,
    pub sample_rate: u32,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

impl Default for ProjectMetadata {
    fn default() -> Self {
        Self {
            name: "Untitled".into(),
            fps: 30.0,
            width: 1920,
            height: 1080,
            sample_rate: 44100,
            created_at: Utc::now(),
            modified_at: Utc::now(),
        }
    }
}

/// Proyecto principal – contiene tracks y el path al archivo .vedit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub metadata: ProjectMetadata,
    pub tracks: Vec<Track>,
    #[serde(skip)]
    pub path: Option<PathBuf>,
}

impl Project {
    /// Crea un proyecto nuevo en memoria
    pub fn new(name: impl Into<String>) -> Self {
        let mut meta = ProjectMetadata::default();
        meta.name = name.into();
        Self {
            id: Uuid::new_v4(),
            metadata: meta,
            tracks: Vec::new(),
            path: None,
        }
    }

    /// Devuelve la duración total del proyecto (máximo de todos los tracks)
    pub fn duration_secs(&self) -> f64 {
        self.tracks
            .iter()
            .map(|t| t.duration_secs())
            .fold(0.0_f64, f64::max)
    }

    /// Agrega un track al proyecto
    pub fn add_track(&mut self, kind: TrackKind, name: impl Into<String>) -> Uuid {
        let track = Track::new(kind, name);
        let id = track.id;
        self.tracks.push(track);
        self.touch();
        id
    }

    /// Elimina un track por id
    pub fn remove_track(&mut self, id: Uuid) -> bool {
        let before = self.tracks.len();
        self.tracks.retain(|t| t.id != id);
        let removed = self.tracks.len() < before;
        if removed { self.touch(); }
        removed
    }

    /// Obtiene referencia mutable a un track
    pub fn track_mut(&mut self, id: Uuid) -> Option<&mut Track> {
        self.tracks.iter_mut().find(|t| t.id == id)
    }

    /// Obtiene referencia inmutable a un track
    pub fn track(&self, id: Uuid) -> Option<&Track> {
        self.tracks.iter().find(|t| t.id == id)
    }

    /// Busca track por nombre (case-insensitive)
    pub fn track_by_name(&self, name: &str) -> Option<&Track> {
        let lower = name.to_lowercase();
        self.tracks.iter().find(|t| t.name.to_lowercase() == lower)
    }

    pub fn track_by_name_mut(&mut self, name: &str) -> Option<&mut Track> {
        let lower = name.to_lowercase();
        self.tracks.iter_mut().find(|t| t.name.to_lowercase() == lower)
    }

    /// Actualiza timestamp de modificación
    pub fn touch(&mut self) {
        self.metadata.modified_at = Utc::now();
    }

    /// Carga proyecto desde disco
    pub async fn load(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let mut proj = io::load_project(&path).await?;
        proj.path = Some(path);
        Ok(proj)
    }

    /// Guarda el proyecto en su path actual
    pub async fn save(&mut self) -> Result<()> {
        let path = self.path.clone().ok_or_else(|| anyhow::anyhow!("No hay path de guardado. Usa save_as."))?;
        self.touch();
        io::save_project(self, &path).await
    }

    /// Guarda el proyecto en un path específico
    pub async fn save_as(&mut self, path: impl Into<PathBuf>) -> Result<()> {
        let path = path.into();
        self.touch();
        io::save_project(self, &path).await?;
        self.path = Some(path);
        Ok(())
    }

    /// Valida que el proyecto esté listo para renderizar (tiene tracks y los archivos fuente existen)
    pub fn validate_for_render(&self) -> Result<()> {
        if self.tracks.is_empty() {
            anyhow::bail!("No hay tracks para renderizar en este proyecto. Agrega al menos un track y clip antes de exportar.");
        }

        for track in &self.tracks {
            for clip in &track.audio_clips {
                if !clip.source_path.exists() {
                    anyhow::bail!("Archivo fuente extraviado o eliminado: {:?}", clip.source_path);
                }
            }
            for clip in &track.video_clips {
                if !clip.source_path.exists() {
                    anyhow::bail!("Archivo fuente extraviado o eliminado: {:?}", clip.source_path);
                }
            }
            for clip in &track.image_clips {
                if !clip.source_path.exists() {
                    anyhow::bail!("Archivo fuente extraviado o eliminado: {:?}", clip.source_path);
                }
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_creation() {
        let proj = Project::new("TestProj");
        assert_eq!(proj.metadata.name, "TestProj");
        assert_eq!(proj.tracks.len(), 0);
    }

    #[test]
    fn test_manage_tracks() {
        let mut proj = Project::new("T");
        let id1 = proj.add_track(TrackKind::Audio, "Voice");
        let id2 = proj.add_track(TrackKind::Video, "B-Roll");
        
        assert_eq!(proj.tracks.len(), 2);
        
        assert!(proj.track_by_name("voice").is_some());
        assert!(proj.track_by_name("VOICE").is_some());
        assert!(proj.track_by_name("B-Roll").is_some());
        
        assert!(proj.remove_track(id1));
        assert_eq!(proj.tracks.len(), 1);
        assert!(!proj.remove_track(id1));
    }

    #[test]
    fn test_empty_render_validation() {
        let proj = Project::new("Empty");
        assert!(proj.validate_for_render().is_err());
    }
}
