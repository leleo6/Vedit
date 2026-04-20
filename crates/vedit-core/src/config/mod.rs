//! # `config` — Configuración global de Vedit
//!
//! Gestiona la configuración persistida en `~/.config/vedit/config.json`.
//! El struct [`VeditConfig`] es el punto de entrada único; se carga al inicio
//! del proceso y se pasa por referencia a los subsistemas que la necesitan.
//!
//! ## Jerarquía de valores
//! 1. Valor explícito en `config.json`
//! 2. Variable de entorno (`VEDIT_*`)
//! 3. Valor por defecto en código (garantizado por `Default`)

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use anyhow::Result;

// ─────────────────────────────────────────────────────────────────────────────
// Tipos auxiliares
// ─────────────────────────────────────────────────────────────────────────────

/// Codificador de video preferido.
///
/// Controla qué codec FFmpeg se elige cuando el usuario no especifica uno
/// explícitamente en el comando de render.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PreferredEncoder {
    /// CPU puro — máxima compatibilidad, sin requisitos de hardware.
    #[default]
    Libx264,
    /// GPU NVIDIA (NVENC) — muy rápido si tienes una GPU NVIDIA compatible.
    H264Nvenc,
    /// GPU Intel (VA-API) — aceleración por hardware en Intel integrado/Arc.
    H264Vaapi,
    /// GPU AMD (AMF) — aceleración por hardware en GPUs AMD.
    H264Amf,
    /// Expresión personalizada (cualquier codec que entienda tu FFmpeg).
    Custom(String),
}

impl PreferredEncoder {
    /// Devuelve el nombre de codec tal como lo entiende FFmpeg.
    pub fn as_ffmpeg_codec(&self) -> &str {
        match self {
            PreferredEncoder::Libx264        => "libx264",
            PreferredEncoder::H264Nvenc      => "h264_nvenc",
            PreferredEncoder::H264Vaapi      => "h264_vaapi",
            PreferredEncoder::H264Amf        => "h264_amf",
            PreferredEncoder::Custom(s)      => s.as_str(),
        }
    }

    /// Indica si el encoder requiere un filtro de carga de hardware adicional
    /// (necesario para VA-API).
    pub fn requires_hwaccel_filter(&self) -> bool {
        matches!(self, PreferredEncoder::H264Vaapi)
    }
}

impl std::fmt::Display for PreferredEncoder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ffmpeg_codec())
    }
}

/// Relación de aspecto predeterminada para nuevos proyectos.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DefaultAspectRatio {
    /// 16:9 panorámica (1920×1080)
    #[default]
    Widescreen,
    /// 9:16 vertical — reels, shorts, TikTok
    Portrait,
    /// 1:1 cuadrado — Instagram
    Square,
    /// Resolución completamente personalizada
    Custom { width: u32, height: u32 },
}

impl DefaultAspectRatio {
    /// Devuelve `(width, height)` en píxeles.
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            DefaultAspectRatio::Widescreen              => (1920, 1080),
            DefaultAspectRatio::Portrait                => (1080, 1920),
            DefaultAspectRatio::Square                  => (1080, 1080),
            DefaultAspectRatio::Custom { width, height } => (*width, *height),
        }
    }
}

impl std::fmt::Display for DefaultAspectRatio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (w, h) = self.dimensions();
        write!(f, "{}x{}", w, h)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// VeditConfig — struct principal
// ─────────────────────────────────────────────────────────────────────────────

/// Configuración global de Vedit.
///
/// Se persiste en `~/.config/vedit/config.json`. Si el archivo no existe,
/// [`VeditConfig::load`] crea uno con los valores por defecto.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VeditConfig {
    // ── Entorno y Rutas ───────────────────────────────────────────────────

    /// Ruta al binario `ffmpeg`. Si es `None`, se busca en el PATH del sistema.
    ///
    /// Útil para usuarios con múltiples versiones de FFmpeg instaladas o builds
    /// personalizados (ej. con soporte de codecs adicionales).
    ///
    /// Variable de entorno equivalente: `VEDIT_FFMPEG_PATH`
    pub ffmpeg_path: Option<PathBuf>,

    /// Ruta al binario `ffprobe`. Si es `None`, se busca en el PATH del sistema.
    ///
    /// Variable de entorno equivalente: `VEDIT_FFPROBE_PATH`
    pub ffprobe_path: Option<PathBuf>,

    /// Directorio global para archivos temporales, proxies y caché de análisis.
    ///
    /// Si es `None`, cada proyecto usa `<project_dir>/.vedit_cache/`.
    /// Con un valor global, todos los proyectos comparten el mismo directorio.
    ///
    /// Variable de entorno equivalente: `VEDIT_CACHE_DIR`
    pub cache_dir: Option<PathBuf>,

    // ── Aceleración por Hardware ──────────────────────────────────────────

    /// Codec de video preferido para renderizado. Controla si usar CPU o GPU.
    ///
    /// Variable de entorno equivalente: `VEDIT_ENCODER`
    pub preferred_encoder: PreferredEncoder,

    /// Dispositivo VA-API a utilizar cuando se activa la aceleración por hardware VA-API.
    ///
    /// Variable de entorno equivalente: `VEDIT_VAAPI_DEVICE`
    pub vaapi_device: String,

    // ── Valores de Proyecto Predeterminados ───────────────────────────────

    /// FPS por defecto para nuevos proyectos.
    ///
    /// Variable de entorno equivalente: `VEDIT_DEFAULT_FPS`
    pub default_fps: u32,

    /// Resolución predeterminada para nuevos proyectos.
    pub default_resolution: DefaultAspectRatio,

    // ── Gestión de Recursos ───────────────────────────────────────────────

    /// Número máximo de hilos que FFmpeg puede usar.
    ///
    /// `0` significa "decidir automáticamente" (comportamiento por defecto de FFmpeg).
    ///
    /// Variable de entorno equivalente: `VEDIT_MAX_THREADS`
    pub max_threads: u32,

    /// Si es `true`, elimina el directorio de caché al terminar un render exitoso.
    ///
    /// Variable de entorno equivalente: `VEDIT_CLEANUP_CACHE` (1/0 o true/false)
    pub cleanup_cache_on_exit: bool,

    /// Nivel de log por defecto cuando no se pasa `--log` en el CLI.
    ///
    /// Valores válidos: `trace`, `debug`, `info`, `warn`, `error`
    pub log_level: String,
}

impl Default for VeditConfig {
    fn default() -> Self {
        Self {
            ffmpeg_path:          None,
            ffprobe_path:         None,
            cache_dir:            None,
            preferred_encoder:    PreferredEncoder::Libx264,
            vaapi_device:         "/dev/dri/renderD128".to_string(),
            default_fps:          30,
            default_resolution:   DefaultAspectRatio::Widescreen,
            max_threads:          0,
            cleanup_cache_on_exit: false,
            log_level:            "info".to_string(),
        }
    }
}

impl VeditConfig {
    // ── Persistencia ──────────────────────────────────────────────────────

    /// Ruta canónica al archivo de configuración (`~/.config/vedit/config.json`).
    pub fn config_path() -> PathBuf {
        dirs_home()
            .join(".config")
            .join("vedit")
            .join("config.json")
    }

    /// Carga la configuración desde disco, aplicando luego las variables de entorno.
    ///
    /// Si el archivo no existe, genera uno con los valores por defecto y lo persiste.
    /// Nunca devuelve error por archivo ausente; solo falla si el JSON es inválido.
    pub fn load() -> Result<Self> {
        let path = Self::config_path();

        let mut cfg = if path.exists() {
            let raw = std::fs::read_to_string(&path)?;
            serde_json::from_str::<VeditConfig>(&raw)
                .map_err(|e| anyhow::anyhow!("config.json inválido: {}. Borra {:?} para regenerarlo.", e, path))?
        } else {
            let cfg = VeditConfig::default();
            cfg.save()?; // Crea el archivo con los valores por defecto
            tracing::info!("Configuración inicial creada en {:?}", path);
            cfg
        };

        // Sobrescribir con variables de entorno (tienen prioridad sobre el archivo)
        cfg.apply_env_overrides();
        Ok(cfg)
    }

    /// Persiste la configuración actual en disco.
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        tracing::debug!("Configuración guardada en {:?}", path);
        Ok(())
    }

    // ── Resolución de rutas ───────────────────────────────────────────────

    /// Ruta efectiva al binario `ffmpeg` (config > PATH).
    pub fn ffmpeg_binary(&self) -> PathBuf {
        self.ffmpeg_path.clone().unwrap_or_else(|| PathBuf::from("ffmpeg"))
    }

    /// Ruta efectiva al binario `ffprobe` (config > PATH).
    pub fn ffprobe_binary(&self) -> PathBuf {
        self.ffprobe_path.clone().unwrap_or_else(|| PathBuf::from("ffprobe"))
    }

    /// Directorio de caché efectivo para un proyecto dado.
    ///
    /// Si `cache_dir` está configurado globalmente, devuelve ese directorio.
    /// Si no, usa `<project_dir>/.vedit_cache/`.
    pub fn resolve_cache_dir(&self, project_dir: &Path) -> PathBuf {
        self.cache_dir.clone().unwrap_or_else(|| project_dir.join(".vedit_cache"))
    }

    /// Devuelve los args de `-threads N` para FFmpeg, o vacío si es auto.
    pub fn ffmpeg_thread_args(&self) -> Vec<String> {
        if self.max_threads > 0 {
            vec!["-threads".to_string(), self.max_threads.to_string()]
        } else {
            vec![]
        }
    }

    /// Devuelve `(width, height)` de la resolución predeterminada configurada.
    pub fn default_dimensions(&self) -> (u32, u32) {
        self.default_resolution.dimensions()
    }

    // ── Variables de entorno ──────────────────────────────────────────────

    /// Aplica las variables de entorno `VEDIT_*` sobre la configuración actual.
    /// Las variables de entorno tienen prioridad sobre el archivo de configuración.
    fn apply_env_overrides(&mut self) {
        if let Ok(p) = std::env::var("VEDIT_FFMPEG_PATH") {
            self.ffmpeg_path = Some(PathBuf::from(p));
        }
        if let Ok(p) = std::env::var("VEDIT_FFPROBE_PATH") {
            self.ffprobe_path = Some(PathBuf::from(p));
        }
        if let Ok(p) = std::env::var("VEDIT_CACHE_DIR") {
            self.cache_dir = Some(PathBuf::from(p));
        }
        if let Ok(enc) = std::env::var("VEDIT_ENCODER") {
            self.preferred_encoder = match enc.to_lowercase().as_str() {
                "libx264"   => PreferredEncoder::Libx264,
                "h264_nvenc"=> PreferredEncoder::H264Nvenc,
                "h264_vaapi"=> PreferredEncoder::H264Vaapi,
                "h264_amf"  => PreferredEncoder::H264Amf,
                other        => PreferredEncoder::Custom(other.to_string()),
            };
        }
        if let Ok(dev) = std::env::var("VEDIT_VAAPI_DEVICE") {
            self.vaapi_device = dev;
        }
        if let Ok(t) = std::env::var("VEDIT_MAX_THREADS") {
            if let Ok(n) = t.parse::<u32>() {
                self.max_threads = n;
            }
        }
        if let Ok(v) = std::env::var("VEDIT_CLEANUP_CACHE") {
            self.cleanup_cache_on_exit = matches!(v.to_lowercase().as_str(), "1" | "true" | "yes");
        }
        if let Ok(lvl) = std::env::var("VEDIT_LOG") {
            self.log_level = lvl;
        }
    }

    // ── Validación ────────────────────────────────────────────────────────

    /// Valida que los binarios de FFmpeg son alcanzables y ejecutables.
    /// Devuelve `Ok(())` si todo está bien, o un error descriptivo.
    pub fn validate_ffmpeg(&self) -> Result<()> {
        let ffmpeg = self.ffmpeg_binary();
        let status = std::process::Command::new(&ffmpeg)
            .arg("-version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();

        match status {
            Ok(s) if s.success() || s.code() == Some(1) => Ok(()), // ffmpeg -version devuelve 0 o 1
            Ok(_) | Err(_) => anyhow::bail!(
                "No se pudo ejecutar FFmpeg en '{}'.\n\
                 Instálalo con: sudo apt install ffmpeg\n\
                 O configura la ruta con: vedit config set ffmpeg-path /ruta/a/ffmpeg",
                ffmpeg.display()
            ),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Utilidades internas
// ─────────────────────────────────────────────────────────────────────────────

/// Obtiene el directorio home del usuario actual.
fn dirs_home() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let cfg = VeditConfig::default();
        assert_eq!(cfg.default_fps, 30);
        assert_eq!(cfg.max_threads, 0);
        assert!(!cfg.cleanup_cache_on_exit);
        assert_eq!(cfg.preferred_encoder, PreferredEncoder::Libx264);
    }

    #[test]
    fn preferred_encoder_codec_names() {
        assert_eq!(PreferredEncoder::Libx264.as_ffmpeg_codec(),   "libx264");
        assert_eq!(PreferredEncoder::H264Nvenc.as_ffmpeg_codec(), "h264_nvenc");
        assert_eq!(PreferredEncoder::H264Vaapi.as_ffmpeg_codec(), "h264_vaapi");
        assert_eq!(PreferredEncoder::H264Amf.as_ffmpeg_codec(),   "h264_amf");
        assert_eq!(PreferredEncoder::Custom("libx265".into()).as_ffmpeg_codec(), "libx265");
    }

    #[test]
    fn vaapi_requires_hwaccel_filter() {
        assert!( PreferredEncoder::H264Vaapi.requires_hwaccel_filter());
        assert!(!PreferredEncoder::Libx264.requires_hwaccel_filter());
        assert!(!PreferredEncoder::H264Nvenc.requires_hwaccel_filter());
    }

    #[test]
    fn default_resolution_dimensions() {
        assert_eq!(DefaultAspectRatio::Widescreen.dimensions(), (1920, 1080));
        assert_eq!(DefaultAspectRatio::Portrait.dimensions(),   (1080, 1920));
        assert_eq!(DefaultAspectRatio::Square.dimensions(),     (1080, 1080));
        assert_eq!(DefaultAspectRatio::Custom { width: 3840, height: 2160 }.dimensions(), (3840, 2160));
    }

    #[test]
    fn thread_args_zero_means_empty() {
        let mut cfg = VeditConfig::default();
        assert!(cfg.ffmpeg_thread_args().is_empty(), "0 threads → args vacíos");
        cfg.max_threads = 8;
        let args = cfg.ffmpeg_thread_args();
        assert_eq!(args, vec!["-threads", "8"]);
    }

    #[test]
    fn cache_dir_falls_back_to_project_dir() {
        let cfg = VeditConfig::default();
        let proj = Path::new("/tmp/mi_proyecto");
        let cache = cfg.resolve_cache_dir(proj);
        assert_eq!(cache, PathBuf::from("/tmp/mi_proyecto/.vedit_cache"));
    }

    #[test]
    fn cache_dir_uses_global_when_set() {
        let mut cfg = VeditConfig::default();
        cfg.cache_dir = Some(PathBuf::from("/tmp/vedit_global_cache"));
        let cache = cfg.resolve_cache_dir(Path::new("/cualquier/proyecto"));
        assert_eq!(cache, PathBuf::from("/tmp/vedit_global_cache"));
    }

    #[test]
    fn config_roundtrip_json() {
        let mut cfg = VeditConfig::default();
        cfg.max_threads = 16;
        cfg.preferred_encoder = PreferredEncoder::H264Nvenc;
        cfg.cleanup_cache_on_exit = true;
        cfg.vaapi_device = "/dev/dri/renderD129".to_string();

        let json  = serde_json::to_string(&cfg).unwrap();
        let back: VeditConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(back.max_threads, 16);
        assert_eq!(back.preferred_encoder, PreferredEncoder::H264Nvenc);
        assert!(back.cleanup_cache_on_exit);
        assert_eq!(back.vaapi_device, "/dev/dri/renderD129");
    }

    #[test]
    fn config_path_is_inside_home() {
        let path = VeditConfig::config_path();
        assert!(path.to_string_lossy().contains("vedit"));
        assert!(path.to_string_lossy().ends_with("config.json"));
    }

    #[test]
    fn env_override_encoder(){ 
        // Guardamos para no contaminar otros tests
        let prev = std::env::var("VEDIT_ENCODER").ok();
        std::env::set_var("VEDIT_ENCODER", "h264_nvenc");

        let mut cfg = VeditConfig::default();
        cfg.apply_env_overrides();
        assert_eq!(cfg.preferred_encoder, PreferredEncoder::H264Nvenc);

        match prev {
            Some(v) => std::env::set_var("VEDIT_ENCODER", v),
            None    => std::env::remove_var("VEDIT_ENCODER"),
        }
    }
}
