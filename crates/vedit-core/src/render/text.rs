use anyhow::{Result, Context};
use std::path::Path;
use std::fs;
use crate::project::Project;
use crate::project::clip::TextClip;
use crate::render::VideoFormat;
use crate::ffmpeg::command::FfmpegCommand;

/// Función auxiliar para generar el filtro `drawtext` de FFmpeg
pub fn build_drawtext_filter(
    clip: &TextClip,
    _frame_w: u32,
    _frame_h: u32,
    temp_dir: &Path,
) -> Result<String> {
    // Escape text or use textfile. textfile is safer for multiline and quotes.
    let text_file = temp_dir.join(format!("text_{}.txt", clip.id));
    fs::write(&text_file, &clip.text)
        .with_context(|| format!("Error al escribir archivo temporal de texto {:?}", text_file))?;

    let mut dt = vec![];
    
    // Configuración base
    dt.push(format!("textfile='{}'", text_file.to_string_lossy().replace("\\", "/")));
    
    // Fuente y tamaño
    if clip.style.font_family.contains("/") || clip.style.font_family.contains("\\") || clip.style.font_family.ends_with(".ttf") || clip.style.font_family.ends_with(".otf") {
        dt.push(format!("fontfile='{}'", clip.style.font_family.replace("\\", "/")));
    } else {
        dt.push(format!("font='{}'", clip.style.font_family));
    }
    dt.push(format!("fontsize={}", clip.style.font_size));
    
    // Color y opacidad
    dt.push(format!("fontcolor={}", clip.style.color.to_ffmpeg_hex()));
    
    // Box (fondo)
    if let Some(ref bg) = clip.style.bg_color {
        dt.push("box=1".into());
        dt.push(format!("boxcolor={}", bg.to_ffmpeg_hex()));
        dt.push("boxborderw=10".into()); // padding
    }
    
    // Alineación multilínea (drawtext lo maneja limitado, pero line_spacing ayuda)
    if clip.style.line_height != 1.0 {
        // En drawtext, line_spacing se da en píxeles. Aproximación basada en fontsize.
        let spacing_px = (clip.style.font_size as f64 * (clip.style.line_height - 1.0)).round() as i32;
        dt.push(format!("line_spacing={}", spacing_px));
    }

    // Shadow
    if let Some(ref sh) = clip.style.shadow {
        dt.push(format!("shadowcolor={}", sh.color.to_ffmpeg_hex()));
        dt.push(format!("shadowx={:.1}", sh.offset_x));
        dt.push(format!("shadowy={:.1}", sh.offset_y));
    }
    
    // Stroke
    if let Some(ref st) = clip.style.stroke {
        dt.push(format!("bordercolor={}", st.color.to_ffmpeg_hex()));
        dt.push(format!("borderw={:.1}", st.width));
    }

    // Posicionamiento
    let (x_expr, y_expr) = clip.resolve_ffmpeg_position();
    dt.push(format!("x={}", x_expr));
    dt.push(format!("y={}", y_expr));

    // Animación y Fade de opacidad (alpha)
    // drawtext permite modificar el alpha mediante ecuaciones en el color, o usar la opción `alpha`.
    // Para simplificar el filtro y hacerlo robusto, el alpha base es el del color, y le aplicamos los fades.
    let start_t = clip.timeline_start;
    let end_t = clip.timeline_start + clip.duration_secs;
    
    let mut alpha_expr = "1.0".to_string(); // opacidad total
    if let Some(ref fi) = clip.fade_in {
        let dur = fi.duration_secs;
        alpha_expr = format!("if(lt(t,{start_t}+{dur}),(t-{start_t})/{dur},1)");
    }
    if let Some(ref fo) = clip.fade_out {
        let dur = fo.duration_secs;
        let fo_start = end_t - dur;
        let out_expr = format!("if(gt(t,{fo_start}),({end_t}-t)/{dur},1)");
        if alpha_expr == "1.0" {
            alpha_expr = out_expr;
        } else {
            alpha_expr = format!("min({},{})", alpha_expr, out_expr);
        }
    }
    if alpha_expr != "1.0" {
        dt.push(format!("alpha='{}'", alpha_expr));
    }

    // Habilitar durante el rango del clip
    dt.push(format!("enable='between(t,{:.3},{:.3})'", start_t, end_t));

    Ok(format!("drawtext={}", dt.join(":")))
}

/// Renderiza un preview que solo contiene fondo negro y los textos
pub async fn render_text_preview(
    project: &Project,
    output: &Path,
    format: &VideoFormat,
    width: u32,
    height: u32,
) -> Result<()> {
    tracing::info!("Renderizando preview de texto → {:?}", output);

    let temp_dir = project.path.as_ref().and_then(|p| p.parent()).unwrap_or(Path::new(".")).join(".vedit_cache");
    fs::create_dir_all(&temp_dir)?;

    let mut cmd = FfmpegCommand::new();
    cmd.hide_banner().overwrite();

    let total_duration = project.duration_secs();
    if total_duration <= 0.0 {
        anyhow::bail!("El proyecto está vacío");
    }

    let mut complex_filters = Vec::new();
    // Fondo base
    complex_filters.push(format!("color=c=black:s={}x{}:d={:.3}[bg0]", width, height, total_duration));

    let mut current_bg = "bg0".to_string();

    let clips: Vec<&TextClip> = project.tracks.iter()
        .filter(|t| !t.muted)
        .flat_map(|t| t.text_clips.iter())
        .collect();

    for (input_idx, clip) in clips.into_iter().enumerate() {
        let filter = build_drawtext_filter(clip, width, height, &temp_dir)?;
        let next_bg = format!("bg{}", input_idx + 1);
        complex_filters.push(format!("[{}]{}[{}]", current_bg, filter, next_bg));
        current_bg = next_bg;
    }

    if !complex_filters.is_empty() {
        cmd.complex_filter(complex_filters.join(";"));
    }

    cmd.raw_args(&["-map", &format!("[{}]", current_bg)]);
    cmd.raw_args(&["-an"]); // sin audio

    let vcodec = match format {
        VideoFormat::Mp4 => "libx264",
        VideoFormat::Mkv => "libx265",
        VideoFormat::Mov => "prores",
    };
    cmd.video_codec(vcodec).output(output);

    let res = cmd.run().await;
    
    // Limpieza de caché temporal
    let _ = fs::remove_dir_all(&temp_dir);
    res
}
