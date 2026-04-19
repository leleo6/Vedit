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
// VideoClip — tipos auxiliares
// ─────────────────────────────────────────────────────────────────────────────

/// Escala del clip de video (porcentaje del frame o resolución exacta)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoScale {
    /// Fracción del ancho del frame (1.0 = 100%)
    pub width: f64,
    /// Fracción del alto del frame (1.0 = 100%)
    pub height: f64,
}

impl Default for VideoScale {
    fn default() -> Self {
        Self { width: 1.0, height: 1.0 }
    }
}

/// Posición del clip en el frame (coordenadas fraccionarias)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoPosition {
    /// Coordenada X (fracción del ancho, 0.0 = izquierda)
    pub x: f64,
    /// Coordenada Y (fracción del alto, 0.0 = arriba)
    pub y: f64,
}

impl Default for VideoPosition {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

/// Recorte del frame del clip de video (en píxeles)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VideoCrop {
    pub top: u32,
    pub bottom: u32,
    pub left: u32,
    pub right: u32,
}

/// Corrección de color del clip
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorCorrection {
    /// Brillo (-1.0 a 1.0, 0.0 = sin cambio)
    pub brightness: f64,
    /// Contraste (0.0 a 2.0, 1.0 = sin cambio)
    pub contrast: f64,
    /// Saturación (0.0 = escala de grises, 1.0 = original, 2.0 = máximo)
    pub saturation: f64,
    /// Temperatura de color en Kelvin (None = sin cambio)
    pub temperature_k: Option<f64>,
    /// Curvas RGB básicas: (shadows, midtones, highlights) por canal
    pub curves_r: Option<(f64, f64, f64)>,
    pub curves_g: Option<(f64, f64, f64)>,
    pub curves_b: Option<(f64, f64, f64)>,
    /// Ruta a archivo LUT .cube para color grading
    pub lut_path: Option<PathBuf>,
}

impl Default for ColorCorrection {
    fn default() -> Self {
        Self {
            brightness: 0.0,
            contrast: 1.0,
            saturation: 1.0,
            temperature_k: None,
            curves_r: None,
            curves_g: None,
            curves_b: None,
            lut_path: None,
        }
    }
}

impl ColorCorrection {
    /// Retorna true si alguna corrección de color difiere de los valores por defecto
    pub fn is_active(&self) -> bool {
        self.brightness.abs() > 0.001
            || (self.contrast - 1.0).abs() > 0.001
            || (self.saturation - 1.0).abs() > 0.001
            || self.temperature_k.is_some()
            || self.curves_r.is_some()
            || self.curves_g.is_some()
            || self.curves_b.is_some()
            || self.lut_path.is_some()
    }
}

/// Efectos visuales del clip de video
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VideoEffects {
    /// Blur gaussiano (radio en píxeles, None = desactivado)
    pub blur_radius: Option<f64>,
    /// Sharpening (0.0–5.0, None = desactivado)
    pub sharpen: Option<f64>,
    /// Viñeta (intensidad 0.0–1.0, None = desactivado)
    pub vignette: Option<f64>,
    /// Ruido/grano (intensidad 0.0–1.0, None = desactivado)
    pub noise: Option<f64>,
    /// Deinterlace (para material de cámara antigua)
    pub deinterlace: bool,
}

impl VideoEffects {
    pub fn is_active(&self) -> bool {
        self.blur_radius.is_some()
            || self.sharpen.is_some()
            || self.vignette.is_some()
            || self.noise.is_some()
            || self.deinterlace
    }
}

/// Tipo de transición de salida del clip
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransitionKind {
    /// Corte directo (sin transición)
    #[default]
    Cut,
    /// Fade a negro
    FadeToBlack,
    /// Fade a blanco
    FadeToWhite,
    /// Cross dissolve (fundido entre dos clips)
    CrossDissolve,
    /// Wipe horizontal de izquierda a derecha
    WipeHorizontal,
    /// Wipe vertical de arriba a abajo
    WipeVertical,
}

impl std::fmt::Display for TransitionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransitionKind::Cut            => write!(f, "cut"),
            TransitionKind::FadeToBlack    => write!(f, "fade_to_black"),
            TransitionKind::FadeToWhite    => write!(f, "fade_to_white"),
            TransitionKind::CrossDissolve  => write!(f, "cross_dissolve"),
            TransitionKind::WipeHorizontal => write!(f, "wipe_horizontal"),
            TransitionKind::WipeVertical   => write!(f, "wipe_vertical"),
        }
    }
}

/// Transición de salida del clip
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoTransition {
    pub kind: TransitionKind,
    /// Duración de la transición en segundos
    pub duration_secs: f64,
}

impl VideoTransition {
    pub fn new(kind: TransitionKind, duration_secs: f64) -> Self {
        Self { kind, duration_secs: duration_secs.max(0.0) }
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
    /// Inicio en el archivo fuente (segundos)
    pub source_start: f64,
    /// Fin en el archivo fuente (segundos, None = hasta el final)
    pub source_end: Option<f64>,
    /// Posición de inicio en el timeline del proyecto (segundos)
    pub timeline_start: f64,
    /// Volumen del audio embebido (0.0–2.0)
    pub volume: f64,
    /// Factor de velocidad (1.0 = normal, 0.5 = la mitad, 2.0 = doble)
    pub speed: f64,
    /// Reproducir en reversa
    pub reverse: bool,
    /// Mantener pitch del audio al cambiar velocidad
    pub maintain_pitch: bool,

    // ── Transformación ────────────────────────────────────────────────────
    pub scale: VideoScale,
    pub position: VideoPosition,
    /// Rotación en grados (sentido horario)
    pub rotation_deg: f64,
    /// Flip horizontal
    pub flip_horizontal: bool,
    /// Flip vertical
    pub flip_vertical: bool,
    /// Recorte del frame
    pub crop: Option<VideoCrop>,

    // ── Corrección de color ───────────────────────────────────────────────
    pub color: ColorCorrection,

    // ── Efectos visuales ─────────────────────────────────────────────────
    pub effects: VideoEffects,

    // ── Transiciones ─────────────────────────────────────────────────────
    pub fade_in: Option<FadeEffect>,
    pub fade_out: Option<FadeEffect>,
    /// Transición de salida hacia el siguiente clip
    pub transition_out: Option<VideoTransition>,

    // ── Estabilización ───────────────────────────────────────────────────
    /// Activar estabilización de video (vidstab)
    pub stabilize: bool,

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
            reverse: false,
            maintain_pitch: true,
            scale: VideoScale::default(),
            position: VideoPosition::default(),
            rotation_deg: 0.0,
            flip_horizontal: false,
            flip_vertical: false,
            crop: None,
            color: ColorCorrection::default(),
            effects: VideoEffects::default(),
            fade_in: None,
            fade_out: None,
            transition_out: None,
            stabilize: false,
            replacement_audio: None,
        }
    }

    /// Duración bruta del clip (sin factor de velocidad)
    pub fn raw_duration(&self) -> f64 {
        self.source_end
            .map(|e| (e - self.source_start).max(0.0))
            .unwrap_or(0.0)
    }

    /// Duración efectiva en el timeline (considerando speed)
    pub fn duration(&self) -> f64 {
        let raw = self.raw_duration();
        if self.speed > 0.0 { raw / self.speed } else { raw }
    }

    pub fn set_fade_in(&mut self, secs: f64) {
        self.fade_in = Some(FadeEffect { duration_secs: secs });
    }

    pub fn set_fade_out(&mut self, secs: f64) {
        self.fade_out = Some(FadeEffect { duration_secs: secs });
    }

    /// Ajusta escala (1.0 = 100% del frame)
    pub fn set_scale(&mut self, width: f64, height: f64) {
        self.scale = VideoScale {
            width:  width.max(0.0),
            height: height.max(0.0),
        };
    }

    /// Ajusta posición en el frame (valores fraccionarios)
    pub fn set_position(&mut self, x: f64, y: f64) {
        self.position = VideoPosition { x, y };
    }

    /// Activa o desactiva estabilización
    pub fn set_stabilize(&mut self, enabled: bool) {
        self.stabilize = enabled;
    }

    /// Divide el clip en `at_secs` segundos relativos al inicio del clip en el timeline.
    /// Devuelve el segundo clip resultante.
    ///
    /// # Ejemplo
    ///
    /// ```
    /// use vedit_core::project::clip::VideoClip;
    /// let mut clip = VideoClip::new("video", "v.mp4", 0.0);
    /// clip.source_end = Some(10.0);
    /// let split = clip.split_at(4.0).unwrap();
    /// assert!((clip.duration() - 4.0).abs() < 0.001);
    /// assert!((split.duration() - 6.0).abs() < 0.001);
    /// assert!((split.timeline_start - 4.0).abs() < 0.001);
    /// ```
    pub fn split_at(&mut self, at_secs: f64) -> Option<VideoClip> {
        let dur = self.raw_duration();
        if dur <= 0.0 || at_secs <= 0.0 || at_secs >= dur {
            return None;
        }
        let mut second = self.clone();
        second.id = Uuid::new_v4();
        second.name = format!("{} (2)", self.name);
        // El segundo clip empieza en `source_start + at_secs` del archivo fuente
        second.source_start = self.source_start + at_secs;
        // El segundo clip comienza en el timeline justo después del primero
        second.timeline_start = self.timeline_start + at_secs / self.speed.max(f64::EPSILON);
        // El primero termina en source_start + at_secs
        self.source_end = Some(self.source_start + at_secs);
        Some(second)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ImageClip — tipos auxiliares
// ─────────────────────────────────────────────────────────────────────────────

/// Posición en el frame (coordenadas relativas al frame, 0.0 = borde izq/top, 1.0 = borde der/bot)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImagePosition {
    /// Coordenada X (fracción del ancho del frame, 0.0–1.0); None = centrado
    pub x: Option<f64>,
    /// Coordenada Y (fracción del alto del frame, 0.0–1.0); None = centrado
    pub y: Option<f64>,
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

// ─────────────────────────────────────────────────────────────────────────────
// TextClip — tipos auxiliares
// ─────────────────────────────────────────────────────────────────────────────

/// Alineación horizontal del texto
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TextAlign {
    Left,
    #[default]
    Center,
    Right,
}

impl std::fmt::Display for TextAlign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextAlign::Left   => write!(f, "left"),
            TextAlign::Center => write!(f, "center"),
            TextAlign::Right  => write!(f, "right"),
        }
    }
}

/// Preset de posición en el frame
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextPositionPreset {
    TopLeft, TopCenter, TopRight,
    MiddleLeft, MiddleCenter, MiddleRight,
    BottomLeft,
    #[default]
    BottomCenter,
    BottomRight,
    /// Posición libre — usa los campos pos_x/pos_y
    Custom,
}

impl std::fmt::Display for TextPositionPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextPositionPreset::TopLeft      => write!(f, "top_left"),
            TextPositionPreset::TopCenter    => write!(f, "top_center"),
            TextPositionPreset::TopRight     => write!(f, "top_right"),
            TextPositionPreset::MiddleLeft   => write!(f, "middle_left"),
            TextPositionPreset::MiddleCenter => write!(f, "middle_center"),
            TextPositionPreset::MiddleRight  => write!(f, "middle_right"),
            TextPositionPreset::BottomLeft   => write!(f, "bottom_left"),
            TextPositionPreset::BottomCenter => write!(f, "bottom_center"),
            TextPositionPreset::BottomRight  => write!(f, "bottom_right"),
            TextPositionPreset::Custom       => write!(f, "custom"),
        }
    }
}

/// Color RGBA (0–255)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RgbaColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    /// Opacidad 0–255 (255 = totalmente opaco)
    pub a: u8,
}

impl RgbaColor {
    pub fn white()       -> Self { Self { r: 255, g: 255, b: 255, a: 255 } }
    pub fn black()       -> Self { Self { r:   0, g:   0, b:   0, a: 255 } }
    pub fn transparent() -> Self { Self { r:   0, g:   0, b:   0, a:   0 } }
    /// Representación hex para FFmpeg drawtext: 0xRRGGBBAA
    pub fn to_ffmpeg_hex(&self) -> String {
        format!("0x{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
    }
}

impl Default for RgbaColor {
    fn default() -> Self { Self::white() }
}

/// Sombra del texto
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextShadow {
    pub offset_x: f64,
    pub offset_y: f64,
    pub blur: f64,
    pub color: RgbaColor,
}

impl Default for TextShadow {
    fn default() -> Self {
        Self { offset_x: 2.0, offset_y: 2.0, blur: 3.0, color: RgbaColor::black() }
    }
}

/// Stroke (borde del texto)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextStroke {
    pub width: f64,
    pub color: RgbaColor,
}

impl Default for TextStroke {
    fn default() -> Self { Self { width: 1.5, color: RgbaColor::black() } }
}

/// Estilo completo de un clip de texto
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextStyle {
    pub font_family:    String,
    pub font_size:      u32,
    pub color:          RgbaColor,
    /// Color del cuadro de fondo (None = sin fondo)
    pub bg_color:       Option<RgbaColor>,
    pub bold:           bool,
    pub italic:         bool,
    pub underline:      bool,
    pub align:          TextAlign,
    /// Interlineado (1.0 = normal)
    pub line_height:    f64,
    /// Espaciado entre letras en píxeles
    pub letter_spacing: f64,
    pub stroke:         Option<TextStroke>,
    pub shadow:         Option<TextShadow>,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_family:    "Sans".to_string(),
            font_size:      48,
            color:          RgbaColor::white(),
            bg_color:       None,
            bold:           false,
            italic:         false,
            underline:      false,
            align:          TextAlign::Center,
            line_height:    1.0,
            letter_spacing: 0.0,
            stroke:         None,
            shadow:         None,
        }
    }
}

/// Animación de entrada o salida del clip de texto
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextAnimation {
    Typewriter,
    SlideUp,
    SlideDown,
    SlideLeft,
    SlideRight,
    Fade,
}

impl std::fmt::Display for TextAnimation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextAnimation::Typewriter => write!(f, "typewriter"),
            TextAnimation::SlideUp    => write!(f, "slide_up"),
            TextAnimation::SlideDown  => write!(f, "slide_down"),
            TextAnimation::SlideLeft  => write!(f, "slide_left"),
            TextAnimation::SlideRight => write!(f, "slide_right"),
            TextAnimation::Fade       => write!(f, "fade"),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// TextClip
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextClip {
    pub id: Uuid,
    pub name: String,
    /// Texto a renderizar (puede incluir \n para multilínea)
    pub text: String,
    pub timeline_start: f64,
    pub duration_secs:  f64,

    // ── Estilo ────────────────────────────────────────────────────────────
    pub style: TextStyle,

    // ── Posicionamiento ───────────────────────────────────────────────────
    pub position_preset: TextPositionPreset,
    /// X absoluta en píxeles (solo cuando preset = Custom)
    pub pos_x: Option<f64>,
    /// Y absoluta en píxeles (solo cuando preset = Custom)
    pub pos_y: Option<f64>,
    /// Margen desde los bordes en píxeles
    pub margin: f64,
    /// Rotación en grados
    pub rotation_deg: f64,

    // ── Efectos ───────────────────────────────────────────────────────────
    pub fade_in:         Option<FadeEffect>,
    pub fade_out:        Option<FadeEffect>,
    pub entry_animation: Option<TextAnimation>,
    pub exit_animation:  Option<TextAnimation>,
}

impl TextClip {
    pub fn new(
        name: impl Into<String>,
        text: impl Into<String>,
        timeline_start: f64,
        duration_secs: f64,
    ) -> Self {
        Self {
            id:              Uuid::new_v4(),
            name:            name.into(),
            text:            text.into(),
            timeline_start,
            duration_secs,
            style:           TextStyle::default(),
            position_preset: TextPositionPreset::default(),
            pos_x:           None,
            pos_y:           None,
            margin:          20.0,
            rotation_deg:    0.0,
            fade_in:         None,
            fade_out:        None,
            entry_animation: None,
            exit_animation:  None,
        }
    }

    pub fn duration(&self) -> f64 { self.duration_secs }

    pub fn set_fade_in(&mut self, secs: f64) {
        self.fade_in = Some(FadeEffect { duration_secs: secs });
    }
    pub fn set_fade_out(&mut self, secs: f64) {
        self.fade_out = Some(FadeEffect { duration_secs: secs });
    }

    /// Divide el clip en `at_secs` segundos relativos a su inicio.
    ///
    /// # Ejemplo
    ///
    /// ```
    /// use vedit_core::project::clip::TextClip;
    /// let mut clip = TextClip::new("sub", "Hello World", 0.0, 10.0);
    /// let split = clip.split_at(4.0).unwrap();
    /// assert_eq!(clip.duration_secs, 4.0);
    /// assert_eq!(split.duration_secs, 6.0);
    /// assert_eq!(split.timeline_start, 4.0);
    /// ```
    pub fn split_at(&mut self, at_secs: f64) -> Option<TextClip> {
        if at_secs <= 0.0 || at_secs >= self.duration_secs {
            return None;
        }
        let mut second = self.clone();
        second.id              = Uuid::new_v4();
        second.name            = format!("{} (2)", self.name);
        second.timeline_start  = self.timeline_start + at_secs;
        second.duration_secs   = self.duration_secs - at_secs;
        self.duration_secs     = at_secs;
        Some(second)
    }

    /// Resuelve posición (x, y) como expresión FFmpeg drawtext
    pub fn resolve_ffmpeg_position(&self) -> (String, String) {
        let m = self.margin as i64;
        match &self.position_preset {
            TextPositionPreset::Custom => {
                let x = self.pos_x.map(|v| format!("{:.0}", v))
                    .unwrap_or_else(|| "(w-text_w)/2".to_string());
                let y = self.pos_y.map(|v| format!("{:.0}", v))
                    .unwrap_or_else(|| "(h-text_h)/2".to_string());
                (x, y)
            }
            TextPositionPreset::TopLeft      => (format!("{m}"),             format!("{m}")),
            TextPositionPreset::TopCenter    => ("(w-text_w)/2".into(),      format!("{m}")),
            TextPositionPreset::TopRight     => (format!("w-text_w-{m}"),   format!("{m}")),
            TextPositionPreset::MiddleLeft   => (format!("{m}"),             "(h-text_h)/2".into()),
            TextPositionPreset::MiddleCenter => ("(w-text_w)/2".into(),      "(h-text_h)/2".into()),
            TextPositionPreset::MiddleRight  => (format!("w-text_w-{m}"),   "(h-text_h)/2".into()),
            TextPositionPreset::BottomLeft   => (format!("{m}"),             format!("h-text_h-{m}")),
            TextPositionPreset::BottomCenter => ("(w-text_w)/2".into(),      format!("h-text_h-{m}")),
            TextPositionPreset::BottomRight  => (format!("w-text_w-{m}"),   format!("h-text_h-{m}")),
        }
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
