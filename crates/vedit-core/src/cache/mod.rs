use std::path::{Path, PathBuf};
use anyhow::Result;

/// Gestión de archivos temporales y proxies para el proyecto
#[derive(Debug)]
pub struct CacheManager {
    cache_dir: PathBuf,
}

impl CacheManager {
    pub fn new(cache_dir: impl Into<PathBuf>) -> Self {
        Self { cache_dir: cache_dir.into() }
    }

    /// Directorio de caché por defecto (~/.cache/vedit/<project-id>)
    pub fn default_for_project(project_id: &uuid::Uuid) -> Result<Self> {
        let base = dirs_next();
        let dir = base.join("vedit").join(project_id.to_string());
        std::fs::create_dir_all(&dir)?;
        Ok(Self::new(dir))
    }

    pub fn proxy_dir(&self) -> PathBuf {
        self.cache_dir.join("proxies")
    }

    pub fn temp_dir(&self) -> PathBuf {
        self.cache_dir.join("tmp")
    }

    pub fn proxy_path(&self, source: &Path) -> PathBuf {
        let stem = source
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("clip");
        self.proxy_dir().join(format!("{}_proxy.mp4", stem))
    }

    /// Limpia todos los archivos temporales
    pub fn clear_temp(&self) -> Result<()> {
        let tmp = self.temp_dir();
        if tmp.exists() {
            std::fs::remove_dir_all(&tmp)?;
        }
        std::fs::create_dir_all(&tmp)?;
        tracing::info!("Caché temporal limpiada en {:?}", tmp);
        Ok(())
    }

    /// Limpia todo el caché del proyecto
    pub fn clear_all(&self) -> Result<()> {
        if self.cache_dir.exists() {
            std::fs::remove_dir_all(&self.cache_dir)?;
        }
        std::fs::create_dir_all(&self.cache_dir)?;
        tracing::info!("Caché completa limpiada en {:?}", self.cache_dir);
        Ok(())
    }
}

fn dirs_next() -> PathBuf {
    std::env::var("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
            PathBuf::from(home).join(".cache")
        })
}
