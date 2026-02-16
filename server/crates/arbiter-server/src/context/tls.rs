use std::string::FromUtf8Error;

use miette::Diagnostic;
use rcgen::{Certificate, KeyPair};
use rustls::pki_types::CertificateDer;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
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

        let key = String::from_utf8(self.key.clone()).map_err(TlsInitError::KeyInvalidFormat)?;

        let keypair = KeyPair::from_pem(&key).map_err(TlsInitError::KeyDeserializationError)?;

        Ok(TlsData { cert, keypair })
    }
}

fn generate_cert(key: &KeyPair) -> Result<Certificate, rcgen::Error> {
    let params =
        rcgen::CertificateParams::new(vec!["arbiter.local".to_string(), "localhost".to_string()])?;

    params.self_signed(key)
}

// TODO: Implement cert rotation
pub struct TlsManager {
    data: TlsData,
}

impl TlsManager {
    pub async fn new(data: Option<TlsDataRaw>) -> Result<Self, TlsInitError> {
        match data {
            Some(raw) => {
                let tls_data = raw.deserialize()?;
                Ok(Self { data: tls_data })
            }
            None => {
                let keypair = KeyPair::generate()?;
                let cert = generate_cert(&keypair)?;
                let tls_data = TlsData {
                    cert: cert.der().clone(),
                    keypair,
                };
                Ok(Self { data: tls_data })
            }
        }
    }

    pub fn bytes(&self) -> TlsDataRaw {
        TlsDataRaw::serialize(&self.data)
    }
}
