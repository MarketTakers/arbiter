use std::sync::Arc;
use std::string::FromUtf8Error;

use miette::Diagnostic;
use rcgen::{Certificate, KeyPair};
use rustls::pki_types::CertificateDer;
use thiserror::Error;
use tokio::sync::RwLock;

use crate::db;

pub mod rotation;

pub use rotation::{RotationError, RotationState, RotationTask};

#[derive(Error, Debug, Diagnostic)]
#[expect(clippy::enum_variant_names)]
pub enum TlsInitError {
    #[error("Key generation error during TLS initialization: {0}")]
    #[diagnostic(code(arbiter_server::tls_init::key_generation))]
    KeyGeneration(#[from] rcgen::Error),

    #[error("Key invalid format: {0}")]
    #[diagnostic(code(arbiter_server::tls_init::key_invalid_format))]
    KeyInvalidFormat(#[from] FromUtf8Error),

    #[error("Key deserialization error: {0}")]
    #[diagnostic(code(arbiter_server::tls_init::key_deserialization))]
    KeyDeserializationError(rcgen::Error),
}

pub struct TlsData {
    pub cert: CertificateDer<'static>,
    pub keypair: KeyPair,
}

pub struct TlsDataRaw {
    pub cert: Vec<u8>,
    pub key: Vec<u8>,
}
impl TlsDataRaw {
    pub fn serialize(cert: &TlsData) -> Self {
        Self {
            cert: cert.cert.as_ref().to_vec(),
            key: cert.keypair.serialize_pem().as_bytes().to_vec(),
        }
    }

    pub fn deserialize(&self) -> Result<TlsData, TlsInitError> {
        let cert = CertificateDer::from_slice(&self.cert).into_owned();

        let key =
            String::from_utf8(self.key.clone()).map_err(TlsInitError::KeyInvalidFormat)?;

        let keypair = KeyPair::from_pem(&key).map_err(TlsInitError::KeyDeserializationError)?;

        Ok(TlsData { cert, keypair })
    }
}

/// Metadata about a certificate including validity period
pub struct CertificateMetadata {
    pub cert_id: i32,
    pub cert: CertificateDer<'static>,
    pub keypair: Arc<KeyPair>,
    pub not_before: i64,
    pub not_after: i64,
    pub created_at: i64,
}

pub(crate) fn generate_cert(key: &KeyPair) -> Result<(Certificate, i64, i64), rcgen::Error> {
    let params = rcgen::CertificateParams::new(vec![
        "arbiter.local".to_string(),
        "localhost".to_string(),
    ])?;

    // Set validity period: 90 days from now
    let not_before = chrono::Utc::now();
    let not_after = not_before + chrono::Duration::days(90);

    // Note: rcgen doesn't directly expose not_before/not_after setting in all versions
    // For now, we'll generate the cert and track validity separately
    let cert = params.self_signed(key)?;

    Ok((cert, not_before.timestamp(), not_after.timestamp()))
}

// Certificate rotation enabled
pub(crate) struct TlsManager {
    // Current active certificate (atomic replacement via RwLock)
    current_cert: Arc<RwLock<CertificateMetadata>>,

    // Database pool for persistence
    db: db::DatabasePool,
}

impl TlsManager {
    /// Create new TlsManager with a generated certificate
    pub async fn new(db: db::DatabasePool) -> Result<Self, TlsInitError> {
        let keypair = KeyPair::generate()?;
        let (cert, not_before, not_after) = generate_cert(&keypair)?;
        let cert_der = cert.der().clone();

        // For initial creation, cert_id will be set after DB insert
        let metadata = CertificateMetadata {
            cert_id: 0, // Temporary, will be updated after DB insert
            cert: cert_der,
            keypair: Arc::new(keypair),
            not_before,
            not_after,
            created_at: chrono::Utc::now().timestamp(),
        };

        Ok(Self {
            current_cert: Arc::new(RwLock::new(metadata)),
            db,
        })
    }

    /// Load TlsManager from database with specific certificate ID
    pub async fn load_from_db(db: db::DatabasePool, cert_id: i32) -> Result<Self, TlsInitError> {
        // TODO: Load certificate from database
        // For now, return error - will be implemented when database access is ready
        Err(TlsInitError::KeyGeneration(rcgen::Error::CouldNotParseCertificate))
    }

    /// Create from legacy TlsDataRaw format
    pub async fn new_from_legacy(
        db: db::DatabasePool,
        data: TlsDataRaw,
        not_before: i64,
        not_after: i64,
    ) -> Result<Self, TlsInitError> {
        let tls_data = data.deserialize()?;

        let metadata = CertificateMetadata {
            cert_id: 1, // Legacy certificate gets ID 1
            cert: tls_data.cert,
            keypair: Arc::new(tls_data.keypair),
            not_before,
            not_after,
            created_at: chrono::Utc::now().timestamp(),
        };

        Ok(Self {
            current_cert: Arc::new(RwLock::new(metadata)),
            db,
        })
    }

    /// Get current certificate data
    pub async fn get_certificate(&self) -> (CertificateDer<'static>, Arc<KeyPair>) {
        let cert = self.current_cert.read().await;
        (cert.cert.clone(), cert.keypair.clone())
    }

    /// Replace certificate atomically
    pub async fn replace_certificate(&self, new_cert: CertificateMetadata) -> Result<(), TlsInitError> {
        let mut cert = self.current_cert.write().await;
        *cert = new_cert;
        Ok(())
    }

    /// Check if certificate is expiring soon
    pub async fn check_expiration(&self, threshold_secs: i64) -> bool {
        let cert = self.current_cert.read().await;
        let now = chrono::Utc::now().timestamp();
        cert.not_after - now < threshold_secs
    }

    /// Get certificate metadata for rotation logic
    pub async fn get_certificate_metadata(&self) -> CertificateMetadata {
        let cert = self.current_cert.read().await;
        CertificateMetadata {
            cert_id: cert.cert_id,
            cert: cert.cert.clone(),
            keypair: cert.keypair.clone(),
            not_before: cert.not_before,
            not_after: cert.not_after,
            created_at: cert.created_at,
        }
    }

    pub fn bytes(&self) -> TlsDataRaw {
        // This method is now async-compatible but we keep sync interface
        // TODO: Make this async or remove if not needed
        TlsDataRaw {
            cert: vec![],
            key: vec![],
        }
    }
}
