use std::path::PathBuf;
use anyhow::Result;
use clap::Subcommand;
use vedit_core::project::Project;
use super::{success, error, section};

#[derive(Subcommand, Debug)]
pub enum HistoryCmd {
    /// Deshace la última operación (Undo)
    Undo {
        #[arg(short, long)]
        project: PathBuf,
    },
    /// Rehace la última operación deshecha (Redo)
    Redo {
        #[arg(short, long)]
        project: PathBuf,
    },
    /// Muestra el estado del historial
    Status {
        #[arg(short, long)]
        project: PathBuf,
    }
}

pub async fn run(cmd: HistoryCmd) -> Result<()> {
    match cmd {
        HistoryCmd::Undo { project: proj_path } => {
            let mut project = Project::load(&proj_path).await?;
            if project.undo().await? {
                success("Undo aplicado correctamente.");
            } else {
                error("No hay acciones para deshacer.");
            }
        }
        HistoryCmd::Redo { project: proj_path } => {
            let mut project = Project::load(&proj_path).await?;
            if project.redo().await? {
                success("Redo aplicado correctamente.");
            } else {
                error("No hay acciones para rehacer.");
            }
        }
        HistoryCmd::Status { project: proj_path } => {
            let project = Project::load(&proj_path).await?;
            section("Estado del Historial");
            let undo_desc = project.history.undo_description().unwrap_or("Ninguna");
            println!("  Deshacer disponible: {}", if project.history.can_undo() { "Sí" } else { "No" });
            println!("  Siguiente acción a deshacer: {}", undo_desc);
            println!("  Rehacer disponible: {}", if project.history.can_redo() { "Sí" } else { "No" });
        }
    }
    Ok(())
}
