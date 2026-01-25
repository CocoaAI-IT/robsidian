//! Markdown preview panel using egui_commonmark

use eframe::egui;
use egui_commonmark::CommonMarkViewer;

use crate::app::RobsidianApp;

/// Markdown preview panel
pub struct PreviewPanel;

impl PreviewPanel {
    /// Show the preview panel
    pub fn show(ui: &mut egui::Ui, app: &mut RobsidianApp) {
        // Get content first to avoid borrow conflicts
        let content = app
            .active_document()
            .map(|doc| doc.content_without_frontmatter().to_string());

        egui::ScrollArea::vertical()
            .id_salt("preview_scroll")
            .show(ui, |ui| {
                if let Some(content) = content {
                    CommonMarkViewer::new()
                        .show(ui, &mut app.commonmark_cache, &content);
                } else {
                    Self::show_empty(ui);
                }
            });
    }

    /// Show empty state
    fn show_empty(ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.label("No document selected");
            ui.label("Open a markdown file to see the preview");
        });
    }
}
