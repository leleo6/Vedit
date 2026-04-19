use anyhow::Result;
use uuid::Uuid;
use crate::project::Project;
use crate::tools::Tool;

/// Aplica transformaciones geométricas a un clip de video
pub struct TransformVideoClip {
    pub track_id: Uuid,
    pub clip_id: Uuid,
    /// Escala ancho (1.0 = 100%)
    pub scale_w: Option<f64>,
    /// Escala alto (1.0 = 100%)
    pub scale_h: Option<f64>,
    /// Posición X fraccionaria del frame
    pub pos_x: Option<f64>,
    /// Posición Y fraccionaria del frame
    pub pos_y: Option<f64>,
    /// Rotación en grados
    pub rotation_deg: Option<f64>,
    /// Flip horizontal
    pub flip_horizontal: Option<bool>,
    /// Flip vertical
    pub flip_vertical: Option<bool>,
    /// Crop: (top, bottom, left, right) en píxeles
    pub crop: Option<(u32, u32, u32, u32)>,
}

impl Tool for TransformVideoClip {
    fn name(&self) -> &str { "transform_video_clip" }

    fn apply(&self, project: &mut Project) -> Result<()> {
        let track = project
            .track_mut(self.track_id)
            .ok_or_else(|| anyhow::anyhow!("Track {} no encontrado", self.track_id))?;

        let clip = track
            .video_clip_mut(self.clip_id)
            .ok_or_else(|| anyhow::anyhow!("VideoClip {} no encontrado", self.clip_id))?;

        if let (Some(sw), Some(sh)) = (self.scale_w, self.scale_h) {
            clip.set_scale(sw, sh);
        }
        if let (Some(px), Some(py)) = (self.pos_x, self.pos_y) {
            clip.set_position(px, py);
        }
        if let Some(r) = self.rotation_deg {
            clip.rotation_deg = r;
        }
        if let Some(fh) = self.flip_horizontal {
            clip.flip_horizontal = fh;
        }
        if let Some(fv) = self.flip_vertical {
            clip.flip_vertical = fv;
        }
        if let Some((top, bottom, left, right)) = self.crop {
            use crate::project::clip::VideoCrop;
            clip.crop = Some(VideoCrop { top, bottom, left, right });
        }

        tracing::info!("Transformación aplicada al VideoClip {}", self.clip_id);
        Ok(())
    }
}
