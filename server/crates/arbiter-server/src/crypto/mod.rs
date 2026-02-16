pub mod aead;
pub mod root_key;

use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum CryptoError {
    #[error("AEAD encryption failed: {0}")]
    #[diagnostic(code(arbiter_server::crypto::aead_encryption))]
    AeadEncryption(String),

    #[error("AEAD decryption failed: {0}")]
    #[diagnostic(code(arbiter_server::crypto::aead_decryption))]
    AeadDecryption(String),

    #[error("Key derivation failed: {0}")]
    #[diagnostic(code(arbiter_server::crypto::key_derivation))]
    KeyDerivation(String),

    #[error("Invalid nonce: {0}")]
    #[diagnostic(code(arbiter_server::crypto::invalid_nonce))]
    InvalidNonce(String),

    #[error("Invalid key format: {0}")]
    #[diagnostic(code(arbiter_server::crypto::invalid_key))]
    InvalidKey(String),
}
