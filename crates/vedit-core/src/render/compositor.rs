use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::project::Project;
use crate::project::clip::{ImageMode, KenBurnsEffect};
use crate::project::track::TrackKind;
use crate::render::{AudioFormat, VideoFormat, RenderJob, RenderOutput};
use crate::render::filter_chain::build_video_clip_filters;
use crate::motion::MovementFormula;
use crate::ffmpeg::command::FfmpegCommand;
use crate::ffmpeg::escape_filter_arg;

/// Compositor final: mezcla video + audio + imágenes y genera el output final
pub async fn composite<F>(job: &RenderJob, project: &Project, on_progress: Option<F>) -> Result<RenderOutput> 
where 
    F: FnMut(f64) + Send + 'static 
{
    tracing::info!("Compositor iniciado → {:?}", job.output_path);

    // Resolver configuración: usa la del job o los valores por defecto
    let cfg = job.config.clone().unwrap_or_default();

    let mut cmd = FfmpegCommand::new();
    cmd.hide_banner().overwrite();

    // Aplicar límite de threads si está configurado
    let thread_args = cfg.ffmpeg_thread_args();
    if !thread_args.is_empty() {
        let refs: Vec<&str> = thread_args.iter().map(|s| s.as_str()).collect();
        cmd.raw_args(&refs);
        tracing::debug!("FFmpeg threads: {}", cfg.max_threads);
    }

    let (frame_w, frame_h) = (project.metadata.width, project.metadata.height);
    let total_duration = project.duration_secs();

    // Si hay una región, el canvas solo dura lo que la región
    let render_duration = job.region
        .map(|r| r.duration_secs.min(total_duration - r.start_secs))
        .unwrap_or(total_duration)
        .max(0.001); // garantizar duración positiva

    if let Some(region) = &job.region {
        tracing::info!("Región de render: {}", region);
    }

    let temp_dir = resolve_temp_dir(project);
    let _ = std::fs::create_dir_all(&temp_dir);

    pre_analyze_stabilization(project, &temp_dir).await;

    let mut complex_filters = Vec::new();
    // Fondo base (negro) — duración limitada a la región si aplica
    complex_filters.push(format!("color=c=black:s={}x{}:d={:.3}:r={}[bg0]", frame_w, frame_h, render_duration, project.metadata.fps));

    let mut input_idx = 0;
    let mut current_bg = "bg0".to_string();

    // ── Inputs: video clips ───────────────────────────────────────────────
    for track in project.tracks.iter().filter(|t| !t.muted) {
        for clip in &track.video_clips {
            cmd.input(&clip.source_path).ss(clip.source_start);
            if let Some(end) = clip.source_end {
                cmd.to(end - clip.source_start);
            }

            let mut vf = build_video_clip_filters(clip, frame_w, frame_h);

            // Estabilización (requiere archivo .trf pre-generado)
            if clip.stabilize {
                let trf_path = temp_dir.join(format!("{}_stab.trf", clip.id));
                vf.push(format!(
                    "vidstabtransform=input='{}':optzoom=1:zoomspeed=0.25",
                    escape_filter_arg(&trf_path.to_string_lossy())
                ));
            }

            // Sincronización de tiempo en el timeline
            vf.push(format!("setpts=PTS-STARTPTS+{:.3}/TB", clip.timeline_start));

            let v_node = format!("v{}", input_idx);
            let filter_chain = if vf.is_empty() {
                format!("[{}:v]null[{}]", input_idx, v_node)
            } else {
                format!("[{}:v]{}[{}]", input_idx, vf.join(","), v_node)
            };
            complex_filters.push(filter_chain);

            let ox = (clip.position.x * frame_w as f64).round() as i64;
            let oy = (clip.position.y * frame_h as f64).round() as i64;
            let clip_end = clip.timeline_start + clip.duration();
            let next_bg = format!("bg{}", input_idx + 1);
            complex_filters.push(format!(
                "[{}][{}]overlay={}:{}:enable='between(t,{:.3},{:.3})':eof_action=pass[{}]",
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
            
            // Sincronización de tiempo (PTS)
            vf_parts.push(format!("setpts=PTS-STARTPTS+{:.3}/TB", clip.timeline_start));

            let img_node = format!("img{}", input_idx);
            let filter_chain = if vf_parts.is_empty() {
                format!("[{}:v]null[{}]", input_idx, img_node)
            } else {
                format!("[{}:v]{}[{}]", input_idx, vf_parts.join(","), img_node)
            };
            complex_filters.push(filter_chain);

            let (ox, oy) = resolve_overlay_position(clip, frame_w, frame_h);
            let next_bg = format!("bg{}", input_idx + 1);

            match clip.mode {
                ImageMode::Background | ImageMode::Fullscreen => {
                    complex_filters.push(format!(
                        "[{}][{}]overlay=0:0:enable='between(t,{:.3},{:.3})':eof_action=pass[{}]",
                        current_bg, img_node,
                        clip.timeline_start, clip.timeline_start + clip.duration_secs,
                        next_bg
                    ));
                }
                ImageMode::Overlay => {
                    complex_filters.push(format!(
                        "[{}][{}]overlay={}:{}:enable='between(t,{:.3},{:.3})':eof_action=pass[{}]",
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

    // ── Text clips (no añaden inputs, usan drawtext sobre el background actual) ──────────
    
    let mut text_tracks: Vec<_> = project
        .tracks
        .iter()
        .filter(|t| t.kind == TrackKind::Text && !t.muted)
        .collect();
    text_tracks.sort_by_key(|t| t.layer_order);

    let mut text_node_idx = 0;
    for track in text_tracks {
        for clip in &track.text_clips {
            if let Ok(filter) = crate::render::text::build_drawtext_filter(clip, frame_w, frame_h, &temp_dir) {
                let next_bg = format!("txt_bg{}", text_node_idx);
                complex_filters.push(format!("[{}]{}[{}]", current_bg, filter, next_bg));
                current_bg = next_bg;
                text_node_idx += 1;
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

    // Map outputs
    // Si hay fórmula de movimiento, la aplicamos como un overlay dinámico final
    // sobre la composición (eval=frame permite re-evaluar por frame)
    if let Some(formula) = &job.motion_formula {
        let motion_filter = build_motion_overlay_filter(formula, frame_w, frame_h, render_duration);
        let motion_bg = "motion_out".to_string();
        complex_filters.push(format!(
            "[{bg}][{bg}]{filter}[{out}]",
            bg     = current_bg,
            filter = motion_filter,
            out    = motion_bg,
        ));
        current_bg = motion_bg;
        tracing::info!("Fórmula de movimiento '{}' aplicada al output final", formula);
    }

    if !complex_filters.is_empty() {
        cmd.complex_filter(complex_filters.join(";"));
    }

    cmd.raw_args(&["-map", &format!("[{}]", current_bg)]);
    if !audio_inputs.is_empty() {
        cmd.raw_args(&["-map", "[aout]"]);
    }

    // ── Codecs de salida ──────────────────────────────────────────────────
    if job.is_live_preview {
        cmd.raw_args(&["-f", "nut"])
           .video_codec("rawvideo")
           .audio_codec("pcm_s16le")
           .output(std::path::Path::new("-"));
           
        cmd.run_piped_to_ffplay().await?;
    } else {
        // Codec de video: MP4 usa el encoder preferido por el usuario
        let vcodec = match &job.video_format {
            Some(VideoFormat::Mp4) | None => cfg.preferred_encoder.as_ffmpeg_codec(),
            Some(VideoFormat::Mkv)        => "libx265",
            Some(VideoFormat::Mov)        => "prores",
        };

        // Filtro de hardware adicional para VA-API
        if cfg.preferred_encoder.requires_hwaccel_filter() {
            cmd.raw_args(&["-vaapi_device", &cfg.vaapi_device]);
        }

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

        // Limitar duración de salida: región o proyecto completo
        let out_duration = job.region
            .map(|r| r.duration_secs)
            .unwrap_or(total_duration);
        cmd.raw_args(&["-t", &format!("{:.3}", out_duration)]);

        // Seek de salida si la región no comienza en 0
        if let Some(region) = &job.region {
            if region.start_secs > 0.001 {
                // -ss antes del output aplica trim en el stream de salida
                cmd.raw_args(&["-ss", &format!("{:.3}", region.start_secs)]);
            }
        }

        let res = if let Some(cb) = on_progress {
            cmd.run_with_progress(out_duration, cb).await
        } else {
            cmd.run().await
        };
        res?;
    }

    // Limpieza del directorio temporal: solo si cleanup_cache_on_exit está activo
    if cfg.cleanup_cache_on_exit {
        let _ = std::fs::remove_dir_all(&temp_dir);
        tracing::info!("Caché temporal eliminado: {:?}", temp_dir);
    } else {
        tracing::debug!("Caché temporal conservado en {:?} (cleanup_cache_on_exit = false)", temp_dir);
    }

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
// Helpers internos del compositor
// ─────────────────────────────────────────────────────────────────────────────

/// Resuelve la ruta al directorio de caché temporal del proyecto,
/// respetando `cache_dir` global si está configurado.
fn resolve_temp_dir(project: &Project) -> PathBuf {
    // Nota: aquí no tenemos acceso directo a la config porque resolve_temp_dir
    // se llama antes de que cfg esté disponible. El caller (composite) puede
    // sobreescribir con cfg.resolve_cache_dir() en el futuro si se pasa.
    project
        .path
        .as_ref()
        .and_then(|p| p.parent())
        .unwrap_or(Path::new("."))
        .join(".vedit_cache")
}

/// Construye un filtro overlay con movimiento dinámico usando `eval=frame`.
///
/// El efecto de movimiento se aplica sobre el stream compuesto completo,
/// re-aplicando el fondo negro como base y usando el `MovementFormula`
/// para posicionar la composición de forma animada.
///
/// El parámetro `duration_secs` limita la duración del efecto al rango de render.
fn build_motion_overlay_filter(
    formula: &MovementFormula,
    frame_w: u32,
    frame_h: u32,
    duration_secs: f64,
) -> String {
    let exprs = formula.to_ffmpeg_exprs(frame_w, frame_h);
    format!(
        "overlay=x='{}':y='{}':enable='between(t,0,{:.3})':eval=frame:eof_action=pass",
        exprs.x, exprs.y, duration_secs,
    )
}

/// Pre-analiza la estabilización de todos los clips que la requieran.
/// Genera los archivos `.trf` necesarios para `vidstabtransform`.
async fn pre_analyze_stabilization(project: &Project, temp_dir: &Path) {
    for track in project.tracks.iter().filter(|t| !t.muted) {
        for clip in &track.video_clips {
            if !clip.stabilize {
                continue;
            }
            let trf_path = temp_dir.join(format!("{}_stab.trf", clip.id));
            if trf_path.exists() {
                continue;
            }
            tracing::info!("Analizando estabilización para '{}'", clip.name);
            let mut stab_cmd = FfmpegCommand::new();
            stab_cmd.hide_banner().overwrite().input(&clip.source_path);
            if clip.source_start > 0.0 {
                stab_cmd.ss(clip.source_start);
            }
            if let Some(end) = clip.source_end {
                stab_cmd.to(end - clip.source_start);
            }
            stab_cmd.video_filter(format!(
                "vidstabdetect=result='{}'",
                escape_filter_arg(&trf_path.to_string_lossy())
            ));
            stab_cmd.output_format("null").output(Path::new("-"));
            let _ = stab_cmd.run().await;
        }
    }
}

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
