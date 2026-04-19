use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─────────────────────────────────────────────────────────────────────────────
// Fade effect
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FadeEffect {
    /// Duración del fade en segundos
    pub duration_secs: f64,
}

// ─────────────────────────────────────────────────────────────────────────────
// AudioClip
// ─────────────────────────────────────────────────────────────────────────────

/// Clip de audio dentro de un track
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioClip {
    pub id: Uuid,
    /// Nombre descriptivo del clip
    pub name: String,
    /// Archivo fuente en disco
    pub source_path: PathBuf,
    /// Inicio en el archivo fuente (segundos)
    pub source_start: f64,
    /// Fin en el archivo fuente (segundos, None = hasta el final)
    pub source_end: Option<f64>,
    /// Posición de inicio en el timeline del proyecto (segundos)
    pub timeline_start: f64,
    /// Volumen individual 0.0–2.0 (1.0 = 100%)
    pub volume: f64,
    /// Repetir N veces (1 = sin loop)
    pub loop_count: u32,
    /// Speed/pitch factor (1.0 = normal)
    pub speed: f64,
    pub fade_in: Option<FadeEffect>,
    pub fade_out: Option<FadeEffect>,
    /// Rangos de silencio en segundos relativos al clip [(start, end)]
    pub mute_ranges: Vec<(f64, f64)>,
}

impl AudioClip {
    pub fn new(name: impl Into<String>, source_path: impl Into<PathBuf>, timeline_start: f64) -> Self {
        let path = source_path.into();
        let path = std::fs::canonicalize(&path).unwrap_or(path);
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            source_path: path,
            source_start: 0.0,
            source_end: None,
            timeline_start,
            volume: 1.0,
            loop_count: 1,
            speed: 1.0,
            fade_in: None,
            fade_out: None,
            mute_ranges: Vec::new(),
        }
    }

    /// Duración neta del clip en el timeline (sin loop)
    pub fn raw_duration(&self) -> f64 {
        match self.source_end {
            Some(end) => (end - self.source_start).max(0.0),
            None => 0.0, // desconocida hasta probar con ffprobe
        }
    }

    /// Duración total considerando loops
    pub fn duration(&self) -> f64 {
        self.raw_duration() * self.loop_count as f64
    }

    pub fn set_fade_in(&mut self, secs: f64) {
        self.fade_in = Some(FadeEffect { duration_secs: secs });
    }

    pub fn set_fade_out(&mut self, secs: f64) {
        self.fade_out = Some(FadeEffect { duration_secs: secs });
    }

    pub fn add_mute_range(&mut self, start: f64, end: f64) {
        self.mute_ranges.push((start, end));
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// VideoClip
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoClip {
    pub id: Uuid,
    pub name: String,
    pub source_path: PathBuf,
    pub source_start: f64,
    pub source_end: Option<f64>,
    pub timeline_start: f64,
    pub volume: f64,
    pub speed: f64,
    pub fade_in: Option<FadeEffect>,
    pub fade_out: Option<FadeEffect>,
    /// Audio de reemplazo (opcional)
    pub replacement_audio: Option<PathBuf>,
}

impl VideoClip {
    pub fn new(name: impl Into<String>, source_path: impl Into<PathBuf>, timeline_start: f64) -> Self {
        let path = source_path.into();
        let path = std::fs::canonicalize(&path).unwrap_or(path);
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            source_path: path,
            source_start: 0.0,
            source_end: None,
            timeline_start,
            volume: 1.0,
            speed: 1.0,
            fade_in: None,
            fade_out: None,
            replacement_audio: None,
        }
    }

    pub fn duration(&self) -> f64 {
        self.source_end
            .map(|e| (e - self.source_start).max(0.0))
            .unwrap_or(0.0)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ImageClip — tipos auxiliares
// ─────────────────────────────────────────────────────────────────────────────

/// Posición en el frame (coordenadas relativas al frame, 0.0 = borde izq/top, 1.0 = borde der/bot)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImagePosition {
    /// Coordenada X (fracción del ancho del frame, 0.0–1.0); None = centrado
    pub x: Option<f64>,
    /// Coordenada Y (fracción del alto del frame, 0.0–1.0); None = centrado
    pub y: Option<f64>,
}

impl Default for ImagePosition {
    fn default() -> Self {
        Self { x: None, y: None }
    }
}

/// Escala del clip
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageScale {
    /// Fracción del ancho del frame (1.0 = 100%)
    pub width: f64,
    /// Fracción del alto del frame (1.0 = 100%)
    pub height: f64,
}

impl Default for ImageScale {
    fn default() -> Self {
        Self { width: 1.0, height: 1.0 }
    }
}

/// Recorte de la imagen fuente antes de colocarla (en píxeles)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImageCrop {
    pub top: u32,
    pub bottom: u32,
    pub left: u32,
    pub right: u32,
}

/// Modo de uso de la imagen en el frame
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ImageMode {
    /// Overlay sobre el video: respeta posición, escala y opacidad
    #[default]
    Overlay,
    /// Fondo — reemplaza el video en ese rango de tiempo (ocupa todo el frame)
    Background,
    /// Pantalla completa con duración fija — para slideshows
    Fullscreen,
}

impl std::fmt::Display for ImageMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageMode::Overlay    => write!(f, "overlay"),
            ImageMode::Background => write!(f, "background"),
            ImageMode::Fullscreen => write!(f, "fullscreen"),
        }
    }
}

/// Efecto Ken Burns: zoom suave + desplazamiento sobre la imagen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KenBurnsEffect {
    /// Escala inicial (1.0 = sin zoom)
    pub zoom_start: f64,
    /// Escala final (ej. 1.2 = zoom del 20%)
    pub zoom_end: f64,
    /// Posición X inicial (fracción)
    pub pan_x_start: f64,
    /// Posición X final (fracción)
    pub pan_x_end: f64,
    /// Posición Y inicial (fracción)
    pub pan_y_start: f64,
    /// Posición Y final (fracción)
    pub pan_y_end: f64,
}

impl Default for KenBurnsEffect {
    fn default() -> Self {
        Self {
            zoom_start: 1.0,
            zoom_end: 1.2,
            pan_x_start: 0.0,
            pan_x_end: 0.0,
            pan_y_start: 0.0,
            pan_y_end: 0.0,
        }
    }
}

/// Animación de entrada del clip
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryAnimation {
    /// Aparece desde el centro (escala)
    ZoomIn,
    /// Entra desde la izquierda
    SlideLeft,
    /// Entra desde la derecha
    SlideRight,
    /// Entra desde arriba
    SlideTop,
    /// Entra desde abajo
    SlideBottom,
}

impl std::fmt::Display for EntryAnimation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntryAnimation::ZoomIn      => write!(f, "zoom_in"),
            EntryAnimation::SlideLeft   => write!(f, "slide_left"),
            EntryAnimation::SlideRight  => write!(f, "slide_right"),
            EntryAnimation::SlideTop    => write!(f, "slide_top"),
            EntryAnimation::SlideBottom => write!(f, "slide_bottom"),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ImageClip
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageClip {
    pub id: Uuid,
    pub name: String,
    pub source_path: PathBuf,
    /// Inicio en el timeline (segundos)
    pub timeline_start: f64,
    /// Duración de la imagen en el timeline (segundos)
    pub duration_secs: f64,

    // ── Transformación ────────────────────────────────────────────────────
    /// Posición en el frame (fracción 0.0–1.0); None = centrado
    pub position: ImagePosition,
    /// Escala relativa al frame (1.0 = 100%)
    pub scale: ImageScale,
    /// Rotación en grados (sentido horario)
    pub rotation_deg: f64,
    /// Opacidad 0.0 (transparente) – 1.0 (opaco)
    pub opacity: f64,
    /// Recorte de la imagen fuente antes de aplicar transformaciones
    pub crop: Option<ImageCrop>,

    // ── Modo de uso ───────────────────────────────────────────────────────
    pub mode: ImageMode,

    // ── Efectos ───────────────────────────────────────────────────────────
    /// Fade de opacidad al entrar
    pub fade_in: Option<FadeEffect>,
    /// Fade de opacidad al salir
    pub fade_out: Option<FadeEffect>,
    /// Efecto Ken Burns (zoom + pan)
    pub ken_burns: Option<KenBurnsEffect>,
    /// Animación de entrada
    pub entry_animation: Option<EntryAnimation>,
}

impl ImageClip {
    pub fn new(
        name: impl Into<String>,
        source_path: impl Into<PathBuf>,
        timeline_start: f64,
        duration_secs: f64,
    ) -> Self {
        let path = source_path.into();
        let path = std::fs::canonicalize(&path).unwrap_or(path);
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            source_path: path,
            timeline_start,
            duration_secs,
            position: ImagePosition::default(),
            scale: ImageScale::default(),
            rotation_deg: 0.0,
            opacity: 1.0,
            crop: None,
            mode: ImageMode::default(),
            fade_in: None,
            fade_out: None,
            ken_burns: None,
            entry_animation: None,
        }
    }

    /// Duración en el timeline
    pub fn duration(&self) -> f64 {
        self.duration_secs
    }

    /// Aplica fade-in de opacidad
    pub fn set_fade_in(&mut self, secs: f64) {
        self.fade_in = Some(FadeEffect { duration_secs: secs });
    }

    /// Aplica fade-out de opacidad
    pub fn set_fade_out(&mut self, secs: f64) {
        self.fade_out = Some(FadeEffect { duration_secs: secs });
    }

    /// Ajusta posición en el frame (valores fraccionarios 0.0–1.0)
    pub fn set_position(&mut self, x: Option<f64>, y: Option<f64>) {
        self.position = ImagePosition { x, y };
    }

    /// Ajusta escala (1.0 = 100% del frame)
    pub fn set_scale(&mut self, width: f64, height: f64) {
        self.scale = ImageScale { width: width.max(0.0), height: height.max(0.0) };
    }

    /// Ajusta opacidad (clamp 0.0–1.0)
    pub fn set_opacity(&mut self, opacity: f64) {
        self.opacity = opacity.clamp(0.0, 1.0);
    }

    /// Activa efecto Ken Burns con valores por defecto
    pub fn apply_ken_burns(&mut self) {
        self.ken_burns = Some(KenBurnsEffect::default());
    }

    /// Activa efecto Ken Burns con valores personalizados
    pub fn apply_ken_burns_custom(&mut self, effect: KenBurnsEffect) {
        self.ken_burns = Some(effect);
    }

    /// Divide el clip en `at_secs` segundos relativos al inicio del clip.
    /// Devuelve el segundo clip resultante (this queda con la primera mitad).
    ///
    /// # Ejemplo
    ///
    /// ```
    /// use vedit_core::project::clip::ImageClip;
    /// let mut clip = ImageClip::new("foto", "img.png", 0.0, 10.0);
    /// let split = clip.split_at(4.0).unwrap();
    /// 
    /// assert_eq!(clip.duration_secs, 4.0);
    /// assert_eq!(split.duration_secs, 6.0);
    /// assert_eq!(split.timeline_start, 4.0);
    /// ```
    pub fn split_at(&mut self, at_secs: f64) -> Option<ImageClip> {
        if at_secs <= 0.0 || at_secs >= self.duration_secs {
            return None;
        }
        let mut second = self.clone();
        second.id = Uuid::new_v4();
        second.name = format!("{} (2)", self.name);
        second.timeline_start = self.timeline_start + at_secs;
        second.duration_secs = self.duration_secs - at_secs;
        self.duration_secs = at_secs;
        Some(second)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_clip_creation() {
        let clip = ImageClip::new("test_image", "foo.png", 5.0, 10.0);
        assert_eq!(clip.name, "test_image");
        assert_eq!(clip.timeline_start, 5.0);
        assert_eq!(clip.duration_secs, 10.0);
        assert_eq!(clip.opacity, 1.0);
    }

    #[test]
    fn test_image_clip_set_opacity() {
        let mut clip = ImageClip::new("test", "test.png", 0.0, 5.0);
        clip.set_opacity(0.5);
        assert_eq!(clip.opacity, 0.5);
        
        // Edge case: boundaries
        clip.set_opacity(1.5);
        assert_eq!(clip.opacity, 1.0);
        
        clip.set_opacity(-0.5);
        assert_eq!(clip.opacity, 0.0);
    }

    #[test]
    fn test_image_clip_split_valid() {
        let mut clip = ImageClip::new("test", "test.png", 10.0, 20.0);
        let split = clip.split_at(5.0);
        
        assert!(split.is_some());
        let second = split.unwrap();
        
        assert_eq!(clip.duration_secs, 5.0);
        assert_eq!(second.timeline_start, 15.0);
        assert_eq!(second.duration_secs, 15.0);
        assert_eq!(second.name, "test (2)");
    }

    #[test]
    fn test_image_clip_split_edge_cases() {
        let mut clip = ImageClip::new("test", "test.png", 0.0, 10.0);
        
        // Split exactly at start (invalid)
        let split_start = clip.split_at(0.0);
        assert!(split_start.is_none());
        assert_eq!(clip.duration_secs, 10.0);
        
        // Split at negative (invalid)
        let split_neg = clip.split_at(-5.0);
        assert!(split_neg.is_none());
        
        // Split at or beyond end (invalid)
        let split_end = clip.split_at(10.0);
        assert!(split_end.is_none());
        let split_beyond = clip.split_at(15.0);
        assert!(split_beyond.is_none());
    }

    #[test]
    fn test_apply_ken_burns() {
        let mut clip = ImageClip::new("test", "test.png", 0.0, 10.0);
        assert!(clip.ken_burns.is_none());
        clip.apply_ken_burns();
        assert!(clip.ken_burns.is_some());
        let kb = clip.ken_burns.unwrap();
        assert_eq!(kb.zoom_start, 1.0);
        assert_eq!(kb.zoom_end, 1.2);
    }

    #[test]
    fn test_image_mode_display() {
        assert_eq!(ImageMode::Overlay.to_string(), "overlay");
        assert_eq!(ImageMode::Background.to_string(), "background");
        assert_eq!(ImageMode::Fullscreen.to_string(), "fullscreen");
    }
}
