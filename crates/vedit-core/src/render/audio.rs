use anyhow::Result;
use std::path::Path;
use crate::project::Project;
use crate::render::AudioFormat;
use crate::ffmpeg::command::FfmpegCommand;

/// Renderiza únicamente el audio del proyecto
pub async fn render_audio(
    project: &Project,
    output: &Path,
    format: &AudioFormat,
) -> Result<()> {
    tracing::info!("Renderizando audio → {:?} ({})", output, format);

    let mut cmd = FfmpegCommand::new();

    // Agregar inputs de todos los clips de audio de todos los tracks
    for track in project.tracks.iter().filter(|t| !t.muted) {
        for clip in &track.audio_clips {
            cmd.input(&clip.source_path)
               .ss(clip.source_start);
            if let Some(end) = clip.source_end {
                cmd.to(end - clip.source_start);
            }
        }
    }

    let codec = match format {
        AudioFormat::Mp3  => "libmp3lame",
        AudioFormat::Wav  => "pcm_s16le",
        AudioFormat::Aac  => "aac",
        AudioFormat::Flac => "flac",
        AudioFormat::Ogg  => "libvorbis",
    };

    cmd.audio_codec(codec)
       .output(output);

    cmd.run().await
}
