use super::formula::MovementFormula;
use std::f64::consts::PI;

// ─────────────────────────────────────────────────────────────────────────────
// Catálogo de presets de movimiento
// ─────────────────────────────────────────────────────────────────────────────

/// Catálogo de fórmulas de movimiento probadas y listas para usar.
///
/// Todas las constantes son valores de tiempo cero: se construyen con
/// parámetros nominales que producen efectos visualmente agradables.
/// Son directamente inyectables en `MovementFormula`.
///
/// # Uso
/// ```rust
/// use vedit_core::motion::presets;
///
/// let formula = presets::dvd_bounce();
/// let filter  = formula.to_overlay_filter(1920, 1080);
/// ```
pub struct MotionPresets;

impl MotionPresets {
    /// Devuelve el preset por nombre (case-insensitive).
    /// Devuelve `None` si el nombre no corresponde a ningún preset conocido.
    pub fn by_name(name: &str) -> Option<MovementFormula> {
        match name.to_lowercase().as_str() {
            "dvd_bounce" | "dvd"         => Some(dvd_bounce()),
            "pulse_slow"                 => Some(pulse_slow()),
            "pulse_fast"                 => Some(pulse_fast()),
            "drift_right"                => Some(drift_right()),
            "drift_left"                 => Some(drift_left()),
            "pendulum"                   => Some(pendulum()),
            "orbit_slow"                 => Some(orbit_slow()),
            "orbit_fast"                 => Some(orbit_fast()),
            "shake"                      => Some(shake()),
            _                            => None,
        }
    }

    /// Lista todos los nombres de presets disponibles.
    pub fn available_names() -> &'static [&'static str] {
        &[
            "dvd_bounce",
            "pulse_slow",
            "pulse_fast",
            "drift_right",
            "drift_left",
            "pendulum",
            "orbit_slow",
            "orbit_fast",
            "shake",
        ]
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Constructores de presets
// ─────────────────────────────────────────────────────────────────────────────

/// Clásico rebote de DVD: logo que rebota diagonalmente en el frame.
///
/// Produce un movimiento sinusoidal combinado en X e Y que simula el rebote
/// de los logos en protectores de pantalla.
///
/// # FFmpeg expression (1920×1080)
/// ```text
/// overlay=x='sin(t*1.0)*960':y='0':eval=frame
/// ```
/// Para el efecto completo de DVD se necesitan dos overlays ortogonales
/// (uno para X, uno para Y). Este preset cubre el eje X; úsalo combinado
/// con `orbit_slow` para el efecto completo.
pub fn dvd_bounce() -> MovementFormula {
    MovementFormula::Bounce {
        amplitude_px: 800.0, // casi todo el ancho del frame en 1080p
        frequency:    1.0,   // 1 ciclo completo / (2π) ≈ cada 6.28 s
    }
}

/// Pulso lento: el clip respira suavemente entre 80% y 120% de escala.
pub fn pulse_slow() -> MovementFormula {
    MovementFormula::Pulse {
        scale_min: 0.8,
        scale_max: 1.2,
        frequency: PI / 4.0, // período ≈ 8 s
    }
}

/// Pulso rápido: latido enérgico entre 90% y 110% de escala.
pub fn pulse_fast() -> MovementFormula {
    MovementFormula::Pulse {
        scale_min: 0.9,
        scale_max: 1.1,
        frequency: PI,       // período ≈ 2 s
    }
}

/// Deriva continua hacia la derecha a 100 px/s.
pub fn drift_right() -> MovementFormula {
    MovementFormula::Linear {
        velocity_x:  100.0,
        velocity_y:    0.0,
    }
}

/// Deriva continua hacia la izquierda a 100 px/s.
pub fn drift_left() -> MovementFormula {
    MovementFormula::Linear {
        velocity_x: -100.0,
        velocity_y:    0.0,
    }
}

/// Péndulo horizontal centrado: oscila ±300 px alrededor del centro del frame.
pub fn pendulum() -> MovementFormula {
    MovementFormula::Bounce {
        amplitude_px: 300.0,
        frequency:    PI / 2.0, // período ≈ 4 s — se siente como péndulo
    }
}

/// Órbita lenta alrededor del centro del frame (radio 200 px, período ≈ 12 s).
pub fn orbit_slow() -> MovementFormula {
    MovementFormula::Orbit {
        center_x_frac: 0.5,
        center_y_frac: 0.5,
        radius_px:     200.0,
        angular_speed: PI / 6.0,
    }
}

/// Órbita rápida alrededor del centro del frame (radio 100 px, período ≈ 4 s).
pub fn orbit_fast() -> MovementFormula {
    MovementFormula::Orbit {
        center_x_frac: 0.5,
        center_y_frac: 0.5,
        radius_px:     100.0,
        angular_speed: PI / 2.0,
    }
}

/// Temblor/shake: vibración aleatoria-determinista rápida mediante composición de senos.
///
/// Simula el efecto de cámara temblando: combina tres frecuencias en X e Y.
pub fn shake() -> MovementFormula {
    // La expresión combina tres armónicos para dar sensación de aleatoriedad
    MovementFormula::Custom {
        x_expr: "sin(t*23)*8+sin(t*47)*5+sin(t*71)*3".to_string(),
        y_expr: "sin(t*31)*6+sin(t*53)*4+sin(t*79)*2".to_string(),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn by_name_resolves_known_presets() {
        assert!(MotionPresets::by_name("dvd_bounce").is_some());
        assert!(MotionPresets::by_name("DVD").is_some()); // case-insensitive
        assert!(MotionPresets::by_name("shake").is_some());
        assert!(MotionPresets::by_name("no_existe").is_none());
    }

    #[test]
    fn all_presets_produce_valid_overlay_filter() {
        for name in MotionPresets::available_names() {
            let formula = MotionPresets::by_name(name).unwrap();
            let filter  = formula.to_overlay_filter(1920, 1080);
            assert!(filter.contains("eval=frame"), "preset '{}' must include eval=frame", name);
            assert!(filter.contains("overlay=x="), "preset '{}' must start with overlay=x=", name);
        }
    }

    #[test]
    fn dvd_bounce_has_high_amplitude() {
        if let MovementFormula::Bounce { amplitude_px, .. } = dvd_bounce() {
            assert!(amplitude_px > 400.0, "DVD bounce debe tener amplitud significativa");
        } else {
            panic!("dvd_bounce debe ser variante Bounce");
        }
    }

    #[test]
    fn shake_is_custom_with_multiple_harmonics() {
        if let MovementFormula::Custom { x_expr, .. } = shake() {
            // Al menos 2 términos aditivos
            assert!(x_expr.contains('+'), "shake debe combinar armónicos");
        } else {
            panic!("shake debe ser variante Custom");
        }
    }
}
