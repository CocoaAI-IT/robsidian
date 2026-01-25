//! Plugin manager for loading and managing plugins

use std::collections::HashMap;
use std::path::PathBuf;

use super::api::{PluginContext, PluginManifest};
use super::loader::{LoadedPlugin, PluginLoader};
use crate::core::document::Document;

/// Plugin manager
pub struct PluginManager {
    /// Plugin loader
    loader: PluginLoader,
    /// Loaded plugins
    plugins: HashMap<String, LoadedPlugin>,
    /// Plugin context
    context: PluginContext,
    /// Available plugin manifests
    available_plugins: Vec<PluginManifest>,
    /// Enabled plugin IDs
    enabled_plugins: Vec<String>,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            loader: PluginLoader::new(),
            plugins: HashMap::new(),
            context: PluginContext::default(),
            available_plugins: Vec::new(),
            enabled_plugins: Vec::new(),
        }
    }

    /// Set the plugin context
    #[allow(dead_code)]
    pub fn set_context(&mut self, context: PluginContext) {
        self.context = context;
    }

    /// Discover available plugins
    pub fn discover(&mut self, plugins_dir: &PathBuf) {
        self.available_plugins = self.loader.discover_plugins(plugins_dir);
        tracing::info!("Discovered {} plugins", self.available_plugins.len());
    }

    /// Get available plugins
    #[allow(dead_code)]
    pub fn available_plugins(&self) -> &[PluginManifest] {
        &self.available_plugins
    }

    /// Enable a plugin
    #[allow(dead_code)]
    pub fn enable_plugin(&mut self, id: &str, plugins_dir: &PathBuf) -> Result<(), String> {
        if self.plugins.contains_key(id) {
            return Ok(());
        }

        let plugin_dir = plugins_dir.join(id);
        match self.loader.load_plugin(&plugin_dir) {
            Ok(plugin) => {
                tracing::info!("Loaded plugin: {} v{}", plugin.name(), plugin.version());
                self.enabled_plugins.push(id.to_string());
                self.plugins.insert(id.to_string(), plugin);
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to load plugin {}: {}", id, e);
                Err(format!("Failed to load plugin: {}", e))
            }
        }
    }

    /// Disable a plugin
    #[allow(dead_code)]
    pub fn disable_plugin(&mut self, id: &str) {
        self.plugins.remove(id);
        self.enabled_plugins.retain(|p| p != id);
        tracing::info!("Disabled plugin: {}", id);
    }

    /// Get enabled plugin IDs
    #[allow(dead_code)]
    pub fn enabled_plugins(&self) -> &[String] {
        &self.enabled_plugins
    }

    /// Notify plugins that a document was opened
    pub fn on_document_open(&mut self, _doc: &Document) {
        // TODO: Call plugin hooks
        for (id, _plugin) in &mut self.plugins {
            tracing::debug!("Notifying plugin {} of document open", id);
        }
    }

    /// Notify plugins that a document was saved
    #[allow(dead_code)]
    pub fn on_document_save(&mut self, _doc: &Document) {
        // TODO: Call plugin hooks
        for (id, _plugin) in &mut self.plugins {
            tracing::debug!("Notifying plugin {} of document save", id);
        }
    }

    /// Execute a plugin command
    #[allow(dead_code)]
    pub fn execute_command(&mut self, plugin_id: &str, command: &str, args: &[&str]) -> Option<String> {
        if let Some(_plugin) = self.plugins.get_mut(plugin_id) {
            // TODO: Execute command in plugin
            tracing::debug!("Executing command {} in plugin {}", command, plugin_id);
            Some(format!("Command '{}' executed with args: {:?}", command, args))
        } else {
            None
        }
    }

    /// Get plugin count
    #[allow(dead_code)]
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_manager_new() {
        let manager = PluginManager::new();
        assert_eq!(manager.plugin_count(), 0);
    }
}
