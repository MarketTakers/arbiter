use arbiter_proto::proto::user_agent::{
    AuthChallenge, AuthOk, UserAgentResponse,
    user_agent_response::Payload as UserAgentResponsePayload,
};
use diesel::{ExpressionMethods as _, OptionalExtension as _, QueryDsl, update};
use diesel_async::RunQueryDsl;
use tracing::error;

use super::Error;
use crate::{
    actors::{bootstrap::ConsumeToken, user_agent::UserAgentConnection},
    db::{models::KeyType, schema},
};

/// Abstraction over Ed25519 / ECDSA-secp256k1 / RSA public keys used during the auth handshake.
#[derive(Clone)]
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

pub struct ChallengeRequest {
    pub pubkey: AuthPublicKey,
}

pub struct BootstrapAuthRequest {
    pub pubkey: AuthPublicKey,
    pub token: String,
}

pub struct ChallengeContext {
    pub challenge: AuthChallenge,
    pub key: AuthPublicKey,
}

pub struct ChallengeSolution {
    pub solution: Vec<u8>,
}

smlang::statemachine!(
    name: Auth,
    custom_error: true,
    transitions: {
        *Init + AuthRequest(ChallengeRequest) / async prepare_challenge = SentChallenge(ChallengeContext),
        Init + BootstrapAuthRequest(BootstrapAuthRequest) [async verify_bootstrap_token] / provide_key_bootstrap = AuthOk(AuthPublicKey),
        SentChallenge(ChallengeContext) + ReceivedSolution(ChallengeSolution) / async verify_solution = AuthOk(AuthPublicKey),
    }
);

async fn create_nonce(db: &crate::db::DatabasePool, pubkey_bytes: &[u8]) -> Result<i32, Error> {
    let mut db_conn = db.get().await.map_err(|e| {
        error!(error = ?e, "Database pool error");
        Error::DatabasePoolUnavailable
    })?;
    db_conn
        .exclusive_transaction(|conn| {
            Box::pin(async move {
                let current_nonce = schema::useragent_client::table
                    .filter(schema::useragent_client::public_key.eq(pubkey_bytes.to_vec()))
                    .select(schema::useragent_client::nonce)
                    .first::<i32>(conn)
                    .await?;

                update(schema::useragent_client::table)
                    .filter(schema::useragent_client::public_key.eq(pubkey_bytes.to_vec()))
                    .set(schema::useragent_client::nonce.eq(current_nonce + 1))
                    .execute(conn)
                    .await?;

                Result::<_, diesel::result::Error>::Ok(current_nonce)
            })
        })
        .await
        .optional()
        .map_err(|e| {
            error!(error = ?e, "Database error");
            Error::DatabaseOperationFailed
        })?
        .ok_or_else(|| {
            error!(?pubkey_bytes, "Public key not found in database");
            Error::PublicKeyNotRegistered
        })
}

async fn register_key(db: &crate::db::DatabasePool, pubkey: &AuthPublicKey) -> Result<(), Error> {
    let pubkey_bytes = pubkey.to_stored_bytes();
    let key_type = pubkey.key_type();
    let mut conn = db.get().await.map_err(|e| {
        error!(error = ?e, "Database pool error");
        Error::DatabasePoolUnavailable
    })?;

    diesel::insert_into(schema::useragent_client::table)
        .values((
            schema::useragent_client::public_key.eq(pubkey_bytes),
            schema::useragent_client::nonce.eq(1),
            schema::useragent_client::key_type.eq(key_type),
        ))
        .execute(&mut conn)
        .await
        .map_err(|e| {
            error!(error = ?e, "Database error");
            Error::DatabaseOperationFailed
        })?;

    Ok(())
}

pub struct AuthContext<'a> {
    pub(super) conn: &'a mut UserAgentConnection,
}

impl<'a> AuthContext<'a> {
    pub fn new(conn: &'a mut UserAgentConnection) -> Self {
        Self { conn }
    }
}

impl AuthStateMachineContext for AuthContext<'_> {
    type Error = Error;

    async fn prepare_challenge(
        &mut self,
        ChallengeRequest { pubkey }: ChallengeRequest,
    ) -> Result<ChallengeContext, Self::Error> {
        let stored_bytes = pubkey.to_stored_bytes();
        let nonce = create_nonce(&self.conn.db, &stored_bytes).await?;

        let challenge = AuthChallenge {
            pubkey: stored_bytes,
            nonce,
        };

        self.conn
            .transport
            .send(Ok(UserAgentResponse {
                payload: Some(UserAgentResponsePayload::AuthChallenge(challenge.clone())),
            }))
            .await
            .map_err(|e| {
                error!(?e, "Failed to send auth challenge");
                Error::Transport
            })?;

        Ok(ChallengeContext {
            challenge,
            key: pubkey,
        })
    }

    #[allow(missing_docs)]
    #[allow(clippy::result_unit_err)]
    async fn verify_bootstrap_token(
        &self,
        BootstrapAuthRequest { pubkey, token }: &BootstrapAuthRequest,
    ) -> Result<bool, Self::Error> {
        let token_ok: bool = self
            .conn
            .actors
            .bootstrapper
            .ask(ConsumeToken {
                token: token.clone(),
            })
            .await
            .map_err(|e| {
                error!(?e, "Failed to consume bootstrap token");
                Error::BootstrapperActorUnreachable
            })?;

        if !token_ok {
            error!("Invalid bootstrap token provided");
            return Err(Error::InvalidBootstrapToken);
        }

        register_key(&self.conn.db, pubkey).await?;

        Ok(true)
    }

    fn provide_key_bootstrap(
        &mut self,
        event_data: BootstrapAuthRequest,
    ) -> Result<AuthPublicKey, Self::Error> {
        Ok(event_data.pubkey)
    }

    #[allow(missing_docs)]
    #[allow(clippy::unused_unit)]
    async fn verify_solution(
        &mut self,
        ChallengeContext { challenge, key  }: &ChallengeContext,
        ChallengeSolution { solution }: ChallengeSolution,
    ) -> Result<AuthPublicKey, Self::Error> {
        let formatted = arbiter_proto::format_challenge(challenge.nonce, &challenge.pubkey);

        let valid = match key {
            AuthPublicKey::Ed25519(vk) => {
                let sig = solution.as_slice().try_into().map_err(|_| {
                    error!(?solution, "Invalid Ed25519 signature length");
                    Error::InvalidChallengeSolution
                })?;
                vk.verify_strict(&formatted, &sig).is_ok()
            }
            AuthPublicKey::EcdsaSecp256k1(vk) => {
                use k256::ecdsa::signature::Verifier as _;
                let sig = k256::ecdsa::Signature::try_from(solution.as_slice()).map_err(|_| {
                    error!(?solution, "Invalid ECDSA signature bytes");
                    Error::InvalidChallengeSolution
                })?;
                vk.verify(&formatted, &sig).is_ok()
            }
            AuthPublicKey::Rsa(pk) => {
                use rsa::signature::Verifier as _;
                let verifying_key = rsa::pss::VerifyingKey::<sha2::Sha256>::new(pk.clone());
                let sig = rsa::pss::Signature::try_from(solution.as_slice()).map_err(|_| {
                    error!(?solution, "Invalid RSA signature bytes");
                    Error::InvalidChallengeSolution
                })?;
                verifying_key.verify(&formatted, &sig).is_ok()
            }
        };

        if valid {
            self.conn
                .transport
                .send(Ok(UserAgentResponse {
                    payload: Some(UserAgentResponsePayload::AuthOk(AuthOk {})),
                }))
                .await
                .map_err(|_| Error::Transport)?;
        }

        Ok(key.clone())
    }
}
