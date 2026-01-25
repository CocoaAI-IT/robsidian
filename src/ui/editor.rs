//! Markdown editor panel

use eframe::egui;

use crate::app::RobsidianApp;

/// Markdown editor panel
pub struct EditorPanel;

impl EditorPanel {
    /// Show the editor panel
    pub fn show(ui: &mut egui::Ui, app: &mut RobsidianApp) {
        ui.vertical(|ui| {
            // Document tabs (if multiple documents open)
            if app.documents.len() > 1 {
                Self::show_tabs(ui, app);
                ui.separator();
            }

            // Editor area
            egui::ScrollArea::vertical()
                .id_salt("editor_scroll")
                .show(ui, |ui| {
                    if let Some(path) = app.active_document.clone() {
                        if let Some(doc) = app.documents.get_mut(&path) {
                            let response = egui::TextEdit::multiline(&mut doc.content)
                                .font(egui::TextStyle::Monospace)
                                .code_editor()
                                .desired_width(f32::INFINITY)
                                .desired_rows(30)
                                .show(ui);

                            if response.response.changed() {
                                doc.modified = true;
                            }
                        }
                    } else {
                        Self::show_welcome(ui);
                    }
                });
        });
    }

    /// Show document tabs
    fn show_tabs(ui: &mut egui::Ui, app: &mut RobsidianApp) {
        ui.horizontal(|ui| {
            let mut paths_to_show: Vec<_> = app.documents.keys().cloned().collect();
            paths_to_show.sort();

            for path in paths_to_show {
                let doc = app.documents.get(&path).unwrap();
                let title = if doc.modified {
                    format!("{}*", doc.title())
                } else {
                    doc.title()
                };

                let is_active = app.active_document.as_ref() == Some(&path);
                if ui.selectable_label(is_active, title).clicked() {
                    app.active_document = Some(path.clone());
                }
            }
        });
    }

    /// Show welcome screen when no document is open
    fn show_welcome(ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);

            ui.heading("Welcome to Robsidian");
            ui.add_space(20.0);

            ui.label("Open a vault or create a new document to get started.");
            ui.add_space(10.0);

            ui.label("Keyboard shortcuts:");
            ui.label("  Ctrl+S - Save");
            ui.label("  Ctrl+B - Toggle sidebar");
            ui.label("  Ctrl+` - Toggle terminal");
        });
    }
}

/// Simple syntax highlighting for markdown
pub struct MarkdownHighlighter;

impl MarkdownHighlighter {
    /// Apply basic markdown styling to text
    #[allow(dead_code)]
    pub fn highlight_line(line: &str) -> Vec<(String, egui::Color32)> {
        let mut result = Vec::new();

        // Headers
        if line.starts_with("# ") {
            result.push((line.to_string(), egui::Color32::from_rgb(129, 162, 190)));
            return result;
        }
        if line.starts_with("## ") || line.starts_with("### ") {
            result.push((line.to_string(), egui::Color32::from_rgb(129, 162, 190)));
            return result;
        }

        // Code blocks
        if line.starts_with("```") {
            result.push((line.to_string(), egui::Color32::from_rgb(152, 195, 121)));
            return result;
        }

        // Lists
        if line.starts_with("- ") || line.starts_with("* ") {
            result.push((line.to_string(), egui::Color32::from_rgb(224, 108, 117)));
            return result;
        }

        // Normal text
        result.push((line.to_string(), egui::Color32::from_rgb(171, 178, 191)));
        result
    }
}
