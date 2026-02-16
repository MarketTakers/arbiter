use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};

use crate::db::models::AeadEncrypted;

use super::{aead, CryptoError};

/// Encrypt root key with user password
///
/// Uses Argon2id for password derivation and ChaCha20Poly1305 for encryption
pub fn encrypt_root_key(
    root_key: &[u8; 32],
    password: &str,
    nonce_counter: i32,
) -> Result<(AeadEncrypted, String), CryptoError> {
    // Derive key from password using Argon2
    let (derived_key, salt) = derive_key_from_password(password)?;

    // Generate nonce from counter
    let nonce = aead::nonce_from_counter(nonce_counter);

    // Encrypt root key
    let ciphertext_with_tag = aead::encrypt(root_key, &derived_key, &nonce)?;

    // Extract tag (last 16 bytes)
    let tag_start = ciphertext_with_tag
        .len()
        .checked_sub(16)
        .ok_or_else(|| CryptoError::AeadEncryption("Ciphertext too short".into()))?;

    let ciphertext = ciphertext_with_tag[..tag_start].to_vec();
    let tag = ciphertext_with_tag[tag_start..].to_vec();

    let aead_encrypted = AeadEncrypted {
        id: 1, // Will be set by database
        current_nonce: nonce_counter,
        ciphertext,
        tag,
        schema_version: 1, // Current version
    };

    Ok((aead_encrypted, salt))
}

/// Decrypt root key with user password
///
/// Verifies password hash and decrypts using ChaCha20Poly1305
pub fn decrypt_root_key(
    encrypted: &AeadEncrypted,
    password: &str,
    salt: &str,
) -> Result<[u8; 32], CryptoError> {
    // Derive key from password using stored salt
    let derived_key = derive_key_with_salt(password, salt)?;

    // Generate nonce from counter
    let nonce = aead::nonce_from_counter(encrypted.current_nonce);

    // Reconstruct ciphertext with tag
    let mut ciphertext_with_tag = encrypted.ciphertext.clone();
    ciphertext_with_tag.extend_from_slice(&encrypted.tag);

    // Decrypt
    let plaintext = aead::decrypt(&ciphertext_with_tag, &derived_key, &nonce)?;

    // Verify length
    if plaintext.len() != 32 {
        return Err(CryptoError::InvalidKey(format!(
            "Expected 32 bytes, got {}",
            plaintext.len()
        )));
    }

    // Convert to fixed-size array
    let mut root_key = [0u8; 32];
    root_key.copy_from_slice(&plaintext);

    Ok(root_key)
}

/// Derive 32-byte key from password using Argon2id
///
/// Generates new random salt and returns (derived_key, salt_string)
fn derive_key_from_password(password: &str) -> Result<([u8; 32], String), CryptoError> {
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| CryptoError::KeyDerivation(e.to_string()))?;

    // Extract hash output (32 bytes)
    let hash_output = password_hash
        .hash
        .ok_or_else(|| CryptoError::KeyDerivation("No hash output".into()))?;

    let hash_bytes = hash_output.as_bytes();

    if hash_bytes.len() != 32 {
        return Err(CryptoError::KeyDerivation(format!(
            "Expected 32 bytes, got {}",
            hash_bytes.len()
        )));
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(hash_bytes);

    Ok((key, salt.to_string()))
}

/// Derive 32-byte key from password using existing salt
fn derive_key_with_salt(password: &str, salt_str: &str) -> Result<[u8; 32], CryptoError> {
    let argon2 = Argon2::default();

    // Parse salt
    let salt =
        SaltString::from_b64(salt_str).map_err(|e| CryptoError::InvalidKey(e.to_string()))?;

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| CryptoError::KeyDerivation(e.to_string()))?;

    // Extract hash output
    let hash_output = password_hash
        .hash
        .ok_or_else(|| CryptoError::KeyDerivation("No hash output".into()))?;

    let hash_bytes = hash_output.as_bytes();

    if hash_bytes.len() != 32 {
        return Err(CryptoError::KeyDerivation(format!(
            "Expected 32 bytes, got {}",
            hash_bytes.len()
        )));
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(hash_bytes);

    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_key_encrypt_decrypt_round_trip() {
        let root_key = [42u8; 32];
        let password = "super_secret_password_123";
        let nonce_counter = 1;

        // Encrypt
        let (encrypted, salt) =
            encrypt_root_key(&root_key, password, nonce_counter).expect("Encryption failed");

        // Verify structure
        assert_eq!(encrypted.current_nonce, nonce_counter);
        assert_eq!(encrypted.schema_version, 1);
        assert_eq!(encrypted.tag.len(), 16); // AEAD tag size

        // Decrypt
        let decrypted =
            decrypt_root_key(&encrypted, password, &salt).expect("Decryption failed");

        // Verify round-trip
        assert_eq!(decrypted, root_key);
    }

    #[test]
    fn test_decrypt_with_wrong_password() {
        let root_key = [99u8; 32];
        let correct_password = "correct_password";
        let wrong_password = "wrong_password";
        let nonce_counter = 1;

        // Encrypt with correct password
        let (encrypted, salt) =
            encrypt_root_key(&root_key, correct_password, nonce_counter).expect("Encryption failed");

        // Attempt decrypt with wrong password
        let result = decrypt_root_key(&encrypted, wrong_password, &salt);

        // Should fail due to authentication tag mismatch
        assert!(result.is_err());
    }

    #[test]
    fn test_password_derivation_different_salts() {
        let password = "same_password";

        // Derive key twice - should produce different salts
        let (key1, salt1) = derive_key_from_password(password).expect("Derivation 1 failed");
        let (key2, salt2) = derive_key_from_password(password).expect("Derivation 2 failed");

        // Salts should be different (randomly generated)
        assert_ne!(salt1, salt2);

        // Keys should be different (due to different salts)
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_password_derivation_with_same_salt() {
        let password = "test_password";

        // Generate key and salt
        let (key1, salt) = derive_key_from_password(password).expect("Derivation failed");

        // Derive key again with same salt
        let key2 = derive_key_with_salt(password, &salt).expect("Re-derivation failed");

        // Keys should be identical
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_different_nonce_produces_different_ciphertext() {
        let root_key = [77u8; 32];
        let password = "password123";

        let (encrypted1, salt1) = encrypt_root_key(&root_key, password, 1).expect("Encryption 1 failed");
        let (encrypted2, salt2) = encrypt_root_key(&root_key, password, 2).expect("Encryption 2 failed");

        // Different nonces should produce different ciphertexts
        assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);

        // But both should decrypt correctly
        let decrypted1 = decrypt_root_key(&encrypted1, password, &salt1).expect("Decryption 1 failed");
        let decrypted2 = decrypt_root_key(&encrypted2, password, &salt2).expect("Decryption 2 failed");

        assert_eq!(decrypted1, root_key);
        assert_eq!(decrypted2, root_key);
    }
}
