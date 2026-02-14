use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export from core for consistency
pub use interceptor_core::{ActivityQuery, DashboardActivity};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HeaderPatch {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepeatRequest {
    pub id: u64,
    pub method: Option<String>,
    pub url: Option<String>,
    pub headers: Option<Vec<HeaderPatch>>,
    pub modified_body: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ManualRequest {
    pub method: String,
    pub url: String,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ManualResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub time_ms: u64,
    pub size_bytes: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub proxy: ProxyConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProxyConfig {
    pub port: u16,
    pub host: String,
    pub intercept_https: bool,
    pub http2: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UiConfig {
    pub theme: String,
    pub animations: bool,
    pub notifications: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub enabled: bool,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginToggle {
    pub enabled: bool,
}
