pub mod command;
pub mod probe;

/// Escapa una cadena para ser usada dentro de un filtro de FFmpeg.
/// Los filtros de FFmpeg usan comillas simples (') para encerrar valores que contienen espacios
/// o caracteres especiales. Si el valor contiene una comilla simple, debe ser escapada.
pub fn escape_filter_arg(arg: &str) -> String {
    // FFmpeg espera que las comillas simples se escapen como '\''
    arg.replace("'", "'\\''")
}

pub use command::FfmpegCommand;
