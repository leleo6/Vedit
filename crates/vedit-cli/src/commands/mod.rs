pub mod project;
pub mod track;
pub mod clip;
pub mod audio;
pub mod image;
pub mod render;

/// Helpers de presentación compartidos
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

pub fn success(msg: &str) {
    println!("{} {}", style("✔").green().bold(), msg);
}

pub fn warn(msg: &str) {
    eprintln!("{} {}", style("⚠").yellow().bold(), msg);
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


