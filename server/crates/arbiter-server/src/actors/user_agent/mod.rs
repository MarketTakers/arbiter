use alloy::primitives::Address;
use arbiter_proto::{transport::Bi};
use kameo::actor::Spawn as _;
use tracing::{error, info};

use crate::{
    actors::{GlobalActors, evm, user_agent::session::UserAgentSession},
    db::{self, models::KeyType}, evm::policies::{Grant, SpecificGrant},
    evm::policies::SharedGrantSettings,
};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum TransportResponseError {
    #[error("Unexpected request payload")]
    UnexpectedRequestPayload,
    #[error("Invalid state for unseal encrypted key")]
    InvalidStateForUnsealEncryptedKey,
    #[error("client_pubkey must be 32 bytes")]
    InvalidClientPubkeyLength,
    #[error("State machine error")]
    StateTransitionFailed,
    #[error("Vault is not available")]
    KeyHolderActorUnreachable,
    #[error(transparent)]
    Auth(#[from] auth::Error),
    #[error("Failed registering connection")]
    ConnectionRegistrationFailed,
}

/// Abstraction over Ed25519 / ECDSA-secp256k1 / RSA public keys used during the auth handshake.
#[derive(Clone, Debug)]
pub enum AuthPublicKey {
    Ed25519(ed25519_dalek::VerifyingKey),
    /// Compressed SEC1 public key; signature bytes are raw 64-byte (r||s).
    EcdsaSecp256k1(k256::ecdsa::VerifyingKey),
    /// RSA-2048+ public key (Windows Hello / KeyCredentialManager); signature bytes are PSS+SHA-256.
    Rsa(rsa::RsaPublicKey),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnsealError {
    InvalidKey,
    Unbootstrapped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootstrapError {
    AlreadyBootstrapped,
    InvalidKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultState {
    Unbootstrapped,
    Sealed,
    Unsealed,
}

#[derive(Debug, Clone)]
pub enum Request {
    AuthChallengeRequest {
        pubkey: AuthPublicKey,
        bootstrap_token: Option<String>,
    },
    AuthChallengeSolution {
        signature: Vec<u8>,
    },
    UnsealStart {
        client_pubkey: x25519_dalek::PublicKey,
    },
    UnsealEncryptedKey {
        nonce: Vec<u8>,
        ciphertext: Vec<u8>,
        associated_data: Vec<u8>,
    },
    BootstrapEncryptedKey {
        nonce: Vec<u8>,
        ciphertext: Vec<u8>,
        associated_data: Vec<u8>,
    },
    QueryVaultState,
    EvmWalletCreate,
    EvmWalletList,
    ClientConnectionResponse {
        approved: bool,
    },

    ListGrants,
    EvmGrantCreate {
        client_id: i32,
        shared: SharedGrantSettings,
        specific: SpecificGrant,
    },
    EvmGrantDelete {
        grant_id: i32,
    },
}

#[derive(Debug)]
pub enum Response {
    AuthChallenge { nonce: i32 },
    AuthOk,
    UnsealStartResponse { server_pubkey: x25519_dalek::PublicKey },
    UnsealResult(Result<(), UnsealError>),
    BootstrapResult(Result<(), BootstrapError>),
    VaultState(VaultState),
    ClientConnectionRequest { pubkey: ed25519_dalek::VerifyingKey },
    ClientConnectionCancel,
    EvmWalletCreate(Result<(), evm::Error>),
    EvmWalletList(Vec<Address>),

    ListGrants(Vec<Grant<SpecificGrant>>),
    EvmGrantCreate(Result<i32, evm::Error>),
    EvmGrantDelete(Result<(), evm::Error>),
}

pub type Transport = Box<dyn Bi<Request, Result<Response, TransportResponseError>> + Send>;

pub struct UserAgentConnection {
    db: db::DatabasePool,
    actors: GlobalActors,
    transport: Transport,
}

impl UserAgentConnection {
    pub fn new(db: db::DatabasePool, actors: GlobalActors, transport: Transport) -> Self {
        Self {
            db,
            actors,
            transport,
        }
    }
}

pub mod auth;
pub mod session;

#[tracing::instrument(skip(props))]
pub async fn connect_user_agent(props: UserAgentConnection) {
    match auth::authenticate_and_create(props).await {
        Ok(session) => {
            UserAgentSession::spawn(session);
            info!("User authenticated, session started");
        }
        Err(err) => {
            error!(?err, "Authentication failed, closing connection");
        }
    }
}
