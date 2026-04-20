use std::path::PathBuf;
use anyhow::Result;
use clap::Subcommand;
use console::style;
use vedit_core::project::Project;
use vedit_core::render::{self, AudioFormat, VideoFormat, AspectRatio, RenderJob};
use vedit_core::motion::{MovementFormula, MotionPresets, RenderRegion};
use super::{success, section, spinner};
use indicatif::{ProgressBar, ProgressStyle};

// ── Enums con ValueEnum para el CLI ───────────────────────────────────────────

#[derive(Debug, Clone, clap::ValueEnum, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, clap::ValueEnum, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, clap::ValueEnum, serde::Serialize, serde::Deserialize)]
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
        /// Usa un preset guardado para los ajustes de renderizado
        #[arg(long)]
        preset: Option<String>,
        /// Sobrescribir el archivo de salida si ya existe
        #[arg(long, default_value_t = false)]
        force: bool,
        /// Inicio de la región a renderizar (segundos). Omite para renderizar desde el inicio.
        #[arg(long)]
        start_time: Option<f64>,
        /// Duración de la región a renderizar (segundos). Requiere --start-time.
        #[arg(long)]
        duration: Option<f64>,
        /// Preset de movimiento dinámico (ej. dvd_bounce, shake, orbit_slow).
        /// Ejecuta `vedit render motion list` para ver todos los disponibles.
        #[arg(long)]
        motion: Option<String>,
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
        /// Sobrescribir el archivo de salida si ya existe
        #[arg(long, default_value_t = false)]
        force: bool,
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
        /// Sobrescribir el archivo de salida si ya existe
        #[arg(long, default_value_t = false)]
        force: bool,
        /// Inicio de la región a renderizar (segundos)
        #[arg(long)]
        start_time: Option<f64>,
        /// Duración de la región a renderizar (segundos)
        #[arg(long)]
        duration: Option<f64>,
    },
    /// Exporta un frame específico como imagen (screenshot)
    ExportFrame {
        #[arg(short, long)]
        project: PathBuf,
        /// Archivo de salida (PNG, JPG, etc.)
        #[arg(short, long)]
        output: PathBuf,
        /// Instante del timeline en segundos
        #[arg(long)]
        at: f64,
        /// Sobrescribir el archivo de salida si ya existe
        #[arg(long, default_value_t = false)]
        force: bool,
    },
    /// Renderiza un preview rápido solo del texto/subtítulos sobre fondo negro
    TextPreview {
        #[arg(short, long)]
        project: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long, value_enum, default_value = "mp4")]
        format: CliVideoFormat,
        #[arg(long, value_enum, default_value = "16:9")]
        aspect: CliAspect,
        /// Sobrescribir el archivo de salida si ya existe
        #[arg(long, default_value_t = false)]
        force: bool,
    },
    /// Reproduce el proyecto en vivo usando ffplay
    LivePreview {
        #[arg(short, long)]
        project: PathBuf,
        #[arg(long, value_enum, default_value = "16:9")]
        aspect: CliAspect,
    },
    /// Gestiona presets de renderizado
    Preset {
        #[command(subcommand)]
        action: PresetAction,
    },
    /// Gestiona fórmulas de movimiento dinámico
    Motion {
        #[command(subcommand)]
        action: MotionAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum PresetAction {
    /// Guarda un nuevo preset
    Save {
        name: String,
        #[arg(long, value_enum, default_value = "mp4")]
        format: CliVideoFormat,
        #[arg(long, value_enum, default_value = "aac")]
        audio: CliAudioFormat,
        #[arg(long, value_enum, default_value = "16:9")]
        aspect: CliAspect,
    },
    /// Lista los presets guardados
    List,
}

/// Subcomandos para inspeccionar y probar fórmulas de movimiento.
#[derive(Subcommand, Debug)]
pub enum MotionAction {
    /// Lista todos los presets de movimiento disponibles
    List,
    /// Muestra la expresión FFmpeg de un preset
    Show {
        /// Nombre del preset (ej. dvd_bounce, shake)
        name: String,
    },
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RenderPreset {
    pub format: CliVideoFormat,
    pub audio: CliAudioFormat,
    pub aspect: CliAspect,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn run(cmd: RenderCmd) -> Result<()> {
    match cmd {
        RenderCmd::Full { project: proj_path, output, format, audio, aspect, preset, force, start_time, duration, motion } => {
            check_output_not_overwritten(&output, force)?;
            let mut format = format;
            let mut audio = audio;
            let mut aspect = aspect;

            if let Some(name) = preset {
                let presets = PresetRepository::load().unwrap_or_default();
                if let Some(p) = presets.get(&name) {
                    format = p.format.clone();
                    audio = p.audio.clone();
                    aspect = p.aspect.clone();
                } else {
                    anyhow::bail!("Preset '{}' no encontrado.", name);
                }
            }

            // Resolver región de render (si se pasa --start-time + --duration)
            let region = resolve_render_region(start_time, duration)?;

            // Resolver fórmula de movimiento (si se pasa --motion)
            let motion_formula = resolve_motion_formula(motion.as_deref())?;

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
            if let Some(ref r) = region {
                println!("  {} {}", style("Región:").dim(), style(r).cyan());
            }
            if let Some(ref f) = motion_formula {
                println!("  {} {}", style("Movimiento:").dim(), style(f).cyan());
            }

            let pb = ProgressBar::new(100);
            pb.set_style(
                ProgressStyle::with_template("{spinner:.cyan} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}% ({eta})")
                    .unwrap()
                    .progress_chars("#>-"),
            );

            let pb_clone = pb.clone();
            let on_progress = move |p: f64| {
                pb_clone.set_position((p * 100.0) as u64);
            };

            let job = RenderJob {
                project_path: proj_path,
                output_path: output,
                audio_only: false,
                video_format: Some(vfmt),
                audio_format: Some(afmt),
                aspect: Some(asp),
                is_live_preview: false,
                motion_formula,
                region,
            };

            let result = render::compositor::composite(&job, &project, Some(on_progress)).await?;
            pb.finish_and_clear();

            print_render_result(result);
        }

        RenderCmd::Audio { project: proj_path, output, format, force } => {
            check_output_not_overwritten(&output, force)?;
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

        RenderCmd::Video { project: proj_path, output, format, aspect, force, start_time, duration } => {
            check_output_not_overwritten(&output, force)?;
            let project = Project::load(&proj_path).await?;
            project.validate_for_render()?;
            let vfmt: VideoFormat = format.into();
            let asp:  AspectRatio = aspect.into();
            let (w, h) = asp.dimensions();

            section("Renderizando video");
            println!("  {} {}x{}", style("Resolución:").dim(), w, h);
            println!("  {} {}", style("Formato:").dim(), vfmt);
            println!("  {} {}", style("Salida:").dim(), style(output.display()).cyan());

            let region = resolve_render_region(start_time, duration)?;
            if let Some(ref r) = region {
                println!("  {} {}", style("Región:").dim(), style(r).cyan());
            }

            let pb = spinner("Renderizando video...");
            render::video::render_video(&project, &output, &vfmt, w, h).await?;
            pb.finish_and_clear();
            success(&format!("Video exportado → {}", output.display()));
        }

        RenderCmd::ExportFrame { project: proj_path, output, at, force } => {
            check_output_not_overwritten(&output, force)?;
            let project = Project::load(&proj_path).await?;
            project.validate_for_render()?;

            section("Exportando frame");
            println!("  {} {:.3}s", style("Instante:").dim(), at);
            println!("  {} {}", style("Salida:").dim(), style(output.display()).cyan());

            let pb = spinner(&format!("Exportando frame en {:.3}s...", at));
            render::video::export_frame(&project, &output, at).await?;
            pb.finish_and_clear();
            success(&format!("Frame exportado → {}", output.display()));
        }

        RenderCmd::TextPreview { project: proj_path, output, format, aspect, force } => {
            check_output_not_overwritten(&output, force)?;
            let project = Project::load(&proj_path).await?;
            project.validate_for_render()?;
            let vfmt: VideoFormat = format.into();
            let asp:  AspectRatio = aspect.into();
            let (w, h) = asp.dimensions();

            section("Renderizando preview de texto");
            println!("  {} {}x{}", style("Resolución:").dim(), w, h);
            println!("  {} {}", style("Formato:").dim(), vfmt);

            let pb = spinner("Generando preview...");
            render::text::render_text_preview(&project, &output, &vfmt, w, h).await?;
            pb.finish_and_clear();
            success(&format!("Preview exportado → {}", output.display()));
        }

        RenderCmd::LivePreview { project: proj_path, aspect } => {
            let project = Project::load(&proj_path).await?;
            project.validate_for_render()?;
            let asp: AspectRatio = aspect.into();

            section("Live Preview 🔴");
            println!("  Iniciando FFplay pipeline...");
            println!("  (Cierra la ventana de FFplay para detener)");

            let job = RenderJob {
                project_path: proj_path,
                output_path: PathBuf::from("-"),
                audio_only: false,
                video_format: None,
                audio_format: None,
                aspect: Some(asp),
                is_live_preview: true,
                motion_formula: None,
                region: None,
            };

            let _ = render::compositor::composite(&job, &project, None::<fn(f64)>).await?;
            success("Preview finalizado.");
        }

        RenderCmd::Preset { action } => match action {
            PresetAction::Save { name, format, audio, aspect } => {
                let preset = RenderPreset { format, audio, aspect };
                let mut presets = PresetRepository::load().unwrap_or_default();
                presets.insert(name.clone(), preset);
                PresetRepository::save(&presets)?;
                success(&format!("Preset '{}' guardado.", name));
            }
            PresetAction::List => {
                let presets = PresetRepository::load().unwrap_or_default();
                section("Presets de Render");
                if presets.is_empty() {
                    println!("  No hay presets guardados.");
                } else {
                    for (name, p) in presets {
                        println!("  • {}: {:?}, {:?}, {:?}", name, p.format, p.audio, p.aspect);
                    }
                }
            }
        },

        RenderCmd::Motion { action } => match action {
            MotionAction::List => {
                section("Presets de Movimiento Disponibles");
                for name in MotionPresets::available_names() {
                    let formula = MotionPresets::by_name(name).unwrap();
                    println!("  {} {}  ({})", style("•").cyan(), style(name).white().bold(), style(&formula).dim());
                }
                println!();
                println!("  {} vedit render full -p mi.vedit -o out.mp4 --motion dvd_bounce",
                    style("Uso:").dim());
            }
            MotionAction::Show { name } => {
                let formula = MotionPresets::by_name(&name)
                    .ok_or_else(|| anyhow::anyhow!("Preset de movimiento '{}' no encontrado. Usa `vedit render motion list`.", name))?;
                let exprs = formula.to_ffmpeg_exprs(1920, 1080);
                section(&format!("Preset: {}", name));
                println!("  {} {}", style("Tipo:").dim(), formula);
                println!("  {} {}", style("x_expr:").dim(), style(&exprs.x).cyan());
                println!("  {} {}", style("y_expr:").dim(), style(&exprs.y).cyan());
                println!("  {} {}", style("Filtro completo (1920×1080):").dim(),
                    style(formula.to_overlay_filter(1920, 1080)).yellow());
            }
        },
    }
    Ok(())
}

// ── Helpers privados ──────────────────────────────────────────────────────────

/// Construye una `RenderRegion` a partir de los flags opcionales del CLI.
/// Retorna `None` si no se especificaron flags de región.
fn resolve_render_region(
    start_time: Option<f64>,
    duration: Option<f64>,
) -> Result<Option<RenderRegion>> {
    match (start_time, duration) {
        (Some(start), Some(dur)) => {
            let region = RenderRegion::new(start, dur)?;
            Ok(Some(region))
        }
        (Some(_), None) => {
            anyhow::bail!("--start-time requiere también --duration para definir la región.");
        }
        (None, Some(_)) => {
            anyhow::bail!("--duration requiere también --start-time para definir la región.");
        }
        (None, None) => Ok(None),
    }
}

/// Resuelve una `MovementFormula` a partir del nombre de preset del CLI.
/// Retorna `None` si no se especificó movimiento.
fn resolve_motion_formula(name: Option<&str>) -> Result<Option<MovementFormula>> {
    match name {
        None => Ok(None),
        Some(n) => {
            let formula = MotionPresets::by_name(n)
                .ok_or_else(|| anyhow::anyhow!(
                    "Preset de movimiento '{}' no reconocido. Usa `vedit render motion list` para ver los disponibles.",
                    n
                ))?;
            Ok(Some(formula))
        }
    }
}

// ── Infraestructura de Presets ────────────────────────────────────────────────

/// Repositorio de presets de renderizado persistidos en `~/.vedit_presets.json`.
/// Encápsula toda la I/O de presets para que los handlers solo gestionen la lógica
/// de presentación (SRP).
struct PresetRepository;

impl PresetRepository {
    fn storage_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        PathBuf::from(home).join(".vedit_presets.json")
    }

    fn load() -> Result<std::collections::HashMap<String, RenderPreset>> {
        let path = Self::storage_path();
        if !path.exists() {
            return Ok(Default::default());
        }
        let content = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }

    fn save(presets: &std::collections::HashMap<String, RenderPreset>) -> Result<()> {
        let content = serde_json::to_string_pretty(presets)?;
        std::fs::write(Self::storage_path(), content)?;
        Ok(())
    }
}

// ── Utilidades de presentación del módulo ─────────────────────────────────────

/// Guard de sobreescritura: evita que el usuario pierda archivos por accidente.
fn check_output_not_overwritten(output: &PathBuf, force: bool) -> Result<()> {
    if output.exists() && !force {
        anyhow::bail!(
            "El archivo de salida ya existe: {:?}. Usa --force para sobrescribir.",
            output
        );
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
