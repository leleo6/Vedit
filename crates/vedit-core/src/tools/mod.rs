pub mod audio;
pub mod image;

use anyhow::Result;
use crate::project::Project;

/// Trait base para todas las herramientas/operaciones
pub trait Tool {
    /// Nombre legible de la herramienta
    fn name(&self) -> &str;
    /// Aplica la herramienta al proyecto
    fn apply(&self, project: &mut Project) -> Result<()>;
}
