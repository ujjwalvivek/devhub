mod app;
mod cache;
mod config;
mod discovery;
mod editor;
mod ui;
mod workspace;

use anyhow::Result;
use tracing_subscriber::{EnvFilter, fmt};

fn main() -> Result<()> {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("devhub=info")),
        )
        .init();

    tracing::info!("devhub starting");

    let config = config::Config::load_or_create()?;
    tracing::info!(scan_dirs = ?config.scan_dirs, "config loaded");

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1920.0, 1080.0])
            .with_min_inner_size([800.0, 500.0])
            .with_decorations(false)
            .with_title("devhub"),
        ..Default::default()
    };

    eframe::run_native(
        "devhub",
        options,
        Box::new(move |cc| Ok(Box::new(app::DevHub::new(cc, config)))),
    )
    .map_err(|e| anyhow::anyhow!("eframe error: {e}"))?;

    Ok(())
}
