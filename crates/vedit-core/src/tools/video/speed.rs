use anyhow::Result;
use uuid::Uuid;
use crate::project::Project;
use crate::tools::Tool;

/// Ajusta velocidad de reproducción y reversa de un clip de video
pub struct SetVideoSpeed {
    pub track_id: Uuid,
    pub clip_id: Uuid,
    /// Factor de velocidad (0.25 = cuarto, 1.0 = normal, 2.0 = doble, etc.)
    pub speed: f64,
    /// Reproducir en reversa
    pub reverse: bool,
    /// Mantener pitch del audio al cambiar velocidad
    pub maintain_pitch: bool,
}

impl Tool for SetVideoSpeed {
    fn name(&self) -> &str { "set_video_speed" }

    fn apply(&self, project: &mut Project) -> Result<()> {
        if self.speed <= 0.0 {
            anyhow::bail!("La velocidad debe ser mayor a 0.0 (recibido: {})", self.speed);
        }

        let track = project
            .track_mut(self.track_id)
            .ok_or_else(|| anyhow::anyhow!("Track {} no encontrado", self.track_id))?;

        let clip = track
            .video_clip_mut(self.clip_id)
            .ok_or_else(|| anyhow::anyhow!("VideoClip {} no encontrado", self.clip_id))?;

        clip.speed          = self.speed;
        clip.reverse        = self.reverse;
        clip.maintain_pitch = self.maintain_pitch;

        tracing::info!(
            "VideoClip {} → speed={:.2}x reverse={} maintain_pitch={}",
            self.clip_id, self.speed, self.reverse, self.maintain_pitch
        );
        Ok(())
    }
}
