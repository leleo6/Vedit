mod commands;

use clap::{Parser, Subcommand};
use anyhow::Result;
use tracing_subscriber::EnvFilter;

use commands::{
    project::ProjectCmd,
    track::TrackCmd,
    clip::ClipCmd,
    audio::AudioCmd,
    image::ImageCmd,
    render::RenderCmd,
};

/// ╔══════════════════════════════════════╗
/// ║         vedit — Video Editor CLI     ║
/// ╚══════════════════════════════════════╝
#[derive(Parser, Debug)]
#[command(
    name = "vedit",
    version,
    about = "💎 vedit – Editor de video/audio por línea de comandos",
    long_about = "vedit te permite gestionar proyectos de video y audio,\nagregar tracks/clips, aplicar efectos y renderizar, todo desde la terminal.",
    propagate_version = true,
)]
struct Cli {
    /// Nivel de log (trace, debug, info, warn, error)
    #[arg(long, global = true, default_value = "info", env = "VEDIT_LOG")]
    log: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Gestión de proyectos (new, open, info)
    #[command(subcommand)]
    Project(ProjectCmd),

    /// Gestión de tracks (add, remove, list, rename, volume, mute)
    #[command(subcommand)]
    Track(TrackCmd),

    /// Gestión de clips (add, remove, move, trim, split, loop)
    #[command(subcommand)]
    Clip(ClipCmd),

    /// Operaciones de audio (mix, mute, normalize, fade, speed)
    #[command(subcommand)]
    Audio(AudioCmd),

    /// Operaciones de imagen (add, transform, fade, ken-burns)
    #[command(subcommand)]
    Image(ImageCmd),

    /// Renderizado del proyecto
    #[command(subcommand)]
    Render(RenderCmd),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Inicializar tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(&cli.log)),
        )
        .with_target(false)
        .compact()
        .init();

    match cli.command {
        Commands::Project(cmd) => commands::project::run(cmd).await,
        Commands::Track(cmd)   => commands::track::run(cmd).await,
        Commands::Clip(cmd)    => commands::clip::run(cmd).await,
        Commands::Audio(cmd)   => commands::audio::run(cmd).await,
        Commands::Image(cmd)   => commands::image::run(cmd).await,
        Commands::Render(cmd)  => commands::render::run(cmd).await,
    }
}
