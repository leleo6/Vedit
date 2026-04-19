use anyhow::Result;
use crate::project::{Project, track::TrackKind};
use crate::tools::Tool;

/// Agrega un nuevo track de imagen al proyecto con nombre y capa visual
pub struct AddImageTrack {
    pub name: String,
    /// Orden de capa visual (mayor número = encima)
    pub layer_order: usize,
}

impl Tool for AddImageTrack {
    fn name(&self) -> &str { "add_image_track" }

    fn apply(&self, project: &mut Project) -> Result<()> {
        let id = project.add_track(TrackKind::Image, &self.name);
        if let Some(track) = project.track_mut(id) {
            track.layer_order = self.layer_order;
        }
        tracing::info!(
            "Track de imagen '{}' agregado (layer={}, id={})",
            self.name, self.layer_order, id
        );
        Ok(())
    }
}
