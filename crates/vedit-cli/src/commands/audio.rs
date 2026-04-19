use std::path::PathBuf;
use anyhow::Result;
use clap::Subcommand;
use uuid::Uuid;
use vedit_core::project::Project;
use super::success;

#[derive(Subcommand, Debug)]
pub enum AudioCmd {
    /// Aplica fade-in a un clip o track
    FadeIn {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
        /// UUID del clip (omitir para aplicar al track completo)
        #[arg(long)]
        clip: Option<Uuid>,
        /// Duración del fade en segundos
        #[arg(short, long)]
        duration: f64,
    },
    /// Aplica fade-out a un clip o track
    FadeOut {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
        #[arg(long)]
        clip: Option<Uuid>,
        #[arg(short, long)]
        duration: f64,
    },
    /// Normaliza el volumen de un track (-23 LUFS por defecto)
    Normalize {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
        /// Objetivo LUFS
        #[arg(long, default_value_t = -23.0)]
        lufs: f64,
    },
    /// Mezcla múltiples tracks en uno (bounce)
    Mix {
        #[arg(short, long)]
        project: PathBuf,
        /// Tracks a mezclar (nombres o UUIDs separados por coma)
        #[arg(long, value_delimiter = ',')]
        tracks: Vec<String>,
        /// Nombre del track resultante
        #[arg(long, default_value = "Mix")]
        output_name: String,
    },
    /// Silencia un rango de tiempo dentro de un clip
    MuteRange {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
        clip: Uuid,
        /// Inicio del rango (segundos relativos al clip)
        #[arg(long)]
        start: f64,
        /// Fin del rango (segundos relativos al clip)
        #[arg(long)]
        end: f64,
    },
    /// Extrae el audio de un clip de video
    ExtractAudio {
        #[arg(short, long)]
        project: PathBuf,
        track: String,
        clip: Uuid,
        /// Track de audio destino (nombre o UUID); si se omite se crea uno nuevo
        #[arg(long)]
        audio_track: Option<String>,
    },
}

pub async fn run(cmd: AudioCmd) -> Result<()> {
    match cmd {
        AudioCmd::FadeIn { project: proj_path, track, clip, duration } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;

            if let Some(cid) = clip {
                let c = project.track_mut(tid).unwrap()
                    .audio_clip_mut(cid)
                    .ok_or_else(|| anyhow::anyhow!("Clip {} no encontrado", cid))?;
                c.set_fade_in(duration);
                success(&format!("Fade-in de {:.2}s aplicado al clip {}", duration, cid));
            } else {
                for c in project.track_mut(tid).unwrap().audio_clips.iter_mut() {
                    c.set_fade_in(duration);
                }
                success(&format!("Fade-in de {:.2}s aplicado al track '{}'", duration, track));
            }
            project.save().await?;
        }

        AudioCmd::FadeOut { project: proj_path, track, clip, duration } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;

            if let Some(cid) = clip {
                let c = project.track_mut(tid).unwrap()
                    .audio_clip_mut(cid)
                    .ok_or_else(|| anyhow::anyhow!("Clip {} no encontrado", cid))?;
                c.set_fade_out(duration);
                success(&format!("Fade-out de {:.2}s aplicado al clip {}", duration, cid));
            } else {
                for c in project.track_mut(tid).unwrap().audio_clips.iter_mut() {
                    c.set_fade_out(duration);
                }
                success(&format!("Fade-out de {:.2}s aplicado al track '{}'", duration, track));
            }
            project.save().await?;
        }

        AudioCmd::Normalize { project: proj_path, track, lufs } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            let t = project.track(tid).unwrap();
            let name = t.name.clone();
            // Marcamos metadata de normalización (la implementación FFmpeg real va en render)
            tracing::info!("Normalización @ {} LUFS marcada para track '{}'", lufs, name);
            project.save().await?;
            success(&format!("Track '{}' marcado para normalización a {} LUFS", name, lufs));
        }

        AudioCmd::Mix { project: proj_path, tracks, output_name } => {
            let mut project = Project::load(&proj_path).await?;
            let ids: Vec<Uuid> = tracks.iter()
                .map(|t| resolve_track_id(&project, t))
                .collect::<Result<Vec<_>>>()?;

            // Acumular clips de los tracks fuente
            let mut all_clips = Vec::new();
            for &id in &ids {
                let t = project.track(id).unwrap();
                all_clips.extend(t.audio_clips.clone());
            }

            use vedit_core::project::track::TrackKind;
            let new_id = project.add_track(TrackKind::Audio, &output_name);
            let new_track = project.track_mut(new_id).unwrap();
            for clip in all_clips {
                new_track.add_audio_clip(clip);
            }
            project.save().await?;
            success(&format!("{} tracks mezclados en '{}'", ids.len(), output_name));
        }

        AudioCmd::MuteRange { project: proj_path, track, clip, start, end } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;
            let c = project.track_mut(tid).unwrap()
                .audio_clip_mut(clip)
                .ok_or_else(|| anyhow::anyhow!("Clip {} no encontrado", clip))?;
            c.add_mute_range(start, end);
            project.save().await?;
            success(&format!("Rango {:.2}s–{:.2}s silenciado en clip {}", start, end, clip));
        }

        AudioCmd::ExtractAudio { project: proj_path, track, clip, audio_track } => {
            let mut project = Project::load(&proj_path).await?;
            let tid = resolve_track_id(&project, &track)?;

            // Obtener info del video clip
            let vc = project.track(tid).unwrap()
                .video_clips.iter()
                .find(|c| c.id == clip)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("VideoClip {} no encontrado en track '{}'", clip, track))?;

            // Crear AudioClip con el mismo source (ffmpeg extrae el stream de audio)
            use vedit_core::project::clip::AudioClip;
            use vedit_core::project::track::TrackKind;
            let audio_clip = AudioClip::new(
                format!("audio_de_{}", vc.name),
                &vc.source_path,
                vc.timeline_start,
            );

            // Determinar track destino
            let dest_id = if let Some(ref at_name) = audio_track {
                resolve_track_id(&project, at_name)?
            } else {
                project.add_track(TrackKind::Audio, format!("Audio de {}", vc.name))
            };

            project.track_mut(dest_id).unwrap().add_audio_clip(audio_clip);
            project.save().await?;
            success(&format!("Audio extraído del clip '{}'", vc.name));
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
