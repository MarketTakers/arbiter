use crate::{
    actors::{GlobalActors, client::ClientProfile}, crypto::integrity::Integrable, db::{self, models::KeyType}
};

fn serialize_ecdsa<S>(key: &k256::ecdsa::VerifyingKey, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    // Serialize as hex string for easier debugging (33 bytes compressed SEC1 format)
    let key = key.to_encoded_point(true);
    let bytes = key.as_bytes();
    serializer.serialize_bytes(bytes)
}

fn deserialize_ecdsa<'de, D>(deserializer: D) -> Result<k256::ecdsa::VerifyingKey, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct EcdsaVisitor;

    impl<'de> serde::de::Visitor<'de> for EcdsaVisitor {
        type Value = k256::ecdsa::VerifyingKey;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a compressed SEC1-encoded ECDSA public key")
        }

        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let point = k256::EncodedPoint::from_bytes(v)
                .map_err(|_| E::custom("invalid compressed SEC1 format"))?;
            k256::ecdsa::VerifyingKey::from_encoded_point(&point)
                .map_err(|_| E::custom("invalid ECDSA public key"))
        }
    }

    deserializer.deserialize_bytes(EcdsaVisitor)
}

/// Abstraction over Ed25519 / ECDSA-secp256k1 / RSA public keys used during the auth handshake.
#[derive(Clone, Debug, Serialize)]
pub enum AuthPublicKey {
    Ed25519(ed25519_dalek::VerifyingKey),
    /// Compressed SEC1 public key; signature bytes are raw 64-byte (r||s).
    #[serde(serialize_with = "serialize_ecdsa", deserialize_with = "deserialize_ecdsa")]
    EcdsaSecp256k1(k256::ecdsa::VerifyingKey),
    /// RSA-2048+ public key (Windows Hello / KeyCredentialManager); signature bytes are PSS+SHA-256.
    Rsa(rsa::RsaPublicKey),
}

#[derive(Debug, Serialize)]
pub struct UserAgentCredentials {
    pub pubkey: AuthPublicKey,
    pub nonce: i32
}

impl Integrable for UserAgentCredentials {
    const KIND: &'static str = "useragent_credentials";
}

impl AuthPublicKey {
    /// Canonical bytes stored in DB and echoed back in the challenge.
    /// Ed25519: raw 32 bytes. ECDSA: SEC1 compressed 33 bytes. RSA: DER-encoded SPKI.
    pub fn to_stored_bytes(&self) -> Vec<u8> {
        match self {
            AuthPublicKey::Ed25519(k) => k.to_bytes().to_vec(),
            // SEC1 compressed (33 bytes) is the natural compact format for secp256k1
            AuthPublicKey::EcdsaSecp256k1(k) => k.to_encoded_point(true).as_bytes().to_vec(),
            AuthPublicKey::Rsa(k) => {
                use rsa::pkcs8::EncodePublicKey as _;
                #[allow(clippy::expect_used)]
                k.to_public_key_der()
                    .expect("rsa SPKI encoding is infallible")
                    .to_vec()
            }
        }
    }

    pub fn key_type(&self) -> KeyType {
        match self {
            AuthPublicKey::Ed25519(_) => KeyType::Ed25519,
            AuthPublicKey::EcdsaSecp256k1(_) => KeyType::EcdsaSecp256k1,
            AuthPublicKey::Rsa(_) => KeyType::Rsa,
        }
    }
}

impl TryFrom<(KeyType, Vec<u8>)> for AuthPublicKey {
    type Error = &'static str;

    fn try_from(value: (KeyType, Vec<u8>)) -> Result<Self, Self::Error> {
        let (key_type, bytes) = value;
        match key_type {
            KeyType::Ed25519 => {
                let bytes: [u8; 32] = bytes.try_into().map_err(|_| "invalid Ed25519 key length")?;
                let key = ed25519_dalek::VerifyingKey::from_bytes(&bytes)
                    .map_err(|_e| "invalid Ed25519 key")?;
                Ok(AuthPublicKey::Ed25519(key))
            }
            KeyType::EcdsaSecp256k1 => {
                let point =
                    k256::EncodedPoint::from_bytes(&bytes).map_err(|_e| "invalid ECDSA key")?;
                let key = k256::ecdsa::VerifyingKey::from_encoded_point(&point)
                    .map_err(|_e| "invalid ECDSA key")?;
                Ok(AuthPublicKey::EcdsaSecp256k1(key))
            }
            KeyType::Rsa => {
                use rsa::pkcs8::DecodePublicKey as _;
                let key = rsa::RsaPublicKey::from_public_key_der(&bytes)
                    .map_err(|_e| "invalid RSA key")?;
                Ok(AuthPublicKey::Rsa(key))
            }
        }
    }
}

// Messages, sent by user agent to connection client without having a request
#[derive(Debug)]
pub enum OutOfBand {
    ClientConnectionRequest { profile: ClientProfile },
    ClientConnectionCancel { pubkey: ed25519_dalek::VerifyingKey },
}

pub struct UserAgentConnection {
    pub(crate) db: db::DatabasePool,
    pub(crate) actors: GlobalActors,
}

impl UserAgentConnection {
    pub fn new(db: db::DatabasePool, actors: GlobalActors) -> Self {
        Self { db, actors }
    }
}

pub mod auth;
pub mod session;

pub use auth::authenticate;
use serde::Serialize;
pub use session::UserAgentSession;
