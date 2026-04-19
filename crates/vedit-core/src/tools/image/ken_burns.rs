use anyhow::Result;
use uuid::Uuid;
use crate::project::clip::KenBurnsEffect;
use crate::project::Project;
use crate::tools::Tool;

/// Aplica el efecto Ken Burns (zoom + pan) a un ImageClip
pub struct ApplyKenBurns {
    pub track_id: Uuid,
    pub clip_id: Uuid,
    /// None = usa valores por defecto (zoom 1.0→1.2, sin pan)
    pub effect: Option<KenBurnsEffect>,
}

impl Tool for ApplyKenBurns {
    fn name(&self) -> &str { "apply_ken_burns" }

    fn apply(&self, project: &mut Project) -> Result<()> {
        let track = project
            .track_mut(self.track_id)
            .ok_or_else(|| anyhow::anyhow!("Track {} no encontrado", self.track_id))?;

        let clip = track
            .image_clip_mut(self.clip_id)
            .ok_or_else(|| anyhow::anyhow!("ImageClip {} no encontrado", self.clip_id))?;

        match &self.effect {
            Some(ef) => clip.apply_ken_burns_custom(ef.clone()),
            None      => clip.apply_ken_burns(),
        }

        tracing::info!("Ken Burns aplicado al ImageClip {}", self.clip_id);
        Ok(())
    }
}
