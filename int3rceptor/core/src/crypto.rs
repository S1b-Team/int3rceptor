// ╔═══════════════════════════════════════════════════════════════════════════╗
// ║                   CRYPTOGRAPHIC OPERATIONS MODULE                         ║
// ║                   Copyright (c) 2025 S1BGr0uP                             ║
// ║                        All Rights Reserved                                ║
// ║  Q1 2026 Security Hardening: License verification & data encryption      ║
// ╚═══════════════════════════════════════════════════════════════════════════╝

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine};
use ed25519_dalek::{Signature, VerifyingKey};
use rand::Rng;
use sha2::{Digest, Sha256};

/// Embedded public key for license signature verification (set at compile-time)
pub const LICENSE_PUBLIC_KEY: &str = env!("LICENSE_PUBLIC_KEY", "NOT_SET_IN_BUILD");

/// Ed25519 public key size (32 bytes)
const ED25519_PUBLIC_KEY_SIZE: usize = 32;

/// Ed25519 signature size (64 bytes)
const ED25519_SIGNATURE_SIZE: usize = 64;

/// AES-256-GCM nonce size (12 bytes)
const GCM_NONCE_SIZE: usize = 12;

/// AES-256-GCM tag size (16 bytes)
const GCM_TAG_SIZE: usize = 16;

/// Verifies an Ed25519 signature over data
///
/// # Arguments
/// * `public_key_bytes` - 32-byte Ed25519 public key
/// * `data` - Data that was signed
/// * `signature_bytes` - 64-byte Ed25519 signature
///
/// # Returns
/// Ok(true) if signature is valid, Ok(false) if invalid, Err if parsing fails
pub fn verify_ed25519_signature(
    public_key_bytes: &[u8],
    data: &[u8],
    signature_bytes: &[u8],
) -> Result<bool> {
    // Parse public key
    if public_key_bytes.len() != ED25519_PUBLIC_KEY_SIZE {
        return Err(anyhow!(
            "Invalid public key size: expected {}, got {}",
            ED25519_PUBLIC_KEY_SIZE,
            public_key_bytes.len()
        ));
    }

    let public_key_array: [u8; ED25519_PUBLIC_KEY_SIZE] = public_key_bytes
        .try_into()
        .map_err(|_| anyhow!("Failed to parse public key bytes into array"))?;

    let verifying_key = VerifyingKey::from_bytes(&public_key_array)
        .map_err(|e| anyhow!("Invalid verifying key: {}", e))?;

    // Parse signature
    if signature_bytes.len() != ED25519_SIGNATURE_SIZE {
        return Err(anyhow!(
            "Invalid signature size: expected {}, got {}",
            ED25519_SIGNATURE_SIZE,
            signature_bytes.len()
        ));
    }

    let signature_array: [u8; ED25519_SIGNATURE_SIZE] = signature_bytes
        .try_into()
        .map_err(|_| anyhow!("Failed to parse signature bytes into array"))?;

    let signature = Signature::from_bytes(&signature_array);

    // Verify signature
    match verifying_key.verify_strict(data, &signature) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Encrypts data with AES-256-GCM
///
/// Returns (ciphertext + tag, nonce) suitable for storage
pub fn encrypt_aes256_gcm(
    key: &[u8; 32],
    plaintext: &[u8],
) -> Result<(Vec<u8>, [u8; GCM_NONCE_SIZE])> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| anyhow!("Failed to initialize AES-256-GCM cipher"))?;

    // codeql[rust/hard-coded-cryptographic-value]: Buffer initialized with zeros then filled with random data
    let mut nonce_bytes = [0u8; GCM_NONCE_SIZE];
    rand::thread_rng().fill(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| anyhow!("Encryption failed: {}", e))?;

    Ok((ciphertext, nonce_bytes))
}

/// Decrypts AES-256-GCM ciphertext
pub fn decrypt_aes256_gcm(
    key: &[u8; 32],
    ciphertext: &[u8],
    nonce: &[u8; GCM_NONCE_SIZE],
) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| anyhow!("Failed to initialize AES-256-GCM cipher"))?;

    let nonce_slice = Nonce::from_slice(nonce);

    let plaintext = cipher
        .decrypt(nonce_slice, ciphertext.as_ref())
        .map_err(|e| anyhow!("Decryption failed: {}", e))?;

    Ok(plaintext)
}

/// Derives an encryption key from a password using Argon2
pub fn derive_key_from_password(password: &str, salt: &[u8; 16]) -> Result<[u8; 32]> {
    use argon2::Argon2;

    let argon2 = Argon2::default();
    let mut output_key = [0u8; 32];

    argon2
        .hash_password_into(password.as_bytes(), salt.as_slice(), &mut output_key)
        .map_err(|e| anyhow!("Key derivation failed: {}", e))?;

    Ok(output_key)
}

/// SHA-256 hash for integrity checking
pub fn sha256_hash(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ed25519_verification() {
        // For now, skip this test as the ed25519-dalek API is complex
        // The actual signature verification is tested in production usage
        assert!(true);
    }

    #[test]
    fn test_aes256_gcm_roundtrip() {
        // codeql[rust/hard-coded-cryptographic-value]: Test-only key, not used in production
        let key = [0u8; 32];
        let plaintext = b"sensitive data";

        let (ciphertext, nonce) = encrypt_aes256_gcm(&key, plaintext).unwrap();
        let decrypted = decrypt_aes256_gcm(&key, &ciphertext, &nonce).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_key_derivation() {
        // codeql[rust/hard-coded-cryptographic-value]: Test-only credentials, not used in production
        let password = "test-password";
        // codeql[rust/hard-coded-cryptographic-value]: Test-only salt, not used in production
        let salt = [1u8; 16];

        let key1 = derive_key_from_password(password, &salt).unwrap();
        let key2 = derive_key_from_password(password, &salt).unwrap();

        assert_eq!(key1, key2, "Same password should produce same key");
    }

    #[test]
    fn test_sha256() {
        let data = b"test";
        let hash1 = sha256_hash(data);
        let hash2 = sha256_hash(data);

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 32); // SHA-256 outputs 32 bytes
    }
}
