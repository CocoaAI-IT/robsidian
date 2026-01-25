//! Sidebar component containing file tree and other panels

use eframe::egui;

use crate::app::RobsidianApp;
use super::file_tree::FileTreePanel;

/// Sidebar with file tree and additional panels
pub struct Sidebar;

impl Sidebar {
    /// Show the sidebar
    #[allow(dead_code)]
    pub fn show(ui: &mut egui::Ui, app: &mut RobsidianApp) {
        ui.vertical(|ui| {
            // File tree takes most of the space
            FileTreePanel::show(ui, app);

            ui.separator();

            // Quick actions
            ui.collapsing("Quick Actions", |ui| {
                if ui.button("New Note").clicked() {
                    // TODO: Create new note
                }
                if ui.button("Daily Note").clicked() {
                    // TODO: Create/open daily note
                }
                if ui.button("Search...").clicked() {
                    // TODO: Open search dialog
                }
            });

            // Recent files
            if !app.documents.is_empty() {
                ui.collapsing("Open Files", |ui| {
                    let paths: Vec<_> = app.documents.keys().cloned().collect();
                    for path in paths {
                        if let Some(file_name) = path.file_name() {
                            if ui.button(file_name.to_string_lossy()).clicked() {
                                app.active_document = Some(path);
                            }
                        }
                    }
                });
            }
        });
    }
}
