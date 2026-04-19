use std::path::PathBuf;
use anyhow::Result;
use clap::Subcommand;
use uuid::Uuid;
use vedit_core::project::Project;
use vedit_core::project::clip::ImageMode;
use super::{success, section};

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum CliImageMode {
    Overlay,
    Background,
    Fullscreen,
}

impl From<CliImageMode> for ImageMode {
    fn from(m: CliImageMode) -> Self {
        match m {
            CliImageMode::Overlay => ImageMode::Overlay,
            CliImageMode::Background => ImageMode::Background,
            CliImageMode::Fullscreen => ImageMode::Fullscreen,
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum ImageCmd {
    /// Agrega un clip de imagen a un track
    Add {
        #[arg(short, long)]
        project: PathBuf,
        /// Nombre o UUID del track de imagen destino
        track: String,
        /// Archivo de imagen fuente
        source: PathBuf,
        /// Posición de inicio en el timeline (segundos)
        #[arg(long, default_value_t = 0.0)]
        at: f64,
        /// Duración de la imagen en pantalla (segundos)
        #[arg(short, long, default_value_t = 5.0)]
        duration: f64,
        /// Nombre descriptivo
        #[arg(long)]
        name: Option<String>,
    },
    /// Transforma posición, escala, rotación u opacidad
    Transform {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
        clip_id: Uuid,
        /// Posición X (fracción 0.0 a 1.0)
        #[arg(long)]
        x: Option<f64>,
        /// Posición Y (fracción 0.0 a 1.0)
        #[arg(long)]
        y: Option<f64>,
        /// Escala en ancho (1.0 = 100%)
        #[arg(long)]
        scale_w: Option<f64>,
        /// Escala en alto (1.0 = 100%)
        #[arg(long)]
        scale_h: Option<f64>,
        /// Rotación en grados
        #[arg(long)]
        rotation: Option<f64>,
        /// Opacidad (0.0 a 1.0)
        #[arg(long)]
        opacity: Option<f64>,
        /// Modo de imagen
        #[arg(long, value_enum)]
        mode: Option<CliImageMode>,
    },
    /// Aplica efecto Ken Burns
    KenBurns {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
        clip_id: Uuid,
    },
    /// Aplica fade-in de opacidad
    FadeIn {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
        clip_id: Uuid,
        #[arg(short, long)]
        duration: f64,
    },
    /// Aplica fade-out de opacidad
    FadeOut {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
        clip_id: Uuid,
        #[arg(short, long)]
        duration: f64,
    },
    /// Elimina un clip de imagen
    Remove {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
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
    /// Ajusta la duración de un clip de imagen
    Duration {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
        clip_id: Uuid,
        /// Nueva duración (segundos)
        secs: f64,
    },
    /// Divide el clip en dos en el tiempo especificado (segundos desde el inicio del clip)
    Split {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
        clip_id: Uuid,
        /// Tiempo relativo al inicio del clip donde dividir
        at: f64,
    },
    /// Lista los clips de un track de imagen
    List {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
    },
}

pub async fn run(cmd: ImageCmd) -> Result<()> {
    match cmd {
        ImageCmd::Add { project: proj_path, track, source, at, duration, name } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            
            use vedit_core::project::clip::ImageClip;
            let clip_name = name.unwrap_or_else(|| {
                source.file_stem().and_then(|s| s.to_str()).unwrap_or("image").to_string()
            });
            let clip = ImageClip::new(&clip_name, &source, at, duration);
            let cid = clip.id;
            
            let t = project.track_mut(tid).unwrap();
            t.add_image_clip(clip);
            project.save().await?;
            success(&format!("ImageClip '{}' agregado @ {:.2}s dur={:.2}s (id: {})", clip_name, at, duration, cid));
        }

        ImageCmd::Transform { project: proj_path, track, clip_id, x, y, scale_w, scale_h, rotation, opacity, mode } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            let clip = project.track_mut(tid).unwrap()
                .image_clip_mut(clip_id)
                .ok_or_else(|| anyhow::anyhow!("ImageClip {} no encontrado", clip_id))?;
            
            if x.is_some() || y.is_some() {
                clip.set_position(x, y);
            }
            if let (Some(sw), Some(sh)) = (scale_w, scale_h) {
                clip.set_scale(sw, sh);
            }
            if let Some(r) = rotation {
                clip.rotation_deg = r;
            }
            if let Some(o) = opacity {
                clip.set_opacity(o);
            }
            if let Some(m) = mode {
                clip.mode = m.into();
            }
            project.save().await?;
            success(&format!("Transformación aplicada al ImageClip {}", clip_id));
        }

        ImageCmd::KenBurns { project: proj_path, track, clip_id } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            let clip = project.track_mut(tid).unwrap()
                .image_clip_mut(clip_id)
                .ok_or_else(|| anyhow::anyhow!("ImageClip {} no encontrado", clip_id))?;
            
            clip.apply_ken_burns();
            project.save().await?;
            success(&format!("Efecto Ken Burns aplicado al ImageClip {}", clip_id));
        }

        ImageCmd::FadeIn { project: proj_path, track, clip_id, duration } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            let clip = project.track_mut(tid).unwrap()
                .image_clip_mut(clip_id)
                .ok_or_else(|| anyhow::anyhow!("ImageClip {} no encontrado", clip_id))?;
            
            clip.set_fade_in(duration);
            project.save().await?;
            success(&format!("Fade-in de {:.2}s aplicado al ImageClip {}", duration, clip_id));
        }

        ImageCmd::FadeOut { project: proj_path, track, clip_id, duration } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            let clip = project.track_mut(tid).unwrap()
                .image_clip_mut(clip_id)
                .ok_or_else(|| anyhow::anyhow!("ImageClip {} no encontrado", clip_id))?;
            
            clip.set_fade_out(duration);
            project.save().await?;
            success(&format!("Fade-out de {:.2}s aplicado al ImageClip {}", duration, clip_id));
        }

        ImageCmd::Remove { project: proj_path, track, clip_id } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            let removed = project.track_mut(tid).unwrap().remove_image_clip(clip_id);
            if removed {
                project.save().await?;
                success(&format!("ImageClip {} eliminado", clip_id));
            } else {
                super::warn(&format!("ImageClip {} no encontrado", clip_id));
            }
        }

        ImageCmd::Move { project: proj_path, track, clip_id, at } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            let clip = project.track_mut(tid).unwrap()
                .image_clip_mut(clip_id)
                .ok_or_else(|| anyhow::anyhow!("ImageClip {} no encontrado", clip_id))?;
            clip.timeline_start = at;
            project.save().await?;
            success(&format!("ImageClip {} movido a {:.2}s", clip_id, at));
        }

        ImageCmd::Duration { project: proj_path, track, clip_id, secs } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            let new_duration = secs.max(0.1);
            {
                let clip = project.track_mut(tid).unwrap()
                    .image_clip_mut(clip_id)
                    .ok_or_else(|| anyhow::anyhow!("ImageClip {} no encontrado", clip_id))?;
                clip.duration_secs = new_duration;
            }
            project.save().await?;
            success(&format!("Duración de ImageClip {} ajustada a {:.2}s", clip_id, new_duration));
        }

        ImageCmd::Split { project: proj_path, track, clip_id, at } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            let t = project.track_mut(tid).unwrap();
            let clip = t.image_clip_mut(clip_id)
                .ok_or_else(|| anyhow::anyhow!("ImageClip {} no encontrado", clip_id))?;
            
            if let Some(new_clip) = clip.split_at(at) {
                let new_id = t.add_image_clip(new_clip);
                project.save().await?;
                success(&format!("ImageClip dividido en {:.2}s. Nuevo clip id: {}", at, new_id));
            } else {
                anyhow::bail!("No se puede dividir el clip en {:.2}s (fuera de rango)", at);
            }
        }

        ImageCmd::List { project: proj_path, track } => {
            use console::style;
            let project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            let t = project.track(tid).unwrap();
            section(&format!("Image Clips del track '{}'", t.name));

            if t.image_clips.is_empty() {
                println!("  {} No hay clips de imagen en este track.", style("ℹ").blue());
                return Ok(());
            }

            for clip in &t.image_clips {
                println!(
                    "  {} {} @{:.2}s [dur: {:.2}s] mode={} opac={:.2}",
                    style("🖼").cyan(),
                    style(&clip.name).white().bold(),
                    clip.timeline_start,
                    clip.duration_secs,
                    clip.mode,
                    clip.opacity,
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

#[cfg(test)]
mod tests {
    use super::*;
    use vedit_core::project::track::TrackKind;

    #[test]
    fn test_cli_image_mode_conversion() {
        assert_eq!(ImageMode::from(CliImageMode::Overlay), ImageMode::Overlay);
        assert_eq!(ImageMode::from(CliImageMode::Background), ImageMode::Background);
        assert_eq!(ImageMode::from(CliImageMode::Fullscreen), ImageMode::Fullscreen);
    }

    #[test]
    fn test_resolve_track_id_by_uuid() {
        let proj = Project::new("test");
        let id = Uuid::new_v4();
        // Even if track doesn't exist, if it's a valid UUID, it returns it
        // (Wait, resolve_track_id only parses it. It doesn't check existence if it's a UUID)
        let resolved = resolve_track_id(&proj, &id.to_string()).unwrap();
        assert_eq!(resolved, id);
    }

    #[test]
    fn test_resolve_track_id_by_name() {
        let mut proj = Project::new("test");
        let id = proj.add_track(TrackKind::Video, "My Video Track");
        
        let resolved = resolve_track_id(&proj, "My Video Track").unwrap();
        assert_eq!(resolved, id);
    }

    #[test]
    fn test_resolve_track_id_not_found() {
        let proj = Project::new("test");
        let result = resolve_track_id(&proj, "Nonexistent");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Track 'Nonexistent' no encontrado");
    }
}
