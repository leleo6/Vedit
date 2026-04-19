use std::path::{Path, PathBuf};
use crate::project::Project;

/// Contexto de sesión activa del editor
#[derive(Debug, Default)]
pub struct AppContext {
    /// Proyecto actualmente cargado
    pub project: Option<Project>,
    /// Path del proyecto
    pub project_path: Option<PathBuf>,
}

impl AppContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn load_project(&mut self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let path = path.as_ref().to_path_buf();
        let project = Project::load(&path).await?;
        self.project_path = Some(path);
        self.project = Some(project);
        Ok(())
    }

    pub fn require_project(&self) -> anyhow::Result<&Project> {
        self.project.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No hay ningún proyecto abierto. Usa `vedit project open <path>`"))
    }

    pub fn require_project_mut(&mut self) -> anyhow::Result<&mut Project> {
        self.project.as_mut()
            .ok_or_else(|| anyhow::anyhow!("No hay ningún proyecto abierto. Usa `vedit project open <path>`"))
    }
}
