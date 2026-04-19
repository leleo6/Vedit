use std::path::Path;
use anyhow::{Context, Result};
use crate::project::Project;

/// Guarda el proyecto en disco como JSON de forma asíncrona
pub async fn save_project(project: &Project, path: &Path) -> Result<()> {
    // Crear directorio padre si no existe
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await
            .with_context(|| format!("No se pudo crear el directorio {:?}", parent))?;
    }
    let json = serde_json::to_string_pretty(project)
        .context("Error al serializar el proyecto")?;
    tokio::fs::write(path, json).await
        .with_context(|| format!("No se pudo escribir el proyecto en {:?}", path))?;
    tracing::info!("Proyecto guardado en {:?}", path);
    Ok(())
}

/// Carga el proyecto desde disco de forma asíncrona
pub async fn load_project(path: &Path) -> Result<Project> {
    let content = tokio::fs::read_to_string(path).await
        .with_context(|| format!("No se pudo leer el archivo {:?}", path))?;
    let project: Project = serde_json::from_str(&content)
        .with_context(|| format!("JSON inválido en {:?}", path))?;
        
    // Validación semántica para evitar panics/división por cero
    if project.metadata.fps <= 0.0 {
        anyhow::bail!("Corrupción semántica: Los FPS del proyecto no pueden ser <= 0.0");
    }
    if project.metadata.sample_rate == 0 {
        anyhow::bail!("Corrupción semántica: El Sample Rate del proyecto no puede ser 0");
    }
    
    tracing::info!("Proyecto cargado desde {:?}", path);
    Ok(project)
}
