use crate::project::Project;


/// Estado guardado para undo/redo (snapshot full del proyecto)
#[derive(Debug, Clone)]
struct HistoryEntry {
    snapshot: Project,
    description: String,
}

/// Gestor de historial de operaciones (Undo/Redo)
#[derive(Debug, Default)]
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
}
