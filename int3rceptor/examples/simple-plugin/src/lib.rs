#[link(wasm_import_module = "env")]
extern "C" {
    fn host_log(level: i32, msg_ptr: *const u8, msg_len: usize) -> i32;
    fn host_set_header(
        name_ptr: *const u8,
        name_len: usize,
        val_ptr: *const u8,
        val_len: usize,
    ) -> i32;
}

fn log(msg: &str) {
    unsafe {
        host_log(1, msg.as_ptr(), msg.len());
    }
}

fn set_header(name: &str, value: &str) {
    unsafe {
        host_set_header(name.as_ptr(), name.len(), value.as_ptr(), value.len());
    }
}

#[no_mangle]
pub extern "C" fn plugin_init() -> i32 {
    log("Plugin initialized!");
    0
}

#[no_mangle]
pub extern "C" fn on_request() -> i32 {
    log("Request intercepted by WASM plugin! Injecting header.");
    set_header("X-WASM-Intercepted", "True");
    0
}

#[no_mangle]
pub extern "C" fn on_response() -> i32 {
    log("Response intercepted by WASM plugin!");
    0
}
