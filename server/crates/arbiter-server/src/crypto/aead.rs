use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};

use super::CryptoError;

/// Encrypt plaintext with AEAD (ChaCha20Poly1305)
///
/// Returns (ciphertext, tag) on success
pub fn encrypt(
    plaintext: &[u8],
    key: &[u8; 32],
    nonce: &[u8; 12],
) -> Result<Vec<u8>, CryptoError> {
    let cipher_key = Key::from_slice(key);
    let cipher = ChaCha20Poly1305::new(cipher_key);
    let nonce_array = Nonce::from_slice(nonce);

    cipher
        .encrypt(nonce_array, plaintext)
        .map_err(|e| CryptoError::AeadEncryption(e.to_string()))
}

/// Decrypt ciphertext with AEAD (ChaCha20Poly1305)
///
/// The ciphertext должен содержать tag (последние 16 bytes)
pub fn decrypt(
    ciphertext_with_tag: &[u8],
    key: &[u8; 32],
    nonce: &[u8; 12],
) -> Result<Vec<u8>, CryptoError> {
    let cipher_key = Key::from_slice(key);
    let cipher = ChaCha20Poly1305::new(cipher_key);
    let nonce_array = Nonce::from_slice(nonce);

    cipher
        .decrypt(nonce_array, ciphertext_with_tag)
        .map_err(|e| CryptoError::AeadDecryption(e.to_string()))
}

/// Generate nonce from counter
///
/// Converts i32 counter to 12-byte nonce (big-endian encoding)
pub fn nonce_from_counter(counter: i32) -> [u8; 12] {
    let mut nonce = [0u8; 12];
    nonce[8..12].copy_from_slice(&counter.to_be_bytes());
    nonce
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aead_encrypt_decrypt_round_trip() {
        let plaintext = b"Hello, World! This is a secret message.";
        let key = [42u8; 32];
        let nonce = nonce_from_counter(1);

        // Encrypt
        let ciphertext = encrypt(plaintext, &key, &nonce).expect("Encryption failed");

        // Verify ciphertext is different from plaintext
        assert_ne!(ciphertext.as_slice(), plaintext);

        // Decrypt
        let decrypted = decrypt(&ciphertext, &key, &nonce).expect("Decryption failed");

        // Verify round-trip
        assert_eq!(decrypted.as_slice(), plaintext);
    }

    #[test]
    fn test_aead_decrypt_with_wrong_key() {
        let plaintext = b"Secret data";
        let key = [1u8; 32];
        let wrong_key = [2u8; 32];
        let nonce = nonce_from_counter(1);

        let ciphertext = encrypt(plaintext, &key, &nonce).expect("Encryption failed");

        // Attempt decrypt with wrong key
        let result = decrypt(&ciphertext, &wrong_key, &nonce);

        // Should fail
        assert!(result.is_err());
    }

    #[test]
    fn test_aead_decrypt_with_wrong_nonce() {
        let plaintext = b"Secret data";
        let key = [1u8; 32];
        let nonce = nonce_from_counter(1);
        let wrong_nonce = nonce_from_counter(2);

        let ciphertext = encrypt(plaintext, &key, &nonce).expect("Encryption failed");

        // Attempt decrypt with wrong nonce
        let result = decrypt(&ciphertext, &key, &wrong_nonce);

        // Should fail
        assert!(result.is_err());
    }

    #[test]
    fn test_nonce_generation_from_counter() {
        let nonce1 = nonce_from_counter(1);
        let nonce2 = nonce_from_counter(2);
        let nonce_max = nonce_from_counter(i32::MAX);

        // Verify nonces are different
        assert_ne!(nonce1, nonce2);

        // Verify nonce format (first 8 bytes should be zero, last 4 contain counter)
        assert_eq!(&nonce1[0..8], &[0u8; 8]);
        assert_eq!(&nonce1[8..12], &1i32.to_be_bytes());

        assert_eq!(&nonce_max[8..12], &i32::MAX.to_be_bytes());
    }

    #[test]
    fn test_aead_tampered_ciphertext() {
        let plaintext = b"Important message";
        let key = [7u8; 32];
        let nonce = nonce_from_counter(5);

        let mut ciphertext = encrypt(plaintext, &key, &nonce).expect("Encryption failed");

        // Tamper with ciphertext (flip a bit)
        if let Some(byte) = ciphertext.get_mut(5) {
            *byte ^= 0x01;
        }

        // Attempt decrypt - should fail due to authentication tag mismatch
        let result = decrypt(&ciphertext, &key, &nonce);
        assert!(result.is_err());
    }
}
