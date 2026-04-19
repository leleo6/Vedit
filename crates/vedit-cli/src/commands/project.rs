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
}

pub async fn run(cmd: ProjectCmd) -> Result<()> {
    match cmd {
        ProjectCmd::New { name, output, fps } => {
            let mut project = Project::new(&name);
            project.metadata.fps = fps;

            // Crear el archivo .vedit
            let filename = format!("{}.vedit", name.replace(' ', "_").to_lowercase());
            let path = output.join(&filename);

            project.save_as(&path).await?;

            section("Nuevo proyecto creado");
            println!("  {} {}", style("Nombre:").dim(), style(&name).white().bold());
            println!("  {} {}", style("FPS:").dim(), style(fps).white());
            println!("  {} {}", style("Archivo:").dim(), style(path.display()).cyan());
            success(&format!("Proyecto '{}' creado exitosamente", name));
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
    }
    Ok(())
}

fn print_project_info(project: &Project, path: &PathBuf) {
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
