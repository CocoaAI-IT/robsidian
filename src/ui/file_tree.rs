//! File tree panel for vault navigation

use std::path::PathBuf;

use eframe::egui;

use crate::app::RobsidianApp;
use crate::core::file_system::FileNode;

/// File tree panel
pub struct FileTreePanel;

impl FileTreePanel {
    /// Show the file tree panel
    pub fn show(ui: &mut egui::Ui, app: &mut RobsidianApp) {
        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.heading("Explorer");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("\u{21BB}").on_hover_text("Refresh").clicked() {
                        let _ = app.file_tree.refresh();
                    }
                    if ui.button("+").on_hover_text("New file").clicked() {
                        // TODO: Create new file dialog
                    }
                });
            });

            ui.separator();

            // File tree
            egui::ScrollArea::vertical()
                .id_salt("file_tree_scroll")
                .show(ui, |ui| {
                    if let Some(ref root) = app.file_tree.root.clone() {
                        Self::show_node(ui, root, app);
                    } else {
                        ui.label("No vault open");
                        ui.add_space(10.0);
                        if ui.button("Open Vault...").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                app.open_vault(path);
                            }
                        }
                    }
                });
        });
    }

    /// Recursively show a file tree node
    fn show_node(ui: &mut egui::Ui, node: &FileNode, app: &mut RobsidianApp) {
        if node.is_dir {
            Self::show_directory(ui, node, app);
        } else {
            Self::show_file(ui, node, app);
        }
    }

    /// Show a directory node
    fn show_directory(ui: &mut egui::Ui, node: &FileNode, app: &mut RobsidianApp) {
        let id = ui.make_persistent_id(&node.path);

        egui::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(),
            id,
            node.expanded,
        )
        .show_header(ui, |ui| {
            let icon = if node.expanded { "\u{1F4C2}" } else { "\u{1F4C1}" };
            if ui
                .selectable_label(false, format!("{} {}", icon, node.name))
                .clicked()
            {
                app.file_tree.toggle_expanded(&node.path);
            }
        })
        .body(|ui| {
            for child in &node.children {
                Self::show_node(ui, child, app);
            }
        });
    }

    /// Show a file node
    fn show_file(ui: &mut egui::Ui, node: &FileNode, app: &mut RobsidianApp) {
        let icon = if node.is_markdown() {
            "\u{1F4DD}"
        } else {
            "\u{1F4C4}"
        };

        let is_active = app.active_document.as_ref() == Some(&node.path);

        // Check if document is modified
        let display_name = if is_active {
            if let Some(doc) = app.documents.get(&node.path) {
                if doc.modified {
                    format!("{} {}*", icon, node.name)
                } else {
                    format!("{} {}", icon, node.name)
                }
            } else {
                format!("{} {}", icon, node.name)
            }
        } else {
            format!("{} {}", icon, node.name)
        };

        ui.horizontal(|ui| {
            ui.add_space(16.0); // Indent for files
            if ui.selectable_label(is_active, display_name).clicked() {
                app.open_document(node.path.clone());
            }
        });
    }
}

/// Dialog for creating a new file
pub struct NewFileDialog {
    pub visible: bool,
    pub file_name: String,
    pub parent_path: Option<PathBuf>,
}

impl Default for NewFileDialog {
    fn default() -> Self {
        Self {
            visible: false,
            file_name: String::new(),
            parent_path: None,
        }
    }
}

impl NewFileDialog {
    #[allow(dead_code)]
    pub fn show(&mut self, ctx: &egui::Context) -> Option<PathBuf> {
        let mut result = None;

        if self.visible {
            egui::Window::new("New File")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("File name:");
                        ui.text_edit_singleline(&mut self.file_name);
                    });

                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.visible = false;
                            self.file_name.clear();
                        }
                        if ui.button("Create").clicked() {
                            if !self.file_name.is_empty() {
                                if let Some(ref parent) = self.parent_path {
                                    let mut path = parent.clone();
                                    let name = if self.file_name.ends_with(".md") {
                                        self.file_name.clone()
                                    } else {
                                        format!("{}.md", self.file_name)
                                    };
                                    path.push(name);
                                    result = Some(path);
                                }
                            }
                            self.visible = false;
                            self.file_name.clear();
                        }
                    });
                });
        }

        result
    }
}
