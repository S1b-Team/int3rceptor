// ╔═══════════════════════════════════════════════════════════════════════════╗
// ║                   LICENSE KEY VALIDATION SYSTEM                           ║
// ║                   Copyright (c) 2025 S1BGr0uP                             ║
// ║                        All Rights Reserved                                ║
// ║  CRITICAL: This module contains proprietary license validation logic.     ║
// ║  Tampering with this code violates the license agreement.                 ║
// ║  Q1 2026: Ed25519 signature verification implemented for security        ║
// ╚═══════════════════════════════════════════════════════════════════════════╝

use crate::crypto;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// License tiers available for Int3rceptor
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LicenseTier {
    /// Free tier - Personal use only, limited features
    Free,
    /// Professional tier - Commercial use, advanced features
    Professional,
    /// Enterprise tier - Unlimited use, priority support
    Enterprise,
}

impl LicenseTier {
    /// Maximum concurrent connections allowed per tier
    pub fn max_connections(&self) -> usize {
        match self {
            LicenseTier::Free => 10,
            LicenseTier::Professional => 100,
            LicenseTier::Enterprise => usize::MAX,
        }
    }

    /// Maximum requests per second allowed per tier
    pub fn max_rps(&self) -> usize {
        match self {
            LicenseTier::Free => 100,
            LicenseTier::Professional => 1000,
            LicenseTier::Enterprise => usize::MAX,
        }
    }

    /// Whether advanced features are enabled
    pub fn has_advanced_features(&self) -> bool {
        matches!(self, LicenseTier::Professional | LicenseTier::Enterprise)
    }

    /// Whether priority support is included
    pub fn has_priority_support(&self) -> bool {
        matches!(self, LicenseTier::Enterprise)
    }
}

/// License information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    /// License key (base64 encoded signature)
    pub key: String,
    /// License tier
    pub tier: LicenseTier,
    /// Licensee name/organization
    pub licensee: String,
    /// Issue timestamp (Unix epoch)
    pub issued_at: u64,
    /// Expiration timestamp (Unix epoch), None = perpetual
    pub expires_at: Option<u64>,
    /// Hardware fingerprint (optional binding)
    pub hardware_id: Option<String>,
}

impl License {
    /// Check if license is currently valid
    pub fn is_valid(&self) -> bool {
        // Check expiration
        if let Some(expires_at) = self.expires_at {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time before UNIX epoch")
                .as_secs();
            if now > expires_at {
                return false;
            }
        }

        // Check hardware binding if present
        if let Some(expected_hw_id) = &self.hardware_id {
            if let Ok(current_hw_id) = get_hardware_id() {
                if &current_hw_id != expected_hw_id {
                    return false;
                }
            }
        }

        true
    }

    /// Get days until expiration (None if perpetual)
    pub fn days_until_expiration(&self) -> Option<u64> {
        self.expires_at.map(|expires_at| {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time before UNIX epoch")
                .as_secs();
            if expires_at > now {
                (expires_at - now) / 86400
            } else {
                0
            }
        })
    }
}

/// License validator
pub struct LicenseValidator {
    /// Public key for signature verification (embedded at compile-time)
    #[allow(dead_code)]
    public_key: &'static str,
}

impl LicenseValidator {
    /// Create new validator with embedded public key
    pub fn new() -> Self {
        Self {
            // This would be your actual public key in production
            public_key: env!("LICENSE_PUBLIC_KEY"),
        }
    }

    /// Validate a license key
    pub fn validate(&self, license_key: &str) -> Result<License> {
        use base64::{engine::general_purpose, Engine};
        
        // Decode base64 license key
        let decoded = general_purpose::STANDARD.decode(license_key)
            .map_err(|_| anyhow!(obfstr::obfstr!("Invalid license key format").to_string()))?;

        // Parse license data
        let license: License = serde_json::from_slice(&decoded)
            .map_err(|_| anyhow!(obfstr::obfstr!("Corrupted license key").to_string()))?;

        // Verify signature (simplified - in production use proper crypto)
        if !self.verify_signature(&license) {
            return Err(anyhow!(obfstr::obfstr!(
                "License signature verification failed"
            )
            .to_string()));
        }

        // Check if license is still valid
        if !license.is_valid() {
            return Err(anyhow!(obfstr::obfstr!(
                "License has expired or is invalid"
            )
            .to_string()));
        }

        Ok(license)
    }

    /// Verify license signature using Ed25519
    fn verify_signature(&self, license: &License) -> bool {
        if license.key.is_empty() {
            // Empty key means free tier (no signature needed)
            return license.tier == LicenseTier::Free;
        }

        // Decode license key from base64 (format: signature|license_json)
        use base64::{engine::general_purpose, Engine};
        let decoded = match general_purpose::STANDARD.decode(&license.key) {
            Ok(d) => d,
            Err(_) => return false,
        };

        // Extract signature (first 64 bytes) and license data (remaining bytes)
        if decoded.len() < 64 {
            return false;
        }

        let (sig_bytes, license_data) = decoded.split_at(64);
        let public_key_bytes = match self.public_key.as_bytes().try_into() {
            Ok(b) => b,
            Err(_) => return false,
        };

        match crypto::verify_ed25519_signature(public_key_bytes, license_data, sig_bytes) {
            Ok(valid) => valid,
            Err(_) => false,
        }
    }
}

impl Default for LicenseValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Get hardware fingerprint for license binding
/// Uses SHA-256 for cryptographic strength (prevents trivial spoofing)
fn get_hardware_id() -> Result<String> {
    use std::process::Command;

    let mut components = Vec::new();

    // Machine ID (most reliable - system-wide unique identifier)
    if let Ok(output) = Command::new("cat").arg("/etc/machine-id").output() {
        if let Ok(content) = String::from_utf8(output.stdout) {
            components.push(format!("machine_id:{}", content.trim()));
        }
    }

    // CPU info (Linux)
    if let Ok(output) = Command::new("cat").arg("/proc/cpuinfo").output() {
        if let Ok(content) = String::from_utf8(output.stdout) {
            if let Some(line) = content.lines().find(|l| l.starts_with("model name")) {
                components.push(format!("cpu:{}", line));
            }
        }
    }

    // OS version
    if let Ok(output) = Command::new("cat").arg("/etc/os-release").output() {
        if let Ok(content) = String::from_utf8(output.stdout) {
            if let Some(line) = content.lines().find(|l| l.starts_with("ID=")) {
                components.push(format!("os:{}", line));
            }
        }
    }

    if components.is_empty() {
        return Err(anyhow!(obfstr::obfstr!(
            "Could not generate hardware fingerprint"
        )
        .to_string()));
    }

    // Use SHA-256 for cryptographic hashing (prevents trivial spoofing via DefaultHasher)
    let combined = components.join("|");
    let hash = crypto::sha256_hash(combined.as_bytes());
    let hex_hash = hex::encode(hash);

    Ok(hex_hash)
}

/// License manager - handles loading and caching of licenses
pub struct LicenseManager {
    validator: LicenseValidator,
    current_license: Option<License>,
}

impl LicenseManager {
    /// Create new license manager
    pub fn new() -> Self {
        Self {
            validator: LicenseValidator::new(),
            current_license: None,
        }
    }

    /// Load license from environment variable or file
    pub fn load_license(&mut self) -> Result<()> {
        // Try environment variable first
        if let Ok(key) = std::env::var("INTERCEPTOR_LICENSE_KEY") {
            self.current_license = Some(self.validator.validate(&key)?);
            return Ok(());
        }

        // Try license file
        let license_path =
            std::env::var("INTERCEPTOR_LICENSE_PATH").unwrap_or_else(|_| "license.key".to_string());

        if let Ok(key) = std::fs::read_to_string(&license_path) {
            self.current_license = Some(self.validator.validate(key.trim())?);
            return Ok(());
        }

        // No license found - use free tier
        self.current_license = Some(License {
            key: String::new(),
            tier: LicenseTier::Free,
            licensee: "Free User".to_string(),
            issued_at: 0,
            expires_at: None,
            hardware_id: None,
        });

        Ok(())
    }

    /// Get current license tier
    pub fn tier(&self) -> LicenseTier {
        self.current_license
            .as_ref()
            .map(|l| l.tier)
            .unwrap_or(LicenseTier::Free)
    }

    /// Get current license (if any)
    pub fn license(&self) -> Option<&License> {
        self.current_license.as_ref()
    }

    /// Check if a feature is available in current license
    pub fn check_feature(&self, feature: &str) -> bool {
        let tier = self.tier();
        match feature {
            "intruder" => tier.has_advanced_features(),
            "websocket" => tier.has_advanced_features(),
            "rules" => tier.has_advanced_features(),
            "export" => tier.has_advanced_features(),
            _ => true, // Basic features always available
        }
    }
}

impl Default for LicenseManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_license_tiers() {
        assert_eq!(LicenseTier::Free.max_connections(), 10);
        assert_eq!(LicenseTier::Professional.max_connections(), 100);
        assert!(LicenseTier::Enterprise.max_connections() > 1000);
    }

    #[test]
    fn test_license_validation() {
        let license = License {
            key: String::new(),
            tier: LicenseTier::Free,
            licensee: "Test User".to_string(),
            issued_at: 0,
            expires_at: None,
            hardware_id: None,
        };

        assert!(license.is_valid());
    }
}
