use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Name of the plugin
    pub name: String,

    /// Path to the .wasm file
    pub path: PathBuf,

    /// Whether this plugin is enabled
    pub enabled: bool,

    /// Plugin permissions
    pub permissions: PluginPermissions,

    /// Maximum execution time
    #[serde(default = "default_max_execution_time")]
    pub max_execution_time: Duration,

    /// Maximum memory usage in bytes
    #[serde(default = "default_max_memory")]
    pub max_memory_bytes: usize,

    /// Plugin priority (lower = higher priority)
    #[serde(default)]
    pub priority: u32,
}

fn default_max_execution_time() -> Duration {
    Duration::from_secs(super::MAX_EXECUTION_TIME_SECS)
}

fn default_max_memory() -> usize {
    super::MAX_MEMORY_BYTES
}

/// Plugin permissions control what a plugin can do
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPermissions {
    /// Can make network requests
    #[serde(default)]
    pub can_make_network_requests: bool,

    /// Can access filesystem (limited WASI access)
    #[serde(default)]
    pub can_access_filesystem: bool,

    /// Can modify HTTP requests
    #[serde(default = "default_true")]
    pub can_modify_requests: bool,

    /// Can modify HTTP responses
    #[serde(default = "default_true")]
    pub can_modify_responses: bool,

    /// Can access request/response bodies
    #[serde(default = "default_true")]
    pub can_access_bodies: bool,
}

fn default_true() -> bool {
    true
}

impl Default for PluginPermissions {
    fn default() -> Self {
        Self {
            can_make_network_requests: false,
            can_access_filesystem: false,
            can_modify_requests: true,
            can_modify_responses: true,
            can_access_bodies: true,
        }
    }
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            path: PathBuf::new(),
            enabled: true,
            permissions: PluginPermissions::default(),
            max_execution_time: default_max_execution_time(),
            max_memory_bytes: default_max_memory(),
            priority: 100,
        }
    }
}

/// Global plugin system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSystemConfig {
    /// Directory containing plugins
    pub plugin_dir: PathBuf,

    /// Whether the plugin system is enabled
    pub enabled: bool,

    /// List of configured plugins
    pub plugins: Vec<PluginConfig>,
}

impl Default for PluginSystemConfig {
    fn default() -> Self {
        Self {
            plugin_dir: PathBuf::from("plugins"),
            enabled: true,
            plugins: Vec::new(),
        }
    }
}
