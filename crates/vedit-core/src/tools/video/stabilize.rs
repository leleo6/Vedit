use anyhow::Result;
use uuid::Uuid;
use crate::project::Project;
use crate::tools::Tool;

/// Activa o desactiva la estabilización de video (vidstab de FFmpeg)
pub struct SetStabilize {
    pub track_id: Uuid,
    pub clip_id: Uuid,
    pub enabled: bool,
}

impl Tool for SetStabilize {
    fn name(&self) -> &str { "set_stabilize" }

    fn apply(&self, project: &mut Project) -> Result<()> {
        let track = project
            .track_mut(self.track_id)
            .ok_or_else(|| anyhow::anyhow!("Track {} no encontrado", self.track_id))?;

        let clip = track
            .video_clip_mut(self.clip_id)
            .ok_or_else(|| anyhow::anyhow!("VideoClip {} no encontrado", self.clip_id))?;

        clip.set_stabilize(self.enabled);
        tracing::info!(
            "Estabilización {} en VideoClip {}",
            if self.enabled { "activada" } else { "desactivada" },
            self.clip_id
        );
        Ok(())
    }
}
