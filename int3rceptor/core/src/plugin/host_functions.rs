use super::hooks::HookContext;
use std::sync::{Arc, RwLock};
use wasmtime::*;

/// Context available to host functions
#[derive(Clone)]
pub struct HostContext {
    /// Shared log buffer
    pub logs: Arc<RwLock<Vec<String>>>,

    /// Plugin name (for logging)
    pub plugin_name: String,

    /// Current hook context (request/response data)
    pub context: Arc<RwLock<Option<HookContext>>>,
}

impl HostContext {
    pub fn new(plugin_name: String) -> Self {
        Self {
            logs: Arc::new(RwLock::new(Vec::new())),
            plugin_name,
            context: Arc::new(RwLock::new(None)),
        }
    }

    pub fn with_context(plugin_name: String, context: HookContext) -> Self {
        Self {
            logs: Arc::new(RwLock::new(Vec::new())),
            plugin_name,
            context: Arc::new(RwLock::new(Some(context))),
        }
    }

    pub fn add_log(&self, message: String) {
        if let Ok(mut logs) = self.logs.write() {
            logs.push(format!("[{}] {}", self.plugin_name, message));
            tracing::info!(plugin = %self.plugin_name, "{}", message);
        }
    }
}

/// Define host functions available to plugins
pub fn define_host_functions(linker: &mut Linker<HostContext>) -> anyhow::Result<()> {
    // host_log(level: i32, msg_ptr: i32, msg_len: i32)
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
                return -1;
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

    // host_get_header(name_ptr: i32, name_len: i32, val_ptr: i32, val_max_len: i32) -> i32
    // Returns length of value, or -1 if not found/error
    linker.func_wrap(
        "env",
        "host_get_header",
        |mut caller: Caller<'_, HostContext>,
         name_ptr: i32,
         name_len: i32,
         val_ptr: i32,
         val_max_len: i32|
         -> i32 {
            let mem = match caller.get_export("memory") {
                Some(Extern::Memory(mem)) => mem,
                _ => return -1,
            };

            let (memory, ctx) = mem.data_and_store_mut(&mut caller);
            let name_ptr = name_ptr as usize;
            let name_len = name_len as usize;

            if name_ptr + name_len > memory.len() {
                return -1;
            }

            let name = match String::from_utf8(memory[name_ptr..name_ptr + name_len].to_vec()) {
                Ok(s) => s,
                Err(_) => return -1,
            };

            if let Ok(guard) = ctx.context.read() {
                if let Some(hook_ctx) = guard.as_ref() {
                    if let Some(value) = hook_ctx.headers.get(&name) {
                        let value_bytes = value.as_bytes();
                        let len = value_bytes.len();

                        if len > val_max_len as usize {
                            return len as i32; // Return needed size if buffer too small
                        }

                        let val_ptr = val_ptr as usize;
                        if val_ptr + len > memory.len() {
                            return -1;
                        }

                        memory[val_ptr..val_ptr + len].copy_from_slice(value_bytes);
                        return len as i32;
                    }
                }
            }
            -1 // Not found
        },
    )?;

    // host_set_header(name_ptr: i32, name_len: i32, val_ptr: i32, val_len: i32) -> i32
    linker.func_wrap(
        "env",
        "host_set_header",
        |mut caller: Caller<'_, HostContext>,
         name_ptr: i32,
         name_len: i32,
         val_ptr: i32,
         val_len: i32|
         -> i32 {
            let mem = match caller.get_export("memory") {
                Some(Extern::Memory(mem)) => mem,
                _ => return -1,
            };

            let data = mem.data(&caller);
            let name_ptr = name_ptr as usize;
            let name_len = name_len as usize;
            let val_ptr = val_ptr as usize;
            let val_len = val_len as usize;

            if name_ptr + name_len > data.len() || val_ptr + val_len > data.len() {
                return -1;
            }

            let name = match String::from_utf8(data[name_ptr..name_ptr + name_len].to_vec()) {
                Ok(s) => s,
                Err(_) => return -1,
            };

            let value = match String::from_utf8(data[val_ptr..val_ptr + val_len].to_vec()) {
                Ok(s) => s,
                Err(_) => return -1,
            };

            let ctx = caller.data();
            if let Ok(mut guard) = ctx.context.write() {
                if let Some(hook_ctx) = guard.as_mut() {
                    hook_ctx.headers.insert(name, value);
                    return 0; // Success
                }
            }
            -1
        },
    )?;

    // host_get_method(buf_ptr: i32, max_len: i32) -> i32
    linker.func_wrap(
        "env",
        "host_get_method",
        |mut caller: Caller<'_, HostContext>, buf_ptr: i32, max_len: i32| -> i32 {
            let mem = match caller.get_export("memory") {
                Some(Extern::Memory(mem)) => mem,
                _ => return -1,
            };

            let (memory, ctx) = mem.data_and_store_mut(&mut caller);

            if let Ok(guard) = ctx.context.read() {
                if let Some(hook_ctx) = guard.as_ref() {
                    if let Some(method) = &hook_ctx.method {
                        let bytes = method.as_bytes();
                        let len = bytes.len();

                        if len > max_len as usize {
                            return len as i32;
                        }

                        let buf_ptr = buf_ptr as usize;
                        if buf_ptr + len > memory.len() {
                            return -1;
                        }

                        memory[buf_ptr..buf_ptr + len].copy_from_slice(bytes);
                        return len as i32;
                    }
                }
            }
            -1
        },
    )?;

    // host_get_memory_size() -> i32
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
