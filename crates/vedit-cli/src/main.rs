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
    video::VideoCmd,
    text::TextCmd,
    render::RenderCmd,
    history::HistoryCmd,
    cache::CacheCmd,
};

const BANNER: &str = r#"
 ██╗   ██╗███████╗██████╗ ██╗████████╗
 ██║   ██║██╔════╝██╔══██╗██║╚══██╔══╝
 ██║   ██║█████╗  ██║  ██║██║   ██║   
 ╚██╗ ██╔╝██╔══╝  ██║  ██║██║   ██║   
  ╚████╔╝ ███████╗██████╔╝██║   ██║   
   ╚═══╝  ╚══════╝╚═════╝ ╚═╝   ╚═╝   
    Video Editor CLI (Stateful NLE)
"#;

#[derive(Parser, Debug)]
#[command(
    name = "vedit",
    version,
    before_help = BANNER,
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

    /// Operaciones de video (add, transform, speed, color, effects, transition, stabilize)
    #[command(subcommand)]
    Video(VideoCmd),

    /// Operaciones de texto y subtítulos (add, style, position, import-srt)
    #[command(subcommand)]
    Text(TextCmd),

    /// Renderizado del proyecto
    #[command(subcommand)]
    Render(RenderCmd),

    /// Deshacer y rehacer cambios (Undo/Redo)
    #[command(subcommand)]
    History(HistoryCmd),

    /// Gestión de archivos temporales y proxies
    #[command(subcommand)]
    Cache(CacheCmd),
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
        Commands::Video(cmd)   => commands::video::run(cmd).await,
        Commands::Text(cmd)    => commands::text::run(cmd).await,
        Commands::Render(cmd)  => commands::render::run(cmd).await,
        Commands::History(cmd) => commands::history::run(cmd).await,
        Commands::Cache(cmd)   => commands::cache::run(cmd).await,
    }
}
