use std::time::{SystemTime, UNIX_EPOCH};

use miette::Diagnostic;
use secrecy::{ExposeSecret, SecretBox};
use thiserror::Error;
use x25519_dalek::{PublicKey, StaticSecret};

const EPHEMERAL_KEY_LIFETIME_SECS: u64 = 60;

#[derive(Error, Debug, Diagnostic)]
pub enum UnsealError {
    #[error("Invalid public key")]
    #[diagnostic(code(arbiter_server::unseal::invalid_pubkey))]
    InvalidPublicKey,
}

/// Ephemeral X25519 keypair for secure password transmission
///
/// Generated on-demand when client requests unseal. Expires after 60 seconds.
/// Uses StaticSecret stored in SecretBox for automatic zeroization on drop.
pub struct EphemeralKeyPair {
    /// Secret key stored securely
    secret: SecretBox<StaticSecret>,
    public: PublicKey,
    expires_at: u64,
}

impl EphemeralKeyPair {
    /// Generate new ephemeral X25519 keypair
    pub fn generate() -> Self {
        // Generate random 32 bytes
        let secret_bytes = rand::random::<[u8; 32]>();
        let secret = StaticSecret::from(secret_bytes);
        let public = PublicKey::from(&secret);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time before UNIX epoch")
            .as_secs();

        Self {
            secret: SecretBox::new(Box::new(secret)),
            public,
            expires_at: now + EPHEMERAL_KEY_LIFETIME_SECS,
        }
    }

    /// Check if this ephemeral key has expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time before UNIX epoch")
            .as_secs();

        now > self.expires_at
    }

    /// Get expiration timestamp (Unix epoch seconds)
    pub fn expires_at(&self) -> u64 {
        self.expires_at
    }

    /// Get public key as bytes for transmission to client
    pub fn public_bytes(&self) -> Vec<u8> {
        self.public.as_bytes().to_vec()
    }

    /// Perform Diffie-Hellman key exchange with client's public key
    ///
    /// Returns 32-byte shared secret for ChaCha20Poly1305 encryption
    pub fn perform_dh(&self, client_pubkey: &[u8]) -> Result<[u8; 32], UnsealError> {
        // Parse client public key
        let client_public = PublicKey::from(
            <[u8; 32]>::try_from(client_pubkey).map_err(|_| UnsealError::InvalidPublicKey)?,
        );

        // Perform ECDH
        let shared_secret = self.secret.expose_secret().diffie_hellman(&client_public);

        Ok(shared_secret.to_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ephemeral_keypair_generation() {
        let keypair = EphemeralKeyPair::generate();

        // Public key should be 32 bytes
        assert_eq!(keypair.public_bytes().len(), 32);

        // Should not be expired immediately
        assert!(!keypair.is_expired());

        // Expiration should be ~60 seconds in future
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let time_until_expiry = keypair.expires_at() - now;
        assert!(time_until_expiry >= 59 && time_until_expiry <= 61);
    }

    #[test]
    fn test_perform_dh_with_valid_key() {
        let server_keypair = EphemeralKeyPair::generate();
        let client_secret_bytes = rand::random::<[u8; 32]>();
        let client_secret = StaticSecret::from(client_secret_bytes);
        let client_public = PublicKey::from(&client_secret);

        // Server performs DH
        let server_shared_secret = server_keypair
            .perform_dh(client_public.as_bytes())
            .expect("DH should succeed");

        // Client performs DH
        let client_shared_secret = client_secret.diffie_hellman(&server_keypair.public);

        // Shared secrets should match
        assert_eq!(server_shared_secret, client_shared_secret.to_bytes());
        assert_eq!(server_shared_secret.len(), 32);
    }

    #[test]
    fn test_perform_dh_with_invalid_key() {
        let keypair = EphemeralKeyPair::generate();

        // Try with invalid length
        let invalid_key = vec![1, 2, 3];
        let result = keypair.perform_dh(&invalid_key);
        assert!(result.is_err());

        // Try with wrong length (not 32 bytes)
        let invalid_key = vec![0u8; 16];
        let result = keypair.perform_dh(&invalid_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_different_keypairs_produce_different_shared_secrets() {
        let server_keypair1 = EphemeralKeyPair::generate();
        let server_keypair2 = EphemeralKeyPair::generate();

        let client_secret_bytes = rand::random::<[u8; 32]>();
        let client_secret = StaticSecret::from(client_secret_bytes);
        let client_public = PublicKey::from(&client_secret);

        let shared1 = server_keypair1
            .perform_dh(client_public.as_bytes())
            .unwrap();
        let shared2 = server_keypair2
            .perform_dh(client_public.as_bytes())
            .unwrap();

        // Different server keys should produce different shared secrets
        assert_ne!(shared1, shared2);
    }
}
