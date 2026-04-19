use anyhow::Result;

use crate::project::Project;
use crate::project::clip::{ImageMode, KenBurnsEffect, TransitionKind};
use crate::project::track::TrackKind;
use crate::render::{AudioFormat, VideoFormat, RenderJob, RenderOutput};
use crate::ffmpeg::command::FfmpegCommand;
use crate::ffmpeg::escape_filter_arg;

/// Compositor final: mezcla video + audio + imágenes y genera el output final
pub async fn composite<F>(job: &RenderJob, project: &Project, on_progress: Option<F>) -> Result<RenderOutput> 
where 
    F: FnMut(f64) + Send + 'static 
{
    tracing::info!("Compositor iniciado → {:?}", job.output_path);

    let mut cmd = FfmpegCommand::new();
    cmd.hide_banner().overwrite();

    let (frame_w, frame_h) = (project.metadata.width, project.metadata.height);
    let total_duration = project.duration_secs();

    let temp_dir = project.path.as_ref().and_then(|p| p.parent()).unwrap_or(std::path::Path::new(".")).join(".vedit_cache");
    let _ = std::fs::create_dir_all(&temp_dir);

    // Pre-procesar estabilización de video
    for track in project.tracks.iter().filter(|t| !t.muted) {
        for clip in &track.video_clips {
            if clip.stabilize {
                let trf_path = temp_dir.join(format!("{}_stab.trf", clip.id));
                if !trf_path.exists() {
                    tracing::info!("Analizando estabilización para {}", clip.name);
                    let mut stab_cmd = FfmpegCommand::new();
                    stab_cmd.hide_banner().overwrite();
                    stab_cmd.input(&clip.source_path);
                    if clip.source_start > 0.0 {
                        stab_cmd.ss(clip.source_start);
                    }
                    if let Some(end) = clip.source_end {
                        stab_cmd.to(end - clip.source_start);
                    }
                    stab_cmd.video_filter(format!("vidstabdetect=result='{}'", escape_filter_arg(&trf_path.to_string_lossy())));
                    stab_cmd.output_format("null").output(std::path::Path::new("-"));
                    let _ = stab_cmd.run().await;
                }
            }
        }
    }

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

            let mut vf: Vec<String> = Vec::new();

            // Reversa
            if clip.reverse {
                vf.push("reverse".to_string());
            }
            // Velocidad
            if (clip.speed - 1.0).abs() > 0.001 {
                vf.push(format!("setpts={:.6}*PTS", 1.0 / clip.speed));
            }
            // Crop
            if let Some(ref c) = clip.crop {
                vf.push(format!(
                    "crop=iw-{}-{}:ih-{}-{}:{}:{}",
                    c.left, c.right, c.top, c.bottom, c.left, c.top
                ));
            }
            // Escalar al frame
            let target_w = (frame_w as f64 * clip.scale.width ).round() as u32;
            let target_h = (frame_h as f64 * clip.scale.height).round() as u32;
            vf.push(format!("scale={}:{}", target_w, target_h));
            // Flip
            if clip.flip_horizontal { vf.push("hflip".to_string()); }
            if clip.flip_vertical   { vf.push("vflip".to_string()); }
            // Rotación
            if clip.rotation_deg.abs() > 0.001 {
                let rad = clip.rotation_deg * std::f64::consts::PI / 180.0;
                vf.push(format!("rotate={:.6}:fillcolor=none", rad));
            }
            // Corrección de color
            if clip.color.is_active() {
                vf.push(format!(
                    "eq=brightness={:.4}:contrast={:.4}:saturation={:.4}",
                    clip.color.brightness, clip.color.contrast, clip.color.saturation
                ));
                if let Some(ref lut) = clip.color.lut_path {
                    vf.push(format!("lut3d='{}'", escape_filter_arg(&lut.to_string_lossy())));
                }
            }
            // Efectos visuales
            if clip.effects.is_active() {
                if let Some(r) = clip.effects.blur_radius {
                    vf.push(format!("gblur=sigma={:.2}", r));
                }
                if let Some(s) = clip.effects.sharpen {
                    vf.push(format!("unsharp=luma_msize_x=5:luma_msize_y=5:luma_amount={:.2}", s));
                }
                if let Some(v) = clip.effects.vignette {
                    let angle = v * std::f64::consts::PI / 2.0;
                    vf.push(format!("vignette=angle={:.4}", angle));
                }
                if let Some(n) = clip.effects.noise {
                    let noise_val = (n * 100.0) as u32;
                    vf.push(format!("noise=alls={}:allf=t+u", noise_val));
                }
                if clip.effects.deinterlace {
                    vf.push("yadif=mode=1".to_string());
                }
            }
            // Fades
            if let Some(ref fi) = clip.fade_in {
                vf.push(format!("fade=t=in:st=0:d={:.3}", fi.duration_secs));
            }
            if let Some(ref fo) = clip.fade_out {
                let clip_dur = clip.duration();
                let fo_start = (clip_dur - fo.duration_secs).max(0.0);
                vf.push(format!("fade=t=out:st={:.3}:d={:.3}", fo_start, fo.duration_secs));
            }
            // Transición de salida
            if let Some(ref trans) = clip.transition_out {
                let clip_dur = clip.duration();
                let tr_start = (clip_dur - trans.duration_secs).max(0.0);
                match trans.kind {
                    TransitionKind::Cut => {}
                    TransitionKind::FadeToBlack => {
                        vf.push(format!("fade=t=out:st={:.3}:d={:.3}:color=black", tr_start, trans.duration_secs));
                    }
                    TransitionKind::FadeToWhite => {
                        vf.push(format!("fade=t=out:st={:.3}:d={:.3}:color=white", tr_start, trans.duration_secs));
                    }
                    TransitionKind::CrossDissolve => {
                        vf.push(format!("format=rgba,fade=t=out:st={:.3}:d={:.3}:alpha=1", tr_start, trans.duration_secs));
                    }
                    TransitionKind::WipeHorizontal => {
                        vf.push(format!("crop=iw*max(0\\,1-(t-{:.3})/{:.3}):ih:0:0", tr_start, trans.duration_secs));
                    }
                    TransitionKind::WipeVertical => {
                        vf.push(format!("crop=iw:ih*max(0\\,1-(t-{:.3})/{:.3}):0:0", tr_start, trans.duration_secs));
                    }
                }
            }
            // Estabilización
            if clip.stabilize {
                let trf_path = temp_dir.join(format!("{}_stab.trf", clip.id));
                vf.push(format!("vidstabtransform=input='{}':optzoom=1:zoomspeed=0.25", escape_filter_arg(&trf_path.to_string_lossy())));
            }

            let v_node = format!("v{}", input_idx);
            let filter_chain = if vf.is_empty() {
                format!("[{}:v]copy[{}]", input_idx, v_node)
            } else {
                format!("[{}:v]{}[{}]", input_idx, vf.join(","), v_node)
            };
            complex_filters.push(filter_chain);

            // Overlay sobre el fondo
            let ox = (clip.position.x * frame_w as f64).round() as i64;
            let oy = (clip.position.y * frame_h as f64).round() as i64;
            let clip_end = clip.timeline_start + clip.duration();
            let next_bg = format!("bg{}", input_idx + 1);
            complex_filters.push(format!(
                "[{}][{}]overlay={}:{}:enable='between(t,{:.3},{:.3})'[{}]",
                current_bg, v_node, ox, oy,
                clip.timeline_start, clip_end,
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

    // ── Text clips ────────────────────────────────────────────────────────
    
    let mut text_tracks: Vec<_> = project
        .tracks
        .iter()
        .filter(|t| t.kind == TrackKind::Text && !t.muted)
        .collect();
    text_tracks.sort_by_key(|t| t.layer_order);

    for track in text_tracks {
        for clip in &track.text_clips {
            if let Ok(filter) = crate::render::text::build_drawtext_filter(clip, frame_w, frame_h, &temp_dir) {
                let next_bg = format!("bg{}", input_idx + 1);
                complex_filters.push(format!("[{}]{}[{}]", current_bg, filter, next_bg));
                current_bg = next_bg;
                input_idx += 1;
            }
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

    // Si el job tiene una duración, la usamos para el progreso
    cmd.raw_args(&["-t", &format!("{:.3}", total_duration)]);

    let res = if let Some(cb) = on_progress {
        cmd.run_with_progress(total_duration, cb).await
    } else {
        cmd.run().await
    };
    
    // Limpieza de archivos de texto temporales
    let _ = std::fs::remove_dir_all(&temp_dir);
    
    res?;

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
