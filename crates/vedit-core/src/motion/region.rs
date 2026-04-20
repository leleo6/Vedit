use serde::{Deserialize, Serialize};
use anyhow::Result;

// ─────────────────────────────────────────────────────────────────────────────
// RenderRegion — DTO de región de tiempo para renderizado
// ─────────────────────────────────────────────────────────────────────────────

/// Define un sub-rango temporal de un proyecto para ser renderizado.
///
/// Es el DTO que viaja desde el CLI hasta el compositor sin exponer detalles
/// de FFmpeg. El compositor lo traduce a `-ss` / `-t` cuando sea necesario.
///
/// # Invariante
/// `duration_secs > 0.0` y `start_secs >= 0.0`.
/// Usa `RenderRegion::new()` para garantizar ambas condiciones.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RenderRegion {
    /// Instante de inicio (segundos desde el inicio del proyecto)
    pub start_secs: f64,
    /// Duración del segmento a renderizar (segundos)
    pub duration_secs: f64,
}

impl RenderRegion {
    /// Crea una región validada.
    ///
    /// # Errores
    /// - `start_secs < 0.0`
    /// - `duration_secs <= 0.0`
    pub fn new(start_secs: f64, duration_secs: f64) -> Result<Self> {
        if start_secs < 0.0 {
            anyhow::bail!(
                "El instante de inicio ({:.3}s) no puede ser negativo.",
                start_secs
            );
        }
        if duration_secs <= 0.0 {
            anyhow::bail!(
                "La duración ({:.3}s) debe ser mayor que cero.",
                duration_secs
            );
        }
        Ok(Self { start_secs, duration_secs })
    }

    /// Instante de fin de la región (no inclusive).
    #[inline]
    pub fn end_secs(&self) -> f64 {
        self.start_secs + self.duration_secs
    }

    /// Verifica si un instante del proyecto cae dentro de la región.
    #[inline]
    pub fn contains(&self, t: f64) -> bool {
        t >= self.start_secs && t < self.end_secs()
    }

    /// Convierte la región en los argumentos FFmpeg `-ss <start> -t <duration>`.
    /// El llamador los inserta antes del output en la línea de comando.
    pub fn to_ffmpeg_args(&self) -> [String; 4] {
        [
            "-ss".to_string(),
            format!("{:.3}", self.start_secs),
            "-t".to_string(),
            format!("{:.3}", self.duration_secs),
        ]
    }
}

impl std::fmt::Display for RenderRegion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:.3}s – {:.3}s (duración: {:.3}s)",
            self.start_secs,
            self.end_secs(),
            self.duration_secs
        )
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_region_is_created() {
        let r = RenderRegion::new(10.0, 30.0).unwrap();
        assert_eq!(r.start_secs, 10.0);
        assert_eq!(r.duration_secs, 30.0);
        assert!((r.end_secs() - 40.0).abs() < f64::EPSILON);
    }

    #[test]
    fn negative_start_is_rejected() {
        assert!(RenderRegion::new(-1.0, 10.0).is_err());
    }

    #[test]
    fn zero_duration_is_rejected() {
        assert!(RenderRegion::new(0.0, 0.0).is_err());
        assert!(RenderRegion::new(5.0, -3.0).is_err());
    }

    #[test]
    fn contains_works_correctly() {
        let r = RenderRegion::new(10.0, 5.0).unwrap(); // 10–15s
        assert!( r.contains(10.0));
        assert!( r.contains(14.999));
        assert!(!r.contains(15.0));  // end no inclusive
        assert!(!r.contains(9.999));
    }

    #[test]
    fn ffmpeg_args_are_well_formed() {
        let r = RenderRegion::new(5.5, 20.0).unwrap();
        let args = r.to_ffmpeg_args();
        assert_eq!(args[0], "-ss");
        assert_eq!(args[1], "5.500");
        assert_eq!(args[2], "-t");
        assert_eq!(args[3], "20.000");
    }
}
