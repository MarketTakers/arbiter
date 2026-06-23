use crate::db::{
    self,
    models::{NewTlsHistory, TlsHistory},
    schema::{
        arbiter_settings,
        tls_history::{self},
    },
};

use diesel::{ExpressionMethods as _, QueryDsl, SelectableHelper as _};
use diesel_async::{AsyncConnection, RunQueryDsl};
use pem::Pem;
use rcgen::{
    BasicConstraints, Certificate, CertificateParams, CertifiedIssuer, DistinguishedName, DnType,
    IsCa, Issuer, KeyPair, KeyUsagePurpose, SanType,
};
use rustls::pki_types::pem::PemObject;
use std::{net::Ipv4Addr, string::FromUtf8Error};
use thiserror::Error;
use tonic::transport::CertificateDer;

const ENCODE_CONFIG: pem::EncodeConfig = {
    let line_ending = if cfg!(target_family = "windows") {
        pem::LineEnding::CRLF
    } else {
        pem::LineEnding::LF
    };
    pem::EncodeConfig::new().set_line_ending(line_ending)
};

#[derive(Error, Debug)]
pub enum InitError {
    #[error("Key generation error during TLS initialization: {0}")]
    KeyGeneration(#[from] rcgen::Error),

    #[error("Key invalid format: {0}")]
    KeyInvalidFormat(#[from] FromUtf8Error),

    #[error("Key deserialization error: {0}")]
    KeyDeserializationError(rcgen::Error),

    #[error("Database error during TLS initialization: {0}")]
    DatabaseError(#[from] diesel::result::Error),

    #[error("Pem deserialization error during TLS initialization: {0}")]
    PemDeserializationError(#[from] rustls::pki_types::pem::Error),

    #[error("Database pool acquire error during TLS initialization: {0}")]
    DatabasePoolAcquire(#[from] db::PoolError),
}

pub type PemCert = String;

pub fn encode_cert_to_pem(cert: &CertificateDer<'_>) -> PemCert {
    pem::encode_config(&Pem::new("CERTIFICATE", cert.to_vec()), ENCODE_CONFIG)
}

#[expect(
    unused,
    reason = "may be needed for future cert rotation implementation"
)]
struct SerializedTls {
    cert_pem: PemCert,
    cert_key_pem: String,
}

struct TlsCa {
    issuer: Issuer<'static, KeyPair>,
    cert: CertificateDer<'static>,
}

impl TlsCa {
    fn generate() -> Result<Self, InitError> {
        let keypair = KeyPair::generate()?;
        let mut params = CertificateParams::new(["Arbiter Instance CA".into()])?;
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        params.key_usages = vec![
            KeyUsagePurpose::KeyCertSign,
            KeyUsagePurpose::CrlSign,
            KeyUsagePurpose::DigitalSignature,
        ];

        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, "Arbiter Instance CA");
        params.distinguished_name = dn;
        let certified_issuer = CertifiedIssuer::self_signed(params, keypair)?;

        let cert_key_pem = certified_issuer.key().serialize_pem();

        #[expect(
            clippy::unwrap_used,
            reason = "Broken cert couldn't bootstrap server anyway"
        )]
        let issuer = Issuer::from_ca_cert_pem(
            &certified_issuer.pem(),
            KeyPair::from_pem(cert_key_pem.as_ref()).unwrap(),
        )
        .unwrap();

        Ok(Self {
            issuer,
            cert: certified_issuer.der().clone(),
        })
    }
    fn generate_leaf(&self) -> Result<TlsCert, InitError> {
        let cert_key = KeyPair::generate()?;
        let mut params = CertificateParams::new(["Arbiter Instance Leaf".into()])?;
        params.is_ca = IsCa::NoCa;
        params.key_usages = vec![
            KeyUsagePurpose::DigitalSignature,
            KeyUsagePurpose::KeyEncipherment,
        ];
        params
            .subject_alt_names
            .push(SanType::IpAddress(Ipv4Addr::LOCALHOST.into()));

        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, "Arbiter Instance Leaf");
        params.distinguished_name = dn;

        let new_cert = params.signed_by(&cert_key, &self.issuer)?;

        Ok(TlsCert {
            cert: new_cert,
            cert_key,
        })
    }

    #[expect(
        unused,
        clippy::unnecessary_wraps,
        reason = "may be needed for future cert rotation implementation"
    )]
    fn serialize(&self) -> Result<SerializedTls, InitError> {
        let cert_key_pem = self.issuer.key().serialize_pem();
        Ok(SerializedTls {
            cert_pem: encode_cert_to_pem(&self.cert),
            cert_key_pem,
        })
    }

    #[expect(
        unused,
        reason = "may be needed for future cert rotation implementation"
    )]
    fn try_deserialize(cert_pem: &str, cert_key_pem: &str) -> Result<Self, InitError> {
        let keypair =
            KeyPair::from_pem(cert_key_pem).map_err(InitError::KeyDeserializationError)?;
        let issuer = Issuer::from_ca_cert_pem(cert_pem, keypair)?;
        Ok(Self {
            issuer,
            cert: CertificateDer::from_pem_slice(cert_pem.as_bytes())?,
        })
    }
}

struct TlsCert {
    cert: Certificate,
    cert_key: KeyPair,
}

// TODO: Implement cert rotation
pub struct TlsManager {
    cert: CertificateDer<'static>,
    keypair: KeyPair,
    ca_cert: CertificateDer<'static>,
    _db: db::DatabasePool,
}

impl TlsManager {
    pub async fn generate_new(db: &db::DatabasePool) -> Result<Self, InitError> {
        let ca = TlsCa::generate()?;
        let new_cert = ca.generate_leaf()?;

        {
            let mut conn = db.get().await?;
            conn.transaction(async |conn| {
                let new_tls_history = NewTlsHistory {
                    cert: new_cert.cert.pem(),
                    cert_key: new_cert.cert_key.serialize_pem(),
                    ca_cert: encode_cert_to_pem(&ca.cert),
                    ca_key: ca.issuer.key().serialize_pem(),
                };

                let inserted_tls_history: i32 = diesel::insert_into(tls_history::table)
                    .values(&new_tls_history)
                    .returning(tls_history::id)
                    .get_result(&mut *conn)
                    .await?;

                diesel::update(arbiter_settings::table)
                    .set(arbiter_settings::tls_id.eq(inserted_tls_history))
                    .execute(&mut *conn)
                    .await?;

                Result::<_, diesel::result::Error>::Ok(())
            })
            .await?;
        }

        Ok(Self {
            cert: new_cert.cert.der().clone(),
            keypair: new_cert.cert_key,
            ca_cert: ca.cert,
            _db: db.clone(),
        })
    }

    pub async fn new(db: db::DatabasePool) -> Result<Self, InitError> {
        let cert_data: Option<TlsHistory> = {
            let mut conn = db.get().await?;
            arbiter_settings::table
                .left_join(tls_history::table)
                .select(Option::<TlsHistory>::as_select())
                .first(&mut conn)
                .await?
        };

        match cert_data {
            Some(data) => {
                let try_load = || -> Result<_, Box<dyn std::error::Error>> {
                    let keypair = KeyPair::from_pem(&data.cert_key)?;
                    let cert = CertificateDer::from_pem_slice(data.cert.as_bytes())?;
                    let ca_cert = CertificateDer::from_pem_slice(data.ca_cert.as_bytes())?;
                    Ok(Self {
                        cert,
                        keypair,
                        ca_cert,
                        _db: db.clone(),
                    })
                };
                match try_load() {
                    Ok(manager) => Ok(manager),
                    Err(e) => {
                        eprintln!("Failed to load existing TLS certs: {e}. Generating new ones.");
                        Self::generate_new(&db).await
                    }
                }
            }
            None => Self::generate_new(&db).await,
        }
    }

    pub const fn cert(&self) -> &CertificateDer<'static> {
        &self.cert
    }
    pub const fn ca_cert(&self) -> &CertificateDer<'static> {
        &self.ca_cert
    }

    pub fn cert_pem(&self) -> PemCert {
        encode_cert_to_pem(&self.cert)
    }
    pub fn key_pem(&self) -> String {
        self.keypair.serialize_pem()
    }
}
