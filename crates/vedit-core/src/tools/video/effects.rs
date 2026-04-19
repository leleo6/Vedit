use anyhow::Result;
use uuid::Uuid;
use crate::project::Project;
use crate::tools::Tool;

/// Aplica efectos visuales a un clip de video
pub struct SetVideoEffects {
    pub track_id: Uuid,
    pub clip_id: Uuid,
    /// Blur gaussiano (radio, None = desactivar)
    pub blur_radius: Option<f64>,
    /// Sharpening (0.0–5.0, None = desactivar)
    pub sharpen: Option<f64>,
    /// Viñeta (0.0–1.0, None = desactivar)
    pub vignette: Option<f64>,
    /// Ruido/grano (0.0–1.0, None = desactivar)
    pub noise: Option<f64>,
    /// Activar/desactivar deinterlace
    pub deinterlace: Option<bool>,
}

impl Tool for SetVideoEffects {
    fn name(&self) -> &str { "set_video_effects" }

    fn apply(&self, project: &mut Project) -> Result<()> {
        let track = project
            .track_mut(self.track_id)
            .ok_or_else(|| anyhow::anyhow!("Track {} no encontrado", self.track_id))?;

        let clip = track
            .video_clip_mut(self.clip_id)
            .ok_or_else(|| anyhow::anyhow!("VideoClip {} no encontrado", self.clip_id))?;

        // Cada campo Some(_) actualiza; None se ignora (preserva el valor actual)
        if let Some(v) = self.blur_radius {
            clip.effects.blur_radius = if v > 0.0 { Some(v) } else { None };
        }
        if let Some(v) = self.sharpen {
            clip.effects.sharpen = if v > 0.0 { Some(v.min(5.0)) } else { None };
        }
        if let Some(v) = self.vignette {
            clip.effects.vignette = if v > 0.0 { Some(v.clamp(0.0, 1.0)) } else { None };
        }
        if let Some(v) = self.noise {
            clip.effects.noise = if v > 0.0 { Some(v.clamp(0.0, 1.0)) } else { None };
        }
        if let Some(di) = self.deinterlace {
            clip.effects.deinterlace = di;
        }

        tracing::info!("Efectos visuales actualizados en VideoClip {}", self.clip_id);
        Ok(())
    }
}
