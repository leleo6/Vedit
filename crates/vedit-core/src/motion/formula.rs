use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────────────────
// MovementFormula
// ─────────────────────────────────────────────────────────────────────────────

/// Fórmula de movimiento dinámico aplicable a posición/escala de un clip.
///
/// Cada variante produce una expresión de texto que FFmpeg evalúa por frame
/// cuando se usa en filtros que soportan `eval=frame`.
///
/// # Ejemplo de uso en un filtro FFmpeg
/// ```text
/// overlay=x='sin(t*3.14)*100+960':y=0:eval=frame
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "params")]
pub enum MovementFormula {
    // ── Movimiento horizontal ────────────────────────────────────────────
    /// Rebote sinusoidal: amplitud × sin(t × frecuencia)
    Bounce {
        /// Amplitud máxima del rebote en píxeles
        amplitude_px: f64,
        /// Frecuencia de oscilación (radianes/segundo)
        frequency: f64,
    },

    // ── Escala ───────────────────────────────────────────────────────────
    /// Pulso de escala: oscila entre `scale_min` y `scale_max`
    Pulse {
        scale_min: f64,
        scale_max: f64,
        /// Frecuencia de oscilación (radianes/segundo)
        frequency: f64,
    },

    // ── Traslación lineal ────────────────────────────────────────────────
    /// Desplazamiento lineal constante en píxeles/segundo
    Linear {
        /// Velocidad horizontal (px/s, puede ser negativa)
        velocity_x: f64,
        /// Velocidad vertical (px/s, puede ser negativa)
        velocity_y: f64,
    },

    // ── Trayectoria circular ─────────────────────────────────────────────
    /// Órbita circular alrededor de un punto central
    Orbit {
        /// Coordenada X del centro (fracción del frame, 0.0–1.0)
        center_x_frac: f64,
        /// Coordenada Y del centro (fracción del frame, 0.0–1.0)
        center_y_frac: f64,
        /// Radio de la órbita en píxeles
        radius_px: f64,
        /// Velocidad angular (radianes/segundo)
        angular_speed: f64,
    },

    // ── Expresión personalizada ──────────────────────────────────────────
    /// Expresión FFmpeg arbitraria evaluada por frame.
    ///
    /// El usuario es responsable de proveer una expresión válida.
    /// Se pasa directamente al filtro sin escapado adicional.
    ///
    /// # Ejemplo
    /// ```text
    /// Custom { x_expr: "sin(t*2)*200+960".into(), y_expr: "cos(t)*100+540".into() }
    /// ```
    Custom {
        /// Expresión para la coordenada X (string de FFmpeg)
        x_expr: String,
        /// Expresión para la coordenada Y (string de FFmpeg)
        y_expr: String,
    },
}

// ─────────────────────────────────────────────────────────────────────────────
// Conversión a expresiones FFmpeg
// ─────────────────────────────────────────────────────────────────────────────

/// Par de expresiones FFmpeg (x, y) listas para insertar en un filtro overlay.
#[derive(Debug, Clone, PartialEq)]
pub struct FfmpegMotionExprs {
    /// Expresión para la coordenada X del overlay
    pub x: String,
    /// Expresión para la coordenada Y del overlay
    pub y: String,
}

impl MovementFormula {
    /// Convierte la fórmula en el par de expresiones FFmpeg `(x_expr, y_expr)`
    /// para su uso en `overlay=x='...':y='...':eval=frame`.
    ///
    /// Los valores de `frame_w` y `frame_h` son necesarios para convertir
    /// las fracciones de posición en píxeles absolutos.
    pub fn to_ffmpeg_exprs(&self, frame_w: u32, frame_h: u32) -> FfmpegMotionExprs {
        match self {
            MovementFormula::Bounce { amplitude_px, frequency } => {
                // x oscila sinusoidalmente; y permanece estático en el origen
                FfmpegMotionExprs {
                    x: format!("sin(t*{:.6})*{:.2}", frequency, amplitude_px),
                    y: "0".to_string(),
                }
            }

            MovementFormula::Pulse { scale_min, scale_max, frequency } => {
                // La escala no es una coordenada (x, y), pero se representa
                // como expresión de escala compacta para el filtro `scale`.
                // Devolvemos la expresión de escala en el campo `x` por convención.
                let mid   = (scale_max + scale_min) / 2.0;
                let half  = (scale_max - scale_min) / 2.0;
                FfmpegMotionExprs {
                    x: format!(
                        "iw*({:.4}+{:.4}*sin(t*{:.6}))",
                        mid, half, frequency
                    ),
                    y: format!(
                        "ih*({:.4}+{:.4}*sin(t*{:.6}))",
                        mid, half, frequency
                    ),
                }
            }

            MovementFormula::Linear { velocity_x, velocity_y } => {
                FfmpegMotionExprs {
                    x: format!("t*{:.4}", velocity_x),
                    y: format!("t*{:.4}", velocity_y),
                }
            }

            MovementFormula::Orbit { center_x_frac, center_y_frac, radius_px, angular_speed } => {
                let cx = center_x_frac * frame_w as f64;
                let cy = center_y_frac * frame_h as f64;
                FfmpegMotionExprs {
                    x: format!("{:.2}+{:.2}*cos(t*{:.6})", cx, radius_px, angular_speed),
                    y: format!("{:.2}+{:.2}*sin(t*{:.6})", cy, radius_px, angular_speed),
                }
            }

            MovementFormula::Custom { x_expr, y_expr } => {
                FfmpegMotionExprs {
                    x: x_expr.clone(),
                    y: y_expr.clone(),
                }
            }
        }
    }

    /// Construye el fragmento de filtro overlay completo con `eval=frame`.
    ///
    /// Ejemplo de salida:
    /// ```text
    /// overlay=x='sin(t*3.14)*100':y='0':eval=frame:eof_action=pass
    /// ```
    pub fn to_overlay_filter(&self, frame_w: u32, frame_h: u32) -> String {
        let exprs = self.to_ffmpeg_exprs(frame_w, frame_h);
        format!(
            "overlay=x='{}':y='{}':eval=frame:eof_action=pass",
            exprs.x, exprs.y
        )
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Display — nombre legible para logs y CLI
// ─────────────────────────────────────────────────────────────────────────────

impl std::fmt::Display for MovementFormula {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MovementFormula::Bounce { .. }  => write!(f, "bounce"),
            MovementFormula::Pulse { .. }   => write!(f, "pulse"),
            MovementFormula::Linear { .. }  => write!(f, "linear"),
            MovementFormula::Orbit { .. }   => write!(f, "orbit"),
            MovementFormula::Custom { .. }  => write!(f, "custom"),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounce_produces_sin_expression() {
        let f = MovementFormula::Bounce { amplitude_px: 100.0, frequency: std::f64::consts::PI };
        let exprs = f.to_ffmpeg_exprs(1920, 1080);
        assert!(exprs.x.contains("sin(t*"));
        assert_eq!(exprs.y, "0");
    }

    #[test]
    fn linear_produces_t_expression() {
        let f = MovementFormula::Linear { velocity_x: 50.0, velocity_y: -30.0 };
        let exprs = f.to_ffmpeg_exprs(1920, 1080);
        assert!(exprs.x.contains("t*"));
        assert!(exprs.y.contains("t*"));
    }

    #[test]
    fn custom_passthrough() {
        let x = "sin(t)*200+960".to_string();
        let y = "cos(t)*100+540".to_string();
        let f = MovementFormula::Custom { x_expr: x.clone(), y_expr: y.clone() };
        let exprs = f.to_ffmpeg_exprs(1920, 1080);
        assert_eq!(exprs.x, x);
        assert_eq!(exprs.y, y);
    }

    #[test]
    fn overlay_filter_contains_eval_frame() {
        let f = MovementFormula::Bounce { amplitude_px: 80.0, frequency: 2.0 };
        let filter = f.to_overlay_filter(1920, 1080);
        assert!(filter.contains("eval=frame"));
    }

    #[test]
    fn display_names_are_stable() {
        assert_eq!(MovementFormula::Bounce { amplitude_px: 1.0, frequency: 1.0 }.to_string(), "bounce");
        assert_eq!(MovementFormula::Pulse { scale_min: 0.8, scale_max: 1.2, frequency: 1.0 }.to_string(), "pulse");
        assert_eq!(MovementFormula::Linear { velocity_x: 1.0, velocity_y: 0.0 }.to_string(), "linear");
    }
}
