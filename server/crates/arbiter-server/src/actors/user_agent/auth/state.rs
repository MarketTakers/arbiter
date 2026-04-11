use arbiter_proto::transport::Bi;
use diesel::{ExpressionMethods as _, OptionalExtension as _, QueryDsl, update};
use diesel_async::{AsyncConnection, RunQueryDsl};
use kameo::actor::ActorRef;
use tracing::error;

use super::Error;
use crate::{
    actors::{
        bootstrap::ConsumeToken,
        keyholder::KeyHolder,
        user_agent::{AuthPublicKey, UserAgentConnection, UserAgentCredentials, auth::Outbound},
    },
    crypto::integrity,
    db::{DatabasePool, schema::useragent_client},
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

/// Returns the current nonce, ready to use for the challenge nonce.
async fn get_current_nonce_and_id(
    db: &DatabasePool,
    key: &AuthPublicKey,
) -> Result<(i32, i32), Error> {
    let mut db_conn = db
        .get()
        .await
        .map_err(|e| Error::internal("Database unavailable", &e))?;
    db_conn
        .exclusive_transaction(|conn| {
            Box::pin(async move {
                useragent_client::table
                    .filter(useragent_client::public_key.eq(key.to_stored_bytes()))
                    .filter(useragent_client::key_type.eq(key.key_type()))
                    .select((useragent_client::id, useragent_client::nonce))
                    .first::<(i32, i32)>(conn)
                    .await
            })
        })
        .await
        .optional()
        .map_err(|e| Error::internal("Database operation failed", &e))?
        .ok_or_else(|| {
            error!(?key, "Public key not found in database");
            Error::UnregisteredPublicKey
        })
}

async fn verify_integrity(
    db: &DatabasePool,
    keyholder: &ActorRef<KeyHolder>,
    pubkey: &AuthPublicKey,
) -> Result<(), Error> {
    let mut db_conn = db
        .get()
        .await
        .map_err(|e| Error::internal("Database unavailable", &e))?;

    let (id, nonce) = get_current_nonce_and_id(db, pubkey).await?;

    let attestation_status = integrity::check_entity_attestation(
        &mut db_conn,
        keyholder,
        &UserAgentCredentials {
            pubkey: pubkey.clone(),
            nonce,
        },
        id,
    )
    .await
    .map_err(|e| Error::internal("Integrity verification failed", &e))?;

    use integrity::AttestationStatus as AS;
    // SAFETY (policy): challenge auth must work in both vault states.
    // While sealed, integrity checks can only report `Unavailable` because key material is not
    // accessible. While unsealed, the same check can report `Attested`.
    // This path intentionally accepts both outcomes to keep challenge auth available across state
    // transitions; stricter verification is enforced in sensitive post-auth flows.
    match attestation_status {
        AS::Attested | AS::Unavailable => Ok(()),
    }
}

async fn create_nonce(
    db: &DatabasePool,
    keyholder: &ActorRef<KeyHolder>,
    pubkey: &AuthPublicKey,
) -> Result<i32, Error> {
    let mut db_conn = db
        .get()
        .await
        .map_err(|e| Error::internal("Database unavailable", &e))?;
    let new_nonce = db_conn
        .exclusive_transaction(|conn| {
            Box::pin(async move {
                let (id, new_nonce): (i32, i32) = update(useragent_client::table)
                    .filter(useragent_client::public_key.eq(pubkey.to_stored_bytes()))
                    .filter(useragent_client::key_type.eq(pubkey.key_type()))
                    .set(useragent_client::nonce.eq(useragent_client::nonce + 1))
                    .returning((useragent_client::id, useragent_client::nonce))
                    .get_result(conn)
                    .await
                    .map_err(|e| Error::internal("Database operation failed", &e))?;

                integrity::sign_entity(
                    conn,
                    keyholder,
                    &UserAgentCredentials {
                        pubkey: pubkey.clone(),
                        nonce: new_nonce,
                    },
                    id,
                )
                .await
                .map_err(|e| Error::internal("Database error", &e))?
                .drop_verification_provenance();

                Result::<_, Error>::Ok(new_nonce)
            })
        })
        .await?;
    Ok(new_nonce)
}

async fn register_key(
    db: &DatabasePool,
    keyholder: &ActorRef<KeyHolder>,
    pubkey: &AuthPublicKey,
) -> Result<(), Error> {
    let pubkey_bytes = pubkey.to_stored_bytes();
    let key_type = pubkey.key_type();
    let mut conn = db
        .get()
        .await
        .map_err(|e| Error::internal("Database unavailable", &e))?;

    conn.transaction(|conn| {
        Box::pin(async move {
            const NONCE_START: i32 = 1;

            let id: i32 = diesel::insert_into(useragent_client::table)
                .values((
                    useragent_client::public_key.eq(pubkey_bytes),
                    useragent_client::nonce.eq(NONCE_START),
                    useragent_client::key_type.eq(key_type),
                ))
                .returning(useragent_client::id)
                .get_result(conn)
                .await
                .map_err(|e| Error::internal("Database operation failed", &e))?;

            if let Err(e) = integrity::sign_entity(
                conn,
                keyholder,
                &UserAgentCredentials {
                    pubkey: pubkey.clone(),
                    nonce: NONCE_START,
                },
                id,
            )
            .await
            {
                match e {
                    integrity::Error::Keyholder(
                        crate::actors::keyholder::Error::NotBootstrapped,
                    ) => {
                        // IMPORTANT: bootstrap-token auth must work before the vault has a root key.
                        // We intentionally allow creating the DB row first and backfill envelopes
                        // after bootstrap/unseal to keep the bootstrap flow possible.
                    }
                    other => {
                        return Err(Error::internal("Failed to register public key", &other));
                    }
                }
            }

            Result::<_, Error>::Ok(())
        })
    })
    .await?;

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
        verify_integrity(&self.conn.db, &self.conn.actors.key_holder, &pubkey).await?;

        let nonce = create_nonce(&self.conn.db, &self.conn.actors.key_holder, &pubkey).await?;

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
            .map_err(|e| Error::internal("Failed to consume bootstrap token", &e))?;

        if !token_ok {
            error!("Invalid bootstrap token provided");
            return Err(Error::InvalidBootstrapToken);
        }

        match token_ok {
            true => {
                register_key(&self.conn.db, &self.conn.actors.key_holder, &pubkey).await?;
                self.transport
                    .send(Ok(Outbound::AuthSuccess))
                    .await
                    .map_err(|_| Error::Transport)?;
                Ok(pubkey)
            }
            false => {
                error!("Invalid bootstrap token provided");
                self.transport
                    .send(Err(Error::InvalidBootstrapToken))
                    .await
                    .map_err(|_| Error::Transport)?;
                Err(Error::InvalidBootstrapToken)
            }
        }
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

        match valid {
            true => {
                self.transport
                    .send(Ok(Outbound::AuthSuccess))
                    .await
                    .map_err(|_| Error::Transport)?;
                Ok(key.clone())
            }
            false => {
                self.transport
                    .send(Err(Error::InvalidChallengeSolution))
                    .await
                    .map_err(|_| Error::Transport)?;
                Err(Error::InvalidChallengeSolution)
            }
        }
    }
}
