use anyhow::Result;
use std::path::Path;
use crate::project::Project;
use crate::project::clip::{VideoClip, TransitionKind};
use crate::render::VideoFormat;
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
        // Agregar input
        cmd.input(&clip.source_path).ss(clip.source_start);
        if let Some(end) = clip.source_end {
            cmd.to(end - clip.source_start);
        }

        let mut vf: Vec<String> = Vec::new();

        // ── Reversa ─────────────────────────────────────────────────────────
        if clip.reverse {
            vf.push("reverse".to_string());
        }

        // ── Velocidad ────────────────────────────────────────────────────────
        if (clip.speed - 1.0).abs() > 0.001 {
            vf.push(format!("setpts={:.6}*PTS", 1.0 / clip.speed));
        }

        // ── Crop ─────────────────────────────────────────────────────────────
        if let Some(ref c) = clip.crop {
            vf.push(format!(
                "crop=iw-{}-{}:ih-{}-{}:{}:{}",
                c.left, c.right, c.top, c.bottom, c.left, c.top
            ));
        }

        // ── Escala ───────────────────────────────────────────────────────────
        let target_w = (width  as f64 * clip.scale.width ).round() as u32;
        let target_h = (height as f64 * clip.scale.height).round() as u32;
        vf.push(format!("scale={}:{}", target_w, target_h));

        // ── Flip ─────────────────────────────────────────────────────────────
        if clip.flip_horizontal { vf.push("hflip".to_string()); }
        if clip.flip_vertical   { vf.push("vflip".to_string()); }

        // ── Rotación ─────────────────────────────────────────────────────────
        if clip.rotation_deg.abs() > 0.001 {
            let rad = clip.rotation_deg * std::f64::consts::PI / 180.0;
            vf.push(format!("rotate={:.6}:fillcolor=none", rad));
        }

        // ── Corrección de color ──────────────────────────────────────────────
        if clip.color.is_active() {
            // EQ filter: brightness, contrast, saturation
            vf.push(format!(
                "eq=brightness={:.4}:contrast={:.4}:saturation={:.4}",
                clip.color.brightness, clip.color.contrast, clip.color.saturation
            ));
            // Temperatura de color (aproximación: ajuste de tint usando hue)
            if let Some(temp) = clip.color.temperature_k {
                // 6500K = neutro; mayor = más cálido (+ rojo/naranja); menor = más frío (+ azul)
                let tint = (temp - 6500.0) / 6500.0 * 0.3; // normalizado
                vf.push(format!("hue=s=1:H={:.4}", tint));
            }
            // LUT .cube
            if let Some(ref lut) = clip.color.lut_path {
                vf.push(format!("lut3d='{}'", lut.display()));
            }
        }

        // ── Efectos visuales ─────────────────────────────────────────────────
        if clip.effects.is_active() {
            if let Some(r) = clip.effects.blur_radius {
                vf.push(format!("gblur=sigma={:.2}", r));
            }
            if let Some(s) = clip.effects.sharpen {
                vf.push(format!("unsharp=luma_msize_x=5:luma_msize_y=5:luma_amount={:.2}", s));
            }
            if let Some(v) = clip.effects.vignette {
                // angle controla la intensidad de la viñeta
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

        // ── Fade ─────────────────────────────────────────────────────────────
        if let Some(ref fi) = clip.fade_in {
            vf.push(format!("fade=t=in:st=0:d={:.3}", fi.duration_secs));
        }
        if let Some(ref fo) = clip.fade_out {
            let clip_dur = clip.duration();
            let fo_start = (clip_dur - fo.duration_secs).max(0.0);
            vf.push(format!("fade=t=out:st={:.3}:d={:.3}", fo_start, fo.duration_secs));
        }

        // ── Transición de salida ──────────────────────────────────────────────
        if let Some(ref tr) = clip.transition_out {
            let clip_dur = clip.duration();
            let tr_start = (clip_dur - tr.duration_secs).max(0.0);
            match tr.kind {
                TransitionKind::FadeToBlack => {
                    vf.push(format!("fade=t=out:st={:.3}:d={:.3}:color=black", tr_start, tr.duration_secs));
                }
                TransitionKind::FadeToWhite => {
                    vf.push(format!("fade=t=out:st={:.3}:d={:.3}:color=white", tr_start, tr.duration_secs));
                }
                TransitionKind::WipeHorizontal => {
                    // Wipe horizontal via crop animado
                    vf.push(format!(
                        "crop=w='if(gt(t,{tr_start:.3}),iw*(1-(t-{tr_start:.3})/{dur:.3}),iw)':h=ih:x=0:y=0",
                        tr_start = tr_start,
                        dur = tr.duration_secs
                    ));
                }
                TransitionKind::WipeVertical => {
                    vf.push(format!(
                        "crop=w=iw:h='if(gt(t,{tr_start:.3}),ih*(1-(t-{tr_start:.3})/{dur:.3}),ih)':x=0:y=0",
                        tr_start = tr_start,
                        dur = tr.duration_secs
                    ));
                }
                // CrossDissolve y Cut no modifican el clip individual
                _ => {}
            }
        }

        // Ensamblar filtro del clip
        let v_node = format!("v{}", input_idx);
        let filter_chain = if vf.is_empty() {
            format!("[{}:v]copy[{}]", input_idx, v_node)
        } else {
            format!("[{}:v]{}[{}]", input_idx, vf.join(","), v_node)
        };
        complex_filters.push(filter_chain);

        // Calcular posición de overlay en píxeles
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

    // Sin audio
    cmd.raw_args(&["-an"]);

    let vcodec = match format {
        VideoFormat::Mp4 => "libx264",
        VideoFormat::Mkv => "libx265",
        VideoFormat::Mov => "prores",
    };
    cmd.video_codec(vcodec).output(output);

    cmd.run().await
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
