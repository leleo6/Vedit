use std::path::{Path, PathBuf};
use std::process::Stdio;
use anyhow::{Context, Result};
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};

/// Constructor fluent para comandos FFmpeg
#[derive(Debug, Default)]
pub struct FfmpegCommand {
    args: Vec<String>,
    video_filters: Vec<String>,
    audio_filters: Vec<String>,
    complex_filters: Vec<String>,
    output: Option<PathBuf>,
}

impl FfmpegCommand {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn hide_banner(&mut self) -> &mut Self {
        self.args.push("-hide_banner".into());
        // Forzar multithreading automático por defecto
        self.args.push("-threads".into());
        self.args.push("0".into());
        self
    }

    pub fn overwrite(&mut self) -> &mut Self {
        self.args.push("-y".into());
        self
    }

    /// Agrega un archivo de entrada
    pub fn input(&mut self, path: &Path) -> &mut Self {
        self.args.push("-i".into());
        self.args.push(path.to_string_lossy().to_string());
        self
    }

    /// Posición de inicio en el input (debe ir ANTES de -i para seek rápido)
    pub fn ss(&mut self, seconds: f64) -> &mut Self {
        self.args.push("-ss".into());
        self.args.push(format!("{:.3}", seconds));
        self
    }

    /// Duración del segmento
    pub fn to(&mut self, seconds: f64) -> &mut Self {
        self.args.push("-t".into());
        self.args.push(format!("{:.3}", seconds));
        self
    }

    pub fn video_codec(&mut self, codec: &str) -> &mut Self {
        self.args.push("-c:v".into());
        self.args.push(codec.into());
        self
    }

    pub fn audio_codec(&mut self, codec: &str) -> &mut Self {
        self.args.push("-c:a".into());
        self.args.push(codec.into());
        self
    }

    pub fn output_format(&mut self, format: &str) -> &mut Self {
        self.args.push("-f".into());
        self.args.push(format.into());
        self
    }

    pub fn video_filter(&mut self, filter: impl Into<String>) -> &mut Self {
        self.video_filters.push(filter.into());
        self
    }

    pub fn audio_filter(&mut self, filter: impl Into<String>) -> &mut Self {
        self.audio_filters.push(filter.into());
        self
    }

    pub fn complex_filter(&mut self, filter: impl Into<String>) -> &mut Self {
        self.complex_filters.push(filter.into());
        self
    }

    pub fn output(&mut self, path: &Path) -> &mut Self {
        self.output = Some(path.to_path_buf());
        self
    }

    /// Agrega argumentos arbitrarios a la línea de comando (útil para flags especiales)
    pub fn raw_args(&mut self, args: &[&str]) -> &mut Self {
        for a in args {
            self.args.push(a.to_string());
        }
        self
    }

    pub fn build_args(&self) -> Vec<String> {
        let mut args = self.args.clone();
        
        if !self.complex_filters.is_empty() {
            args.push("-filter_complex".into());
            args.push(self.complex_filters.join(";"));
        } else {
            if !self.video_filters.is_empty() {
                args.push("-vf".into());
                args.push(self.video_filters.join(","));
            }
            if !self.audio_filters.is_empty() {
                args.push("-af".into());
                args.push(self.audio_filters.join(","));
            }
        }

        if let Some(ref out) = self.output {
            args.push(out.to_string_lossy().to_string());
        }
        args
    }

    /// Ejecuta el comando ffmpeg de forma asíncrona
    pub async fn run(&self) -> Result<()> {
        let args = self.build_args();
        tracing::debug!("ffmpeg {}", args.join(" "));

        let status = Command::new("ffmpeg")
            .args(&args)
            .status()
            .await
            .context("No se pudo ejecutar ffmpeg. ¿Está instalado?")?;

        if !status.success() {
            anyhow::bail!(
                "ffmpeg terminó con código de error: {}",
                status.code().unwrap_or(-1)
            );
        }
        Ok(())
    }

    /// Ejecuta el comando ffmpeg capturando el progreso
    pub async fn run_with_progress<F>(&self, total_duration_secs: f64, mut on_progress: F) -> Result<()> 
    where 
        F: FnMut(f64) + Send + 'static 
    {
        let mut args = self.build_args();
        let total_duration_ms = (total_duration_secs * 1000.0) as i64;

        // Insertar -progress pipe:1 antes del output
        // Buscamos el último argumento (el output) para insertar justo antes
        if !args.is_empty() {
            let last_idx = args.len() - 1;
            args.insert(last_idx, "-progress".into());
            args.insert(last_idx + 1, "pipe:1".into());
        } else {
             args.push("-progress".into());
             args.push("pipe:1".into());
        }

        let mut child = Command::new("ffmpeg")
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("No se pudo ejecutar ffmpeg")?;

        let stdout = child.stdout.take().unwrap();
        let mut reader = BufReader::new(stdout).lines();

        loop {
            tokio::select! {
                line = reader.next_line() => {
                    if let Ok(Some(l)) = line {
                    if let Some(stripped) = l.strip_prefix("out_time_ms=") {
                            if let Ok(ms) = stripped.parse::<i64>() {
                                if total_duration_ms > 0 {
                                    let progress = (ms as f64 / total_duration_ms as f64).clamp(0.0, 1.0);
                                    on_progress(progress);
                                }
                            }
                        }
                        if l == "progress=end" {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                status = child.wait() => {
                    let status = status?;
                    if !status.success() {
                        anyhow::bail!("ffmpeg terminó con error: {}", status.code().unwrap_or(-1));
                    }
                    break;
                }
                _ = tokio::signal::ctrl_c() => {
                    let _ = child.kill().await;
                    anyhow::bail!("Renderizado cancelado por el usuario.");
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    

    #[test]
    fn test_ffmpeg_command_builder() {
        let mut cmd = FfmpegCommand::new();
        cmd.hide_banner()
           .overwrite()
           .ss(15.250)
           .input(Path::new("video.mp4"))
           .video_codec("libx264")
           .to(5.0)
           .output(Path::new("output.mp4"));

        let args = cmd.build_args();
        assert_eq!(
            args,
            vec![
                "-hide_banner", "-threads", "0", "-y", 
                "-ss", "15.250", 
                "-i", "video.mp4", 
                "-c:v", "libx264", 
                "-t", "5.000", 
                "output.mp4"
            ]
        );
    }
}
