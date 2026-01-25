//! Main application state and UI coordination

use std::collections::HashMap;
use std::path::PathBuf;

use eframe::egui;

use crate::core::{config::AppConfig, document::Document, file_system::FileTree};
use crate::plugin::manager::PluginManager;
use crate::terminal::TerminalState;
use crate::ui::{editor::EditorPanel, file_tree::FileTreePanel, preview::PreviewPanel, terminal::TerminalPanel};

/// View mode for the editor area
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    #[default]
    Editor,
    Preview,
    Split,
}

/// Main application state
pub struct RobsidianApp {
    /// Path to the current vault (workspace)
    pub vault_path: Option<PathBuf>,
    /// Open documents indexed by path
    pub documents: HashMap<PathBuf, Document>,
    /// Currently active document path
    pub active_document: Option<PathBuf>,
    /// File tree state
    pub file_tree: FileTree,
    /// Terminal state
    pub terminal: TerminalState,
    /// Plugin manager
    pub plugin_manager: PluginManager,
    /// Application configuration
    pub config: AppConfig,
    /// Current view mode
    pub view_mode: ViewMode,
    /// Whether sidebar is visible
    pub sidebar_visible: bool,
    /// Whether terminal panel is visible
    pub terminal_visible: bool,
    /// Commonmark cache for preview
    pub commonmark_cache: egui_commonmark::CommonMarkCache,
}

impl RobsidianApp {
    /// Create a new application instance
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Configure fonts and styles
        Self::configure_fonts(&cc.egui_ctx);

        // Load config or use defaults
        let config = AppConfig::load().unwrap_or_default();

        // Load last vault if configured
        let vault_path = config.last_vault.clone();
        let file_tree = if let Some(ref path) = vault_path {
            FileTree::from_path(path).unwrap_or_default()
        } else {
            FileTree::default()
        };

        Self {
            vault_path,
            documents: HashMap::new(),
            active_document: None,
            file_tree,
            terminal: TerminalState::new(),
            plugin_manager: PluginManager::new(),
            config,
            view_mode: ViewMode::Split,
            sidebar_visible: true,
            terminal_visible: false,
            commonmark_cache: egui_commonmark::CommonMarkCache::default(),
        }
    }

    /// Configure custom fonts
    fn configure_fonts(_ctx: &egui::Context) {
        // Use default fonts for now
        // Custom fonts can be added by placing font files in assets/fonts/
        // and using include_bytes! to embed them
        //
        // Example:
        // let mut fonts = egui::FontDefinitions::default();
        // fonts.font_data.insert(
        //     "custom".to_owned(),
        //     egui::FontData::from_static(include_bytes!("../assets/fonts/Font.ttf")),
        // );
        // fonts.families
        //     .entry(egui::FontFamily::Monospace)
        //     .or_default()
        //     .insert(0, "custom".to_owned());
        // ctx.set_fonts(fonts);
    }

    /// Open a vault (workspace directory)
    pub fn open_vault(&mut self, path: PathBuf) {
        self.vault_path = Some(path.clone());
        self.file_tree = FileTree::from_path(&path).unwrap_or_default();
        self.config.last_vault = Some(path);
        let _ = self.config.save();
    }

    /// Open a document
    pub fn open_document(&mut self, path: PathBuf) {
        if !self.documents.contains_key(&path) {
            match Document::open(&path) {
                Ok(doc) => {
                    // Notify plugins
                    self.plugin_manager.on_document_open(&doc);
                    self.documents.insert(path.clone(), doc);
                }
                Err(e) => {
                    tracing::error!("Failed to open document: {}", e);
                    return;
                }
            }
        }
        self.active_document = Some(path);
    }

    /// Save the active document
    pub fn save_active_document(&mut self) {
        if let Some(ref path) = self.active_document {
            if let Some(doc) = self.documents.get(path) {
                if let Err(e) = doc.save() {
                    tracing::error!("Failed to save document: {}", e);
                }
            }
        }
    }

    /// Get the active document mutably
    pub fn active_document_mut(&mut self) -> Option<&mut Document> {
        self.active_document
            .as_ref()
            .and_then(|path| self.documents.get_mut(path))
    }

    /// Get the active document
    pub fn active_document(&self) -> Option<&Document> {
        self.active_document
            .as_ref()
            .and_then(|path| self.documents.get(path))
    }

    /// Render the top menu bar
    fn render_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open Vault...").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.open_vault(path);
                        }
                        ui.close();
                    }
                    if ui.button("Save").clicked() {
                        self.save_active_document();
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("View", |ui| {
                    if ui.button("Toggle Sidebar").clicked() {
                        self.sidebar_visible = !self.sidebar_visible;
                        ui.close();
                    }
                    if ui.button("Toggle Terminal").clicked() {
                        self.terminal_visible = !self.terminal_visible;
                        ui.close();
                    }
                    ui.separator();
                    if ui.selectable_label(self.view_mode == ViewMode::Editor, "Editor Only").clicked() {
                        self.view_mode = ViewMode::Editor;
                        ui.close();
                    }
                    if ui.selectable_label(self.view_mode == ViewMode::Preview, "Preview Only").clicked() {
                        self.view_mode = ViewMode::Preview;
                        ui.close();
                    }
                    if ui.selectable_label(self.view_mode == ViewMode::Split, "Split View").clicked() {
                        self.view_mode = ViewMode::Split;
                        ui.close();
                    }
                });

                ui.menu_button("Plugins", |ui| {
                    if ui.button("Manage Plugins...").clicked() {
                        // TODO: Open plugin manager dialog
                        ui.close();
                    }
                });
            });
        });
    }
}

impl eframe::App for RobsidianApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle keyboard shortcuts
        ctx.input(|i| {
            if i.modifiers.ctrl && i.key_pressed(egui::Key::S) {
                self.save_active_document();
            }
            if i.modifiers.ctrl && i.key_pressed(egui::Key::B) {
                self.sidebar_visible = !self.sidebar_visible;
            }
            if i.modifiers.ctrl && i.key_pressed(egui::Key::Backtick) {
                self.terminal_visible = !self.terminal_visible;
            }
        });

        // Render menu bar
        self.render_menu_bar(ctx);

        // Render sidebar with file tree
        if self.sidebar_visible {
            egui::SidePanel::left("sidebar")
                .resizable(true)
                .default_width(250.0)
                .min_width(150.0)
                .show(ctx, |ui| {
                    FileTreePanel::show(ui, self);
                });
        }

        // Render terminal panel at bottom
        if self.terminal_visible {
            egui::TopBottomPanel::bottom("terminal_panel")
                .resizable(true)
                .default_height(200.0)
                .min_height(100.0)
                .show(ctx, |ui| {
                    TerminalPanel::show(ui, &mut self.terminal);
                });
        }

        // Render main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.view_mode {
                ViewMode::Editor => {
                    EditorPanel::show(ui, self);
                }
                ViewMode::Preview => {
                    PreviewPanel::show(ui, self);
                }
                ViewMode::Split => {
                    // Split view: editor on left, preview on right
                    let available_width = ui.available_width();
                    ui.horizontal(|ui| {
                        ui.set_min_width(available_width);

                        // Editor panel
                        ui.vertical(|ui| {
                            ui.set_width(available_width / 2.0 - 4.0);
                            EditorPanel::show(ui, self);
                        });

                        ui.separator();

                        // Preview panel
                        ui.vertical(|ui| {
                            ui.set_width(available_width / 2.0 - 4.0);
                            PreviewPanel::show(ui, self);
                        });
                    });
                }
            }
        });
    }
}
