use anyhow::{bail, Result};
use uuid::Uuid;
use crate::project::{Project, clip::AudioClip, track::TrackKind};
use crate::tools::Tool;


/// Mezcla múltiples tracks de audio en uno solo (bounce)
pub struct MixTracks {
    /// IDs de los tracks a mezclar
    pub track_ids: Vec<Uuid>,
    /// Nombre del track resultante
    pub output_name: String,
}

impl Tool for MixTracks {
    fn name(&self) -> &str { "mix_tracks" }

    fn apply(&self, project: &mut Project) -> Result<()> {
        if self.track_ids.len() < 2 {
            bail!("Se necesitan al menos 2 tracks para mezclar");
        }

        // Recopilar clips de los tracks fuente
        let mut all_clips: Vec<AudioClip> = Vec::new();
        for &tid in &self.track_ids {
            let track = project.track(tid)
                .ok_or_else(|| anyhow::anyhow!("Track {} no encontrado", tid))?;
            all_clips.extend(track.audio_clips.clone());
        }

        // Crear track destino
        let new_id = project.add_track(TrackKind::Audio, &self.output_name);
        let new_track = project.track_mut(new_id).unwrap();
        for clip in all_clips {
            new_track.add_audio_clip(clip);
        }

        tracing::info!("Tracks mezclados en nuevo track '{}'", self.output_name);
        Ok(())
    }
}
