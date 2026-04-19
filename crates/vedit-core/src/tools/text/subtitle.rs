use std::path::PathBuf;
use std::fs;
use anyhow::{Result, Context};
use uuid::Uuid;
use crate::project::Project;
use crate::project::clip::TextClip;
use crate::tools::Tool;

/// Importa un archivo .srt y lo convierte en clips de texto en un track
pub struct ImportSrt {
    pub track_id: Uuid,
    pub file_path: PathBuf,
}

impl Tool for ImportSrt {
    fn name(&self) -> &str { "import_srt" }

    fn apply(&self, project: &mut Project) -> Result<()> {
        let track = project
            .track_mut(self.track_id)
            .ok_or_else(|| anyhow::anyhow!("Track {} no encontrado", self.track_id))?;

        let content = fs::read_to_string(&self.file_path)
            .with_context(|| format!("No se pudo leer el archivo {:?}", self.file_path))?;

        let mut clips_added = 0;
        let blocks: Vec<&str> = content.trim().split("\n\n").collect();

        for block in blocks {
            let lines: Vec<&str> = block.lines().collect();
            if lines.len() >= 3 {
                // lines[0] is the index
                // lines[1] is the timestamp range, e.g., "00:00:01,000 --> 00:00:04,000"
                let times = lines[1];
                if let Some((start_str, end_str)) = times.split_once(" --> ") {
                    if let (Ok(start), Ok(end)) = (parse_srt_time(start_str), parse_srt_time(end_str)) {
                        let text = lines[2..].join("\n");
                        let duration = (end - start).max(0.1);
                        let name = format!("Sub {}", lines[0]);
                        let mut clip = TextClip::new(name, text, start, duration);
                        // Default position for subtitles: bottom center
                        clip.position_preset = crate::project::clip::TextPositionPreset::BottomCenter;
                        clip.margin = 40.0;
                        track.add_text_clip(clip);
                        clips_added += 1;
                    }
                }
            }
        }

        tracing::info!(
            "Importados {} subtítulos desde {:?} al track {}",
            clips_added, self.file_path, self.track_id
        );
        Ok(())
    }
}

pub struct ImportVtt {
    pub track_id: Uuid,
    pub file_path: PathBuf,
}

impl Tool for ImportVtt {
    fn name(&self) -> &str { "import_vtt" }

    fn apply(&self, project: &mut Project) -> Result<()> {
        let track = project
            .track_mut(self.track_id)
            .ok_or_else(|| anyhow::anyhow!("Track {} no encontrado", self.track_id))?;

        let content = fs::read_to_string(&self.file_path)
            .with_context(|| format!("No se pudo leer el archivo {:?}", self.file_path))?;

        if !content.trim_start().starts_with("WEBVTT") {
            anyhow::bail!("El archivo no parece ser un archivo WebVTT válido.");
        }

        let mut clips_added = 0;
        // Normalizar saltos de línea y separar por doble salto
        let normalized = content.replace("\r\n", "\n");
        let blocks: Vec<&str> = normalized.trim().split("\n\n").collect();

        // Ignorar el primer bloque que suele ser "WEBVTT"
        for block in blocks.into_iter().skip(1) {
            let lines: Vec<&str> = block.lines().collect();
            if lines.is_empty() { continue; }
            
            // Un bloque VTT puede tener un identificador opcional en la primera línea
            let mut time_line_idx = 0;
            if !lines[0].contains("-->") && lines.len() > 1 {
                time_line_idx = 1;
            }

            if lines.len() > time_line_idx {
                let times = lines[time_line_idx];
                if let Some((start_str, end_str)) = times.split_once(" --> ") {
                    // Ignorar settings opcionales de VTT al final (ej. " line:0 align:center")
                    let end_time_str = end_str.split_whitespace().next().unwrap_or(end_str);
                    
                    if let (Ok(start), Ok(end)) = (parse_vtt_time(start_str), parse_vtt_time(end_time_str)) {
                        let text = lines[time_line_idx + 1..].join("\n");
                        // Remover tags HTML simples como <b>, <i>
                        let clean_text = text.replace("<b>", "").replace("</b>", "")
                                             .replace("<i>", "").replace("</i>", "")
                                             .replace("<u>", "").replace("</u>", "");
                                             
                        let duration = (end - start).max(0.1);
                        let name = format!("Sub {}", clips_added + 1);
                        let mut clip = TextClip::new(name, clean_text, start, duration);
                        clip.position_preset = crate::project::clip::TextPositionPreset::BottomCenter;
                        clip.margin = 40.0;
                        track.add_text_clip(clip);
                        clips_added += 1;
                    }
                }
            }
        }

        tracing::info!(
            "Importados {} subtítulos desde {:?} al track {}",
            clips_added, self.file_path, self.track_id
        );
        Ok(())
    }
}

pub struct ExportSrt {
    pub track_id: Uuid,
    pub file_path: PathBuf,
}

impl Tool for ExportSrt {
    fn name(&self) -> &str { "export_srt" }

    fn apply(&self, project: &mut Project) -> Result<()> {
        let track = project
            .track_mut(self.track_id)
            .ok_or_else(|| anyhow::anyhow!("Track {} no encontrado", self.track_id))?;

        let mut srt_content = String::new();
        
        let mut clips = track.text_clips.clone();
        clips.sort_by(|a, b| a.timeline_start.partial_cmp(&b.timeline_start).unwrap());

        for (idx, clip) in clips.iter().enumerate() {
            let start = format_srt_time(clip.timeline_start);
            let end = format_srt_time(clip.timeline_start + clip.duration_secs);
            
            srt_content.push_str(&format!("{}\n", idx + 1));
            srt_content.push_str(&format!("{} --> {}\n", start, end));
            srt_content.push_str(&format!("{}\n\n", clip.text));
        }

        fs::write(&self.file_path, srt_content.trim())
            .with_context(|| format!("No se pudo escribir en {:?}", self.file_path))?;

        tracing::info!(
            "Exportados {} subtítulos a {:?}",
            clips.len(), self.file_path
        );
        Ok(())
    }
}

// Helpers

fn parse_srt_time(time_str: &str) -> Result<f64> {
    let parts: Vec<&str> = time_str.trim().split(&[':', ','][..]).collect();
    if parts.len() == 4 {
        let h: f64 = parts[0].parse()?;
        let m: f64 = parts[1].parse()?;
        let s: f64 = parts[2].parse()?;
        let ms: f64 = parts[3].parse()?;
        Ok(h * 3600.0 + m * 60.0 + s + ms / 1000.0)
    } else {
        anyhow::bail!("Formato de tiempo inválido: {}", time_str)
    }
}

fn format_srt_time(seconds: f64) -> String {
    let hours = (seconds / 3600.0).floor() as u32;
    let mins = ((seconds % 3600.0) / 60.0).floor() as u32;
    let secs = (seconds % 60.0).floor() as u32;
    let millis = ((seconds.fract()) * 1000.0).round() as u32;
    format!("{:02}:{:02}:{:02},{:03}", hours, mins, secs, millis)
}

fn parse_vtt_time(time_str: &str) -> Result<f64> {
    let parts: Vec<&str> = time_str.trim().split(&[':', '.'][..]).collect();
    if parts.len() == 3 {
        let m: f64 = parts[0].parse()?;
        let s: f64 = parts[1].parse()?;
        let ms: f64 = parts[2].parse()?;
        Ok(m * 60.0 + s + ms / 1000.0)
    } else if parts.len() == 4 {
        let h: f64 = parts[0].parse()?;
        let m: f64 = parts[1].parse()?;
        let s: f64 = parts[2].parse()?;
        let ms: f64 = parts[3].parse()?;
        Ok(h * 3600.0 + m * 60.0 + s + ms / 1000.0)
    } else {
        anyhow::bail!("Formato de tiempo VTT inválido: {}", time_str)
    }
}
