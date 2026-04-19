use crate::project::Project;
use serde::{Deserialize, Serialize};
use std::path::Path;
use anyhow::Result;

/// Estado guardado para undo/redo (snapshot full del proyecto)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub snapshot: Project,
    pub description: String,
}

/// Gestor de historial de operaciones (Undo/Redo)
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct History {
    past: Vec<HistoryEntry>,
    future: Vec<HistoryEntry>,
}

impl History {
    pub fn new() -> Self {
        Self::default()
    }

    /// Guarda el estado actual antes de aplicar una operación
    pub fn push(&mut self, project: &Project, description: impl Into<String>) {
        self.past.push(HistoryEntry {
            snapshot: project.clone(),
            description: description.into(),
        });
        
        // Limitar el historial a 50 pasos para evitar fugas de memoria
        if self.past.len() > 50 {
            self.past.remove(0);
        }

        // Al hacer una nueva acción, se pierde el futuro
        self.future.clear();
    }

    /// Deshace la última operación; devuelve el estado anterior
    pub fn undo(&mut self, current: &Project) -> Option<Project> {
        let entry = self.past.pop()?;
        self.future.push(HistoryEntry {
            snapshot: current.clone(),
            description: format!("undo: {}", entry.description),
        });
        tracing::info!("Undo: {}", entry.description);
        Some(entry.snapshot)
    }

    /// Rehace la última operación deshecha
    pub fn redo(&mut self, current: &Project) -> Option<Project> {
        let entry = self.future.pop()?;
        self.past.push(HistoryEntry {
            snapshot: current.clone(),
            description: format!("redo: {}", entry.description),
        });
        tracing::info!("Redo: {}", entry.description);
        Some(entry.snapshot)
    }

    pub fn can_undo(&self) -> bool { !self.past.is_empty() }
    pub fn can_redo(&self) -> bool { !self.future.is_empty() }

    pub fn undo_description(&self) -> Option<&str> {
        self.past.last().map(|e| e.description.as_str())
    }

    pub async fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let data = tokio::fs::read(path).await?;
        let history: Self = serde_json::from_slice(&data)?;
        Ok(history)
    }

    pub async fn save(&self, path: &Path) -> Result<()> {
        let data = serde_json::to_vec(self)?;
        tokio::fs::write(path, data).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::track::TrackKind;

    #[test]
    fn test_undo_redo_flow() {
        let mut project = Project::new("Test Project");
        let mut history = History::new();

        // Operación 1: Añadir track
        history.push(&project, "Añadir track audio");
        project.add_track(TrackKind::Audio, "Audio 1");
        assert_eq!(project.tracks.len(), 1);

        // Operación 2: Cambiar nombre
        history.push(&project, "Renombrar proyecto");
        project.metadata.name = "Vedit Updated".to_string();

        // Undo 1: Volver a antes de renombrar
        project = history.undo(&project).expect("Debería poder hacer undo");
        assert_eq!(project.metadata.name, "Test Project");
        assert_eq!(project.tracks.len(), 1);

        // Undo 2: Volver a antes de añadir track
        project = history.undo(&project).expect("Debería poder hacer undo");
        assert_eq!(project.tracks.len(), 0);

        // Redo 1: Recuperar track
        project = history.redo(&project).expect("Debería poder hacer redo");
        assert_eq!(project.tracks.len(), 1);
        assert_eq!(project.metadata.name, "Test Project");

        // Nueva acción después de undo debería limpiar el futuro
        history.undo(&project); // Volvemos a 0 tracks
        history.push(&project, "Acción nueva");
        assert!(!history.can_redo());
    }
}
