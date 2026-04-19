use std::path::PathBuf;
use anyhow::Result;
use clap::Subcommand;
use uuid::Uuid;
use console::style;
use vedit_core::project::Project;
use vedit_core::project::clip::{TextAlign, TextPositionPreset};
use super::{success, section};

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum CliTextAlign {
    Left, Center, Right,
}

impl From<CliTextAlign> for TextAlign {
    fn from(c: CliTextAlign) -> Self {
        match c {
            CliTextAlign::Left => TextAlign::Left,
            CliTextAlign::Center => TextAlign::Center,
            CliTextAlign::Right => TextAlign::Right,
        }
    }
}

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum CliTextPreset {
    TopLeft, TopCenter, TopRight,
    MiddleLeft, MiddleCenter, MiddleRight,
    BottomLeft, BottomCenter, BottomRight,
    Custom,
}

impl From<CliTextPreset> for TextPositionPreset {
    fn from(c: CliTextPreset) -> Self {
        match c {
            CliTextPreset::TopLeft => TextPositionPreset::TopLeft,
            CliTextPreset::TopCenter => TextPositionPreset::TopCenter,
            CliTextPreset::TopRight => TextPositionPreset::TopRight,
            CliTextPreset::MiddleLeft => TextPositionPreset::MiddleLeft,
            CliTextPreset::MiddleCenter => TextPositionPreset::MiddleCenter,
            CliTextPreset::MiddleRight => TextPositionPreset::MiddleRight,
            CliTextPreset::BottomLeft => TextPositionPreset::BottomLeft,
            CliTextPreset::BottomCenter => TextPositionPreset::BottomCenter,
            CliTextPreset::BottomRight => TextPositionPreset::BottomRight,
            CliTextPreset::Custom => TextPositionPreset::Custom,
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum TextCmd {
    /// Agrega un track de texto al proyecto
    AddTrack {
        #[arg(short, long)]
        project: PathBuf,
        name: String,
        /// Orden de capa visual
        #[arg(long, default_value_t = 100)]
        layer: usize,
    },
    
    /// Agrega un clip de texto a un track
    Add {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
        /// Nombre del clip
        name: String,
        /// Contenido del texto
        text: String,
        /// Inicio en timeline (segundos)
        #[arg(long)]
        at: f64,
        /// Duración en segundos
        #[arg(long, default_value_t = 5.0)]
        duration: f64,
    },

    /// Mueve un clip en el timeline
    Move {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
        clip_id: Uuid,
        /// Nueva posición (segundos)
        at: f64,
    },

    /// Divide un clip en un punto dado
    Split {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
        clip_id: Uuid,
        /// Punto de corte relativo al inicio del clip (segundos)
        at: f64,
    },

    /// Elimina un clip de texto
    Remove {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
        clip_id: Uuid,
    },

    /// Ajusta el estilo del texto (fuente, tamaño, color, negrita, etc.)
    Style {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
        clip_id: Uuid,
        #[arg(long)]
        font: Option<String>,
        #[arg(long)]
        size: Option<u32>,
        #[arg(long)]
        bold: bool,
        #[arg(long)]
        italic: bool,
        #[arg(long)]
        align: Option<CliTextAlign>,
    },

    /// Ajusta la posición del texto
    Position {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
        clip_id: Uuid,
        #[arg(long)]
        preset: Option<CliTextPreset>,
        #[arg(long)]
        x: Option<f64>,
        #[arg(long)]
        y: Option<f64>,
        #[arg(long)]
        margin: Option<f64>,
    },

    /// Importa un archivo .srt como clips de texto
    ImportSrt {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
        srt_file: PathBuf,
    },

    /// Importa un archivo .vtt como clips de texto
    ImportVtt {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
        vtt_file: PathBuf,
    },

    /// Exporta los clips de texto como archivo .srt
    ExportSrt {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
        out_file: PathBuf,
    },

    /// Lista los clips de texto de un track
    List {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
    },
}

pub async fn run(cmd: TextCmd) -> Result<()> {
    match cmd {
        TextCmd::AddTrack { project: proj_path, name, layer } => {
            let mut project = Project::load(&proj_path).await?;
            use vedit_core::project::track::TrackKind;
            let id = project.add_track(TrackKind::Text, &name);
            project.track_mut(id).unwrap().layer_order = layer;
            project.save().await?;
            success(&format!("Track de texto '{}' creado (id: {})", name, id));
        }

        TextCmd::Add { project: proj_path, track, name, text, at, duration } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            
            use vedit_core::project::clip::TextClip;
            let clip = TextClip::new(&name, &text, at, duration);
            let cid = clip.id;
            project.track_mut(tid).unwrap().add_text_clip(clip);
            project.save().await?;
            success(&format!("TextClip '{}' agregado @ {:.2}s (id: {})", name, at, cid));
        }

        TextCmd::Move { project: proj_path, track, clip_id, at } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            let clip = project.track_mut(tid).unwrap()
                .text_clip_mut(clip_id)
                .ok_or_else(|| anyhow::anyhow!("Clip {} no encontrado", clip_id))?;
            clip.timeline_start = at;
            project.save().await?;
            success(&format!("TextClip {} movido a {:.2}s", clip_id, at));
        }

        TextCmd::Split { project: proj_path, track, clip_id, at } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            let t = project.track_mut(tid).unwrap();
            let clip = t.text_clip_mut(clip_id)
                .ok_or_else(|| anyhow::anyhow!("Clip {} no encontrado", clip_id))?;

            if let Some(new_clip) = clip.split_at(at) {
                let new_id = t.add_text_clip(new_clip);
                project.save().await?;
                success(&format!("TextClip dividido. Nuevo clip id: {}", new_id));
            } else {
                anyhow::bail!("No se puede dividir en {:.2}s", at);
            }
        }

        TextCmd::Remove { project: proj_path, track, clip_id } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            if project.track_mut(tid).unwrap().remove_text_clip(clip_id) {
                project.save().await?;
                success(&format!("TextClip {} eliminado", clip_id));
            } else {
                super::warn(&format!("Clip {} no encontrado", clip_id));
            }
        }

        TextCmd::Style { project: proj_path, track, clip_id, font, size, bold, italic, align } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            let clip = project.track_mut(tid).unwrap()
                .text_clip_mut(clip_id)
                .ok_or_else(|| anyhow::anyhow!("Clip {} no encontrado", clip_id))?;

            if let Some(f) = font { clip.style.font_family = f; }
            if let Some(s) = size { clip.style.font_size = s; }
            if bold { clip.style.bold = !clip.style.bold; }
            if italic { clip.style.italic = !clip.style.italic; }
            if let Some(a) = align { clip.style.align = a.into(); }

            project.save().await?;
            success(&format!("Estilo aplicado al TextClip {}", clip_id));
        }

        TextCmd::Position { project: proj_path, track, clip_id, preset, x, y, margin } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            let clip = project.track_mut(tid).unwrap()
                .text_clip_mut(clip_id)
                .ok_or_else(|| anyhow::anyhow!("Clip {} no encontrado", clip_id))?;

            if let Some(p) = preset { clip.position_preset = p.into(); }
            if let Some(px) = x { clip.pos_x = Some(px); }
            if let Some(py) = y { clip.pos_y = Some(py); }
            if let Some(m) = margin { clip.margin = m; }

            project.save().await?;
            success(&format!("Posición aplicada al TextClip {}", clip_id));
        }

        TextCmd::ImportSrt { project: proj_path, track, srt_file } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            
            use vedit_core::tools::text::ImportSrt;
            use vedit_core::tools::Tool;
            let tool = ImportSrt { track_id: tid, file_path: srt_file };
            tool.apply(&mut project)?;
            project.save().await?;
            success("SRT importado correctamente");
        }

        TextCmd::ImportVtt { project: proj_path, track, vtt_file } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            
            use vedit_core::tools::text::ImportVtt;
            use vedit_core::tools::Tool;
            let tool = ImportVtt { track_id: tid, file_path: vtt_file };
            tool.apply(&mut project)?;
            project.save().await?;
            success("VTT importado correctamente");
        }

        TextCmd::ExportSrt { project: proj_path, track, out_file } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;

            use vedit_core::tools::text::ExportSrt;
            use vedit_core::tools::Tool;
            let tool = ExportSrt { track_id: tid, file_path: out_file };
            tool.apply(&mut project)?;
            success("SRT exportado correctamente");
        }

        TextCmd::List { project: proj_path, track } => {
            let project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            let t = project.track(tid).unwrap();
            section(&format!("Text Clips del track '{}'", t.name));

            if t.text_clips.is_empty() {
                println!("  {} No hay clips de texto en este track.", style("ℹ").blue());
                return Ok(());
            }

            for clip in &t.text_clips {
                let text_preview = if clip.text.len() > 30 {
                    format!("{}...", &clip.text[..30].replace("\n", " "))
                } else {
                    clip.text.replace("\n", " ")
                };

                println!(
                    "  {} {} @{:.2}s [dur: {:.2}s] \"{}\"",
                    style("T").cyan(),
                    style(&clip.name).white().bold(),
                    clip.timeline_start,
                    clip.duration(),
                    style(text_preview).yellow()
                );
                println!("    {} {}", style("id:").dim(),  style(clip.id).dim());
                println!("    {} {} {}px {}", 
                    style("style:").dim(), 
                    clip.style.font_family, 
                    clip.style.font_size,
                    if clip.style.bold { "bold" } else { "" }
                );
                println!("    {} {}", style("pos:").dim(), clip.position_preset);
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
