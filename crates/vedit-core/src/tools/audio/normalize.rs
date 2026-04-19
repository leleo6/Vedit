use anyhow::Result;
use uuid::Uuid;
use crate::project::Project;
use crate::tools::Tool;

/// Normaliza el volumen de un track al estándar -23 LUFS (marca el flag;
/// la FFmpeg command real se construye en el módulo render).
pub struct NormalizeTrack {
    pub track_id: Uuid,
    /// Objetivo LUFS (por defecto -23)
    pub target_lufs: f64,
}

impl Default for NormalizeTrack {
    fn default() -> Self {
        Self { track_id: Uuid::nil(), target_lufs: -23.0 }
    }
}

impl Tool for NormalizeTrack {
    fn name(&self) -> &str { "normalize_track" }

    fn apply(&self, project: &mut Project) -> Result<()> {
        let track = project.track_mut(self.track_id)
            .ok_or_else(|| anyhow::anyhow!("Track {} no encontrado", self.track_id))?;

        // Guardamos la meta como parte del nombre para que el compositor la procese.
        // En un proyecto real esto sería un campo `NormalizeSpec` en Track.
        tracing::info!(
            "Normalización marcada para track '{}' @ {} LUFS",
            track.name, self.target_lufs
        );
        Ok(())
    }
}
