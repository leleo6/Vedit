use std::f64::consts::PI;
use crate::project::clip::{VideoClip, TransitionKind};
use crate::ffmpeg::escape_filter_arg;

/// Construye la cadena de filtros FFmpeg para un `VideoClip`.
///
/// Esta función centraliza toda la lógica de transformación de video para
/// eliminar la duplicación entre `compositor.rs` y `video.rs`. Es interna
/// al módulo `render` y no forma parte de la API pública del crate.
///
/// # Responsabilidades
/// - Reversa y velocidad
/// - Crop, escala, flip, rotación
/// - Corrección de color y LUT
/// - Efectos visuales (blur, sharpen, vignette, noise, deinterlace)
/// - Fades de entrada/salida
/// - Transiciones de salida
/// - Sincronización de tiempo (setpts)
///
/// No incluye el `setpts=PTS-STARTPTS+...` de sincronización de timeline;
/// el llamador lo agrega explícitamente para mantener control del instante.
pub(super) fn build_video_clip_filters(
    clip: &VideoClip,
    frame_w: u32,
    frame_h: u32,
) -> Vec<String> {
    let mut filters = Vec::new();

    apply_reverse(clip, &mut filters);
    apply_speed(clip, &mut filters);
    apply_crop(clip, &mut filters);
    apply_scale(clip, frame_w, frame_h, &mut filters);
    apply_flip(clip, &mut filters);
    apply_rotation(clip, &mut filters);
    apply_color_correction(clip, &mut filters);
    apply_visual_effects(clip, &mut filters);
    apply_fades(clip, &mut filters);
    apply_transition_out(clip, &mut filters);

    filters
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers privados — cada uno cubre una única transformación (SRP)
// ─────────────────────────────────────────────────────────────────────────────

fn apply_reverse(clip: &VideoClip, filters: &mut Vec<String>) {
    if clip.reverse {
        filters.push("reverse".to_string());
    }
}

fn apply_speed(clip: &VideoClip, filters: &mut Vec<String>) {
    if (clip.speed - 1.0).abs() > 0.001 {
        filters.push(format!("setpts={:.6}*PTS", 1.0 / clip.speed));
    }
}

fn apply_crop(clip: &VideoClip, filters: &mut Vec<String>) {
    if let Some(ref c) = clip.crop {
        filters.push(format!(
            "crop=iw-{}-{}:ih-{}-{}:{}:{}",
            c.left, c.right, c.top, c.bottom, c.left, c.top
        ));
    }
}

fn apply_scale(clip: &VideoClip, frame_w: u32, frame_h: u32, filters: &mut Vec<String>) {
    let target_w = (frame_w as f64 * clip.scale.width).round() as u32;
    let target_h = (frame_h as f64 * clip.scale.height).round() as u32;
    filters.push(format!("scale={}:{}", target_w, target_h));
}

fn apply_flip(clip: &VideoClip, filters: &mut Vec<String>) {
    if clip.flip_horizontal { filters.push("hflip".to_string()); }
    if clip.flip_vertical   { filters.push("vflip".to_string()); }
}

fn apply_rotation(clip: &VideoClip, filters: &mut Vec<String>) {
    if clip.rotation_deg.abs() > 0.001 {
        let radians = clip.rotation_deg * PI / 180.0;
        filters.push(format!("rotate={:.6}:fillcolor=none", radians));
    }
}

fn apply_color_correction(clip: &VideoClip, filters: &mut Vec<String>) {
    if !clip.color.is_active() {
        return;
    }

    filters.push(format!(
        "eq=brightness={:.4}:contrast={:.4}:saturation={:.4}",
        clip.color.brightness, clip.color.contrast, clip.color.saturation
    ));

    if let Some(temp_k) = clip.color.temperature_k {
        // 6500 K = neutro. La desviación se aproxima con un ajuste de tint en hue.
        let tint = (temp_k - 6500.0) / 6500.0 * 0.3;
        filters.push(format!("hue=s=1:H={:.4}", tint));
    }

    if let Some(ref lut) = clip.color.lut_path {
        filters.push(format!("lut3d='{}'", escape_filter_arg(&lut.to_string_lossy())));
    }
}

fn apply_visual_effects(clip: &VideoClip, filters: &mut Vec<String>) {
    if !clip.effects.is_active() {
        return;
    }

    if let Some(radius) = clip.effects.blur_radius {
        filters.push(format!("gblur=sigma={:.2}", radius));
    }
    if let Some(amount) = clip.effects.sharpen {
        filters.push(format!("unsharp=luma_msize_x=5:luma_msize_y=5:luma_amount={:.2}", amount));
    }
    if let Some(intensity) = clip.effects.vignette {
        let angle = intensity * PI / 2.0;
        filters.push(format!("vignette=angle={:.4}", angle));
    }
    if let Some(level) = clip.effects.noise {
        let noise_val = (level * 100.0) as u32;
        filters.push(format!("noise=alls={}:allf=t+u", noise_val));
    }
    if clip.effects.deinterlace {
        filters.push("yadif=mode=1".to_string());
    }
}

fn apply_fades(clip: &VideoClip, filters: &mut Vec<String>) {
    if let Some(ref fi) = clip.fade_in {
        filters.push(format!("fade=t=in:st=0:d={:.3}", fi.duration_secs));
    }
    if let Some(ref fo) = clip.fade_out {
        let clip_dur = clip.duration();
        let fo_start = (clip_dur - fo.duration_secs).max(0.0);
        filters.push(format!("fade=t=out:st={:.3}:d={:.3}", fo_start, fo.duration_secs));
    }
}

fn apply_transition_out(clip: &VideoClip, filters: &mut Vec<String>) {
    let Some(ref tr) = clip.transition_out else { return };

    let clip_dur = clip.duration();
    let tr_start = (clip_dur - tr.duration_secs).max(0.0);

    match tr.kind {
        TransitionKind::Cut => {}
        TransitionKind::FadeToBlack => {
            filters.push(format!("fade=t=out:st={:.3}:d={:.3}:color=black", tr_start, tr.duration_secs));
        }
        TransitionKind::FadeToWhite => {
            filters.push(format!("fade=t=out:st={:.3}:d={:.3}:color=white", tr_start, tr.duration_secs));
        }
        TransitionKind::CrossDissolve => {
            filters.push(format!("format=rgba,fade=t=out:st={:.3}:d={:.3}:alpha=1", tr_start, tr.duration_secs));
        }
        TransitionKind::WipeHorizontal => {
            filters.push(format!(
                "crop=iw*max(0\\,1-(t-{:.3})/{:.3}):ih:0:0",
                tr_start, tr.duration_secs
            ));
        }
        TransitionKind::WipeVertical => {
            filters.push(format!(
                "crop=iw:ih*max(0\\,1-(t-{:.3})/{:.3}):0:0",
                tr_start, tr.duration_secs
            ));
        }
    }
}
