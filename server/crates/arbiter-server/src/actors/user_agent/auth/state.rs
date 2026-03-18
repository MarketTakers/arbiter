use arbiter_proto::transport::Bi;
use diesel::{ExpressionMethods as _, OptionalExtension as _, QueryDsl, update};
use diesel_async::RunQueryDsl;
use tracing::error;

use super::Error;
use crate::{
    actors::{
        bootstrap::ConsumeToken,
        user_agent::{AuthPublicKey, UserAgentConnection, auth::Outbound},
    },
    db::schema,
};

pub struct ChallengeRequest {
    pub pubkey: AuthPublicKey,
}

pub struct BootstrapAuthRequest {
    pub pubkey: AuthPublicKey,
    pub token: String,
}

pub struct ChallengeContext {
    pub challenge_nonce: i32,
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
        Init + BootstrapAuthRequest(BootstrapAuthRequest) / async verify_bootstrap_token = AuthOk(AuthPublicKey),
        SentChallenge(ChallengeContext) + ReceivedSolution(ChallengeSolution) / async verify_solution = AuthOk(AuthPublicKey),
    }
);

async fn create_nonce(db: &crate::db::DatabasePool, pubkey_bytes: &[u8]) -> Result<i32, Error> {
    let mut db_conn = db.get().await.map_err(|e| {
        error!(error = ?e, "Database pool error");
        Error::internal("Database unavailable")
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
            Error::internal("Database operation failed")
        })?
        .ok_or_else(|| {
            error!(?pubkey_bytes, "Public key not found in database");
            Error::UnregisteredPublicKey
        })
}

async fn register_key(db: &crate::db::DatabasePool, pubkey: &AuthPublicKey) -> Result<(), Error> {
    let pubkey_bytes = pubkey.to_stored_bytes();
    let key_type = pubkey.key_type();
    let mut conn = db.get().await.map_err(|e| {
        error!(error = ?e, "Database pool error");
        Error::internal("Database unavailable")
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
            Error::internal("Database operation failed")
        })?;

    Ok(())
}

pub struct AuthContext<'a, T> {
    pub(super) conn: &'a mut UserAgentConnection,
    pub(super) transport: T,
}

impl<'a, T> AuthContext<'a, T> {
    pub fn new(conn: &'a mut UserAgentConnection, transport: T) -> Self {
        Self { conn, transport }
    }
}

impl<T> AuthStateMachineContext for AuthContext<'_, T>
where
    T: Bi<super::Inbound, Result<super::Outbound, Error>> + Send,
{
    type Error = Error;

    async fn prepare_challenge(
        &mut self,
        ChallengeRequest { pubkey }: ChallengeRequest,
    ) -> Result<ChallengeContext, Self::Error> {
        let stored_bytes = pubkey.to_stored_bytes();
        let nonce = create_nonce(&self.conn.db, &stored_bytes).await?;

        self.transport
            .send(Ok(Outbound::AuthChallenge { nonce }))
            .await
            .map_err(|e| {
                error!(?e, "Failed to send auth challenge");
                Error::Transport
            })?;

        Ok(ChallengeContext {
            challenge_nonce: nonce,
            key: pubkey,
        })
    }

    #[allow(missing_docs)]
    #[allow(clippy::result_unit_err)]
    async fn verify_bootstrap_token(
        &mut self,
        BootstrapAuthRequest { pubkey, token }: BootstrapAuthRequest,
    ) -> Result<AuthPublicKey, Self::Error> {
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
                Error::internal("Failed to consume bootstrap token")
            })?;

        if !token_ok {
            error!("Invalid bootstrap token provided");
            return Err(Error::InvalidBootstrapToken);
        }

        register_key(&self.conn.db, &pubkey).await?;

        self.transport
            .send(Ok(Outbound::AuthSuccess))
            .await
            .map_err(|_| Error::Transport)?;

        Ok(pubkey)
    }

    #[allow(missing_docs)]
    #[allow(clippy::unused_unit)]
    async fn verify_solution(
        &mut self,
        ChallengeContext {
            challenge_nonce,
            key,
        }: &ChallengeContext,
        ChallengeSolution { solution }: ChallengeSolution,
    ) -> Result<AuthPublicKey, Self::Error> {
        let formatted = arbiter_proto::format_challenge(*challenge_nonce, &key.to_stored_bytes());

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
            self.transport
                .send(Ok(Outbound::AuthSuccess))
                .await
                .map_err(|_| Error::Transport)?;
        }

        Ok(key.clone())
    }
}
