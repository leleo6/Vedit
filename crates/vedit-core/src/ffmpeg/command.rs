use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use tokio::process::Command;

/// Constructor fluent para comandos FFmpeg
#[derive(Debug, Default)]
pub struct FfmpegCommand {
    args: Vec<String>,
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

    pub fn video_filter(&mut self, filter: impl Into<String>) -> &mut Self {
        self.args.push("-vf".into());
        self.args.push(filter.into());
        self
    }

    pub fn audio_filter(&mut self, filter: impl Into<String>) -> &mut Self {
        self.args.push("-af".into());
        self.args.push(filter.into());
        self
    }

    pub fn output(&mut self, path: &Path) -> &mut Self {
        self.output = Some(path.to_path_buf());
        self
    }

    pub fn build_args(&self) -> Vec<String> {
        let mut args = self.args.clone();
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

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
