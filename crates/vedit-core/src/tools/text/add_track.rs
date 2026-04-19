use anyhow::Result;
use crate::project::{Project, track::TrackKind};
use crate::tools::Tool;

/// Agrega un nuevo track de texto/subtítulos al proyecto
pub struct AddTextTrack {
    pub name: String,
    /// Orden de capa visual (mayor = encima)
    pub layer_order: usize,
}

impl Tool for AddTextTrack {
    fn name(&self) -> &str { "add_text_track" }

    fn apply(&self, project: &mut Project) -> Result<()> {
        let id = project.add_track(TrackKind::Text, &self.name);
        if let Some(track) = project.track_mut(id) {
            track.layer_order = self.layer_order;
        }
        tracing::info!(
            "Track de texto '{}' agregado (layer={}, id={})",
            self.name, self.layer_order, id
        );
        Ok(())
    }
}
