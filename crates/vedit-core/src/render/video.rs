use anyhow::Result;
use std::path::Path;
use crate::project::Project;
use crate::project::clip::VideoClip;
use crate::render::VideoFormat;
use crate::render::filter_chain::build_video_clip_filters;
use crate::ffmpeg::command::FfmpegCommand;

/// Renderiza únicamente el video del proyecto (sin audio)
/// Aplica transformaciones, velocidad, corrección de color, efectos y transiciones.
pub async fn render_video(
    project: &Project,
    output: &Path,
    format: &VideoFormat,
    width: u32,
    height: u32,
) -> Result<()> {
    tracing::info!("Renderizando video → {:?} ({}) {}x{}", output, format, width, height);

    let mut cmd = FfmpegCommand::new();
    cmd.hide_banner().overwrite();

    let total_duration = project.duration_secs();

    // Recopilar todos los clips de video no muteados
    let clips: Vec<&VideoClip> = project
        .tracks
        .iter()
        .filter(|t| !t.muted)
        .flat_map(|t| t.video_clips.iter())
        .collect();

    if clips.is_empty() {
        anyhow::bail!("No hay clips de video para renderizar");
    }

    let mut complex_filters: Vec<String> = Vec::new();

    // Fondo negro base
    complex_filters.push(format!(
        "color=c=black:s={}x{}:d={:.3}[bg0]",
        width, height, total_duration
    ));

    let mut current_bg = "bg0".to_string();

    for (input_idx, clip) in clips.iter().enumerate() {
        cmd.input(&clip.source_path).ss(clip.source_start);
        if let Some(end) = clip.source_end {
            cmd.to(end - clip.source_start);
        }

        let mut vf = build_video_clip_filters(clip, width, height);

        // Sincronización de tiempo en el timeline
        vf.push(format!("setpts=PTS-STARTPTS+{:.3}/TB", clip.timeline_start));

        let v_node = format!("v{}", input_idx);
        let filter_chain = if vf.is_empty() {
            format!("[{}:v]copy[{}]", input_idx, v_node)
        } else {
            format!("[{}:v]{}[{}]", input_idx, vf.join(","), v_node)
        };
        complex_filters.push(filter_chain);

        let ox = (clip.position.x * width as f64).round() as i64;
        let oy = (clip.position.y * height as f64).round() as i64;
        let clip_end = clip.timeline_start + clip.duration();

        let next_bg = format!("bg{}", input_idx + 1);
        complex_filters.push(format!(
            "[{}][{}]overlay={}:{}:enable='between(t,{:.3},{:.3})'[{}]",
            current_bg, v_node, ox, oy,
            clip.timeline_start, clip_end,
            next_bg
        ));
        current_bg = next_bg;
    }

    if !complex_filters.is_empty() {
        cmd.complex_filter(complex_filters.join(";"));
    }

    cmd.raw_args(&["-map", &format!("[{}]", current_bg)]);

    cmd.raw_args(&["-an"]);
    cmd.video_codec(video_codec_for(format)).output(output);
    cmd.run().await
}

/// Selecciona el codec FFmpeg correspondiente al formato de video solicitado.
fn video_codec_for(format: &VideoFormat) -> &'static str {
    match format {
        VideoFormat::Mp4 => "libx264",
        VideoFormat::Mkv => "libx265",
        VideoFormat::Mov => "prores",
    }
}

/// Exporta un frame específico del proyecto como imagen (screenshot)
pub async fn export_frame(
    project: &Project,
    output: &Path,
    at_secs: f64,
) -> Result<()> {
    tracing::info!("Exportando frame en {:.3}s → {:?}", at_secs, output);

    let clips: Vec<&VideoClip> = project
        .tracks
        .iter()
        .filter(|t| !t.muted)
        .flat_map(|t| t.video_clips.iter())
        .filter(|c| c.timeline_start <= at_secs && c.timeline_start + c.duration() > at_secs)
        .collect();

    if clips.is_empty() {
        anyhow::bail!("No hay clip de video en el instante {:.3}s", at_secs);
    }

    // Tomar el primer clip encontrado en ese instante
    let clip = clips[0];
    let offset_in_clip = at_secs - clip.timeline_start;
    let source_seek = clip.source_start + offset_in_clip * clip.speed;

    let mut cmd = FfmpegCommand::new();
    cmd.hide_banner().overwrite();
    cmd.raw_args(&["-ss", &format!("{:.3}", source_seek)]);
    cmd.input(&clip.source_path);
    cmd.raw_args(&["-vframes", "1"]);
    cmd.output(output);
    cmd.run().await
}
