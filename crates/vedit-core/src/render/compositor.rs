use anyhow::Result;

use crate::project::Project;
use crate::project::clip::{ImageMode, KenBurnsEffect};
use crate::project::track::TrackKind;
use crate::render::{AudioFormat, VideoFormat, RenderJob, RenderOutput};
use crate::ffmpeg::command::FfmpegCommand;

/// Compositor final: mezcla video + audio + imágenes y genera el output final
pub async fn composite(job: &RenderJob, project: &Project) -> Result<RenderOutput> {
    tracing::info!("Compositor iniciado → {:?}", job.output_path);

    let mut cmd = FfmpegCommand::new();
    cmd.hide_banner().overwrite();

    let (frame_w, frame_h) = (project.metadata.width, project.metadata.height);
    let total_duration = project.duration_secs();

    let mut complex_filters = Vec::new();
    // Generar un fondo negro base para todo el proyecto
    complex_filters.push(format!("color=c=black:s={}x{}:d={:.3}[bg0]", frame_w, frame_h, total_duration));

    let mut input_idx = 0;
    let mut current_bg = "bg0".to_string();

    // ── Inputs: video clips ───────────────────────────────────────────────
    for track in project.tracks.iter().filter(|t| !t.muted) {
        for clip in &track.video_clips {
            cmd.input(&clip.source_path).ss(clip.source_start);
            if let Some(end) = clip.source_end {
                cmd.to(end - clip.source_start);
            }
            
            // Escalar video al tamaño del frame
            let v_node = format!("v{}", input_idx);
            complex_filters.push(format!("[{}:v]scale={}:{}[{}]", input_idx, frame_w, frame_h, v_node));
            
            // Overlay sobre el fondo
            let next_bg = format!("bg{}", input_idx + 1);
            complex_filters.push(format!(
                "[{}][{}]overlay=0:0:enable='between(t,{:.3},{:.3})'[{}]",
                current_bg, v_node,
                clip.timeline_start, clip.timeline_start + clip.duration(),
                next_bg
            ));
            current_bg = next_bg;
            input_idx += 1;
        }
    }

    // ── Inputs: image clips ───────────────────────────────────────────────
    let mut image_tracks: Vec<_> = project
        .tracks
        .iter()
        .filter(|t| t.kind == TrackKind::Image && !t.muted)
        .collect();
    image_tracks.sort_by_key(|t| t.layer_order);

    for track in &image_tracks {
        for clip in &track.image_clips {
            cmd.raw_args(&[
                "-loop", "1",
                "-t", &format!("{:.3}", clip.duration_secs),
                "-i", clip.source_path.to_str().unwrap_or(""),
            ]);

            let mut vf_parts = Vec::new();
            if let Some(ref crop) = clip.crop {
                vf_parts.push(format!("crop=iw-{}-{}:ih-{}-{}:{}:{}", crop.left, crop.right, crop.top, crop.bottom, crop.left, crop.top));
            }
            let target_w = (frame_w as f64 * clip.scale.width).round() as u32;
            let target_h = (frame_h as f64 * clip.scale.height).round() as u32;
            vf_parts.push(format!("scale={}:{}", target_w, target_h));

            if let Some(ref kb) = clip.ken_burns {
                vf_parts.push(build_ken_burns_filter(kb, target_w, target_h, clip.duration_secs, project.metadata.fps));
            }
            if clip.rotation_deg.abs() > 0.001 {
                let rad = clip.rotation_deg * std::f64::consts::PI / 180.0;
                vf_parts.push(format!("rotate={:.6}:fillcolor=none", rad));
            }
            if (clip.opacity - 1.0).abs() > 0.001 {
                vf_parts.push(format!("format=rgba,colorchannelmixer=aa={:.3}", clip.opacity));
            }
            if let Some(ref fi) = clip.fade_in {
                vf_parts.push(format!("fade=t=in:st=0:d={:.3}:alpha=1", fi.duration_secs));
            }
            if let Some(ref fo) = clip.fade_out {
                let fo_start = (clip.duration_secs - fo.duration_secs).max(0.0);
                vf_parts.push(format!("fade=t=out:st={:.3}:d={:.3}:alpha=1", fo_start, fo.duration_secs));
            }
            if let Some(ref anim) = clip.entry_animation {
                vf_parts.push(build_entry_animation_filter(anim, target_w, target_h));
            }

            let img_node = format!("img{}", input_idx);
            let filter_chain = if vf_parts.is_empty() {
                format!("[{}:v]copy[{}]", input_idx, img_node)
            } else {
                format!("[{}:v]{}[{}]", input_idx, vf_parts.join(","), img_node)
            };
            complex_filters.push(filter_chain);

            let (ox, oy) = resolve_overlay_position(clip, frame_w, frame_h);
            let next_bg = format!("bg{}", input_idx + 1);

            match clip.mode {
                ImageMode::Background | ImageMode::Fullscreen => {
                    complex_filters.push(format!(
                        "[{}][{}]overlay=0:0:enable='between(t,{:.3},{:.3})'[{}]",
                        current_bg, img_node,
                        clip.timeline_start, clip.timeline_start + clip.duration_secs,
                        next_bg
                    ));
                }
                ImageMode::Overlay => {
                    complex_filters.push(format!(
                        "[{}][{}]overlay={}:{}:enable='between(t,{:.3},{:.3})'[{}]",
                        current_bg, img_node, ox, oy,
                        clip.timeline_start, clip.timeline_start + clip.duration_secs,
                        next_bg
                    ));
                }
            }
            current_bg = next_bg;
            input_idx += 1;
        }
    }

    // ── Audio clips (amix) ───────────────────────────────────────────────
    let mut audio_inputs = Vec::new();
    for track in project.tracks.iter().filter(|t| !t.muted) {
        for clip in &track.audio_clips {
            cmd.input(&clip.source_path).ss(clip.source_start);
            if let Some(end) = clip.source_end {
                cmd.to(end - clip.source_start);
            }
            
            let mut af_parts = Vec::new();
            // Delay para sincronizar en el timeline
            let delay_ms = (clip.timeline_start * 1000.0) as i64;
            af_parts.push(format!("adelay={}|{}", delay_ms, delay_ms));
            
            if let Some(ref fi) = clip.fade_in {
                af_parts.push(format!("afade=t=in:d={}", fi.duration_secs));
            }
            if let Some(ref fo) = clip.fade_out {
                af_parts.push(format!("afade=t=out:d={}", fo.duration_secs));
            }
            
            let a_node = format!("a{}", input_idx);
            complex_filters.push(format!("[{}:a]{}[{}]", input_idx, af_parts.join(","), a_node));
            audio_inputs.push(a_node);
            
            input_idx += 1;
        }
    }

    // Mix audio si hay streams
    if !audio_inputs.is_empty() {
        let amix_inputs: String = audio_inputs.iter().map(|n| format!("[{}]", n)).collect();
        complex_filters.push(format!("{}amix=inputs={}:duration=longest[aout]", amix_inputs, audio_inputs.len()));
    }

    if !complex_filters.is_empty() {
        cmd.complex_filter(complex_filters.join(";"));
    }

    // Map outputs
    cmd.raw_args(&["-map", &format!("[{}]", current_bg)]);
    if !audio_inputs.is_empty() {
        cmd.raw_args(&["-map", "[aout]"]);
    }

    // ── Codecs de salida ──────────────────────────────────────────────────
    let vcodec = match &job.video_format {
        Some(VideoFormat::Mp4) | None => "libx264",
        Some(VideoFormat::Mkv)        => "libx265",
        Some(VideoFormat::Mov)        => "prores",
    };

    let acodec = match &job.audio_format {
        Some(AudioFormat::Mp3)        => "libmp3lame",
        Some(AudioFormat::Aac) | None => "aac",
        Some(AudioFormat::Wav)        => "pcm_s16le",
        Some(AudioFormat::Flac)       => "flac",
        Some(AudioFormat::Ogg)        => "libvorbis",
    };

    cmd.video_codec(vcodec)
       .audio_codec(acodec)
       .output(&job.output_path);

    cmd.run().await?;

    let size_bytes = std::fs::metadata(&job.output_path)
        .map(|m| m.len())
        .unwrap_or(0);

    Ok(RenderOutput {
        output_path: job.output_path.clone(),
        duration_secs: project.duration_secs(),
        size_bytes,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers internos
// ─────────────────────────────────────────────────────────────────────────────

/// Construye el filtro zoompan de FFmpeg para el efecto Ken Burns
fn build_ken_burns_filter(
    kb: &KenBurnsEffect,
    w: u32,
    h: u32,
    duration_secs: f64,
    fps: f64,
) -> String {
    let frames = (duration_secs * fps).round() as u64;
    let zoom_step = (kb.zoom_end - kb.zoom_start) / frames.max(1) as f64;
    let zoom_expr = format!("'min(zoom+{:.6},{:.4})'", zoom_step, kb.zoom_end);
    let x_expr = format!(
        "'{:.0}+{:.4}*on/{}'",
        kb.pan_x_start * w as f64,
        (kb.pan_x_end - kb.pan_x_start) * w as f64,
        frames.max(1)
    );
    let y_expr = format!(
        "'{:.0}+{:.4}*on/{}'",
        kb.pan_y_start * h as f64,
        (kb.pan_y_end - kb.pan_y_start) * h as f64,
        frames.max(1)
    );
    format!(
        "zoompan=z={}:x={}:y={}:d={}:s={}x{}:fps={}",
        zoom_expr, x_expr, y_expr, frames, w, h, fps as u32
    )
}

/// Construye filtro de animación de entrada (30 frames de transición)
fn build_entry_animation_filter(
    anim: &crate::project::clip::EntryAnimation,
    w: u32,
    h: u32,
) -> String {
    use crate::project::clip::EntryAnimation;
    match anim {
        EntryAnimation::ZoomIn =>
            format!("scale='if(lt(n,30),{}*n/30,{})':'if(lt(n,30),{}*n/30,{})'", w, w, h, h),
        EntryAnimation::SlideLeft =>
            format!("pad={}+iw:ih:x='if(lt(n,30),iw*(1-n/30),0)':y=0", w),
        EntryAnimation::SlideRight =>
            format!("pad={}+iw:ih:x='if(lt(n,30),-(iw*(1-n/30)),0)':y=0", w),
        EntryAnimation::SlideTop =>
            format!("pad=iw:{}+ih:x=0:y='if(lt(n,30),ih*(1-n/30),0)'", h),
        EntryAnimation::SlideBottom =>
            format!("pad=iw:{}+ih:x=0:y='if(lt(n,30),-(ih*(1-n/30)),0)'", h),
    }
}

/// Calcula la posición (esquina sup-izquierda) del overlay en píxeles o como expresión centrada
fn resolve_overlay_position(
    clip: &crate::project::clip::ImageClip,
    frame_w: u32,
    frame_h: u32,
) -> (String, String) {
    let ox = match clip.position.x {
        Some(fx) => format!("{}", (fx * frame_w as f64).round() as i64),
        None     => "(main_w-overlay_w)/2".to_string(),
    };
    let oy = match clip.position.y {
        Some(fy) => format!("{}", (fy * frame_h as f64).round() as i64),
        None     => "(main_h-overlay_h)/2".to_string(),
    };
    (ox, oy)
}
