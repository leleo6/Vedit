//! # `motion` — Movimiento dinámico y control de regiones
//!
//! Este módulo agrupa:
//! - [`formula::MovementFormula`]: enum tipado de fórmulas de movimiento FFmpeg.
//! - [`presets`]: catálogo de presets matemáticos listos para usar.
//! - [`region::RenderRegion`]: DTO de rango temporal para renderizado parcial.

pub mod formula;
pub mod presets;
pub mod region;

// Reexports de conveniencia para los consumidores del crate
pub use formula::{MovementFormula, FfmpegMotionExprs};
pub use presets::MotionPresets;
pub use region::RenderRegion;
