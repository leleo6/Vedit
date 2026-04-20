use std::path::PathBuf;
use anyhow::Result;
use clap::Subcommand;
use console::style;
use vedit_core::config::{VeditConfig, PreferredEncoder, DefaultAspectRatio};
use super::{success, warn, section};

// ─────────────────────────────────────────────────────────────────────────────
// Subcomandos
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum ConfigCmd {
    /// Muestra la configuración actual (archivo + variables de entorno)
    Show,

    /// Establece un valor de configuración
    Set {
        #[command(subcommand)]
        key: ConfigKey,
    },

    /// Resetea la configuración a los valores por defecto
    Reset {
        /// Confirmar el reset sin pedir confirmación interactiva
        #[arg(long, default_value_t = false)]
        yes: bool,
    },

    /// Muestra la ruta al archivo de configuración
    Path,

    /// Verifica que FFmpeg y FFprobe son alcanzables con la configuración actual
    Check,
}

/// Claves de configuración disponibles para el subcomando `set`.
#[derive(Subcommand, Debug)]
pub enum ConfigKey {
    /// Ruta al binario de FFmpeg (usa 'auto' para buscar en PATH)
    FfmpegPath {
        /// Ruta absoluta o 'auto'
        value: String,
    },
    /// Ruta al binario de FFprobe (usa 'auto' para buscar en PATH)
    FfprobePath {
        /// Ruta absoluta o 'auto'
        value: String,
    },
    /// Directorio global de caché (usa 'auto' para usar directorio del proyecto)
    CacheDir {
        /// Ruta absoluta o 'auto'
        value: String,
    },
    /// Codec de video preferido: libx264, h264_nvenc, h264_vaapi, h264_amf, o nombre personalizado
    PreferredEncoder {
        /// Nombre del encoder
        value: String,
    },
    /// FPS por defecto para nuevos proyectos (ej. 24, 30, 60)
    DefaultFps {
        /// Frames por segundo
        value: u32,
    },
    /// Resolución por defecto: widescreen (16:9), portrait (9:16), square (1:1), o WxH
    DefaultResolution {
        /// Preset o resolución como '1920x1080'
        value: String,
    },
    /// Número máximo de hilos de CPU para FFmpeg (0 = automático)
    MaxThreads {
        /// Número de hilos
        value: u32,
    },
    /// Limpiar caché automáticamente al terminar un render (true/false)
    CleanupCacheOnExit {
        /// true o false
        value: bool,
    },
    /// Nivel de log por defecto (trace, debug, info, warn, error)
    LogLevel {
        /// Nivel de log
        value: String,
    },
}

// ─────────────────────────────────────────────────────────────────────────────
// Handler principal
// ─────────────────────────────────────────────────────────────────────────────

pub async fn run(cmd: ConfigCmd) -> Result<()> {
    match cmd {
        ConfigCmd::Show => {
            let cfg = VeditConfig::load()?;
            print_config(&cfg);
        }

        ConfigCmd::Set { key } => {
            let mut cfg = VeditConfig::load()?;
            apply_key(&mut cfg, key)?;
            cfg.save()?;
            success("Configuración actualizada.");
        }

        ConfigCmd::Reset { yes } => {
            if !yes {
                warn("Este comando resetea toda la configuración a los valores por defecto.");
                warn("Usa --yes para confirmar: vedit config reset --yes");
                return Ok(());
            }
            let cfg = VeditConfig::default();
            cfg.save()?;
            success("Configuración reseteada a valores por defecto.");
            print_config(&cfg);
        }

        ConfigCmd::Path => {
            let path = VeditConfig::config_path();
            println!("{}", style(path.display()).cyan());
        }

        ConfigCmd::Check => {
            let cfg = VeditConfig::load()?;
            section("Verificando entorno de Vedit");

            // FFmpeg
            print!("  {} ffmpeg ({})... ", style("▶").cyan(), style(cfg.ffmpeg_binary().display()).dim());
            match cfg.validate_ffmpeg() {
                Ok(()) => println!("{}", style("✔ OK").green()),
                Err(e) => println!("{}\n    {}", style("✖ ERROR").red(), style(e).red()),
            }

            // FFprobe
            print!("  {} ffprobe ({})... ", style("▶").cyan(), style(cfg.ffprobe_binary().display()).dim());
            let ffprobe_ok = std::process::Command::new(cfg.ffprobe_binary())
                .arg("-version")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .is_ok();
            if ffprobe_ok {
                println!("{}", style("✔ OK").green());
            } else {
                println!(
                    "{}\n    {}",
                    style("✖ ERROR").red(),
                    style("ffprobe no encontrado. Instala ffmpeg con ffprobe incluido.").red()
                );
            }

            // Encoder
            println!(
                "  {} Encoder preferido: {}",
                style("▶").cyan(),
                style(cfg.preferred_encoder.as_ffmpeg_codec()).yellow()
            );
            if cfg.preferred_encoder.requires_hwaccel_filter() {
                warn("VA-API requiere que el sistema tenga drivers de Intel instalados (libva).");
            }

            // Caché
            let sample_cache = cfg.resolve_cache_dir(std::path::Path::new("/tmp/sample"));
            println!(
                "  {} Directorio de caché: {}",
                style("▶").cyan(),
                style(sample_cache.display()).dim()
            );

            // Threads
            let thread_info = if cfg.max_threads == 0 {
                "automático".to_string()
            } else {
                format!("{} hilos", cfg.max_threads)
            };
            println!(
                "  {} CPU threads: {}",
                style("▶").cyan(),
                style(thread_info).dim()
            );

            println!();
            success("Verificación completada.");
        }
    }

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers privados
// ─────────────────────────────────────────────────────────────────────────────

/// Aplica un `ConfigKey` sobre la configuración mutable.
fn apply_key(cfg: &mut VeditConfig, key: ConfigKey) -> Result<()> {
    match key {
        ConfigKey::FfmpegPath { value } => {
            cfg.ffmpeg_path = if value == "auto" { None } else { Some(PathBuf::from(value)) };
            println!(
                "  {} ffmpeg_path = {}",
                style("→").cyan(),
                style(cfg.ffmpeg_binary().display()).yellow()
            );
        }

        ConfigKey::FfprobePath { value } => {
            cfg.ffprobe_path = if value == "auto" { None } else { Some(PathBuf::from(value)) };
            println!(
                "  {} ffprobe_path = {}",
                style("→").cyan(),
                style(cfg.ffprobe_binary().display()).yellow()
            );
        }

        ConfigKey::CacheDir { value } => {
            cfg.cache_dir = if value == "auto" { None } else { Some(PathBuf::from(value)) };
            println!(
                "  {} cache_dir = {}",
                style("→").cyan(),
                match &cfg.cache_dir {
                    Some(p) => style(p.display().to_string()).yellow(),
                    None    => style("<por proyecto>".into()).dim(),
                }
            );
        }

        ConfigKey::PreferredEncoder { value } => {
            cfg.preferred_encoder = parse_encoder(&value);
            println!(
                "  {} preferred_encoder = {}",
                style("→").cyan(),
                style(cfg.preferred_encoder.as_ffmpeg_codec()).yellow()
            );
            if cfg.preferred_encoder.requires_hwaccel_filter() {
                warn("VA-API requiere drivers Intel. Ejecuta `vedit config check` para verificar.");
            }
        }

        ConfigKey::DefaultFps { value } => {
            if value == 0 || value > 240 {
                anyhow::bail!("FPS debe estar entre 1 y 240. Recibido: {}", value);
            }
            cfg.default_fps = value;
            println!("  {} default_fps = {}", style("→").cyan(), style(value).yellow());
        }

        ConfigKey::DefaultResolution { value } => {
            cfg.default_resolution = parse_resolution(&value)?;
            let (w, h) = cfg.default_resolution.dimensions();
            println!("  {} default_resolution = {}x{}", style("→").cyan(), style(w).yellow(), style(h).yellow());
        }

        ConfigKey::MaxThreads { value } => {
            cfg.max_threads = value;
            let info = if value == 0 { "automático".to_string() } else { format!("{} hilos", value) };
            println!("  {} max_threads = {}", style("→").cyan(), style(info).yellow());
        }

        ConfigKey::CleanupCacheOnExit { value } => {
            cfg.cleanup_cache_on_exit = value;
            println!("  {} cleanup_cache_on_exit = {}", style("→").cyan(), style(value).yellow());
        }

        ConfigKey::LogLevel { value } => {
            let valid = ["trace", "debug", "info", "warn", "error"];
            if !valid.contains(&value.as_str()) {
                anyhow::bail!("Nivel de log inválido: '{}'. Opciones válidas: {:?}", value, valid);
            }
            cfg.log_level = value.clone();
            println!("  {} log_level = {}", style("→").cyan(), style(value).yellow());
        }
    }
    Ok(())
}

/// Formatea e imprime la configuración completa.
fn print_config(cfg: &VeditConfig) {
    section("Configuración de Vedit");
    println!("  {} {:?}", style(fmt_key("Archivo")).dim(), VeditConfig::config_path());
    println!();

    section_row("Entorno y Rutas");
    row("ffmpeg_path",  &match &cfg.ffmpeg_path {
        Some(p) => p.display().to_string(),
        None    => format!("auto ({})", cfg.ffmpeg_binary().display()),
    });
    row("ffprobe_path", &match &cfg.ffprobe_path {
        Some(p) => p.display().to_string(),
        None    => format!("auto ({})", cfg.ffprobe_binary().display()),
    });
    row("cache_dir", &match &cfg.cache_dir {
        Some(p) => p.display().to_string(),
        None    => "<por proyecto>/.vedit_cache".to_string(),
    });

    section_row("Aceleración por Hardware");
    row("preferred_encoder", cfg.preferred_encoder.as_ffmpeg_codec());
    if cfg.preferred_encoder.requires_hwaccel_filter() {
        println!("    {} Requiere drivers Intel VA-API", style("⚠").yellow());
    }

    section_row("Valores de Proyecto Predeterminados");
    let (w, h) = cfg.default_resolution.dimensions();
    row("default_fps",        &cfg.default_fps.to_string());
    row("default_resolution", &format!("{}x{}", w, h));

    section_row("Gestión de Recursos");
    row("max_threads", &if cfg.max_threads == 0 {
        "0 (automático)".to_string()
    } else {
        format!("{} hilos", cfg.max_threads)
    });
    row("cleanup_cache_on_exit", &cfg.cleanup_cache_on_exit.to_string());
    row("log_level",             &cfg.log_level);

    println!();
    println!("  {} vedit config set <KEY> <VALUE>", style("Editar:").dim());
    println!("  {} vedit config check", style("Verificar:").dim());
}

// ── Utilidades de presentación ────────────────────────────────────────────────

fn section_row(title: &str) {
    println!("\n  {}", style(title).cyan().bold());
}

fn row(key: &str, value: &str) {
    println!("    {} {}", style(fmt_key(key)).dim(), style(value).white());
}

fn fmt_key(key: &str) -> String {
    format!("{:<28}", format!("{}:", key))
}

// ── Parsers ───────────────────────────────────────────────────────────────────

fn parse_encoder(value: &str) -> PreferredEncoder {
    match value.to_lowercase().as_str() {
        "libx264"    => PreferredEncoder::Libx264,
        "h264_nvenc" => PreferredEncoder::H264Nvenc,
        "h264_vaapi" => PreferredEncoder::H264Vaapi,
        "h264_amf"   => PreferredEncoder::H264Amf,
        other        => PreferredEncoder::Custom(other.to_string()),
    }
}

fn parse_resolution(value: &str) -> Result<DefaultAspectRatio> {
    match value.to_lowercase().as_str() {
        "widescreen" | "16:9" => Ok(DefaultAspectRatio::Widescreen),
        "portrait"   | "9:16" => Ok(DefaultAspectRatio::Portrait),
        "square"     | "1:1"  => Ok(DefaultAspectRatio::Square),
        custom => {
            // Intentar parsear "WxH" (ej. "3840x2160" o "2560x1440")
            let parts: Vec<&str> = custom.split('x').collect();
            if parts.len() == 2 {
                let w = parts[0].trim().parse::<u32>()
                    .map_err(|_| anyhow::anyhow!("Ancho inválido: '{}'", parts[0]))?;
                let h = parts[1].trim().parse::<u32>()
                    .map_err(|_| anyhow::anyhow!("Alto inválido: '{}'", parts[1]))?;
                if w == 0 || h == 0 {
                    anyhow::bail!("La resolución no puede tener dimensiones en cero.");
                }
                Ok(DefaultAspectRatio::Custom { width: w, height: h })
            } else {
                anyhow::bail!(
                    "Resolución '{}' no reconocida. Usa: widescreen, portrait, square, o WxH (ej. 3840x2160)",
                    value
                )
            }
        }
    }
}
