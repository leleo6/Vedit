use anyhow::Result;
use uuid::Uuid;
use crate::project::Project;
use crate::tools::Tool;

/// Aplica fade-in de opacidad a un ImageClip
pub struct FadeInImageClip {
    pub track_id: Uuid,
    pub clip_id: Uuid,
    pub duration_secs: f64,
}

impl Tool for FadeInImageClip {
    fn name(&self) -> &str { "fade_in_image_clip" }

    fn apply(&self, project: &mut Project) -> Result<()> {
        let track = project
            .track_mut(self.track_id)
            .ok_or_else(|| anyhow::anyhow!("Track {} no encontrado", self.track_id))?;

        let clip = track
            .image_clip_mut(self.clip_id)
            .ok_or_else(|| anyhow::anyhow!("ImageClip {} no encontrado", self.clip_id))?;

        clip.set_fade_in(self.duration_secs);
        tracing::info!(
            "Fade-in de {:.2}s aplicado al ImageClip {}",
            self.duration_secs, self.clip_id
        );
        Ok(())
    }
}

/// Aplica fade-out de opacidad a un ImageClip
pub struct FadeOutImageClip {
    pub track_id: Uuid,
    pub clip_id: Uuid,
    pub duration_secs: f64,
}

impl Tool for FadeOutImageClip {
    fn name(&self) -> &str { "fade_out_image_clip" }

    fn apply(&self, project: &mut Project) -> Result<()> {
        let track = project
            .track_mut(self.track_id)
            .ok_or_else(|| anyhow::anyhow!("Track {} no encontrado", self.track_id))?;

        let clip = track
            .image_clip_mut(self.clip_id)
            .ok_or_else(|| anyhow::anyhow!("ImageClip {} no encontrado", self.clip_id))?;

        clip.set_fade_out(self.duration_secs);
        tracing::info!(
            "Fade-out de {:.2}s aplicado al ImageClip {}",
            self.duration_secs, self.clip_id
        );
        Ok(())
    }
}
