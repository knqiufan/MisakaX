use misaka_x_lib::crypto;

#[test]
fn test_encrypt_decrypt_roundtrip() {
    let original = "sk-test-api-key-12345678";
    let encrypted = crypto::encrypt(original).unwrap();
    assert_ne!(encrypted, original);
    let decrypted = crypto::decrypt(&encrypted).unwrap();
    assert_eq!(decrypted, original);
}

#[test]
fn test_encrypt_produces_different_ciphertext() {
    let original = "sk-test-key";
    let enc1 = crypto::encrypt(original).unwrap();
    let enc2 = crypto::encrypt(original).unwrap();
    assert_ne!(enc1, enc2, "Each encryption should use a unique nonce");
}

#[test]
fn test_mask_api_key() {
    assert_eq!(crypto::mask_api_key("sk-1234567890abcdef"), "sk-1...cdef");
    assert_eq!(crypto::mask_api_key("short"), "*****");
    assert_eq!(crypto::mask_api_key("12345678"), "1234...5678");
}
