// Example Logger Plugin for INT3RCEPTOR
// This demonstrates the plugin API

// Import host functions that the proxy provides
extern "C" {
    fn host_log(level: i32, msg_ptr: *const u8, msg_len: i32);
}

/// Log levels matching the host
const LOG_DEBUG: i32 = 0;
const LOG_INFO: i32 = 1;
const LOG_WARN: i32 = 2;
const LOG_ERROR: i32 = 3;

/// Helper to log messages to the host
fn log(level: i32, message: &str) {
    unsafe {
        host_log(level, message.as_ptr(), message.len() as i32);
    }
}

/// Plugin initialization - called once when plugin is loaded
#[no_mangle]
pub extern "C" fn plugin_init() -> i32 {
    log(LOG_INFO, "Example Logger Plugin initialized!");
    log(LOG_INFO, "This plugin logs all requests and responses");
    0 // 0 = success
}

/// Called when a new request is received
#[no_mangle]
pub extern "C" fn on_request() -> i32 {
    log(LOG_INFO, "🔵 REQUEST intercepted");
    log(LOG_DEBUG, "Processing request through logger plugin");
    0 // Don't modify the request
}

/// Called when a response is received
#[no_mangle]
pub extern "C" fn on_response() -> i32 {
    log(LOG_INFO, "🟢 RESPONSE intercepted");
    log(LOG_DEBUG, "Processing response through logger plugin");
    0 // Don't modify the response
}

/// Called when a new connection is established
#[no_mangle]
pub extern "C" fn on_connect() -> i32 {
    log(LOG_WARN, "⚡ NEW CONNECTION established");
    0
}

/// Called when traffic is captured
#[no_mangle]
pub extern "C" fn on_capture() -> i32 {
    log(LOG_DEBUG, "📦 Traffic captured and stored");
    0
}

/// Called when a rule matches
#[no_mangle]
pub extern "C" fn on_rule_match() -> i32 {
    log(LOG_WARN, "✅ Rule matched!");
    0
}
