// ╔═══════════════════════════════════════════════════════════════════════════╗
// ║                   PLUGIN SECURITY MODULE                                  ║
// ║                   Copyright (c) 2025 S1BGr0uP                             ║
// ║                        All Rights Reserved                                ║
// ║  P5 Security: Plugin path validation and resource limits                 ║
// ╚═══════════════════════════════════════════════════════════════════════════╝

use crate::error::Result;
use crate::error::ProxyError;
use std::path::{Path, PathBuf};

// P5 Security: Plugin resource limits
pub const PLUGIN_MAX_MEMORY_MB: u32 = 256; // Per plugin
pub const PLUGIN_MAX_CPU_PERCENT: u32 = 50; // Per plugin
pub const PLUGIN_TIMEOUT_SECS: u64 = 30; // Per plugin invocation
pub const PLUGIN_MAX_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100 MB

/// Plugin security validator
pub struct PluginSecurityValidator {
    allowed_dirs: Vec<PathBuf>,
}

impl PluginSecurityValidator {
    /// Create new validator with allowed directories
    pub fn new(allowed_dirs: Vec<PathBuf>) -> Self {
        Self { allowed_dirs }
    }

    /// Validate plugin path is within allowed directories (prevents path traversal)
    pub fn validate_plugin_path(&self, plugin_path: &Path) -> Result<()> {
        // Reject if path contains .. (path traversal attempt)
        if plugin_path
            .components()
            .any(|c| c.as_os_str() == "..")
        {
            return Err(ProxyError::internal(
                "Plugin path contains '..': path traversal not allowed".to_string(),
            ));
        }

        // Resolve to canonical path (follow symlinks, normalize)
        let canonical = match plugin_path.canonicalize() {
            Ok(path) => path,
            Err(e) => {
                return Err(ProxyError::internal(format!(
                    "Failed to resolve plugin path: {}",
                    e
                )))
            }
        };

        // Check if canonical path is within an allowed directory
        let is_allowed = self.allowed_dirs.iter().any(|allowed_dir| {
            canonical.starts_with(allowed_dir)
        });

        if !is_allowed {
            return Err(ProxyError::internal(format!(
                "Plugin path is outside allowed directories: {}",
                canonical.display()
            )));
        }

        // Check file size limit
        match std::fs::metadata(&canonical) {
            Ok(metadata) => {
                if metadata.len() > PLUGIN_MAX_FILE_SIZE {
                    return Err(ProxyError::internal(format!(
                        "Plugin file size {} exceeds limit of {}",
                        metadata.len(),
                        PLUGIN_MAX_FILE_SIZE
                    )));
                }
            }
            Err(e) => {
                return Err(ProxyError::internal(format!(
                    "Failed to read plugin file metadata: {}",
                    e
                )))
            }
        }

        Ok(())
    }

    /// Validate plugin is a WebAssembly file
    pub fn validate_wasm_file(&self, plugin_path: &Path) -> Result<()> {
        // Read first 4 bytes (WASM magic number: \0asm)
        let bytes = std::fs::read(plugin_path)
            .map_err(|e| ProxyError::internal(format!("Failed to read plugin file: {}", e)))?;

        const WASM_MAGIC: &[u8] = &[0x00, 0x61, 0x73, 0x6d];

        if bytes.len() < 4 || &bytes[..4] != WASM_MAGIC {
            return Err(ProxyError::internal(
                "File is not a valid WebAssembly module".to_string(),
            ));
        }

        Ok(())
    }
}

/// P5 Security: Plugin configuration with resource limits
#[derive(Debug, Clone)]
pub struct PluginResourceLimits {
    pub max_memory_mb: u32,
    pub max_cpu_percent: u32,
    pub timeout_secs: u64,
    pub max_file_size: u64,
}

impl Default for PluginResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: PLUGIN_MAX_MEMORY_MB,
            max_cpu_percent: PLUGIN_MAX_CPU_PERCENT,
            timeout_secs: PLUGIN_TIMEOUT_SECS,
            max_file_size: PLUGIN_MAX_FILE_SIZE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_path_traversal_prevention() {
        let temp_dir = TempDir::new().unwrap();
        let validator = PluginSecurityValidator::new(vec![temp_dir.path().to_path_buf()]);

        // Attempt path traversal
        let malicious_path = temp_dir.path().join("../../../etc/passwd");
        let result = validator.validate_plugin_path(&malicious_path);

        assert!(result.is_err(), "Path traversal should be rejected");
    }

    #[test]
    fn test_valid_plugin_path() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_file = temp_dir.path().join("plugin.wasm");
        
        // Create a minimal WASM file (magic + version)
        let wasm_header = [0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
        std::fs::write(&plugin_file, &wasm_header).unwrap();

        let validator = PluginSecurityValidator::new(vec![temp_dir.path().to_path_buf()]);
        let result = validator.validate_plugin_path(&plugin_file);

        assert!(result.is_ok(), "Valid plugin path should be accepted");
    }

    #[test]
    fn test_outside_allowed_dir() {
        let temp_dir = TempDir::new().unwrap();
        let other_dir = TempDir::new().unwrap();
        let plugin_file = other_dir.path().join("plugin.wasm");

        std::fs::write(&plugin_file, &[0x00, 0x61, 0x73, 0x6d]).unwrap();

        let validator = PluginSecurityValidator::new(vec![temp_dir.path().to_path_buf()]);
        let result = validator.validate_plugin_path(&plugin_file);

        assert!(
            result.is_err(),
            "Plugin outside allowed directory should be rejected"
        );
    }

    #[test]
    fn test_wasm_validation() {
        let temp_dir = TempDir::new().unwrap();

        // Valid WASM file
        let valid_wasm = temp_dir.path().join("valid.wasm");
        std::fs::write(&valid_wasm, &[0x00, 0x61, 0x73, 0x6d]).unwrap();

        // Invalid WASM file
        let invalid_wasm = temp_dir.path().join("invalid.wasm");
        std::fs::write(&invalid_wasm, &[0xff, 0xff, 0xff, 0xff]).unwrap();

        let validator = PluginSecurityValidator::new(vec![temp_dir.path().to_path_buf()]);

        assert!(
            validator.validate_wasm_file(&valid_wasm).is_ok(),
            "Valid WASM should pass"
        );
        assert!(
            validator.validate_wasm_file(&invalid_wasm).is_err(),
            "Invalid WASM should fail"
        );
    }
}
