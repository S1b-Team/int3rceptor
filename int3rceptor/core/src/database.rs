// ╔═══════════════════════════════════════════════════════════════════════════╗
// ║                   ENCRYPTED DATABASE MODULE                              ║
// ║                   Copyright (c) 2025 S1BGr0uP                             ║
// ║                        All Rights Reserved                                ║
// ║  Q1 2026 Security: AES-256-GCM field-level encryption for captures       ║
// ╚═══════════════════════════════════════════════════════════════════════════╝

use crate::crypto;
use anyhow::{anyhow, Result};
use rand::Rng;
use std::path::{Path, PathBuf};

/// Environment variable holding the master key as 64 hex characters.
pub const ENV_KEY: &str = "INTERCEPTOR_ENCRYPTION_KEY";
/// Environment variable pointing at a file that holds the master key.
pub const ENV_KEY_FILE: &str = "INTERCEPTOR_ENCRYPTION_KEY_FILE";
/// Environment variable that explicitly opts into plaintext storage.
pub const ENV_UNENCRYPTED: &str = "INTERCEPTOR_UNENCRYPTED";
/// Key file looked up in the working directory when no other source is set.
pub const DEFAULT_KEY_FILE: &str = ".interceptor_key";

/// Master key provider - handles encryption key loading and management
pub struct EncryptionKeyProvider {
    /// Master encryption key (256-bit)
    master_key: [u8; 32],
    /// Whether encryption is enabled
    enabled: bool,
}

impl std::fmt::Debug for EncryptionKeyProvider {
    // Never print key material.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EncryptionKeyProvider")
            .field("enabled", &self.enabled)
            .finish_non_exhaustive()
    }
}

/// Raw configuration inputs for [`EncryptionKeyProvider::from_sources`].
///
/// Kept separate from the process environment so the resolution logic can be
/// tested without mutating global state.
#[derive(Debug, Default, Clone)]
pub struct KeySources {
    /// Value of `INTERCEPTOR_ENCRYPTION_KEY`, if set.
    pub key_hex: Option<String>,
    /// Value of `INTERCEPTOR_ENCRYPTION_KEY_FILE`, if set.
    pub key_file: Option<PathBuf>,
    /// Fallback key file checked only when `key_file` is not set.
    pub default_key_file: Option<PathBuf>,
    /// Value of `INTERCEPTOR_UNENCRYPTED`, if set.
    pub unencrypted: Option<String>,
}

impl KeySources {
    /// Read sources from the process environment.
    pub fn from_env() -> Self {
        Self {
            key_hex: std::env::var(ENV_KEY).ok(),
            key_file: std::env::var_os(ENV_KEY_FILE).map(PathBuf::from),
            default_key_file: Some(PathBuf::from(DEFAULT_KEY_FILE)),
            unencrypted: std::env::var(ENV_UNENCRYPTED).ok(),
        }
    }
}

/// Parse an `INTERCEPTOR_UNENCRYPTED` value. Only `1` and `true`
/// (case-insensitive) enable plaintext; anything else is rejected so a typo
/// or `0`/`false` never silently turns encryption off.
fn parse_unencrypted_flag(value: &str) -> Result<bool> {
    let v = value.trim();
    if v == "1" || v.eq_ignore_ascii_case("true") {
        Ok(true)
    } else {
        Err(anyhow!(
            "{ENV_UNENCRYPTED} must be '1' or 'true' to enable plaintext storage; \
             unset it to require an encryption key"
        ))
    }
}

impl EncryptionKeyProvider {
    /// Load the key from the process environment.
    ///
    /// Resolution order:
    /// 1. `INTERCEPTOR_ENCRYPTION_KEY` (must be valid if set)
    /// 2. `INTERCEPTOR_ENCRYPTION_KEY_FILE` (must exist and be valid if set)
    /// 3. `./.interceptor_key` (must be valid if it exists)
    /// 4. `INTERCEPTOR_UNENCRYPTED=1|true` for plaintext (dev/testing only)
    ///
    /// Any invalid source is a hard error; there is no silent fallthrough.
    pub fn new() -> Result<Self> {
        let provider = Self::from_sources(&KeySources::from_env())?;
        if !provider.enabled {
            tracing::warn!(
                "Capture storage is UNENCRYPTED ({ENV_UNENCRYPTED} is set). Request/response \
                 headers and bodies, including cookies and auth tokens, are written to disk \
                 in plaintext. Use {ENV_KEY_FILE} for anything beyond local testing."
            );
        }
        Ok(provider)
    }

    /// Resolve a provider from explicit sources. See [`Self::new`].
    pub fn from_sources(sources: &KeySources) -> Result<Self> {
        // Validate the opt-out flag up front so a bad value is reported even
        // when a key is also configured.
        let unencrypted = match &sources.unencrypted {
            Some(v) => parse_unencrypted_flag(v)?,
            None => false,
        };

        if let Some(key_hex) = &sources.key_hex {
            let key =
                Self::from_hex(key_hex.trim()).map_err(|e| anyhow!("{ENV_KEY} is invalid: {e}"))?;
            return Ok(Self::with_key(key));
        }

        if let Some(path) = &sources.key_file {
            return Self::from_key_file(path)
                .map_err(|e| anyhow!("{ENV_KEY_FILE} ({}): {e}", path.display()));
        }

        if let Some(path) = &sources.default_key_file {
            if path.exists() {
                return Self::from_key_file(path)
                    .map_err(|e| anyhow!("key file {}: {e}", path.display()));
            }
        }

        if unencrypted {
            return Ok(Self::disabled());
        }

        Err(anyhow!(
            "No encryption key found. Set {ENV_KEY_FILE} (recommended) or {ENV_KEY}, \
             or set {ENV_UNENCRYPTED}=1 to store captures in plaintext (testing only)."
        ))
    }

    fn from_key_file(path: &Path) -> Result<Self> {
        let contents =
            std::fs::read_to_string(path).map_err(|e| anyhow!("cannot read key file: {e}"))?;
        Ok(Self::with_key(Self::from_hex(contents.trim())?))
    }

    /// Provider that encrypts with the given key.
    pub fn with_key(key: [u8; 32]) -> Self {
        Self {
            master_key: key,
            enabled: true,
        }
    }

    /// Provider with encryption explicitly disabled (plaintext storage).
    ///
    /// Only for tests and explicit opt-out; never used as an error fallback.
    pub fn disabled() -> Self {
        Self {
            // codeql[rust/hard-coded-cryptographic-value]: Encryption explicitly disabled; key is never used
            master_key: [0u8; 32],
            enabled: false,
        }
    }

    /// Parse a 64-character hex string into a 32-byte key.
    ///
    /// Returns an error (never panics) on wrong length, non-hex or non-ASCII input.
    pub fn from_hex(hex_str: &str) -> Result<[u8; 32]> {
        if !hex_str.is_ascii() {
            return Err(anyhow!("Key must be ASCII hex"));
        }
        if hex_str.len() != 64 {
            return Err(anyhow!("Key must be 64 hex characters (256 bits)"));
        }
        let mut key = [0u8; 32];
        hex::decode_to_slice(hex_str, &mut key).map_err(|_| anyhow!("Invalid hex in key"))?;
        Ok(key)
    }

    /// Convert key to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.master_key)
    }

    /// Generate new random encryption key
    pub fn generate_new() -> [u8; 32] {
        // codeql[rust/hard-coded-cryptographic-value]: Buffer initialized then filled with random data
        let mut key = [0u8; 32];
        rand::thread_rng().fill(&mut key);
        key
    }

    /// Get master key
    pub fn key(&self) -> &[u8; 32] {
        &self.master_key
    }

    /// Check if encryption is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Encrypted data container
#[derive(Debug, Clone)]
pub struct EncryptedData {
    /// AES-256-GCM ciphertext + tag
    pub ciphertext: Vec<u8>,
    /// 12-byte nonce
    pub nonce: [u8; 12],
}

impl EncryptedData {
    /// Serialize to bytes for storage
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(12 + 4 + self.ciphertext.len());
        bytes.extend_from_slice(&self.nonce);
        bytes.extend_from_slice(&(self.ciphertext.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&self.ciphertext);
        bytes
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 16 {
            return Err(anyhow!("Invalid encrypted data format (too short)"));
        }

        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&bytes[0..12]);

        let ciphertext_len = u32::from_le_bytes(
            bytes[12..16]
                .try_into()
                .map_err(|_| anyhow!("Invalid length field"))?,
        ) as usize;

        if bytes.len() != 16 + ciphertext_len {
            return Err(anyhow!("Invalid encrypted data format (length mismatch)"));
        }

        let ciphertext = bytes[16..16 + ciphertext_len].to_vec();

        Ok(EncryptedData { ciphertext, nonce })
    }
}

/// Encrypt plaintext using AES-256-GCM
pub fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> Result<EncryptedData> {
    let (ciphertext, nonce) = crypto::encrypt_aes256_gcm(key, plaintext)?;
    Ok(EncryptedData { ciphertext, nonce })
}

/// Decrypt ciphertext using AES-256-GCM
pub fn decrypt(key: &[u8; 32], encrypted: &EncryptedData) -> Result<Vec<u8>> {
    crypto::decrypt_aes256_gcm(key, &encrypted.ciphertext, &encrypted.nonce)
}

/// Encrypt data if encryption is enabled
pub fn encrypt_if_enabled(provider: &EncryptionKeyProvider, plaintext: &[u8]) -> Result<Vec<u8>> {
    if !provider.is_enabled() {
        return Ok(plaintext.to_vec());
    }

    let encrypted = encrypt(provider.key(), plaintext)?;
    Ok(encrypted.to_bytes())
}

/// Decrypt data if encryption is enabled
pub fn decrypt_if_enabled(provider: &EncryptionKeyProvider, data: &[u8]) -> Result<Vec<u8>> {
    if !provider.is_enabled() {
        return Ok(data.to_vec());
    }

    let encrypted = EncryptedData::from_bytes(data)?;
    decrypt(provider.key(), &encrypted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let key = EncryptionKeyProvider::generate_new();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_key_hex_roundtrip() {
        let original_key = EncryptionKeyProvider::generate_new();
        let hex = hex::encode(original_key);
        let restored_key = EncryptionKeyProvider::from_hex(&hex).unwrap();
        assert_eq!(original_key, restored_key);
    }

    #[test]
    fn test_encrypted_data_roundtrip() {
        let key = EncryptionKeyProvider::generate_new();
        let plaintext = b"sensitive data";

        let encrypted = encrypt(&key, plaintext).unwrap();
        let bytes = encrypted.to_bytes();
        let restored = EncryptedData::from_bytes(&bytes).unwrap();
        let decrypted = decrypt(&key, &restored).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_encryption_uniqueness() {
        let key = EncryptionKeyProvider::generate_new();
        let plaintext = b"same data";

        let enc1 = encrypt(&key, plaintext).unwrap();
        let enc2 = encrypt(&key, plaintext).unwrap();

        // Different nonces should produce different ciphertexts
        assert_ne!(enc1.nonce, enc2.nonce);
        assert_ne!(enc1.ciphertext, enc2.ciphertext);
    }

    // Test fixture only: a fixed, obviously fake key.
    const TEST_KEY_HEX: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";

    fn sources() -> KeySources {
        KeySources::default()
    }

    #[test]
    fn test_unencrypted_flag_accepts_only_1_or_true() {
        for v in ["1", "true", "TRUE", "True", " true "] {
            let p = EncryptionKeyProvider::from_sources(&KeySources {
                unencrypted: Some(v.into()),
                ..sources()
            })
            .unwrap();
            assert!(!p.is_enabled(), "value {v:?} should enable plaintext");
        }
        for v in ["0", "false", "", "yes", "on", "2"] {
            let r = EncryptionKeyProvider::from_sources(&KeySources {
                unencrypted: Some(v.into()),
                ..sources()
            });
            assert!(r.is_err(), "value {v:?} should be rejected");
        }
    }

    #[test]
    fn test_bad_unencrypted_flag_rejected_even_with_key() {
        let r = EncryptionKeyProvider::from_sources(&KeySources {
            key_hex: Some(TEST_KEY_HEX.into()),
            unencrypted: Some("0".into()),
            ..sources()
        });
        assert!(r.is_err());
    }

    #[test]
    fn test_no_sources_is_error() {
        assert!(EncryptionKeyProvider::from_sources(&sources()).is_err());
    }

    #[test]
    fn test_valid_env_key_enables_encryption() {
        let p = EncryptionKeyProvider::from_sources(&KeySources {
            key_hex: Some(TEST_KEY_HEX.into()),
            ..sources()
        })
        .unwrap();
        assert!(p.is_enabled());
        assert_eq!(p.to_hex(), TEST_KEY_HEX);
    }

    #[test]
    fn test_invalid_env_key_is_hard_error_not_fallthrough() {
        for bad in [
            "abc",
            &TEST_KEY_HEX[..62],
            &format!("{}zz", &TEST_KEY_HEX[..62]),
        ] {
            let r = EncryptionKeyProvider::from_sources(&KeySources {
                key_hex: Some(bad.to_string()),
                unencrypted: Some("1".into()),
                ..sources()
            });
            assert!(
                r.is_err(),
                "bad key {bad:?} must not fall through to plaintext"
            );
        }
    }

    #[test]
    fn test_key_file_valid_and_invalid() {
        let dir = tempfile::tempdir().unwrap();
        let good = dir.path().join("good");
        std::fs::write(&good, format!("{TEST_KEY_HEX}\n")).unwrap();
        let p = EncryptionKeyProvider::from_sources(&KeySources {
            key_file: Some(good),
            ..sources()
        })
        .unwrap();
        assert!(p.is_enabled());

        let bad = dir.path().join("bad");
        std::fs::write(&bad, "not-a-key").unwrap();
        let r = EncryptionKeyProvider::from_sources(&KeySources {
            key_file: Some(bad),
            unencrypted: Some("1".into()),
            ..sources()
        });
        assert!(r.is_err());

        // Explicitly configured but missing file is an error too.
        let r = EncryptionKeyProvider::from_sources(&KeySources {
            key_file: Some(dir.path().join("missing")),
            unencrypted: Some("1".into()),
            ..sources()
        });
        assert!(r.is_err());
    }

    #[test]
    fn test_invalid_default_key_file_is_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".interceptor_key");
        std::fs::write(&path, "garbage").unwrap();
        let r = EncryptionKeyProvider::from_sources(&KeySources {
            default_key_file: Some(path),
            unencrypted: Some("1".into()),
            ..sources()
        });
        assert!(r.is_err());

        // Missing default file is fine and falls through to the opt-out.
        let p = EncryptionKeyProvider::from_sources(&KeySources {
            default_key_file: Some(dir.path().join("nope")),
            unencrypted: Some("1".into()),
            ..sources()
        })
        .unwrap();
        assert!(!p.is_enabled());
    }

    #[test]
    fn test_from_hex_non_ascii_does_not_panic() {
        // 64 bytes total but contains multi-byte characters.
        let s = format!("{}é", &TEST_KEY_HEX[..62]);
        assert_eq!(s.len(), 64);
        assert!(EncryptionKeyProvider::from_hex(&s).is_err());
        assert!(EncryptionKeyProvider::from_hex(&"é".repeat(32)).is_err());
        assert!(EncryptionKeyProvider::from_hex("").is_err());
    }

    #[test]
    fn test_debug_does_not_leak_key() {
        let p =
            EncryptionKeyProvider::with_key(EncryptionKeyProvider::from_hex(TEST_KEY_HEX).unwrap());
        assert!(!format!("{p:?}").contains("0001020304"));
    }

    #[test]
    fn test_disabled_is_plaintext_passthrough() {
        let p = EncryptionKeyProvider::disabled();
        assert_eq!(encrypt_if_enabled(&p, b"x").unwrap(), b"x");
    }
}
