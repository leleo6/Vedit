use anyhow::Result;

use crate::project::Project;
use crate::render::{AudioFormat, VideoFormat, RenderJob, RenderOutput};
use crate::ffmpeg::command::FfmpegCommand;

/// Compositor final: mezcla video + audio y genera el output final
pub async fn composite(job: &RenderJob, project: &Project) -> Result<RenderOutput> {
    tracing::info!("Compositor iniciado → {:?}", job.output_path);

    // Construimos la línea de comando FFmpeg compuesta
    let mut cmd = FfmpegCommand::new();
    cmd.hide_banner().overwrite();

    // Agregar tracks de video como inputs
    for track in project.tracks.iter().filter(|t| !t.muted) {
        for clip in &track.video_clips {
            cmd.input(&clip.source_path).ss(clip.source_start);
            if let Some(end) = clip.source_end {
                cmd.to(end - clip.source_start);
            }
        }
        // Agregar clips de audio
        for clip in &track.audio_clips {
            cmd.input(&clip.source_path).ss(clip.source_start);
            if let Some(end) = clip.source_end {
                cmd.to(end - clip.source_start);
            }
            if let Some(ref fi) = clip.fade_in {
                cmd.audio_filter(format!("afade=t=in:d={}", fi.duration_secs));
            }
            if let Some(ref fo) = clip.fade_out {
                cmd.audio_filter(format!("afade=t=out:d={}", fo.duration_secs));
            }
        }
    }

    // Codec de video y audio según formato
    let vcodec = match &job.video_format {
        Some(VideoFormat::Mp4) | None => "libx264",
        Some(VideoFormat::Mkv) => "libx265",
        Some(VideoFormat::Mov) => "prores",
    };

    let acodec = match &job.audio_format {
        Some(AudioFormat::Mp3) => "libmp3lame",
        Some(AudioFormat::Aac) | None => "aac",
        Some(AudioFormat::Wav) => "pcm_s16le",
        Some(AudioFormat::Flac) => "flac",
        Some(AudioFormat::Ogg) => "libvorbis",
    };

    cmd.video_codec(vcodec)
       .audio_codec(acodec)
       .output(&job.output_path);

    cmd.run().await?;

    // Obtener tamaño del archivo output
    let size_bytes = std::fs::metadata(&job.output_path)
        .map(|m| m.len())
        .unwrap_or(0);

    Ok(RenderOutput {
        output_path: job.output_path.clone(),
        duration_secs: project.duration_secs(),
        size_bytes,
    })
}
