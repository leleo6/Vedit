use std::path::PathBuf;
use anyhow::Result;
use clap::Subcommand;
use console::style;
use vedit_core::project::Project;
use super::{success, section};

#[derive(Subcommand, Debug)]
pub enum ProjectCmd {
    /// Crea un proyecto nuevo
    New {
        /// Nombre del proyecto
        name: String,
        /// Directorio de salida
        #[arg(short, long, default_value = ".")]
        output: PathBuf,
        /// FPS del proyecto
        #[arg(long, default_value_t = 30.0)]
        fps: f64,
    },
    /// Abre e inspecciona un proyecto existente
    Open {
        /// Ruta al archivo .vedit
        path: PathBuf,
    },
    /// Muestra información detallada del proyecto
    Info {
        /// Ruta al archivo .vedit
        path: PathBuf,
    },
    /// Deshace la última operación (Undo)
    Undo {
        /// Ruta al archivo .vedit
        path: PathBuf,
    },
    /// Rehace la última operación deshecha (Redo)
    Redo {
        /// Ruta al archivo .vedit
        path: PathBuf,
    },
}

pub async fn run(cmd: ProjectCmd) -> Result<()> {
    match cmd {
        ProjectCmd::New { name, output, fps } => {
            let mut project = Project::new(&name);
            project.metadata.fps = fps;

            // Creamos una carpeta para el proyecto (enfoque de gestión oculta)
            let project_dir = output.join(&name.replace(' ', "_").to_lowercase());
            
            // save_as ahora manejará la creación de la carpeta .vedit interna
            project.save_as(&project_dir).await?;

            section("Nuevo proyecto creado (Estructura de gestión)");
            println!("  {} {}", style("Nombre:").dim(), style(&name).white().bold());
            println!("  {} {}", style("FPS:").dim(), style(fps).white());
            println!("  {} {}", style("Carpeta:").dim(), style(project_dir.display()).cyan());
            println!("  {} {}", style("Metadatos:").dim(), style(".vedit/project.json").dim());
            success(&format!("Proyecto '{}' inicializado exitosamente", name));
        }

        ProjectCmd::Open { path } => {
            let project = Project::load(&path).await?;
            print_project_info(&project, &path);
            success(&format!("Proyecto '{}' cargado", project.metadata.name));
        }

        ProjectCmd::Info { path } => {
            let project = Project::load(&path).await?;
            print_project_info(&project, &path);
        }

        ProjectCmd::Undo { path } => {
            let mut project = Project::load(&path).await?;
            if project.undo().await? {
                success("Operación deshecha correctamente (Undo)");
            } else {
                println!("{} Nada que deshacer.", style("!").yellow());
            }
        }

        ProjectCmd::Redo { path } => {
            let mut project = Project::load(&path).await?;
            if project.redo().await? {
                success("Operación rehecha correctamente (Redo)");
            } else {
                println!("{} Nada que rehacer.", style("!").yellow());
            }
        }
    }
    Ok(())
}

fn print_project_info(project: &Project, path: &std::path::Path) {
    section("Información del proyecto");
    println!("  {} {}", style("Nombre:").dim(),    style(&project.metadata.name).white().bold());
    println!("  {} {}", style("ID:").dim(),         style(project.id).yellow());
    println!("  {} {}x{}", style("Resolución:").dim(), project.metadata.width, project.metadata.height);
    println!("  {} {} fps", style("FPS:").dim(),    project.metadata.fps);
    println!("  {} {} Hz", style("Sample rate:").dim(), project.metadata.sample_rate);
    println!("  {} {:.2}s", style("Duración:").dim(), project.duration_secs());
    println!("  {} {}", style("Tracks:").dim(),     style(project.tracks.len()).cyan());
    println!("  {} {}", style("Creado:").dim(),     project.metadata.created_at.format("%Y-%m-%d %H:%M UTC"));
    println!("  {} {}", style("Modificado:").dim(), project.metadata.modified_at.format("%Y-%m-%d %H:%M UTC"));
    println!("  {} {}", style("Archivo:").dim(),    style(path.display()).cyan());

    if !project.tracks.is_empty() {
        println!("\n  {}", style("Tracks:").dim());
        for t in &project.tracks {
            let muted = if t.muted { style(" [MUTED]").red().to_string() } else { String::new() };
            println!(
                "    {} {} ({}) vol={:.1}{}",
                style("•").cyan(),
                style(&t.name).white(),
                style(&t.kind).dim(),
                t.volume,
                muted,
            );
        }
    }
}
