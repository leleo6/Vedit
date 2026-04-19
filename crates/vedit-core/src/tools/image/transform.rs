use anyhow::Result;
use uuid::Uuid;

use crate::project::clip::ImageMode;
use crate::project::Project;
use crate::tools::Tool;

/// Modifica posición, escala, rotación, opacidad y modo de un ImageClip
pub struct TransformImageClip {
    pub track_id: Uuid,
    pub clip_id: Uuid,
    pub position_x: Option<f64>,
    pub position_y: Option<f64>,
    pub scale_w: Option<f64>,
    pub scale_h: Option<f64>,
    pub rotation_deg: Option<f64>,
    pub opacity: Option<f64>,
    pub mode: Option<ImageMode>,
}

impl Tool for TransformImageClip {
    fn name(&self) -> &str { "transform_image_clip" }

    fn apply(&self, project: &mut Project) -> Result<()> {
        let track = project
            .track_mut(self.track_id)
            .ok_or_else(|| anyhow::anyhow!("Track {} no encontrado", self.track_id))?;

        let clip = track
            .image_clip_mut(self.clip_id)
            .ok_or_else(|| anyhow::anyhow!("ImageClip {} no encontrado", self.clip_id))?;

        // Posición
        if self.position_x.is_some() || self.position_y.is_some() {
            clip.set_position(self.position_x, self.position_y);
        }

        // Escala
        if let (Some(w), Some(h)) = (self.scale_w, self.scale_h) {
            clip.set_scale(w, h);
        }

        // Rotación
        if let Some(deg) = self.rotation_deg {
            clip.rotation_deg = deg;
        }

        // Opacidad
        if let Some(op) = self.opacity {
            clip.set_opacity(op);
        }

        // Modo
        if let Some(mode) = self.mode.clone() {
            clip.mode = mode;
        }

        tracing::info!("ImageClip {} transformado", self.clip_id);
        Ok(())
    }
}
