use std::path::PathBuf;
use anyhow::Result;
use uuid::Uuid;

use crate::project::{Project, clip::ImageClip};
use crate::tools::Tool;

/// Agrega un ImageClip a un track de imagen existente
pub struct AddImageClip {
    pub track_id: Uuid,
    pub source_path: PathBuf,
    pub timeline_start: f64,
    pub duration_secs: f64,
    pub name: Option<String>,
}

impl Tool for AddImageClip {
    fn name(&self) -> &str { "add_image_clip" }

    fn apply(&self, project: &mut Project) -> Result<()> {
        let clip_name = self.name.clone().unwrap_or_else(|| {
            self.source_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("image")
                .to_string()
        });

        let clip = ImageClip::new(
            clip_name,
            &self.source_path,
            self.timeline_start,
            self.duration_secs,
        );
        let clip_id = clip.id;

        let track = project
            .track_mut(self.track_id)
            .ok_or_else(|| anyhow::anyhow!("Track {} no encontrado", self.track_id))?;

        track.add_image_clip(clip);
        tracing::info!(
            "ImageClip {} agregado al track {} en t={:.2}s dur={:.2}s",
            clip_id, self.track_id, self.timeline_start, self.duration_secs
        );
        Ok(())
    }
}
