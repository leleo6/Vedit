use std::path::Path;
use anyhow::{Context, Result};
use serde::Deserialize;
use tokio::process::Command;

/// Información de un stream de media obtenida por ffprobe
#[derive(Debug, Deserialize)]
pub struct StreamInfo {
    pub codec_type: String,
    pub codec_name: String,
    pub duration: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub sample_rate: Option<String>,
    pub channels: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ProbeResult {
    pub streams: Vec<StreamInfo>,
}

/// Obtiene información de duración y streams de un archivo media
pub async fn probe_file(path: &Path) -> Result<ProbeResult> {
    let output = Command::new("ffprobe")
        .args([
            "-v", "quiet",
            "-print_format", "json",
            "-show_streams",
            path.to_str().ok_or_else(|| anyhow::anyhow!("La ruta contiene caracteres no-UTF8: {:?}", path))?,
        ])
        .output()
        .await
        .context("No se pudo ejecutar ffprobe")?;

    if !output.status.success() {
        anyhow::bail!("ffprobe falló para {:?}", path);
    }

    let result: ProbeResult = serde_json::from_slice(&output.stdout)
        .context("ffprobe devolvió JSON inválido")?;

    Ok(result)
}

/// Devuelve la duración en segundos del primer stream de un archivo
pub async fn get_duration(path: &Path) -> Result<f64> {
    let probe = probe_file(path).await?;
    probe
        .streams
        .iter()
        .filter_map(|s| s.duration.as_ref())
        .filter_map(|d| d.parse::<f64>().ok())
        .next()
        .ok_or_else(|| anyhow::anyhow!("No se encontró duración en {:?}", path))
}
