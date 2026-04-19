use std::path::PathBuf;
use anyhow::Result;
use clap::Subcommand;
use vedit_core::project::Project;
use vedit_core::cache::CacheManager;
use super::{success, section};

#[derive(Subcommand, Debug)]
pub enum CacheCmd {
    /// Limpia los archivos temporales generados (tmp/) de un proyecto
    Clean {
        #[arg(short, long)]
        project: PathBuf,
    },
    /// Borra toda la caché del proyecto (incluye proxies y temporales)
    ClearAll {
        #[arg(short, long)]
        project: PathBuf,
    },
}

pub async fn run(cmd: CacheCmd) -> Result<()> {
    match cmd {
        CacheCmd::Clean { project: proj_path } => {
            let project = Project::load(&proj_path).await?;
            let cache = CacheManager::default_for_project(&project.id)?;
            section("Limpiando temporales...");
            cache.clear_temp()?;
            success("Temporales eliminados.");
        }
        CacheCmd::ClearAll { project: proj_path } => {
            let project = Project::load(&proj_path).await?;
            let cache = CacheManager::default_for_project(&project.id)?;
            section("Limpiando caché completa...");
            cache.clear_all()?;
            success("Caché borrada.");
        }
    }
    Ok(())
}
