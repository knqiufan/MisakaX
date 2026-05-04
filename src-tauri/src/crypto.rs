use aes_gcm::aead::{Aead, OsRng};
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use hkdf::Hkdf;
use rand::RngCore;
use sha2::Sha256;

const APP_SALT: &[u8] = b"misakax-api-key-encryption-v1";
const NONCE_LEN: usize = 12;

fn derive_key() -> [u8; 32] {
    let machine_seed = get_machine_seed();
    let hk = Hkdf::<Sha256>::new(Some(APP_SALT), &machine_seed);
    let mut key = [0u8; 32];
    hk.expand(b"aes-256-gcm-key", &mut key)
        .expect("HKDF expand should not fail for 32 bytes");
    key
}

fn get_machine_seed() -> Vec<u8> {
    let mut parts = Vec::new();

    if let Ok(dir) = crate::config::config_dir() {
        parts.extend_from_slice(dir.to_string_lossy().as_bytes());
    }

    parts.extend_from_slice(b"misakax-desktop-agent");

    if let Ok(hostname) = std::env::var("COMPUTERNAME").or_else(|_| std::env::var("HOSTNAME")) {
        parts.extend_from_slice(hostname.as_bytes());
    }

    parts
}

pub fn encrypt(plaintext: &str) -> anyhow::Result<String> {
    let key = derive_key();
    let cipher = Aes256Gcm::new_from_slice(&key)?;

    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    let mut combined = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(BASE64.encode(&combined))
}

pub fn decrypt(encoded: &str) -> anyhow::Result<String> {
    let key = derive_key();
    let cipher = Aes256Gcm::new_from_slice(&key)?;

    let combined = BASE64
        .decode(encoded)
        .map_err(|e| anyhow::anyhow!("Base64 decode failed: {}", e))?;

    if combined.len() < NONCE_LEN {
        return Err(anyhow::anyhow!("Encrypted data too short"));
    }

    let (nonce_bytes, ciphertext) = combined.split_at(NONCE_LEN);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

    String::from_utf8(plaintext).map_err(|e| anyhow::anyhow!("UTF-8 decode failed: {}", e))
}

pub fn mask_api_key(key: &str) -> String {
    if key.len() <= 8 {
        return "*".repeat(key.len());
    }
    let prefix = &key[..4];
    let suffix = &key[key.len() - 4..];
    format!("{}...{}", prefix, suffix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let original = "sk-test-api-key-12345678";
        let encrypted = encrypt(original).unwrap();
        assert_ne!(encrypted, original);
        let decrypted = decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, original);
    }

    #[test]
    fn test_encrypt_produces_different_ciphertext() {
        let original = "sk-test-key";
        let enc1 = encrypt(original).unwrap();
        let enc2 = encrypt(original).unwrap();
        assert_ne!(enc1, enc2, "Each encryption should use a unique nonce");
    }

    #[test]
    fn test_mask_api_key() {
        assert_eq!(mask_api_key("sk-1234567890abcdef"), "sk-1...cdef");
        assert_eq!(mask_api_key("short"), "*****");
        assert_eq!(mask_api_key("12345678"), "1234...5678");
    }
}
