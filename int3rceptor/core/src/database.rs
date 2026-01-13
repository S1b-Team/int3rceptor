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

/// Master key provider - handles encryption key generation and management
#[derive(Debug)]
pub struct EncryptionKeyProvider {
    /// Master encryption key (256-bit)
    master_key: [u8; 32],
    /// Whether encryption is enabled
    enabled: bool,
}

impl EncryptionKeyProvider {
    /// Create new key provider from environment variable or generate new key
    pub fn new() -> Result<Self> {
        // Try to load from environment variable
        if let Ok(key_hex) = std::env::var("INTERCEPTOR_ENCRYPTION_KEY") {
            if let Ok(key) = Self::from_hex(&key_hex) {
                return Ok(Self {
                    master_key: key,
                    enabled: true,
                });
            }
        }

        // Try to load from file
        let key_path = std::env::var("INTERCEPTOR_ENCRYPTION_KEY_FILE")
            .unwrap_or_else(|_| ".interceptor_key".to_string());

        if Path::new(&key_path).exists() {
            let key_hex = std::fs::read_to_string(&key_path)?;
            let key = Self::from_hex(key_hex.trim())?;
            return Ok(Self {
                master_key: key,
                enabled: true,
            });
        }

        // If INTERCEPTOR_UNENCRYPTED is set, allow unencrypted mode (dev/testing only)
        if std::env::var("INTERCEPTOR_UNENCRYPTED").is_ok() {
            return Ok(Self {
                master_key: [0u8; 32],
                enabled: false,
            });
        }

        Err(anyhow!(
            "No encryption key found. Set INTERCEPTOR_ENCRYPTION_KEY, \
             INTERCEPTOR_ENCRYPTION_KEY_FILE, or INTERCEPTOR_UNENCRYPTED."
        ))
    }

    /// Parse hex string to 32-byte key
    pub fn from_hex(hex: &str) -> Result<[u8; 32]> {
        if hex.len() != 64 {
            return Err(anyhow!("Key must be 64 hex characters (256 bits)"));
        }

        let mut key = [0u8; 32];
        for i in 0..32 {
            let byte_hex = &hex[i * 2..i * 2 + 2];
            key[i] = u8::from_str_radix(byte_hex, 16)
                .map_err(|_| anyhow!("Invalid hex in key"))?;
        }
        Ok(key)
    }

    /// Convert key to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.master_key)
    }

    /// Generate new random encryption key
    pub fn generate_new() -> [u8; 32] {
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

impl Default for EncryptionKeyProvider {
    fn default() -> Self {
        Self::new().unwrap_or(Self {
            master_key: [0u8; 32],
            enabled: false,
        })
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
            bytes[12..16].try_into().map_err(|_| anyhow!("Invalid length field"))?
        ) as usize;

        if bytes.len() != 16 + ciphertext_len {
            return Err(anyhow!(
                "Invalid encrypted data format (length mismatch)"
            ));
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
pub fn encrypt_if_enabled(
    provider: &EncryptionKeyProvider,
    plaintext: &[u8],
) -> Result<Vec<u8>> {
    if !provider.is_enabled() {
        return Ok(plaintext.to_vec());
    }

    let encrypted = encrypt(provider.key(), plaintext)?;
    Ok(encrypted.to_bytes())
}

/// Decrypt data if encryption is enabled
pub fn decrypt_if_enabled(
    provider: &EncryptionKeyProvider,
    data: &[u8],
) -> Result<Vec<u8>> {
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
}
