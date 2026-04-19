use anyhow::Result;
use uuid::Uuid;
use crate::project::Project;
use crate::tools::Tool;

/// Muta o desmuta un track por su id
pub struct MuteTrack {
    pub track_id: Uuid,
    pub mute: bool,
}

impl Tool for MuteTrack {
    fn name(&self) -> &str { "mute_track" }

    fn apply(&self, project: &mut Project) -> Result<()> {
        let track = project.track_mut(self.track_id)
            .ok_or_else(|| anyhow::anyhow!("Track {} no encontrado", self.track_id))?;

        if self.mute {
            track.mute();
            tracing::info!("Track '{}' muteado", track.name);
        } else {
            track.unmute();
            tracing::info!("Track '{}' desmuteado", track.name);
        }
        Ok(())
    }
}
