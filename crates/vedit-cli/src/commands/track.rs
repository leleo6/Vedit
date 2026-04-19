use std::path::PathBuf;
use anyhow::Result;
use clap::Subcommand;
use console::style;
use uuid::Uuid;
use vedit_core::project::{Project, track::TrackKind};
use super::{success, section, warn};

#[derive(Subcommand, Debug)]
pub enum TrackCmd {
    /// Agrega un track al proyecto
    Add {
        /// Ruta al proyecto .vedit
        #[arg(short, long)]
        project: PathBuf,
        /// Nombre del track
        name: String,
        /// Tipo de track: audio | video | image
        #[arg(short, long, default_value = "audio")]
        kind: String,
    },
    /// Elimina un track del proyecto
    Remove {
        #[arg(short, long)]
        project: PathBuf,
        /// Nombre o UUID del track a eliminar
        track: String,
    },
    /// Lista todos los tracks del proyecto
    List {
        #[arg(short, long)]
        project: PathBuf,
    },
    /// Renombra un track
    Rename {
        #[arg(short, long)]
        project: PathBuf,
        /// Nombre o UUID del track
        track: String,
        /// Nuevo nombre
        new_name: String,
    },
    /// Ajusta el volumen global de un track (0.0–2.0)
    Volume {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
        #[arg(value_parser = clap::value_parser!(f64))]
        volume: f64,
    },
    /// Mutea un track
    Mute {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
    },
    /// Desmutea un track
    Unmute {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
    },
    /// Reordena el track (cambia prioridad en el mix)
    Reorder {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
        /// Nueva posición (índice 0-based)
        position: usize,
    },
}

pub async fn run(cmd: TrackCmd) -> Result<()> {
    match cmd {
        TrackCmd::Add { project: proj_path, name, kind } => {
            let mut project = Project::load(&proj_path).await?;
            let kind = parse_kind(&kind)?;
            let id = project.add_track(kind.clone(), &name);
            project.save().await?;
            success(&format!("Track '{}' ({}) agregado (id: {})", name, kind, id));
        }

        TrackCmd::Remove { project: proj_path, track } => {
            let mut project = Project::load(&proj_path).await?;
            let id = resolve_track_id(&project, &track)?;
            if project.remove_track(id) {
                project.save().await?;
                success(&format!("Track '{}' eliminado", track));
            } else {
                warn(&format!("Track '{}' no encontrado", track));
            }
        }

        TrackCmd::List { project: proj_path } => {
            let project = Project::load(&proj_path).await?;
            section("Tracks del proyecto");
            if project.tracks.is_empty() {
                println!("  {} No hay tracks en este proyecto.", style("ℹ").blue());
                return Ok(());
            }
            for (i, t) in project.tracks.iter().enumerate() {
                let muted = if t.muted { style(" [MUTED]").red().to_string() } else { String::new() };
                let clips = t.audio_clips.len() + t.video_clips.len();
                println!(
                    "  {} {} [{}] {} vol={:.2} clips={}{} id={}",
                    style(format!("{:02}", i)).dim(),
                    style(&t.name).white().bold(),
                    style(&t.kind).cyan(),
                    if t.muted { style("●").red() } else { style("●").green() },
                    t.volume,
                    clips,
                    muted,
                    style(t.id).dim(),
                );
            }
        }

        TrackCmd::Rename { project: proj_path, track, new_name } => {
            let mut project = Project::load(&proj_path).await?;
            let id = resolve_track_id(&project, &track)?;
            project.track_mut(id).unwrap().rename(&new_name);
            project.save().await?;
            success(&format!("Track renombrado a '{}'", new_name));
        }

        TrackCmd::Volume { project: proj_path, track, volume } => {
            let mut project = Project::load(&proj_path).await?;
            let id = resolve_track_id(&project, &track)?;
            project.track_mut(id).unwrap().set_volume(volume);
            project.save().await?;
            success(&format!("Volumen del track '{}' ajustado a {:.2}", track, volume));
        }

        TrackCmd::Mute { project: proj_path, track } => {
            let mut project = Project::load(&proj_path).await?;
            let id = resolve_track_id(&project, &track)?;
            project.track_mut(id).unwrap().mute();
            project.save().await?;
            success(&format!("Track '{}' muteado", track));
        }

        TrackCmd::Unmute { project: proj_path, track } => {
            let mut project = Project::load(&proj_path).await?;
            let id = resolve_track_id(&project, &track)?;
            project.track_mut(id).unwrap().unmute();
            project.save().await?;
            success(&format!("Track '{}' desmuteado", track));
        }

        TrackCmd::Reorder { project: proj_path, track, position } => {
            let mut project = Project::load(&proj_path).await?;
            let id = resolve_track_id(&project, &track)?;
            let idx = project.tracks.iter().position(|t| t.id == id)
                .ok_or_else(|| anyhow::anyhow!("Track no encontrado"))?;
            let t = project.tracks.remove(idx);
            let new_pos = position.min(project.tracks.len());
            project.tracks.insert(new_pos, t);
            project.touch();
            project.save().await?;
            success(&format!("Track '{}' movido a posición {}", track, position));
        }
    }
    Ok(())
}

fn parse_kind(s: &str) -> Result<TrackKind> {
    match s.to_lowercase().as_str() {
        "audio" => Ok(TrackKind::Audio),
        "video" => Ok(TrackKind::Video),
        "image" => Ok(TrackKind::Image),
        other   => anyhow::bail!("Tipo de track desconocido: '{}'. Usa: audio, video, image", other),
    }
}

/// Resuelve un track por nombre o UUID
fn resolve_track_id(project: &Project, name_or_id: &str) -> Result<Uuid> {
    // Intenta parsear como UUID
    if let Ok(id) = name_or_id.parse::<Uuid>() {
        return Ok(id);
    }
    // Busca por nombre
    project
        .track_by_name(name_or_id)
        .map(|t| t.id)
        .ok_or_else(|| anyhow::anyhow!("Track '{}' no encontrado", name_or_id))
}
