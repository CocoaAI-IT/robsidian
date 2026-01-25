//! Application configuration management

use std::path::PathBuf;

use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Last opened vault path
    pub last_vault: Option<PathBuf>,
    /// Recent vaults
    pub recent_vaults: Vec<PathBuf>,
    /// Editor settings
    pub editor: EditorConfig,
    /// UI settings
    pub ui: UiConfig,
    /// Plugin settings
    pub plugins: PluginConfig,
}

/// Editor-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    /// Font size in pixels
    pub font_size: f32,
    /// Tab size in spaces
    pub tab_size: usize,
    /// Use soft tabs (spaces instead of tabs)
    pub soft_tabs: bool,
    /// Word wrap
    pub word_wrap: bool,
    /// Auto-save interval in seconds (0 = disabled)
    pub auto_save_interval: u64,
    /// Show line numbers
    pub show_line_numbers: bool,
}

/// UI settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Theme (light/dark)
    pub theme: String,
    /// Sidebar width
    pub sidebar_width: f32,
    /// Terminal height
    pub terminal_height: f32,
}

/// Plugin settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Plugin directory
    pub plugin_dir: Option<PathBuf>,
    /// Enabled plugins
    pub enabled_plugins: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            last_vault: None,
            recent_vaults: Vec::new(),
            editor: EditorConfig::default(),
            ui: UiConfig::default(),
            plugins: PluginConfig::default(),
        }
    }
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            font_size: 14.0,
            tab_size: 4,
            soft_tabs: true,
            word_wrap: true,
            auto_save_interval: 0,
            show_line_numbers: true,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            sidebar_width: 250.0,
            terminal_height: 200.0,
        }
    }
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            plugin_dir: None,
            enabled_plugins: Vec::new(),
        }
    }
}

impl AppConfig {
    /// Get the config file path
    fn config_path() -> Option<PathBuf> {
        ProjectDirs::from("com", "robsidian", "Robsidian")
            .map(|dirs| dirs.config_dir().join("config.json"))
    }

    /// Load configuration from disk
    pub fn load() -> Result<Self> {
        let path = Self::config_path()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&path)?;
        let config: Self = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to disk
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

        // Ensure config directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;

        tracing::info!("Saved config to: {}", path.display());
        Ok(())
    }

    /// Add a vault to recent vaults
    pub fn add_recent_vault(&mut self, path: PathBuf) {
        // Remove if already exists
        self.recent_vaults.retain(|p| p != &path);
        // Add to front
        self.recent_vaults.insert(0, path);
        // Keep only last 10
        self.recent_vaults.truncate(10);
    }

    /// Get the plugin directory
    pub fn get_plugin_dir(&self) -> PathBuf {
        self.plugins.plugin_dir.clone().unwrap_or_else(|| {
            ProjectDirs::from("com", "robsidian", "Robsidian")
                .map(|dirs| dirs.data_dir().join("plugins"))
                .unwrap_or_else(|| PathBuf::from("plugins"))
        })
    }
}
