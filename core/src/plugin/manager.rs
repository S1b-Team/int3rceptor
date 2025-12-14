use super::config::{PluginConfig, PluginSystemConfig};
use super::hooks::{HookContext, PluginHook};
use super::runtime::PluginRuntime;
use crate::error::{ProxyError, Result};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tracing::{info, warn};

/// Plugin manager - orchestrates all plugins
pub struct PluginManager {
    /// System configuration
    config: PluginSystemConfig,

    /// Loaded plugins (name -> runtime)
    plugins: Arc<RwLock<HashMap<String, Arc<PluginRuntime>>>>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new(config: PluginSystemConfig) -> Self {
        Self {
            config,
            plugins: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Load all configured plugins
    pub fn load_all(&self) -> Result<()> {
        if !self.config.enabled {
            info!("Plugin system is disabled");
            return Ok(());
        }

        info!(plugin_dir = ?self.config.plugin_dir, "Loading plugins");

        // Ensure plugin directory exists
        if !self.config.plugin_dir.exists() {
            fs::create_dir_all(&self.config.plugin_dir).map_err(|e| {
                ProxyError::internal(format!("Failed to create plugin directory: {}", e))
            })?;
            info!(dir = ?self.config.plugin_dir, "Created plugin directory");
        }

        let mut loaded_count = 0;
        let mut failed_count = 0;

        // Load each configured plugin
        for plugin_config in &self.config.plugins {
            if !plugin_config.enabled {
                info!(plugin = %plugin_config.name, "Plugin is disabled, skipping");
                continue;
            }

            match self.load_plugin(plugin_config.clone()) {
                Ok(_) => {
                    info!(plugin = %plugin_config.name, "Plugin loaded successfully");
                    loaded_count += 1;
                }
                Err(e) => {
                    warn!(plugin = %plugin_config.name, error = %e, "Failed to load plugin");
                    failed_count += 1;
                }
            }
        }

        info!(
            loaded = loaded_count,
            failed = failed_count,
            "Plugin loading complete"
        );
        Ok(())
    }

    /// Load a single plugin
    pub fn load_plugin(&self, config: PluginConfig) -> Result<()> {
        // Resolve plugin path relative to plugin_dir if not absolute
        let plugin_path = if config.path.is_absolute() {
            config.path.clone()
        } else {
            self.config.plugin_dir.join(&config.path)
        };

        // Check if file exists
        if !plugin_path.exists() {
            return Err(ProxyError::internal(format!(
                "Plugin file not found: {}",
                plugin_path.display()
            )));
        }

        // Create config with resolved path
        let mut resolved_config = config.clone();
        resolved_config.path = plugin_path;

        // Load the plugin runtime
        let runtime = Arc::new(PluginRuntime::load(resolved_config)?);

        // Initialize the plugin
        runtime.initialize()?;

        // Store the runtime
        let mut plugins = self
            .plugins
            .write()
            .map_err(|e| ProxyError::internal(format!("Failed to acquire plugin lock: {}", e)))?;

        plugins.insert(config.name.clone(), runtime);

        Ok(())
    }

    /// Unload a plugin by name
    pub fn unload_plugin(&self, name: &str) -> Result<()> {
        let mut plugins = self
            .plugins
            .write()
            .map_err(|e| ProxyError::internal(format!("Failed to acquire plugin lock: {}", e)))?;

        if plugins.remove(name).is_some() {
            info!(plugin = %name, "Plugin unloaded");
            Ok(())
        } else {
            Err(ProxyError::internal(format!("Plugin '{}' not found", name)))
        }
    }

    /// Reload a plugin (unload and load again)
    pub fn reload_plugin(&self, name: &str) -> Result<()> {
        // Find the config for this plugin
        let plugin_config = self
            .config
            .plugins
            .iter()
            .find(|p| p.name == name)
            .ok_or_else(|| ProxyError::internal(format!("Plugin '{}' not in configuration", name)))?
            .clone();

        // Unload if currently loaded
        self.unload_plugin(name).ok(); // Ignore error if not loaded

        // Reload
        self.load_plugin(plugin_config)?;

        info!(plugin = %name, "Plugin reloaded");
        Ok(())
    }

    /// Execute a hook across all enabled plugins
    pub fn execute_hook(&self, hook: PluginHook, mut context: HookContext) -> Result<HookContext> {
        let plugins = self
            .plugins
            .read()
            .map_err(|e| ProxyError::internal(format!("Failed to acquire plugin lock: {}", e)))?;

        if plugins.is_empty() {
            return Ok(context); // No plugins loaded
        }

        // Collect plugins and sort by priority
        let mut plugin_list: Vec<_> = plugins.values().cloned().collect();
        plugin_list.sort_by_key(|p| p.name().to_string()); // Could use priority field from config

        // Execute hook in each plugin
        for plugin_runtime in plugin_list {
            if !plugin_runtime.is_enabled() {
                continue;
            }

            match plugin_runtime.call_hook(hook, &context) {
                Ok(result) => {
                    if !result.should_continue {
                        warn!(
                            plugin = %plugin_runtime.name(),
                            hook = ?hook,
                            message = ?result.message,
                            "Plugin blocked further processing"
                        );
                        // Plugin wants to block - return immediately
                        return Ok(context);
                    }

                    if result.modified {
                        if let Some(new_context) = result.context {
                            info!(
                                plugin = %plugin_runtime.name(),
                                hook = ?hook,
                                "Plugin modified context"
                            );
                            context = new_context;
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        plugin = %plugin_runtime.name(),
                        hook = ?hook,
                        error = %e,
                        "Plugin hook failed"
                    );
                    // Continue with other plugins even if one fails
                }
            }
        }

        Ok(context)
    }

    /// Get list of loaded plugins
    pub fn list_plugins(&self) -> Vec<String> {
        self.plugins
            .read()
            .map(|plugins| plugins.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Check if a plugin is loaded
    pub fn is_loaded(&self, name: &str) -> bool {
        self.plugins
            .read()
            .map(|plugins| plugins.contains_key(name))
            .unwrap_or(false)
    }

    /// Get plugin count
    pub fn count(&self) -> usize {
        self.plugins
            .read()
            .map(|plugins| plugins.len())
            .unwrap_or(0)
    }

    /// Discover plugins in the plugin directory
    /// Returns a list of .wasm files found
    pub fn discover_plugins(&self) -> Result<Vec<PathBuf>> {
        if !self.config.plugin_dir.exists() {
            return Ok(Vec::new());
        }

        let mut plugins = Vec::new();

        for entry in fs::read_dir(&self.config.plugin_dir)
            .map_err(|e| ProxyError::internal(format!("Failed to read plugin directory: {}", e)))?
        {
            let entry = entry.map_err(|e| {
                ProxyError::internal(format!("Failed to read directory entry: {}", e))
            })?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                plugins.push(path);
            }
        }

        Ok(plugins)
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new(PluginSystemConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_manager_creation() {
        let config = PluginSystemConfig::default();
        let manager = PluginManager::new(config);

        assert_eq!(manager.count(), 0);
        assert!(!manager.is_loaded("test"));
    }

    #[test]
    fn test_list_plugins() {
        let mut config = PluginSystemConfig::default();
        config.enabled = false; // Don't actually load

        let manager = PluginManager::new(config);
        let plugins = manager.list_plugins();

        assert_eq!(plugins.len(), 0);
    }
}
