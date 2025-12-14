// Plugin system for INT3RCEPTOR
// Allows extending functionality with WASM plugins

pub mod config;
pub mod hooks;
pub mod host_functions;
pub mod manager;
pub mod runtime;

pub use config::{PluginConfig, PluginPermissions};
pub use hooks::{HookContext, HookResult, PluginHook};
pub use manager::PluginManager;
pub use runtime::PluginRuntime;

/// Plugin API version - plugins must match this version
pub const PLUGIN_API_VERSION: u32 = 1;

/// Maximum plugin execution time (5 seconds)
pub const MAX_EXECUTION_TIME_SECS: u64 = 5;

/// Maximum memory per plugin (10 MB)
pub const MAX_MEMORY_BYTES: usize = 10 * 1024 * 1024;
