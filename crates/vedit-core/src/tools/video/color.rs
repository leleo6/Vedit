use std::path::PathBuf;
use anyhow::Result;
use uuid::Uuid;
use crate::project::Project;
use crate::project::clip::ColorCorrection;
use crate::tools::Tool;

/// Aplica corrección de color a un clip de video
pub struct SetColorCorrection {
    pub track_id: Uuid,
    pub clip_id: Uuid,
    /// Brillo (-1.0 a 1.0)
    pub brightness: Option<f64>,
    /// Contraste (0.0 a 2.0)
    pub contrast: Option<f64>,
    /// Saturación (0.0 = grises, 1.0 = normal)
    pub saturation: Option<f64>,
    /// Temperatura en Kelvin (None = sin cambio)
    pub temperature_k: Option<f64>,
    /// Ruta a archivo LUT .cube
    pub lut_path: Option<PathBuf>,
}

impl Tool for SetColorCorrection {
    fn name(&self) -> &str { "set_color_correction" }

    fn apply(&self, project: &mut Project) -> Result<()> {
        let track = project
            .track_mut(self.track_id)
            .ok_or_else(|| anyhow::anyhow!("Track {} no encontrado", self.track_id))?;

        let clip = track
            .video_clip_mut(self.clip_id)
            .ok_or_else(|| anyhow::anyhow!("VideoClip {} no encontrado", self.clip_id))?;

        if let Some(b) = self.brightness {
            clip.color.brightness = b.clamp(-1.0, 1.0);
        }
        if let Some(c) = self.contrast {
            clip.color.contrast = c.max(0.0);
        }
        if let Some(s) = self.saturation {
            clip.color.saturation = s.max(0.0);
        }
        if self.temperature_k.is_some() {
            clip.color.temperature_k = self.temperature_k;
        }
        if let Some(ref lut) = self.lut_path {
            if !lut.exists() {
                anyhow::bail!("Archivo LUT no encontrado: {:?}", lut);
            }
            clip.color.lut_path = Some(lut.clone());
        }

        tracing::info!(
            "Corrección de color aplicada al VideoClip {} (brightness={:?} contrast={:?} sat={:?})",
            self.clip_id, self.brightness, self.contrast, self.saturation
        );
        Ok(())
    }
}

/// Resetea la corrección de color a los valores por defecto
pub struct ResetColorCorrection {
    pub track_id: Uuid,
    pub clip_id: Uuid,
}

impl Tool for ResetColorCorrection {
    fn name(&self) -> &str { "reset_color_correction" }

    fn apply(&self, project: &mut Project) -> Result<()> {
        let track = project
            .track_mut(self.track_id)
            .ok_or_else(|| anyhow::anyhow!("Track {} no encontrado", self.track_id))?;
        let clip = track
            .video_clip_mut(self.clip_id)
            .ok_or_else(|| anyhow::anyhow!("VideoClip {} no encontrado", self.clip_id))?;
        clip.color = ColorCorrection::default();
        tracing::info!("Corrección de color reseteada en VideoClip {}", self.clip_id);
        Ok(())
    }
}
