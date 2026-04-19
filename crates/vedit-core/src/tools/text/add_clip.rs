use anyhow::Result;
use uuid::Uuid;
use crate::project::Project;
use crate::project::clip::TextClip;
use crate::tools::Tool;

/// Agrega un clip de texto a un track existente
pub struct AddTextClip {
    pub track_id: Uuid,
    pub name: String,
    pub text: String,
    pub timeline_start: f64,
    pub duration_secs: f64,
}

impl Tool for AddTextClip {
    fn name(&self) -> &str { "add_text_clip" }

    fn apply(&self, project: &mut Project) -> Result<()> {
        let track = project
            .track_mut(self.track_id)
            .ok_or_else(|| anyhow::anyhow!("Track {} no encontrado", self.track_id))?;

        let clip = TextClip::new(&self.name, &self.text, self.timeline_start, self.duration_secs);
        let cid = track.add_text_clip(clip);
        tracing::info!(
            "TextClip '{}' agregado al track {} @ {:.2}s (id={})",
            self.name, self.track_id, self.timeline_start, cid
        );
        Ok(())
    }
}
