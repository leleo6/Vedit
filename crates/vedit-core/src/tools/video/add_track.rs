use anyhow::Result;
use crate::project::{Project, track::TrackKind};
use crate::tools::Tool;

/// Agrega un nuevo track de video al proyecto
pub struct AddVideoTrack {
    pub name: String,
    /// Orden visual (mayor número = encima en el compositing)
    pub layer_order: usize,
}

impl Tool for AddVideoTrack {
    fn name(&self) -> &str { "add_video_track" }

    fn apply(&self, project: &mut Project) -> Result<()> {
        let id = project.add_track(TrackKind::Video, &self.name);
        if let Some(track) = project.track_mut(id) {
            track.layer_order = self.layer_order;
        }
        tracing::info!(
            "Track de video '{}' agregado (layer={}, id={})",
            self.name, self.layer_order, id
        );
        Ok(())
    }
}
