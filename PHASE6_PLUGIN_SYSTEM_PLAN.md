# Phase 6 - Plugin System (WASM) Implementation Plan

## 🎯 Objective

Build a secure, performant WASM-based plugin system that allows users to extend INT3RCEPTOR with custom logic.

## 📋 Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                   INT3RCEPTOR Core                           │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌────────────────────────────────────────────────────┐    │
│  │              Plugin Manager                         │    │
│  ├────────────────────────────────────────────────────┤    │
│  │  - Load plugins from directory                     │    │
│  │  - Initialize WASM runtime (wasmtime)              │    │
│  │  - Manage plugin lifecycle                         │    │
│  │  - Execute plugin hooks                            │    │
│  └────────────────────────────────────────────────────┘    │
│                        │                                     │
│       ┌────────────────┼────────────────┐                   │
│       │                │                │                   │
│       ▼                ▼                ▼                   │
│  ┌────────┐      ┌────────┐      ┌────────┐               │
│  │Plugin 1│      │Plugin 2│      │Plugin 3│               │
│  │(.wasm) │      │(.wasm) │      │(.wasm) │               │
│  └────────┘      └────────┘      └────────┘               │
│       │                │                │                   │
│       └────────────────┼────────────────┘                   │
│                        │                                     │
│                        ▼                                     │
│  ┌─────────────────────────────────────────────────────┐   │
│  │             Hook Points                              │   │
│  ├─────────────────────────────────────────────────────┤   │
│  │  • on_request       - Modify outgoing requests      │   │
│  │  • on_response      - Modify incoming responses     │   │
│  │  • on_connect       - Handle new connections        │   │
│  │  • on_capture       - Process captured traffic      │   │
│  │  • on_rule_match    - Custom rule logic             │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                              │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
              ┌──────────────────────────┐
              │   WASM Runtime (Host)    │
              ├──────────────────────────┤
              │  • Memory isolation      │
              │  • CPU/time limits       │
              │  • No file system access │
              │  • Controlled imports    │
              └──────────────────────────┘
```

## 🔧 Technical Implementation

### 1. Dependencies

```toml
[dependencies]
wasmtime = "17.0"           # WASM runtime
wasmtime-wasi = "17.0"      # WASI support (optional)
serde_json = "1.0"          # For plugin communication
```

### 2. Plugin API Design

#### Plugin Interface (WASM Side)

```rust
// Plugins must export these functions:

#[no_mangle]
pub extern "C" fn plugin_init() -> i32 {
    // Initialize plugin
    0 // 0 = success
}

#[no_mangle]
pub extern "C" fn on_request(request_ptr: *const u8, request_len: usize) -> i32 {
    // Process request, return modified data pointer
    // Communication via shared memory
    0
}

#[no_mangle]
pub extern "C" fn on_response(response_ptr: *const u8, response_len: usize) -> i32 {
    // Process response
    0
}
```

#### Host Functions (Provided to Plugins)

```rust
// Functions the host provides to plugins:

host_log(level: i32, msg_ptr: *const u8, msg_len: usize)
host_get_header(key_ptr: *const u8, key_len: usize) -> i32
host_set_header(key_ptr: *const u8, value_ptr: *const u8, ...)
host_get_body() -> i32
host_set_body(data_ptr: *const u8, data_len: usize)
```

### 3. Plugin Manager

```rust
pub struct PluginManager {
    engine: Engine,
    plugins: Vec<LoadedPlugin>,
    config: PluginConfig,
}

struct LoadedPlugin {
    name: String,
    instance: Instance,
    store: Store<PluginContext>,
}

impl PluginManager {
    pub fn new(plugin_dir: PathBuf) -> Result<Self>;
    pub fn load_plugins(&mut self) -> Result<()>;
    pub fn call_hook(&self, hook: PluginHook, data: &[u8]) -> Result<Vec<u8>>;
    pub fn reload_plugin(&mut self, name: &str) -> Result<()>;
}
```

### 4. Hook System

```rust
pub enum PluginHook {
    OnRequest,
    OnResponse,
    OnConnect,
    OnCapture,
    OnRuleMatch,
}

pub struct HookContext {
    pub request: Option<HttpRequest>,
    pub response: Option<HttpResponse>,
    pub metadata: HashMap<String, String>,
}
```

### 5. Security & Sandboxing

#### Resource Limits

```rust
// Limit plugin execution time
config.max_execution_time = Duration::from_secs(5);

// Limit memory usage
config.max_memory_bytes = 10 * 1024 * 1024; // 10MB
```

#### Permissions

```rust
pub struct PluginPermissions {
    pub can_make_network_requests: bool,
    pub can_access_filesystem: bool,
    pub can_modify_requests: bool,
    pub can_modify_responses: bool,
}
```

## 📁 File Structure

```
interceptor/
├── core/
│   └── src/
│       ├── plugin/
│       │   ├── mod.rs              # Plugin module exports
│       │   ├── manager.rs          # Plugin manager
│       │   ├── runtime.rs          # WASM runtime wrapper
│       │   ├── host_functions.rs  # Host functions for plugins
│       │   ├── hooks.rs            # Hook definitions
│       │   └── config.rs           # Plugin configuration
│       └── lib.rs                  # Add plugin module
├── plugins/                        # Plugin directory
│   ├── example-logger/
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   └── lib.rs
│   │   └── plugin.wasm
│   └── example-modifier/
│       └── plugin.wasm
└── docs/
    └── PLUGIN_DEVELOPMENT.md       # Plugin development guide
```

## 🎯 Implementation Phases

### Phase 1: Core Infrastructure ✅

-   [x] Add wasmtime dependencies
-   [ ] Create plugin module structure
-   [ ] Implement basic WASM runtime wrapper
-   [ ] Create plugin configuration

### Phase 2: Host Functions

-   [ ] Implement logging functions
-   [ ] Implement header access functions
-   [ ] Implement body access functions
-   [ ] Memory management utilities

### Phase 3: Hook System

-   [ ] Define hook points in proxy code
-   [ ] Implement hook execution
-   [ ] Add hook context builders
-   [ ] Error handling for failed hooks

### Phase 4: Plugin Manager

-   [ ] Plugin discovery and loading
-   [ ] Plugin lifecycle management
-   [ ] Hot reload support
-   [ ] Plugin metadata parsing

### Phase 5: Examples & Documentation

-   [ ] Create example plugin (logger)
-   [ ] Create example plugin (header modifier)
-   [ ] Write plugin development guide
-   [ ] Create plugin API reference

### Phase 6: Testing & Security

-   [ ] Unit tests for plugin system
-   [ ] Integration tests with real plugins
-   [ ] Security audit
-   [ ] Performance benchmarks

## 🔐 Security Considerations

### Sandboxing

-   ✅ WASM provides memory isolation
-   ✅ No direct file system access
-   ✅ Controlled host function imports
-   ✅ Execution time limits
-   ✅ Memory limits

### Plugin Verification

-   [ ] Optional signature verification
-   [ ] Plugin checksum validation
-   [ ] Trust levels (trusted, untrusted)
-   [ ] Audit logging of plugin actions

### Resource Limits

```rust
// Per-plugin limits
max_execution_time: 5s
max_memory: 10MB
max_instances: 100
gas_limit: 1_000_000 // Computational limit
```

## 📊 Example Plugin (Rust)

```rust
// plugin-logger/src/lib.rs

#[no_mangle]
pub extern "C" fn plugin_init() -> i32 {
    host_log(0, "Logger plugin initialized\0".as_ptr(), 27);
    0
}

#[no_mangle]
pub extern "C" fn on_request(request_ptr: *const u8, request_len: usize) -> i32 {
    let request_data = unsafe {
        std::slice::from_raw_parts(request_ptr, request_len)
    };

    // Parse request (JSON)
    // Log request details
    host_log(1, format!("Request: {} bytes", request_len).as_ptr(), ...);

    0 // Don't modify
}

// Build with:
// cargo build --target wasm32-unknown-unknown --release
```

## 🎓 Plugin Development Workflow

### 1. Create Plugin

```bash
# Create new plugin project
cargo new --lib my-plugin
cd my-plugin

# Add to Cargo.toml:
[lib]
crate-type = ["cdylib"]

# Write plugin code in src/lib.rs
```

### 2. Build Plugin

```bash
# Build for WASM target
cargo build --target wasm32-unknown-unknown --release

# Output: target/wasm32-unknown-unknown/release/my_plugin.wasm
```

### 3. Install Plugin

```bash
# Copy to plugins directory
cp target/wasm32-unknown-unknown/release/my_plugin.wasm \
   ~/.interceptor/plugins/my-plugin.wasm

# Or use interceptor CLI:
interceptor plugin install my-plugin.wasm
```

### 4. Configure Plugin

```toml
# plugins/config.toml
[[plugin]]
name = "my-plugin"
path = "my-plugin.wasm"
enabled = true
permissions = { can_modify_requests = true }
```

## 📈 Performance Considerations

### WASM Overhead

-   **Compilation**: First load is slow (~100ms), then cached
-   **Execution**: ~5-10% slower than native Rust
-   **Memory**: Isolated memory space per plugin

### Optimization Strategies

-   [ ] Ahead-of-time (AOT) compilation
-   [ ] Plugin result caching
-   [ ] Lazy plugin loading
-   [ ] Parallel plugin execution (where safe)

## 🚀 Future Enhancements

-   [ ] JavaScript/TypeScript plugin support (via quickjs-wasm)
-   [ ] Python plugin support (via rustpython-wasm)
-   [ ] Plugin marketplace/registry
-   [ ] Visual plugin editor
-   [ ] Plugin debugging tools
-   [ ] Plugin performance profiler

## 📝 Success Criteria

-   [ ] Load and execute WASM plugins successfully
-   [ ] Plugins can modify requests/responses
-   [ ] Sandboxing prevents malicious plugins
-   [ ] Performance overhead <10%
-   [ ] Clear developer documentation
-   [ ] At least 2 working example plugins

---

**Status**: 🚧 **PLANNING COMPLETE - READY TO IMPLEMENT**

**Estimated Time**: 8-12 hours (complex feature)

**Dependencies**: wasmtime, wasmtime-wasi

**Priority**: High (Core Phase 6 feature)
