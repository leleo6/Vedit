use std::path::PathBuf;
use anyhow::Result;
use clap::Subcommand;
use console::style;
use vedit_core::project::Project;
use vedit_core::render::{self, AudioFormat, VideoFormat, AspectRatio, RenderJob};
use super::{success, section, spinner};

// ── Enums con ValueEnum para el CLI ───────────────────────────────────────────

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum CliVideoFormat {
    Mp4,
    Mkv,
    Mov,
}

impl From<CliVideoFormat> for VideoFormat {
    fn from(v: CliVideoFormat) -> Self {
        match v {
            CliVideoFormat::Mp4 => VideoFormat::Mp4,
            CliVideoFormat::Mkv => VideoFormat::Mkv,
            CliVideoFormat::Mov => VideoFormat::Mov,
        }
    }
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum CliAudioFormat {
    Mp3,
    Wav,
    Aac,
    Flac,
    Ogg,
}

impl From<CliAudioFormat> for AudioFormat {
    fn from(v: CliAudioFormat) -> Self {
        match v {
            CliAudioFormat::Mp3  => AudioFormat::Mp3,
            CliAudioFormat::Wav  => AudioFormat::Wav,
            CliAudioFormat::Aac  => AudioFormat::Aac,
            CliAudioFormat::Flac => AudioFormat::Flac,
            CliAudioFormat::Ogg  => AudioFormat::Ogg,
        }
    }
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum CliAspect {
    /// 16:9 panorámica (1920x1080)
    #[value(name = "16:9")]
    Widescreen,
    /// 9:16 vertical reels/shorts (1080x1920)
    #[value(name = "9:16")]
    Portrait,
}

impl From<CliAspect> for AspectRatio {
    fn from(a: CliAspect) -> Self {
        match a {
            CliAspect::Widescreen => AspectRatio::Widescreen,
            CliAspect::Portrait   => AspectRatio::Portrait,
        }
    }
}

// ── Subcomandos ───────────────────────────────────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum RenderCmd {
    /// Renderiza el proyecto completo (video + audio)
    Full {
        /// Ruta al proyecto .vedit
        #[arg(short, long)]
        project: PathBuf,
        /// Archivo de salida
        #[arg(short, long)]
        output: PathBuf,
        /// Formato de video de salida
        #[arg(long, value_enum, default_value = "mp4")]
        format: CliVideoFormat,
        /// Codec de audio embebido
        #[arg(long, value_enum, default_value = "aac")]
        audio: CliAudioFormat,
        /// Relación de aspecto: 16:9 | 9:16
        #[arg(long, value_enum, default_value = "16:9")]
        aspect: CliAspect,
    },
    /// Exporta únicamente el audio del proyecto
    Audio {
        #[arg(short, long)]
        project: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        /// Formato de audio: mp3, wav, aac, flac, ogg
        #[arg(long, value_enum, default_value = "mp3")]
        format: CliAudioFormat,
    },
    /// Renderiza únicamente el video (sin audio)
    Video {
        #[arg(short, long)]
        project: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long, value_enum, default_value = "mp4")]
        format: CliVideoFormat,
        #[arg(long, value_enum, default_value = "16:9")]
        aspect: CliAspect,
    },
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn run(cmd: RenderCmd) -> Result<()> {
    match cmd {
        RenderCmd::Full { project: proj_path, output, format, audio, aspect } => {
            let project = Project::load(&proj_path).await?;
            project.validate_for_render()?;
            let vfmt: VideoFormat = format.into();
            let afmt: AudioFormat = audio.into();
            let asp:  AspectRatio = aspect.into();
            let (w, h) = asp.dimensions();

            section("Renderizando proyecto");
            println!("  {} {}x{}", style("Resolución:").dim(), w, h);
            println!("  {} {}", style("Video:").dim(), vfmt);
            println!("  {} {}", style("Audio:").dim(), afmt);
            println!("  {} {}", style("Salida:").dim(), style(output.display()).cyan());

            let pb = spinner("Procesando con FFmpeg...");

            let job = RenderJob {
                project_path: proj_path,
                output_path: output,
                audio_only: false,
                video_format: Some(vfmt),
                audio_format: Some(afmt),
                aspect: Some(asp),
            };

            let result = render::compositor::composite(&job, &project).await?;
            pb.finish_and_clear();

            print_render_result(result);
        }

        RenderCmd::Audio { project: proj_path, output, format } => {
            let project = Project::load(&proj_path).await?;
            project.validate_for_render()?;
            let afmt: AudioFormat = format.into();

            section("Exportando audio");
            println!("  {} {}", style("Formato:").dim(), afmt);
            println!("  {} {}", style("Salida:").dim(), style(output.display()).cyan());

            let pb = spinner("Exportando audio...");
            render::audio::render_audio(&project, &output, &afmt).await?;
            pb.finish_and_clear();
            success(&format!("Audio exportado → {}", output.display()));
        }

        RenderCmd::Video { project: proj_path, output, format, aspect } => {
            let project = Project::load(&proj_path).await?;
            project.validate_for_render()?;
            let vfmt: VideoFormat = format.into();
            let asp:  AspectRatio = aspect.into();
            let (w, h) = asp.dimensions();

            section("Renderizando video");
            println!("  {} {}x{}", style("Resolución:").dim(), w, h);
            println!("  {} {}", style("Formato:").dim(), vfmt);
            println!("  {} {}", style("Salida:").dim(), style(output.display()).cyan());

            let pb = spinner("Renderizando video...");
            render::video::render_video(&project, &output, &vfmt, w, h).await?;
            pb.finish_and_clear();
            success(&format!("Video exportado → {}", output.display()));
        }
    }
    Ok(())
}

fn print_render_result(result: vedit_core::render::RenderOutput) {
    let size_mb = result.size_bytes as f64 / 1_048_576.0;
    section("Render completado 🎬");
    println!("  {} {}", style("Archivo:").dim(), style(result.output_path.display()).cyan().bold());
    println!("  {} {:.2}s", style("Duración:").dim(), result.duration_secs);
    println!("  {} {:.2} MB", style("Tamaño:").dim(), size_mb);
    println!("\n  {} Render completado exitosamente", style("✔").green().bold());
}
