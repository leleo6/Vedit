use anyhow::Result;
use uuid::Uuid;
use crate::project::Project;
use crate::project::clip::{VideoTransition, TransitionKind};
use crate::tools::Tool;

/// Configura la transición de salida de un clip de video
pub struct SetVideoTransition {
    pub track_id: Uuid,
    pub clip_id: Uuid,
    pub kind: TransitionKind,
    /// Duración de la transición en segundos
    pub duration_secs: f64,
}

impl Tool for SetVideoTransition {
    fn name(&self) -> &str { "set_video_transition" }

    fn apply(&self, project: &mut Project) -> Result<()> {
        if self.duration_secs < 0.0 {
            anyhow::bail!("La duración de la transición no puede ser negativa");
        }

        let track = project
            .track_mut(self.track_id)
            .ok_or_else(|| anyhow::anyhow!("Track {} no encontrado", self.track_id))?;

        let clip = track
            .video_clip_mut(self.clip_id)
            .ok_or_else(|| anyhow::anyhow!("VideoClip {} no encontrado", self.clip_id))?;

        if self.kind == TransitionKind::Cut {
            clip.transition_out = None;
        } else {
            clip.transition_out = Some(VideoTransition::new(self.kind.clone(), self.duration_secs));
        }

        tracing::info!(
            "Transición '{}' ({:.2}s) aplicada al VideoClip {}",
            self.kind, self.duration_secs, self.clip_id
        );
        Ok(())
    }
}
