use std::path::PathBuf;
use anyhow::Result;
use uuid::Uuid;
use crate::project::Project;
use crate::project::clip::VideoClip;
use crate::tools::Tool;

/// Agrega un clip de video a un track existente
pub struct AddVideoClip {
    /// ID del track destino
    pub track_id: Uuid,
    pub name: String,
    pub source_path: PathBuf,
    /// Inicio en el timeline (segundos)
    pub timeline_start: f64,
    /// Fin en el archivo fuente (None = hasta el final del archivo)
    pub source_end: Option<f64>,
}

impl Tool for AddVideoClip {
    fn name(&self) -> &str { "add_video_clip" }

    fn apply(&self, project: &mut Project) -> Result<()> {
        let track = project
            .track_mut(self.track_id)
            .ok_or_else(|| anyhow::anyhow!("Track {} no encontrado", self.track_id))?;

        let mut clip = VideoClip::new(&self.name, &self.source_path, self.timeline_start);
        clip.source_end = self.source_end;

        let cid = track.add_video_clip(clip);
        tracing::info!(
            "VideoClip '{}' agregado al track {} @ {}s (id={})",
            self.name, self.track_id, self.timeline_start, cid
        );
        Ok(())
    }
}
