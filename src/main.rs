//! Robsidian - Obsidian-like markdown note application
//!
//! A Rust-based markdown editor with file explorer, terminal, and plugin support.

mod app;
mod core;
mod plugin;
mod terminal;
mod ui;

use app::RobsidianApp;
use eframe::egui;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> eframe::Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::filter::LevelFilter::INFO)
        .init();

    tracing::info!("Starting Robsidian...");

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Robsidian"),
        ..Default::default()
    };

    eframe::run_native(
        "Robsidian",
        native_options,
        Box::new(|cc| Ok(Box::new(RobsidianApp::new(cc)))),
    )
}
