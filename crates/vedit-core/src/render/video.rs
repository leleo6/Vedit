use anyhow::Result;
use std::path::Path;
use crate::project::Project;
use crate::render::VideoFormat;
use crate::ffmpeg::command::FfmpegCommand;

/// Renderiza únicamente el video del proyecto (sin audio)
pub async fn render_video(
    project: &Project,
    output: &Path,
    format: &VideoFormat,
    width: u32,
    height: u32,
) -> Result<()> {
    tracing::info!("Renderizando video → {:?} ({}) {}x{}", output, format, width, height);

    let mut cmd = FfmpegCommand::new();

    for track in project.tracks.iter().filter(|t| !t.muted) {
        for clip in &track.video_clips {
            cmd.input(&clip.source_path)
               .ss(clip.source_start);
            if let Some(end) = clip.source_end {
                cmd.to(end - clip.source_start);
            }
        }
    }

    cmd.video_filter(format!("scale={}:{}", width, height))
       .video_codec("libx264")
       .output(output);

    cmd.run().await
}
