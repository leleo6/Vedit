use std::path::PathBuf;
use anyhow::Result;

use uuid::Uuid;
use crate::project::{Project, clip::AudioClip};
use crate::tools::Tool;

/// Agrega un AudioClip a un track existente
pub struct AddAudioClip {
    pub track_id: Uuid,
    pub source_path: PathBuf,
    pub timeline_start: f64,
    pub name: Option<String>,
}

impl Tool for AddAudioClip {
    fn name(&self) -> &str { "add_audio_clip" }

    fn apply(&self, project: &mut Project) -> Result<()> {
        let clip_name = self.name.clone().unwrap_or_else(|| {
            self.source_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("clip")
                .to_string()
        });

        let clip = AudioClip::new(clip_name, &self.source_path, self.timeline_start);
        let clip_id = clip.id;

        let track = project.track_mut(self.track_id)
            .ok_or_else(|| anyhow::anyhow!("Track {} no encontrado", self.track_id))?;

        track.add_audio_clip(clip);
        tracing::info!("AudioClip {} agregado al track {}", clip_id, self.track_id);
        Ok(())
    }
}
