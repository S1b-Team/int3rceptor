use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Available plugin hooks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PluginHook {
    /// Called when a new request is received (before processing)
    OnRequest,

    /// Called when a response is received from upstream (before returning to client)
    OnResponse,

    /// Called when a new connection is established
    OnConnect,

    /// Called when traffic is captured and stored
    OnCapture,

    /// Called when a rule matches
    OnRuleMatch,
}

impl PluginHook {
    /// Get the string name of this hook
    pub fn as_str(&self) -> &'static str {
        match self {
            PluginHook::OnRequest => "on_request",
            PluginHook::OnResponse => "on_response",
            PluginHook::OnConnect => "on_connect",
            PluginHook::OnCapture => "on_capture",
            PluginHook::OnRuleMatch => "on_rule_match",
        }
    }

    /// All available hooks
    pub fn all() -> &'static [PluginHook] {
        &[
            PluginHook::OnRequest,
            PluginHook::OnResponse,
            PluginHook::OnConnect,
            PluginHook::OnCapture,
            PluginHook::OnRuleMatch,
        ]
    }
}

/// Context passed to plugin hooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookContext {
    /// HTTP method (GET, POST, etc.)
    pub method: Option<String>,

    /// Request URL
    pub url: Option<String>,

    /// HTTP headers
    pub headers: HashMap<String, String>,

    /// Request/Response body
    pub body: Option<Vec<u8>>,

    /// HTTP status code (for responses)
    pub status_code: Option<u16>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl Default for HookContext {
    fn default() -> Self {
        Self {
            method: None,
            url: None,
            headers: HashMap::new(),
            body: None,
            status_code: None,
            metadata: HashMap::new(),
        }
    }
}

impl HookContext {
    /// Create a new hook context
    pub fn new() -> Self {
        Self::default()
    }

    /// Set HTTP method
    pub fn with_method(mut self, method: impl Into<String>) -> Self {
        self.method = Some(method.into());
        self
    }

    /// Set URL
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Set headers
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = headers;
        self
    }

    /// Set body
    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }

    /// Set status code
    pub fn with_status_code(mut self, code: u16) -> Self {
        self.status_code = Some(code);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Serialize to JSON bytes
    pub fn to_json_bytes(&self) -> crate::error::Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(|e| crate::error::ProxyError::internal(e.to_string()))
    }

    /// Deserialize from JSON bytes
    pub fn from_json_bytes(data: &[u8]) -> crate::error::Result<Self> {
        serde_json::from_slice(data).map_err(|e| crate::error::ProxyError::internal(e.to_string()))
    }
}

/// Result returned from a plugin hook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookResult {
    /// Whether the plugin modified the context
    pub modified: bool,

    /// Updated context (if modified)
    pub context: Option<HookContext>,

    /// Whether to continue processing this request/response
    pub should_continue: bool,

    /// Optional message from the plugin
    pub message: Option<String>,
}

impl Default for HookResult {
    fn default() -> Self {
        Self {
            modified: false,
            context: None,
            should_continue: true,
            message: None,
        }
    }
}

impl HookResult {
    /// Create a new hook result with no modifications
    pub fn unmodified() -> Self {
        Self::default()
    }

    /// Create a modified result
    pub fn modified(context: HookContext) -> Self {
        Self {
            modified: true,
            context: Some(context),
            should_continue: true,
            message: None,
        }
    }

    /// Create a result that blocks further processing
    pub fn block(message: impl Into<String>) -> Self {
        Self {
            modified: false,
            context: None,
            should_continue: false,
            message: Some(message.into()),
        }
    }

    /// Serialize to JSON bytes
    pub fn to_json_bytes(&self) -> crate::error::Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(|e| crate::error::ProxyError::internal(e.to_string()))
    }

    /// Deserialize from JSON bytes
    pub fn from_json_bytes(data: &[u8]) -> crate::error::Result<Self> {
        serde_json::from_slice(data).map_err(|e| crate::error::ProxyError::internal(e.to_string()))
    }
}
