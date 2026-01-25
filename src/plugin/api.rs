//! Plugin API definitions

use crate::core::document::Document;

/// Context provided to plugins
#[derive(Debug, Clone)]
pub struct PluginContext {
    /// Plugin data directory
    pub data_dir: std::path::PathBuf,
    /// Current vault path
    pub vault_path: Option<std::path::PathBuf>,
}

impl Default for PluginContext {
    fn default() -> Self {
        Self {
            data_dir: std::path::PathBuf::from("plugins"),
            vault_path: None,
        }
    }
}

/// Plugin trait that all plugins must implement
pub trait Plugin: Send + Sync {
    /// Get the plugin name
    fn name(&self) -> &str;

    /// Get the plugin version
    fn version(&self) -> &str;

    /// Get the plugin description
    fn description(&self) -> &str {
        ""
    }

    /// Called when the plugin is loaded
    fn on_load(&mut self, ctx: &PluginContext);

    /// Called when the plugin is unloaded
    fn on_unload(&mut self) {}

    /// Called when a document is opened
    fn on_document_open(&mut self, _doc: &Document) {}

    /// Called when a document is saved
    fn on_document_save(&mut self, _doc: &Document) {}

    /// Called when a document is closed
    fn on_document_close(&mut self, _path: &std::path::Path) {}

    /// Handle a command from the user
    fn on_command(&mut self, _cmd: &str, _args: &[&str]) -> Option<String> {
        None
    }

    /// Get commands provided by this plugin
    fn commands(&self) -> Vec<PluginCommand> {
        Vec::new()
    }
}

/// A command provided by a plugin
#[derive(Debug, Clone)]
pub struct PluginCommand {
    /// Command name
    pub name: String,
    /// Command description
    pub description: String,
    /// Command usage
    pub usage: String,
}

impl PluginCommand {
    /// Create a new plugin command
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            usage: String::new(),
        }
    }

    /// Set command usage
    #[allow(dead_code)]
    pub fn with_usage(mut self, usage: impl Into<String>) -> Self {
        self.usage = usage.into();
        self
    }
}

/// Plugin metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginManifest {
    /// Plugin ID
    pub id: String,
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin description
    pub description: String,
    /// Plugin author
    pub author: String,
    /// Entry point (WASM file)
    pub entry_point: String,
    /// Required permissions
    pub permissions: Vec<PluginPermission>,
}

/// Plugin permissions
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginPermission {
    /// Read files in the vault
    ReadVault,
    /// Write files in the vault
    WriteVault,
    /// Execute shell commands
    Shell,
    /// Network access
    Network,
    /// Access to clipboard
    Clipboard,
}

/// Events that can be sent to plugins
#[derive(Debug, Clone)]
pub enum PluginEvent {
    /// A document was opened
    DocumentOpened(std::path::PathBuf),
    /// A document was saved
    DocumentSaved(std::path::PathBuf),
    /// A document was closed
    DocumentClosed(std::path::PathBuf),
    /// The vault was changed
    VaultChanged(Option<std::path::PathBuf>),
    /// A command was invoked
    Command { name: String, args: Vec<String> },
}
