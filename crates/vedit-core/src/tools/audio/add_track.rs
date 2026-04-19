use anyhow::Result;
use crate::project::{Project, track::TrackKind};
use crate::tools::Tool;

/// Agrega un nuevo track de audio al proyecto
pub struct AddTrack {
    pub name: String,
    pub kind: TrackKind,
}

impl Tool for AddTrack {
    fn name(&self) -> &str { "add_track" }

    fn apply(&self, project: &mut Project) -> Result<()> {
        let id = project.add_track(self.kind.clone(), &self.name);
        tracing::info!("Track '{}' ({}) agregado con id={}", self.name, self.kind, id);
        Ok(())
    }
}
