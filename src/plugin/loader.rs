//! Plugin loader for WASM plugins

use std::path::Path;

use anyhow::Result;

use super::api::PluginManifest;

/// Plugin loader for loading WASM plugins
pub struct PluginLoader {
    /// Wasmtime engine
    #[allow(dead_code)]
    engine: wasmtime::Engine,
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginLoader {
    /// Create a new plugin loader
    pub fn new() -> Self {
        let engine = wasmtime::Engine::default();
        Self { engine }
    }

    /// Load a plugin manifest from a directory
    pub fn load_manifest(&self, plugin_dir: &Path) -> Result<PluginManifest> {
        let manifest_path = plugin_dir.join("manifest.json");
        let content = std::fs::read_to_string(&manifest_path)?;
        let manifest: PluginManifest = serde_json::from_str(&content)?;
        Ok(manifest)
    }

    /// Load a WASM plugin
    pub fn load_plugin(&self, plugin_dir: &Path) -> Result<LoadedPlugin> {
        let manifest = self.load_manifest(plugin_dir)?;
        let wasm_path = plugin_dir.join(&manifest.entry_point);

        // Read WASM bytes
        let wasm_bytes = std::fs::read(&wasm_path)?;

        // Compile the module
        let module = wasmtime::Module::new(&self.engine, &wasm_bytes)?;

        // Create store and instance
        let mut store = wasmtime::Store::new(&self.engine, ());
        let instance = wasmtime::Instance::new(&mut store, &module, &[])?;

        Ok(LoadedPlugin {
            manifest,
            _module: module,
            _instance: instance,
            _store: store,
        })
    }

    /// Discover plugins in a directory
    pub fn discover_plugins(&self, plugins_dir: &Path) -> Vec<PluginManifest> {
        let mut manifests = Vec::new();

        if let Ok(entries) = std::fs::read_dir(plugins_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Ok(manifest) = self.load_manifest(&path) {
                        manifests.push(manifest);
                    }
                }
            }
        }

        manifests
    }
}

/// A loaded WASM plugin
pub struct LoadedPlugin {
    /// Plugin manifest
    pub manifest: PluginManifest,
    /// Compiled WASM module
    _module: wasmtime::Module,
    /// WASM instance
    _instance: wasmtime::Instance,
    /// WASM store
    _store: wasmtime::Store<()>,
}

impl LoadedPlugin {
    /// Get the plugin ID
    pub fn id(&self) -> &str {
        &self.manifest.id
    }

    /// Get the plugin name
    pub fn name(&self) -> &str {
        &self.manifest.name
    }

    /// Get the plugin version
    pub fn version(&self) -> &str {
        &self.manifest.version
    }

    /// Call a function in the plugin
    #[allow(dead_code)]
    pub fn call(&mut self, _func_name: &str, _args: &[wasmtime::Val]) -> Result<Vec<wasmtime::Val>> {
        // TODO: Implement function calls
        // This requires proper WIT bindings to be implemented
        Ok(Vec::new())
    }
}
