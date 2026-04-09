use arbiter_crypto::authn::{self, USERAGENT_CONTEXT};
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
        user_agent::{UserAgentConnection, UserAgentCredentials, auth::Outbound},
    },
    crypto::integrity,
    db::{DatabasePool, schema::useragent_client},
};

pub struct ChallengeRequest {
    pub pubkey: authn::PublicKey,
}

pub struct BootstrapAuthRequest {
    pub pubkey: authn::PublicKey,
    pub token: String,
}

pub struct ChallengeContext {
    pub challenge_nonce: i32,
    pub key: authn::PublicKey,
}

pub struct ChallengeSolution {
    pub solution: Vec<u8>,
}

smlang::statemachine!(
    name: Auth,
    custom_error: true,
    transitions: {
        *Init + AuthRequest(ChallengeRequest) / async prepare_challenge = SentChallenge(ChallengeContext),
        Init + BootstrapAuthRequest(BootstrapAuthRequest) / async verify_bootstrap_token = AuthOk(authn::PublicKey),
        SentChallenge(ChallengeContext) + ReceivedSolution(ChallengeSolution) / async verify_solution = AuthOk(authn::PublicKey),
    }
);

/// Returns the current nonce, ready to use for the challenge nonce.
async fn get_current_nonce_and_id(
    db: &DatabasePool,
    key: &authn::PublicKey,
) -> Result<(i32, i32), Error> {
    let mut db_conn = db.get().await.map_err(|e| {
        error!(error = ?e, "Database pool error");
        Error::internal("Database unavailable")
    })?;
    db_conn
        .exclusive_transaction(|conn| {
            Box::pin(async move {
                useragent_client::table
                    .filter(useragent_client::public_key.eq(key.to_bytes()))
                    .select((useragent_client::id, useragent_client::nonce))
                    .first::<(i32, i32)>(conn)
                    .await
            })
        })
        .await
        .optional()
        .map_err(|e| {
            error!(error = ?e, "Database error");
            Error::internal("Database operation failed")
        })?
        .ok_or_else(|| {
            error!(?key, "Public key not found in database");
            Error::UnregisteredPublicKey
        })
}

async fn verify_integrity(
    db: &DatabasePool,
    keyholder: &ActorRef<KeyHolder>,
    pubkey: &authn::PublicKey,
) -> Result<(), Error> {
    let mut db_conn = db.get().await.map_err(|e| {
        error!(error = ?e, "Database pool error");
        Error::internal("Database unavailable")
    })?;

    let (id, nonce) = get_current_nonce_and_id(db, pubkey).await?;

    let _result = integrity::verify_entity(
        &mut db_conn,
        keyholder,
        &UserAgentCredentials {
            pubkey: pubkey.clone(),
            nonce,
        },
        id,
    )
    .await
    .map_err(|e| {
        error!(?e, "Integrity verification failed");
        Error::internal("Integrity verification failed")
    })?;

    Ok(())
}

async fn create_nonce(
    db: &DatabasePool,
    keyholder: &ActorRef<KeyHolder>,
    pubkey: &authn::PublicKey,
) -> Result<i32, Error> {
    let mut db_conn = db.get().await.map_err(|e| {
        error!(error = ?e, "Database pool error");
        Error::internal("Database unavailable")
    })?;
    let new_nonce = db_conn
        .exclusive_transaction(|conn| {
            Box::pin(async move {
                let (id, new_nonce): (i32, i32) = update(useragent_client::table)
                    .filter(useragent_client::public_key.eq(pubkey.to_bytes()))
                    .set(useragent_client::nonce.eq(useragent_client::nonce + 1))
                    .returning((useragent_client::id, useragent_client::nonce))
                    .get_result(conn)
                    .await
                    .map_err(|e| {
                        error!(error = ?e, "Database error");
                        Error::internal("Database operation failed")
                    })?;

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
                .map_err(|e| {
                    error!(?e, "Integrity signature update failed");
                    Error::internal("Database error")
                })?;

                Result::<_, Error>::Ok(new_nonce)
            })
        })
        .await?;
    Ok(new_nonce)
}

async fn register_key(
    db: &DatabasePool,
    keyholder: &ActorRef<KeyHolder>,
    pubkey: &authn::PublicKey,
) -> Result<(), Error> {
    let pubkey_bytes = pubkey.to_bytes();
    let mut conn = db.get().await.map_err(|e| {
        error!(error = ?e, "Database pool error");
        Error::internal("Database unavailable")
    })?;

    conn.transaction(|conn| {
        Box::pin(async move {
            const NONCE_START: i32 = 1;

            let id: i32 = diesel::insert_into(useragent_client::table)
                .values((
                    useragent_client::public_key.eq(pubkey_bytes),
                    useragent_client::nonce.eq(NONCE_START),
                ))
                .returning(useragent_client::id)
                .get_result(conn)
                .await
                .map_err(|e| {
                    error!(error = ?e, "Database error");
                    Error::internal("Database operation failed")
                })?;

            let entity = UserAgentCredentials {
                pubkey: pubkey.clone(),
                nonce: NONCE_START,
            };

            integrity::sign_entity(conn, keyholder, &entity, id)
                .await
                .map_err(|e| {
                    error!(error = ?e, "Failed to sign integrity tag for new user-agent key");
                    Error::internal("Failed to register public key")
                })?;

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
    pub const fn new(conn: &'a mut UserAgentConnection, transport: T) -> Self {
        Self { conn, transport }
    }
}

impl<T> AuthStateMachineContext for AuthContext<'_, T>
where
    T: Bi<super::Inbound, Result<Outbound, Error>> + Send,
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

    async fn verify_bootstrap_token(
        &mut self,
        BootstrapAuthRequest { pubkey, token }: BootstrapAuthRequest,
    ) -> Result<authn::PublicKey, Self::Error> {
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

        if token_ok {
            register_key(&self.conn.db, &self.conn.actors.key_holder, &pubkey).await?;
            self.transport
                .send(Ok(Outbound::AuthSuccess))
                .await
                .map_err(|_| Error::Transport)?;
            Ok(pubkey)
        } else {
            error!("Invalid bootstrap token provided");
            self.transport
                .send(Err(Error::InvalidBootstrapToken))
                .await
                .map_err(|_| Error::Transport)?;
            Err(Error::InvalidBootstrapToken)
        }
    }

    async fn verify_solution(
        &mut self,
        ChallengeContext {
            challenge_nonce,
            key,
        }: &ChallengeContext,
        ChallengeSolution { solution }: ChallengeSolution,
    ) -> Result<authn::PublicKey, Self::Error> {
        let signature = authn::Signature::try_from(solution.as_slice()).map_err(|()| {
            error!("Failed to decode signature in challenge solution");
            Error::InvalidChallengeSolution
        })?;

        let valid = key.verify(*challenge_nonce, USERAGENT_CONTEXT, &signature);

        if valid {
            self.transport
                .send(Ok(Outbound::AuthSuccess))
                .await
                .map_err(|_| Error::Transport)?;
            Ok(key.clone())
        } else {
            self.transport
                .send(Err(Error::InvalidChallengeSolution))
                .await
                .map_err(|_| Error::Transport)?;
            Err(Error::InvalidChallengeSolution)
        }
    }
}
