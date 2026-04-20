pub mod project;
pub mod track;
pub mod clip;
pub mod audio;
pub mod image;
pub mod video;
pub mod text;
pub mod render;
pub mod history;
pub mod cache;
pub mod config;

/// Helpers de presentación compartidos
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

pub fn success(msg: &str) {
    println!("{} {}", style("✔").green().bold(), msg);
}

pub fn warn(msg: &str) {
    eprintln!("{} {}", style("⚠").yellow().bold(), msg);
}

pub fn error(msg: &str) {
    eprintln!("{} {}", style("✖").red().bold(), msg);
}



pub fn section(title: &str) {
    println!("\n{}", style(title).cyan().bold().underlined());
}

pub fn spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb
}

/// Obtiene la duración de un archivo multimedia usando ffprobe
pub async fn get_media_duration(path: &std::path::Path) -> Result<f64, anyhow::Error> {
    let output = tokio::process::Command::new("ffprobe")
        .args(&[
            "-v", "error",
            "-show_entries", "format=duration",
            "-of", "default=noprint_wrappers=1:nokey=1",
        ])
        .arg(path)
        .output()
        .await?;

    if !output.status.success() {
        anyhow::bail!("ffprobe falló al procesar el archivo: {:?}", path);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let dur: f64 = stdout.trim().parse().map_err(|_| anyhow::anyhow!("No se pudo parsear la duración de ffprobe"))?;
    Ok(dur)
}


