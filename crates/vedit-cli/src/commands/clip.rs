use std::path::PathBuf;
use anyhow::Result;
use clap::Subcommand;
use console::style;
use uuid::Uuid;
use vedit_core::project::{Project, clip::AudioClip};
use super::{success, warn, section};

#[derive(Subcommand, Debug)]
pub enum ClipCmd {
    /// Agrega un clip a un track
    Add {
        #[arg(short, long)]
        project: PathBuf,
        /// Nombre o UUID del track destino
        track: String,
        /// Archivo fuente (audio o video)
        source: PathBuf,
        /// Posición de inicio en el timeline (segundos)
        #[arg(long, default_value_t = 0.0)]
        at: f64,
        /// Nombre descriptivo del clip
        #[arg(long)]
        name: Option<String>,
    },
    /// Elimina un clip de un track
    Remove {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
        /// UUID del clip a eliminar
        clip_id: Uuid,
    },
    /// Mueve un clip en el timeline
    Move {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
        clip_id: Uuid,
        /// Nueva posición en el timeline (segundos)
        at: f64,
    },
    /// Recorta un clip (source_start y source_end)
    Trim {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
        clip_id: Uuid,
        /// Inicio en el archivo fuente (segundos)
        #[arg(long)]
        start: f64,
        /// Fin en el archivo fuente (segundos)
        #[arg(long)]
        end: f64,
    },
    /// Ajusta el volumen individual de un clip
    Volume {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
        clip_id: Uuid,
        volume: f64,
    },
    /// Activa loop en un clip (N repeticiones)
    Loop {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
        clip_id: Uuid,
        /// Número de repeticiones
        #[arg(default_value_t = 2)]
        times: u32,
    },
    /// Ajusta velocidad/pitch de un clip
    Speed {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
        clip_id: Uuid,
        /// Factor de velocidad (0.5 = lento, 2.0 = rápido)
        factor: f64,
    },
    /// Lista los clips de un track
    List {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
    },
}

pub async fn run(cmd: ClipCmd) -> Result<()> {
    match cmd {
        ClipCmd::Add { project: proj_path, track, source, at, name } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            let clip_name = name.unwrap_or_else(|| {
                source.file_stem().and_then(|s| s.to_str()).unwrap_or("clip").to_string()
            });
            let clip = AudioClip::new(&clip_name, &source, at);
            let cid = clip.id;
            project.track_mut(tid).unwrap().add_audio_clip(clip);
            project.save().await?;
            success(&format!("Clip '{}' agregado al track '{}' @ {:.2}s (id: {})", clip_name, track, at, cid));
        }

        ClipCmd::Remove { project: proj_path, track, clip_id } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            let removed = project.track_mut(tid).unwrap().remove_audio_clip(clip_id);
            if removed {
                project.save().await?;
                success(&format!("Clip {} eliminado", clip_id));
            } else {
                warn(&format!("Clip {} no encontrado", clip_id));
            }
        }

        ClipCmd::Move { project: proj_path, track, clip_id, at } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            let clip = project.track_mut(tid).unwrap()
                .audio_clip_mut(clip_id)
                .ok_or_else(|| anyhow::anyhow!("Clip {} no encontrado", clip_id))?;
            clip.timeline_start = at;
            project.save().await?;
            success(&format!("Clip {} movido a {:.2}s", clip_id, at));
        }

        ClipCmd::Trim { project: proj_path, track, clip_id, start, end } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            let clip = project.track_mut(tid).unwrap()
                .audio_clip_mut(clip_id)
                .ok_or_else(|| anyhow::anyhow!("Clip {} no encontrado", clip_id))?;
            clip.source_start = start;
            clip.source_end = Some(end);
            project.save().await?;
            success(&format!("Clip {} recortado: {:.2}s – {:.2}s", clip_id, start, end));
        }

        ClipCmd::Volume { project: proj_path, track, clip_id, volume } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            let clip = project.track_mut(tid).unwrap()
                .audio_clip_mut(clip_id)
                .ok_or_else(|| anyhow::anyhow!("Clip {} no encontrado", clip_id))?;
            clip.volume = volume.clamp(0.0, 2.0);
            project.save().await?;
            success(&format!("Volumen del clip {} ajustado a {:.2}", clip_id, volume));
        }

        ClipCmd::Loop { project: proj_path, track, clip_id, times } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            let clip = project.track_mut(tid).unwrap()
                .audio_clip_mut(clip_id)
                .ok_or_else(|| anyhow::anyhow!("Clip {} no encontrado", clip_id))?;
            clip.loop_count = times;
            project.save().await?;
            success(&format!("Clip {} configurado para repetir {} veces", clip_id, times));
        }

        ClipCmd::Speed { project: proj_path, track, clip_id, factor } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            let clip = project.track_mut(tid).unwrap()
                .audio_clip_mut(clip_id)
                .ok_or_else(|| anyhow::anyhow!("Clip {} no encontrado", clip_id))?;
            clip.speed = factor.max(0.1);
            project.save().await?;
            success(&format!("Velocidad del clip {} ajustada a {:.2}x", clip_id, factor));
        }

        ClipCmd::List { project: proj_path, track } => {
            let project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            let t = project.track(tid).unwrap();
            section(&format!("Clips del track '{}'", t.name));

            if t.audio_clips.is_empty() && t.video_clips.is_empty() {
                println!("  {} No hay clips en este track.", style("ℹ").blue());
                return Ok(());
            }

            for clip in &t.audio_clips {
                let end = clip.source_end.map(|e| format!("{:.2}s", e)).unwrap_or("∞".into());
                println!(
                    "  {} {} @{:.2}s [{:.2}s–{}] vol={:.2} loop={} speed={:.2}x",
                    style("♪").cyan(),
                    style(&clip.name).white().bold(),
                    clip.timeline_start,
                    clip.source_start,
                    end,
                    clip.volume,
                    clip.loop_count,
                    clip.speed,
                );
                println!("    {} {}", style("id:").dim(), style(clip.id).dim());
                println!("    {} {}", style("src:").dim(), clip.source_path.display());
            }
        }
    }
    Ok(())
}

fn resolve_track_id(project: &Project, name_or_id: &str) -> Result<Uuid> {
    if let Ok(id) = name_or_id.parse::<Uuid>() {
        return Ok(id);
    }
    project.track_by_name(name_or_id).map(|t| t.id)
        .ok_or_else(|| anyhow::anyhow!("Track '{}' no encontrado", name_or_id))
}
