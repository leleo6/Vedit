use std::path::PathBuf;
use anyhow::Result;
use clap::Subcommand;
use console::{style, Term};
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
    /// Muestra una representación visual de la línea de tiempo
    Timeline {
        /// Ruta al directorio del proyecto o archivo .vedit
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

        ProjectCmd::Timeline { path } => {
            let project = Project::load(&path).await?;
            print_timeline(&project);
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

fn print_timeline(project: &Project) {
    let duration = project.duration_secs();
    section(&format!("Línea de Tiempo: {} (Total: {:.1}s)", project.metadata.name, duration));

    if duration <= 0.0 || project.tracks.is_empty() {
        println!("  {} No hay clips en el proyecto o la duración es 0.", style("!").yellow());
        return;
    }

    // Usar la anchura de la terminal
    let term_width = Term::stdout().size().1 as usize;
    // Reservar un pequeño margen. Mínimo 40 caracteres, máximo ancho terminal
    let bar_width = term_width.saturating_sub(4).max(40);

    // Encabezado de tiempo
    println!("\n  0s {:>width$} {:.1}s", "", duration, width = bar_width - 12);
    println!("  |{}|", "-".repeat(bar_width - 2));

    for track in project.tracks.iter().rev() {
        println!("\n  {} {} ({})", style("Track:").dim(), style(&track.name).bold().white(), style(&track.kind).dim());

        // Recolectar todos los clips iterando por tipo (ya que cada uno está en su array)
        struct ClipData {
            name: String,
            start: f64,
            end: f64,
        }

        let mut clips = Vec::new();

        for c in &track.audio_clips {
            clips.push(ClipData { name: c.name.clone(), start: c.timeline_start, end: c.timeline_start + c.duration() });
        }
        for c in &track.video_clips {
            clips.push(ClipData { name: c.name.clone(), start: c.timeline_start, end: c.timeline_start + c.duration() });
        }
        for c in &track.image_clips {
            clips.push(ClipData { name: c.name.clone(), start: c.timeline_start, end: c.timeline_start + c.duration() });
        }
        for c in &track.text_clips {
            clips.push(ClipData { name: c.text.chars().take(15).collect::<String>(), start: c.timeline_start, end: c.timeline_start + c.duration() });
        }

        // Ordenar por start
        clips.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());

        // Renderizar barra ASCII para el track entero
        let mut bar = vec!['░'; bar_width];
        for clip in &clips {
            let start_idx = ((clip.start / duration) * (bar_width as f64)).floor() as usize;
            let end_idx = ((clip.end / duration) * (bar_width as f64)).ceil() as usize;
            
            let start_idx = start_idx.min(bar_width - 1);
            let end_idx = end_idx.clamp(start_idx + 1, bar_width);

            for i in start_idx..end_idx {
                bar[i] = '█';
            }
        }
        
        let track_color = match track.kind {
            vedit_core::project::track::TrackKind::Video => console::Style::new().blue(),
            vedit_core::project::track::TrackKind::Audio => console::Style::new().green(),
            vedit_core::project::track::TrackKind::Image => console::Style::new().magenta(),
            vedit_core::project::track::TrackKind::Text  => console::Style::new().cyan(),
        };

        let bar_str: String = bar.into_iter().collect();
        println!("  [{}]", track_color.apply_to(bar_str));

        for clip in &clips {
            println!("    {} {:.1}s - {:.1}s: {}", style("├─").dim(), clip.start, clip.end, style(&clip.name).dim());
        }
    }
}

