use super::config::PluginConfig;
use super::hooks::{HookContext, HookResult, PluginHook};
use super::host_functions::{define_host_functions, HostContext};
use crate::error::{ProxyError, Result};
use std::time::Instant;
use wasmtime::*;

/// WASM plugin runtime
pub struct PluginRuntime {
    /// Plugin configuration
    config: PluginConfig,

    /// WASM engine
    engine: Engine,

    /// Compiled module
    module: Module,

    /// Linker with host functions
    linker: Linker<HostContext>,
}

impl PluginRuntime {
    /// Load a plugin from a file
    pub fn load(config: PluginConfig) -> Result<Self> {
        // Create WASM engine with resource limits
        let mut wasm_config = Config::new();
        wasm_config.consume_fuel(true); // Enable fuel metering for time limits

        let engine = Engine::new(&wasm_config)
            .map_err(|e| ProxyError::internal(format!("Failed to create WASM engine: {}", e)))?;

        // Load and compile the module
        let module = Module::from_file(&engine, &config.path).map_err(|e| {
            ProxyError::internal(format!("Failed to load plugin '{}': {}", config.name, e))
        })?;

        // Create linker and add host functions
        let mut linker = Linker::new(&engine);
        define_host_functions(&mut linker)
            .map_err(|e| ProxyError::internal(format!("Failed to define host functions: {}", e)))?;

        tracing::info!(plugin = %config.name, path = ?config.path, "Plugin loaded successfully");

        Ok(Self {
            config,
            engine,
            module,
            linker,
        })
    }

    /// Call a hook function in the plugin
    /// This is a simplified version that returns unmodified context for now
    pub fn call_hook(&self, hook: PluginHook, context: &HookContext) -> Result<HookResult> {
        let start = Instant::now();

        // Create store with host context initialized with current request data
        let host_ctx = HostContext::with_context(self.config.name.clone(), context.clone());
        // Keep a reference to the shared context to retrieve modifications later
        let shared_context = host_ctx.context.clone();

        let mut store = Store::new(&self.engine, host_ctx);

        // Set fuel limit (computational budget)
        store
            .set_fuel(1_000_000) // 1M instructions
            .map_err(|e| ProxyError::internal(format!("Failed to set fuel: {}", e)))?;

        // Instantiate the module
        let instance = self
            .linker
            .instantiate(&mut store, &self.module)
            .map_err(|e| {
                ProxyError::internal(format!(
                    "Failed to instantiate plugin '{}': {}",
                    self.config.name, e
                ))
            })?;

        // Try to get the hook function
        let hook_name = hook.as_str();

        // For now, if the function exists, we just call it without arguments
        if let Some(func) = instance.get_func(&mut store, hook_name) {
            // Try to call as a no-arg function returning i32
            if let Ok(typed_func) = func.typed::<(), i32>(&store) {
                match typed_func.call(&mut store, ()) {
                    Ok(result) => {
                        tracing::debug!(
                            plugin = %self.config.name,
                            hook = %hook_name,
                            result = result,
                            "Hook executed"
                        );

                        // Check if context was modified
                        if let Ok(guard) = shared_context.read() {
                            if let Some(modified_ctx) = guard.as_ref() {
                                // Simple check: if we have a context here, we assume it might be modified
                                // Ideally we would compare with original, but for now we return it
                                return Ok(HookResult::modified(modified_ctx.clone()));
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!(
                            plugin = %self.config.name,
                            hook = %hook_name,
                            error = %e,
                            "Hook execution failed"
                        );
                    }
                }
            }
        }

        // Check execution time
        let elapsed = start.elapsed();
        if elapsed > self.config.max_execution_time {
            return Err(ProxyError::internal(format!(
                "Plugin '{}' exceeded time limit ({:?} > {:?})",
                self.config.name, elapsed, self.config.max_execution_time
            )));
        }

        Ok(HookResult::unmodified())
    }

    /// Initialize the plugin
    pub fn initialize(&self) -> Result<()> {
        let host_ctx = HostContext::new(self.config.name.clone());
        let mut store = Store::new(&self.engine, host_ctx);

        let instance = self
            .linker
            .instantiate(&mut store, &self.module)
            .map_err(|e| ProxyError::internal(format!("Failed to instantiate: {}", e)))?;

        // Call plugin_init if it exists
        if let Some(init_func) = instance.get_func(&mut store, "plugin_init") {
            if let Ok(typed_func) = init_func.typed::<(), i32>(&store) {
                let result = typed_func
                    .call(&mut store, ())
                    .map_err(|e| ProxyError::internal(format!("plugin_init failed: {}", e)))?;

                if result != 0 {
                    return Err(ProxyError::internal(format!(
                        "Plugin initialization failed with code: {}",
                        result
                    )));
                }

                tracing::info!(plugin = %self.config.name, "Plugin initialized");
            }
        }

        Ok(())
    }

    /// Get plugin name
    pub fn name(&self) -> &str {
        &self.config.name
    }

    /// Check if plugin is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_plugin_config() {
        let config = PluginConfig {
            name: "test".to_string(),
            path: PathBuf::from("test.wasm"),
            enabled: true,
            ..Default::default()
        };

        assert_eq!(config.name, "test");
        assert!(config.enabled);
    }
}
