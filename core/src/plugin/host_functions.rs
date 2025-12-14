use std::sync::{Arc, RwLock};
use wasmtime::*;

/// Context available to host functions
#[derive(Clone)]
pub struct HostContext {
    /// Shared log buffer
    pub logs: Arc<RwLock<Vec<String>>>,

    /// Plugin name (for logging)
    pub plugin_name: String,
}

impl HostContext {
    pub fn new(plugin_name: String) -> Self {
        Self {
            logs: Arc::new(RwLock::new(Vec::new())),
            plugin_name,
        }
    }

    pub fn add_log(&self, message: String) {
        if let Ok(mut logs) = self.logs.write() {
            logs.push(format!("[{}] {}", self.plugin_name, message));

            // Also log via tracing
            tracing::info!(plugin = %self.plugin_name, "{}", message);
        }
    }
}

/// Define host functions available to plugins
pub fn define_host_functions(linker: &mut Linker<HostContext>) -> anyhow::Result<()> {
    // host_log(level: i32, msg_ptr: i32, msg_len: i32)
    // Log a message from the plugin
    linker.func_wrap(
        "env",
        "host_log",
        |mut caller: Caller<'_, HostContext>, level: i32, msg_ptr: i32, msg_len: i32| -> i32 {
            let mem = match caller.get_export("memory") {
                Some(Extern::Memory(mem)) => mem,
                _ => return -1,
            };

            let data = mem.data(&caller);
            let msg_ptr = msg_ptr as usize;
            let msg_len = msg_len as usize;

            if msg_ptr + msg_len > data.len() {
                return -1; // Out of bounds
            }

            let msg_bytes = &data[msg_ptr..msg_ptr + msg_len];
            let message = match String::from_utf8(msg_bytes.to_vec()) {
                Ok(s) => s,
                Err(_) => return -1,
            };

            let context = caller.data();
            let level_str = match level {
                0 => "DEBUG",
                1 => "INFO",
                2 => "WARN",
                3 => "ERROR",
                _ => "UNKNOWN",
            };

            context.add_log(format!("[{}] {}", level_str, message));
            0
        },
    )?;

    // host_get_memory_size() -> i32
    // Get current memory size allocated to the plugin
    linker.func_wrap(
        "env",
        "host_get_memory_size",
        |mut caller: Caller<'_, HostContext>| -> i32 {
            let mem = match caller.get_export("memory") {
                Some(Extern::Memory(mem)) => mem,
                _ => return 0,
            };

            (mem.data_size(&caller) / (64 * 1024)) as i32 // Pages
        },
    )?;

    // host_abort(msg_ptr: i32, msg_len: i32)
    // Abort plugin execution with an error message
    linker.func_wrap(
        "env",
        "host_abort",
        |mut caller: Caller<'_, HostContext>, msg_ptr: i32, msg_len: i32| {
            let mem = match caller.get_export("memory") {
                Some(Extern::Memory(mem)) => mem,
                _ => return,
            };

            let data = mem.data(&caller);
            let msg_ptr = msg_ptr as usize;
            let msg_len = msg_len as usize;

            if msg_ptr + msg_len <= data.len() {
                if let Ok(message) = String::from_utf8(data[msg_ptr..msg_ptr + msg_len].to_vec()) {
                    let context = caller.data();
                    context.add_log(format!("[ABORT] {}", message));
                }
            }
        },
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_context() {
        let ctx = HostContext::new("test-plugin".to_string());
        ctx.add_log("Test message".to_string());

        let logs = ctx.logs.read().unwrap();
        assert_eq!(logs.len(), 1);
        assert!(logs[0].contains("test-plugin"));
        assert!(logs[0].contains("Test message"));
    }
}
