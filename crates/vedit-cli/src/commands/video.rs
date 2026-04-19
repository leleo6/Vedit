use std::path::PathBuf;
use anyhow::Result;
use clap::Subcommand;
use uuid::Uuid;
use console::style;
use vedit_core::project::Project;
use vedit_core::project::clip::{TransitionKind, VideoCrop};
use super::{success, section};

// ── Enums CLI ────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum CliTransitionKind {
    Cut,
    FadeToBlack,
    FadeToWhite,
    CrossDissolve,
    WipeHorizontal,
    WipeVertical,
}

impl From<CliTransitionKind> for TransitionKind {
    fn from(k: CliTransitionKind) -> Self {
        match k {
            CliTransitionKind::Cut            => TransitionKind::Cut,
            CliTransitionKind::FadeToBlack    => TransitionKind::FadeToBlack,
            CliTransitionKind::FadeToWhite    => TransitionKind::FadeToWhite,
            CliTransitionKind::CrossDissolve  => TransitionKind::CrossDissolve,
            CliTransitionKind::WipeHorizontal => TransitionKind::WipeHorizontal,
            CliTransitionKind::WipeVertical   => TransitionKind::WipeVertical,
        }
    }
}

// ── Subcomandos ───────────────────────────────────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum VideoCmd {
    /// Agrega un clip de video a un track
    Add {
        #[arg(short, long)]
        project: PathBuf,
        /// Nombre o UUID del track de video destino
        track: String,
        /// Archivo de video fuente
        source: PathBuf,
        /// Posición de inicio en el timeline (segundos)
        #[arg(long, default_value_t = 0.0)]
        at: f64,
        /// Duración de fuente en segundos (None = hasta el final)
        #[arg(long)]
        duration: Option<f64>,
        /// Nombre descriptivo del clip
        #[arg(long)]
        name: Option<String>,
    },

    /// Elimina un clip de video de un track
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

    /// Recorta el clip (source_start / source_end)
    Trim {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
        clip_id: Uuid,
        /// Inicio en el archivo fuente (segundos)
        #[arg(long, default_value_t = 0.0)]
        start: f64,
        /// Fin en el archivo fuente (segundos)
        #[arg(long)]
        end: Option<f64>,
    },

    /// Divide el clip en el tiempo indicado (relativo al inicio del archivo fuente)
    Split {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
        clip_id: Uuid,
        /// Tiempo en el archivo fuente donde dividir (segundos)
        at: f64,
    },

    /// Aplica transformaciones geométricas al clip
    Transform {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
        clip_id: Uuid,
        /// Escala ancho (1.0 = 100%)
        #[arg(long)]
        scale_w: Option<f64>,
        /// Escala alto (1.0 = 100%)
        #[arg(long)]
        scale_h: Option<f64>,
        /// Posición X fraccionaria (0.0 = izquierda)
        #[arg(long)]
        x: Option<f64>,
        /// Posición Y fraccionaria (0.0 = arriba)
        #[arg(long)]
        y: Option<f64>,
        /// Rotación en grados
        #[arg(long)]
        rotation: Option<f64>,
        /// Flip horizontal
        #[arg(long)]
        flip_h: bool,
        /// Flip vertical
        #[arg(long)]
        flip_v: bool,
        /// Crop: píxeles a recortar por arriba
        #[arg(long, default_value_t = 0)]
        crop_top: u32,
        /// Crop: píxeles a recortar por abajo
        #[arg(long, default_value_t = 0)]
        crop_bottom: u32,
        /// Crop: píxeles a recortar por la izquierda
        #[arg(long, default_value_t = 0)]
        crop_left: u32,
        /// Crop: píxeles a recortar por la derecha
        #[arg(long, default_value_t = 0)]
        crop_right: u32,
    },

    /// Ajusta velocidad de reproducción y reversa
    Speed {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
        clip_id: Uuid,
        /// Factor de velocidad (ej: 0.5 = mitad, 2.0 = doble)
        #[arg(long, default_value_t = 1.0)]
        factor: f64,
        /// Reproducir en reversa
        #[arg(long)]
        reverse: bool,
        /// Mantener pitch del audio
        #[arg(long, default_value_t = true)]
        maintain_pitch: bool,
    },

    /// Aplica corrección de color
    Color {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
        clip_id: Uuid,
        /// Brillo (-1.0 a 1.0)
        #[arg(long)]
        brightness: Option<f64>,
        /// Contraste (0.0 a 2.0, 1.0 = normal)
        #[arg(long)]
        contrast: Option<f64>,
        /// Saturación (0.0 = grises, 1.0 = normal)
        #[arg(long)]
        saturation: Option<f64>,
        /// Temperatura de color en Kelvin (6500 = neutro)
        #[arg(long)]
        temperature: Option<f64>,
        /// Ruta a archivo LUT .cube
        #[arg(long)]
        lut: Option<PathBuf>,
    },

    /// Aplica o actualiza efectos visuales
    Effects {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
        clip_id: Uuid,
        /// Blur gaussiano (radio en píxeles, 0 = desactivar)
        #[arg(long)]
        blur: Option<f64>,
        /// Sharpening (0.0–5.0, 0 = desactivar)
        #[arg(long)]
        sharpen: Option<f64>,
        /// Viñeta (0.0–1.0, 0 = desactivar)
        #[arg(long)]
        vignette: Option<f64>,
        /// Ruido/grano (0.0–1.0, 0 = desactivar)
        #[arg(long)]
        noise: Option<f64>,
        /// Activar deinterlace
        #[arg(long)]
        deinterlace: bool,
    },

    /// Aplica fade-in de video
    FadeIn {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
        clip_id: Uuid,
        #[arg(short, long)]
        duration: f64,
    },

    /// Aplica fade-out de video
    FadeOut {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
        clip_id: Uuid,
        #[arg(short, long)]
        duration: f64,
    },

    /// Configura la transición de salida del clip
    Transition {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
        clip_id: Uuid,
        /// Tipo de transición
        #[arg(long, value_enum)]
        kind: CliTransitionKind,
        /// Duración en segundos
        #[arg(long, default_value_t = 1.0)]
        duration: f64,
    },

    /// Activa o desactiva estabilización de video (vidstab)
    Stabilize {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
        clip_id: Uuid,
        /// Desactivar estabilización
        #[arg(long)]
        disable: bool,
    },

    /// Lista los clips de un track de video
    List {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
    },
}

// ── Handler ───────────────────────────────────────────────────────────────────

pub async fn run(cmd: VideoCmd) -> Result<()> {
    match cmd {
        VideoCmd::Add { project: proj_path, track, source, at, duration, name } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;

            use vedit_core::project::clip::VideoClip;
            let clip_name = name.unwrap_or_else(|| {
                source.file_stem().and_then(|s| s.to_str()).unwrap_or("video").to_string()
            });
            let mut clip = VideoClip::new(&clip_name, &source, at);
            if let Some(dur) = duration {
                clip.source_end = Some(clip.source_start + dur);
            }
            let cid = clip.id;
            project.track_mut(tid).unwrap().add_video_clip(clip);
            project.save().await?;
            success(&format!(
                "VideoClip '{}' agregado @ {:.2}s (id: {})",
                clip_name, at, cid
            ));
        }

        VideoCmd::Remove { project: proj_path, track, clip_id } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            let removed = project.track_mut(tid).unwrap().remove_video_clip(clip_id);
            if removed {
                project.save().await?;
                success(&format!("VideoClip {} eliminado", clip_id));
            } else {
                super::warn(&format!("VideoClip {} no encontrado", clip_id));
            }
        }

        VideoCmd::Move { project: proj_path, track, clip_id, at } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            let clip = project.track_mut(tid).unwrap()
                .video_clip_mut(clip_id)
                .ok_or_else(|| anyhow::anyhow!("VideoClip {} no encontrado", clip_id))?;
            clip.timeline_start = at;
            project.save().await?;
            success(&format!("VideoClip {} movido a {:.2}s", clip_id, at));
        }

        VideoCmd::Trim { project: proj_path, track, clip_id, start, end } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            let clip = project.track_mut(tid).unwrap()
                .video_clip_mut(clip_id)
                .ok_or_else(|| anyhow::anyhow!("VideoClip {} no encontrado", clip_id))?;
            clip.source_start = start;
            clip.source_end   = end;
            project.save().await?;
            success(&format!(
                "VideoClip {} recortado: start={:.2}s end={:?}",
                clip_id, start, end
            ));
        }

        VideoCmd::Split { project: proj_path, track, clip_id, at } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            let t = project.track_mut(tid).unwrap();
            let clip = t.video_clip_mut(clip_id)
                .ok_or_else(|| anyhow::anyhow!("VideoClip {} no encontrado", clip_id))?;

            if let Some(new_clip) = clip.split_at(at) {
                let new_id = t.add_video_clip(new_clip);
                project.save().await?;
                success(&format!("VideoClip dividido en {:.2}s. Nuevo clip id: {}", at, new_id));
            } else {
                anyhow::bail!("No se puede dividir el clip en {:.2}s (fuera de rango)", at);
            }
        }

        VideoCmd::Transform {
            project: proj_path, track, clip_id,
            scale_w, scale_h, x, y, rotation,
            flip_h, flip_v,
            crop_top, crop_bottom, crop_left, crop_right,
        } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            let clip = project.track_mut(tid).unwrap()
                .video_clip_mut(clip_id)
                .ok_or_else(|| anyhow::anyhow!("VideoClip {} no encontrado", clip_id))?;

            if let (Some(sw), Some(sh)) = (scale_w, scale_h) {
                clip.set_scale(sw, sh);
            }
            if let (Some(px), Some(py)) = (x, y) {
                clip.set_position(px, py);
            }
            if let Some(r) = rotation {
                clip.rotation_deg = r;
            }
            if flip_h { clip.flip_horizontal = !clip.flip_horizontal; }
            if flip_v { clip.flip_vertical   = !clip.flip_vertical; }
            if crop_top > 0 || crop_bottom > 0 || crop_left > 0 || crop_right > 0 {
                clip.crop = Some(VideoCrop {
                    top: crop_top, bottom: crop_bottom,
                    left: crop_left, right: crop_right,
                });
            }
            project.save().await?;
            success(&format!("Transformación aplicada al VideoClip {}", clip_id));
        }

        VideoCmd::Speed { project: proj_path, track, clip_id, factor, reverse, maintain_pitch } => {
            if factor <= 0.0 {
                anyhow::bail!("La velocidad debe ser mayor a 0.0");
            }
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            let clip = project.track_mut(tid).unwrap()
                .video_clip_mut(clip_id)
                .ok_or_else(|| anyhow::anyhow!("VideoClip {} no encontrado", clip_id))?;
            clip.speed          = factor;
            clip.reverse        = reverse;
            clip.maintain_pitch = maintain_pitch;
            project.save().await?;
            success(&format!(
                "VideoClip {} → speed={:.2}x reverse={} maintain_pitch={}",
                clip_id, factor, reverse, maintain_pitch
            ));
        }

        VideoCmd::Color { project: proj_path, track, clip_id, brightness, contrast, saturation, temperature, lut } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            let clip = project.track_mut(tid).unwrap()
                .video_clip_mut(clip_id)
                .ok_or_else(|| anyhow::anyhow!("VideoClip {} no encontrado", clip_id))?;

            if let Some(b) = brightness { clip.color.brightness = b.clamp(-1.0, 1.0); }
            if let Some(c) = contrast   { clip.color.contrast   = c.max(0.0); }
            if let Some(s) = saturation { clip.color.saturation = s.max(0.0); }
            if let Some(t) = temperature { clip.color.temperature_k = Some(t); }
            if let Some(ref path) = lut {
                if !path.exists() {
                    anyhow::bail!("Archivo LUT no encontrado: {:?}", path);
                }
                clip.color.lut_path = Some(path.clone());
            }
            project.save().await?;
            success(&format!("Corrección de color aplicada al VideoClip {}", clip_id));
        }

        VideoCmd::Effects { project: proj_path, track, clip_id, blur, sharpen, vignette, noise, deinterlace } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            let clip = project.track_mut(tid).unwrap()
                .video_clip_mut(clip_id)
                .ok_or_else(|| anyhow::anyhow!("VideoClip {} no encontrado", clip_id))?;

            if let Some(v) = blur     { clip.effects.blur_radius = if v > 0.0 { Some(v) } else { None }; }
            if let Some(v) = sharpen  { clip.effects.sharpen     = if v > 0.0 { Some(v.min(5.0)) } else { None }; }
            if let Some(v) = vignette { clip.effects.vignette    = if v > 0.0 { Some(v.clamp(0.0, 1.0)) } else { None }; }
            if let Some(v) = noise    { clip.effects.noise       = if v > 0.0 { Some(v.clamp(0.0, 1.0)) } else { None }; }
            if deinterlace { clip.effects.deinterlace = true; }

            project.save().await?;
            success(&format!("Efectos visuales aplicados al VideoClip {}", clip_id));
        }

        VideoCmd::FadeIn { project: proj_path, track, clip_id, duration } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            let clip = project.track_mut(tid).unwrap()
                .video_clip_mut(clip_id)
                .ok_or_else(|| anyhow::anyhow!("VideoClip {} no encontrado", clip_id))?;
            clip.set_fade_in(duration);
            project.save().await?;
            success(&format!("Fade-in de {:.2}s aplicado al VideoClip {}", duration, clip_id));
        }

        VideoCmd::FadeOut { project: proj_path, track, clip_id, duration } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            let clip = project.track_mut(tid).unwrap()
                .video_clip_mut(clip_id)
                .ok_or_else(|| anyhow::anyhow!("VideoClip {} no encontrado", clip_id))?;
            clip.set_fade_out(duration);
            project.save().await?;
            success(&format!("Fade-out de {:.2}s aplicado al VideoClip {}", duration, clip_id));
        }

        VideoCmd::Transition { project: proj_path, track, clip_id, kind, duration } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            let clip = project.track_mut(tid).unwrap()
                .video_clip_mut(clip_id)
                .ok_or_else(|| anyhow::anyhow!("VideoClip {} no encontrado", clip_id))?;

            use vedit_core::project::clip::VideoTransition;
            let tk: TransitionKind = kind.into();
            if tk == TransitionKind::Cut {
                clip.transition_out = None;
            } else {
                clip.transition_out = Some(VideoTransition::new(tk, duration));
            }
            project.save().await?;
            success(&format!("Transición aplicada al VideoClip {}", clip_id));
        }

        VideoCmd::Stabilize { project: proj_path, track, clip_id, disable } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            let clip = project.track_mut(tid).unwrap()
                .video_clip_mut(clip_id)
                .ok_or_else(|| anyhow::anyhow!("VideoClip {} no encontrado", clip_id))?;
            clip.set_stabilize(!disable);
            project.save().await?;
            let state = if disable { "desactivada" } else { "activada" };
            success(&format!("Estabilización {} en VideoClip {}", state, clip_id));
        }

        VideoCmd::List { project: proj_path, track } => {
            let project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            let t = project.track(tid).unwrap();
            section(&format!("Video Clips del track '{}'", t.name));

            if t.video_clips.is_empty() {
                println!("  {} No hay clips de video en este track.", style("ℹ").blue());
                return Ok(());
            }

            for clip in &t.video_clips {
                println!(
                    "  {} {} @{:.2}s [dur: {:.2}s] speed={:.2}x{}",
                    style("🎬").cyan(),
                    style(&clip.name).white().bold(),
                    clip.timeline_start,
                    clip.duration(),
                    clip.speed,
                    if clip.reverse { " ⟵rev" } else { "" },
                );
                println!("    {} {}", style("id:").dim(),  style(clip.id).dim());
                println!("    {} {}", style("src:").dim(), clip.source_path.display());
                if clip.color.is_active() {
                    println!("    {} brightness={:.2} contrast={:.2} saturation={:.2}",
                        style("color:").dim(),
                        clip.color.brightness, clip.color.contrast, clip.color.saturation);
                }
                if clip.effects.is_active() {
                    let mut fx = Vec::new();
                    if let Some(b) = clip.effects.blur_radius { fx.push(format!("blur={:.1}", b)); }
                    if let Some(s) = clip.effects.sharpen     { fx.push(format!("sharpen={:.1}", s)); }
                    if let Some(v) = clip.effects.vignette    { fx.push(format!("vignette={:.2}", v)); }
                    if let Some(n) = clip.effects.noise       { fx.push(format!("noise={:.2}", n)); }
                    if clip.effects.deinterlace               { fx.push("deinterlace".into()); }
                    println!("    {} {}", style("fx:").dim(), fx.join(", "));
                }
                if let Some(ref tr) = clip.transition_out {
                    println!("    {} {} ({:.2}s)", style("transition:").dim(), tr.kind, tr.duration_secs);
                }
                if clip.stabilize {
                    println!("    {} estabilización activa", style("⚡").yellow());
                }
            }
        }
    }
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn resolve_track_id(project: &Project, name_or_id: &str) -> Result<Uuid> {
    if let Ok(id) = name_or_id.parse::<Uuid>() {
        return Ok(id);
    }
    project.track_by_name(name_or_id).map(|t| t.id)
        .ok_or_else(|| anyhow::anyhow!("Track '{}' no encontrado", name_or_id))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use vedit_core::project::track::TrackKind;

    #[test]
    fn test_cli_transition_kind_conversion() {
        assert_eq!(TransitionKind::from(CliTransitionKind::Cut),            TransitionKind::Cut);
        assert_eq!(TransitionKind::from(CliTransitionKind::FadeToBlack),    TransitionKind::FadeToBlack);
        assert_eq!(TransitionKind::from(CliTransitionKind::FadeToWhite),    TransitionKind::FadeToWhite);
        assert_eq!(TransitionKind::from(CliTransitionKind::CrossDissolve),  TransitionKind::CrossDissolve);
        assert_eq!(TransitionKind::from(CliTransitionKind::WipeHorizontal), TransitionKind::WipeHorizontal);
        assert_eq!(TransitionKind::from(CliTransitionKind::WipeVertical),   TransitionKind::WipeVertical);
    }

    #[test]
    fn test_resolve_track_id_by_name() {
        let mut proj = Project::new("test");
        let id = proj.add_track(TrackKind::Video, "Main Video");
        let resolved = resolve_track_id(&proj, "Main Video").unwrap();
        assert_eq!(resolved, id);
    }

    #[test]
    fn test_resolve_track_id_by_uuid() {
        let proj = Project::new("test");
        let id = Uuid::new_v4();
        let resolved = resolve_track_id(&proj, &id.to_string()).unwrap();
        assert_eq!(resolved, id);
    }

    #[test]
    fn test_resolve_track_id_not_found() {
        let proj = Project::new("test");
        let result = resolve_track_id(&proj, "Nonexistent");
        assert!(result.is_err());
    }
}
