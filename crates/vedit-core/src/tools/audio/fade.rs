use anyhow::Result;
use uuid::Uuid;
use crate::project::Project;
use crate::tools::Tool;

/// Aplica fade-in al track completo
pub struct FadeInTrack {
    pub track_id: Uuid,
    pub duration_secs: f64,
}

impl Tool for FadeInTrack {
    fn name(&self) -> &str { "fade_in_track" }

    fn apply(&self, project: &mut Project) -> Result<()> {
        let track = project.track_mut(self.track_id)
            .ok_or_else(|| anyhow::anyhow!("Track {} no encontrado", self.track_id))?;

        for clip in track.audio_clips.iter_mut() {
            clip.set_fade_in(self.duration_secs);
        }
        tracing::info!("Fade-in de {}s aplicado al track '{}'", self.duration_secs, track.name);
        Ok(())
    }
}

/// Aplica fade-out al track completo
pub struct FadeOutTrack {
    pub track_id: Uuid,
    pub duration_secs: f64,
}

impl Tool for FadeOutTrack {
    fn name(&self) -> &str { "fade_out_track" }

    fn apply(&self, project: &mut Project) -> Result<()> {
        let track = project.track_mut(self.track_id)
            .ok_or_else(|| anyhow::anyhow!("Track {} no encontrado", self.track_id))?;

        for clip in track.audio_clips.iter_mut() {
            clip.set_fade_out(self.duration_secs);
        }
        tracing::info!("Fade-out de {}s aplicado al track '{}'", self.duration_secs, track.name);
        Ok(())
    }
}
