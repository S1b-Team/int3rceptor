# Plugin System Integration - Implementation Tracker

## ✅ COMPLETED SO FAR

### Core Infrastructure

-   ✅ All plugin modules created and compiling
-   ✅ Example plugin created (686 bytes WASM)
-   ✅ ProxyServer structure updated with plugins field

### Remaining Integration Work

## 🚧 IN PROGRESS - Proxy Integration

### 1. Update ProxyServer Constructor

**File**: `core/src/proxy.rs`

```rust
pub fn new(
    addr: SocketAddr,
    capture: Arc<RequestCapture>,
    rules: Arc<RuleEngine>,
    scope: Arc<ScopeManager>,
    tls: Option<Arc<TlsInterceptor>>,
    plugins: Option<Arc<crate::plugin::PluginManager>>,  // ADD THIS
) -> Self {
    Self {
        addr,
        capture,
        pool: ConnectionPool::new(),
        rules,
        scope,
        tls,
        plugins,  // ADD THIS
    }
}
```

### 2. Add Hook Calls in forward_request

**File**: `core/src/proxy.rs` (around line 194-292)

Add BEFORE forwarding:

```rust
// Execute on_request plugin hook
if let Some(ref plugins) = plugins {
    use crate::plugin::hooks::{HookContext, PluginHook};

    let mut hook_context = HookContext::new()
        .with_method(parts.method.as_str())
        .with_url(target_uri.to_string())
        .with_body(body_bytes.clone());

    if let Ok(updated_context) = plugins.execute_hook(PluginHook::OnRequest, hook_context) {
        // Apply any modifications from plugins
        if let Some(body) = updated_context.body {
            body_bytes = body;
        }
    }
}
```

Add AFTER receiving response:

```rust
// Execute on_response plugin hook
if let Some(ref plugins) = plugins {
    let mut hook_context = HookContext::new()
        .with_status_code(parts.status.as_u16())
        .with_body(body_bytes.clone());

    if let Ok(updated_context) = plugins.execute_hook(PluginHook::OnResponse, hook_context) {
        if let Some(body) = updated_context.body {
            body_bytes = body;
        }
    }
}
```

### 3. Add Hook Call in handle_connect

**File**: `core/src/proxy.rs` (around line 294)

```rust
// Execute on_connect plugin hook
if let Some(ref plugins) = capture {
    let hook_context = HookContext::new()
        .with_url(format!("https://{}", authority))
        .with_metadata("connection_type".to_string(), "CONNECT".to_string());

    let _ = plugins.execute_hook(PluginHook::OnConnect, hook_context);
}
```

### 4. Update main.rs to Initialize Plugins

**File**: `cli/src/main.rs` or wherever ProxyServer is created

```rust
use interceptor_core::plugin::{PluginManager, PluginSystemConfig};

// Load plugin configuration
let plugin_config = PluginSystemConfig {
    plugin_dir: PathBuf::from("plugins"),
    enabled: true,
    plugins: vec![
        PluginConfig {
            name: "example-logger".to_string(),
            path: PathBuf::from("example_logger.wasm"),
            enabled: true,
            ..Default::default()
        }
    ],
};

// Create and initialize plugin manager
let plugin_manager = Arc::new(PluginManager::new(plugin_config));
plugin_manager.load_all()?;

// Pass to ProxyServer
let proxy = ProxyServer::new(
    addr,
    capture,
    rules,
    scope,
    tls,
    Some(plugin_manager),  // ADD THIS
);
```

---

## 📝 TODO - API Endpoints

### File to Create: `api/src/plugin_routes.rs`

```rust
use axum::{
    extract::Path,
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use interceptor_core::plugin::PluginManager;

#[derive(Serialize)]
struct PluginInfo {
    name: String,
    loaded: bool,
    enabled: bool,
}

#[derive(Serialize)]
struct PluginListResponse {
    plugins: Vec<PluginInfo>,
    total: usize,
}

// GET /api/plugins - List all plugins
pub async fn list_plugins(
    Extension(plugins): Extension<Arc<PluginManager>>,
) -> Json<PluginListResponse> {
    let loaded = plugins.list_plugins();
    let plugin_info: Vec<PluginInfo> = loaded.iter().map(|name| {
        PluginInfo {
            name: name.clone(),
            loaded: true,
            enabled: plugins.is_loaded(name),
        }
    }).collect();

    Json(PluginListResponse {
        total: plugin_info.len(),
        plugins: plugin_info,
    })
}

// POST /api/plugins/reload/:name - Reload a plugin
pub async fn reload_plugin(
    Path(name): Path<String>,
    Extension(plugins): Extension<Arc<PluginManager>>,
) -> Json<serde_json::Value> {
    match plugins.reload_plugin(&name) {
        Ok(_) => Json(json!({"success": true, "message": "Plugin reloaded"})),
        Err(e) => Json(json!({"success": false, "error": e.to_string()})),
    }
}

// GET /api/plugins/discover - Discover available plugins
pub async fn discover_plugins(
    Extension(plugins): Extension<Arc<PluginManager>>,
) -> Json<Vec<String>> {
    match plugins.discover_plugins() {
        Ok(paths) => Json(paths.iter().map(|p| p.display().to_string()).collect()),
        Err(_) => Json(vec![]),
    }
}
```

### Add routes in `api/src/routes.rs`

```rust
use crate::plugin_routes;

// In build_router function:
.route("/api/plugins", get(plugin_routes::list_plugins))
.route("/api/plugins/reload/:name", post(plugin_routes::reload_plugin))
.route("/api/plugins/discover", get(plugin_routes::discover_plugins))
```

---

## 🧪 TODO - Write Tests

### File to Create: `core/src/plugin/tests.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_plugin_config_creation() {
        let config = PluginConfig {
            name: "test".to_string(),
            path: PathBuf::from("test.wasm"),
            enabled: true,
            ..Default::default()
        };

        assert_eq!(config.name, "test");
        assert!(config.enabled);
    }

    #[test]
    fn test_hook_context_builder() {
        let context = HookContext::new()
            .with_method("GET")
            .with_url("http://example.com");

        assert_eq!(context.method, Some("GET".to_string()));
        assert_eq!(context.url, Some("http://example.com".to_string()));
    }

    #[test]
    fn test_plugin_manager_creation() {
        let config = PluginSystemConfig::default();
        let manager = PluginManager::new(config);

        assert_eq!(manager.count(), 0);
    }
}
```

---

## 🔌 TODO - More Example Plugins

### Plugin 2: Header Modifier

**File**: `plugins/header-modifier/src/lib.rs`

```rust
extern "C" {
    fn host_log(level: i32, msg_ptr: *const u8, msg_len: i32);
}

const LOG_INFO: i32 = 1;

fn log(level: i32, message: &str) {
    unsafe {
        host_log(level, message.as_ptr(), message.len() as i32);
    }
}

#[no_mangle]
pub extern "C" fn plugin_init() -> i32 {
    log(LOG_INFO, "Header Modifier Plugin initialized");
    0
}

#[no_mangle]
pub extern "C" fn on_request() -> i32 {
    log(LOG_INFO, "Adding custom header: X-Plugin-Modified: true");
    // In full implementation, this would modify headers
    0
}

#[no_mangle]
pub extern "C" fn on_response() -> i32 {
    log(LOG_INFO, "Removing sensitive headers");
    0
}
```

### Plugin 3: Request Counter

**File**: `plugins/request-counter/src/lib.rs`

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

static REQUEST_COUNT: AtomicUsize = AtomicUsize::new(0);
static RESPONSE_COUNT: AtomicUsize = AtomicUsize::new(0);

extern "C" {
    fn host_log(level: i32, msg_ptr: *const u8, msg_len: i32);
}

const LOG_INFO: i32 = 1;

fn log(level: i32, message: &str) {
    unsafe {
        host_log(level, message.as_ptr(), message.len() as i32);
    }
}

#[no_mangle]
pub extern "C" fn plugin_init() -> i32 {
    log(LOG_INFO, "Request Counter Plugin initialized");
    0
}

#[no_mangle]
pub extern "C" fn on_request() -> i32 {
    let count = REQUEST_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
    log(LOG_INFO, &format!("Total requests: {}", count));
    0
}

#[no_mangle]
pub extern "C" fn on_response() -> i32 {
    let count = RESPONSE_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
    log(LOG_INFO, &format!("Total responses: {}", count));
    0
}
```

---

## 📊 Progress Tracker

### Task 1: Proxy Integration (1 hour)

-   [x] Add plugins field to ProxyServer (DONE)
-   [ ] Update constructor
-   [ ] Add hook calls in forward_request (on_request)
-   [ ] Add hook calls in forward_request (on_response)
-   [ ] Add hook call in handle_connect
-   [ ] Update main.rs to initialize plugins
-   [ ] Test integration

**Status**: 20% Complete

### Task 2: API Endpoints (1 hour)

-   [ ] Create plugin_routes.rs
-   [ ] Implement list_plugins endpoint
-   [ ] Implement reload_plugin endpoint
-   [ ] Implement discover_plugins endpoint
-   [ ] Add routes to router
-   [ ] Test endpoints

**Status**: 0% Complete

### Task 3: Tests (1 hour)

-   [ ] Unit tests for config
-   [ ] Unit tests for hooks
-   [ ] Unit tests for manager
-   [ ] Integration test with WASM
-   [ ] API endpoint tests

**Status**: 0% Complete

### Task 4: More Plugins (1 hour)

-   [x] Example Logger (DONE - 686 bytes)
-   [ ] Header Modifier
-   [ ] Request Counter
-   [ ] Build all plugins
-   [ ] Test all plugins

**Status**: 33% Complete

---

## 🎯 Overall Progress: 13% Complete

**Estimated Time Remaining**: ~3.5 hours

**Next Steps**:

1. Finish proxy integration
2. Add API endpoints
3. Write tests
4. Create more example plugins

---

**Created**: 2025-12-14
**Last Updated**: 2025-12-14
