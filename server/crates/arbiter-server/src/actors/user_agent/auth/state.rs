use arbiter_proto::proto::user_agent::{
    AuthChallenge, UserAgentResponse,
    user_agent_response::Payload as UserAgentResponsePayload,
};
use diesel::{ExpressionMethods as _, OptionalExtension as _, QueryDsl, update};
use diesel_async::RunQueryDsl;
use ed25519_dalek::VerifyingKey;
use tracing::error;

use super::Error;
use crate::{
    actors::{bootstrap::ConsumeToken, user_agent::UserAgentConnection},
    db::schema,
};

pub struct ChallengeRequest {
    pub pubkey: VerifyingKey,
}

pub struct BootstrapAuthRequest {
    pub pubkey: VerifyingKey,
    pub token: String,
}

pub struct ChallengeContext {
    pub challenge: AuthChallenge,
    pub key: VerifyingKey,
}

pub struct ChallengeSolution {
    pub solution: Vec<u8>,
}

smlang::statemachine!(
    name: Auth,
    custom_error: true,
    transitions: {
        *Init + AuthRequest(ChallengeRequest) / async prepare_challenge = SentChallenge(ChallengeContext),
        Init + BootstrapAuthRequest(BootstrapAuthRequest) [async verify_bootstrap_token] / provide_key_bootstrap = AuthOk(VerifyingKey),
        SentChallenge(ChallengeContext) + ReceivedSolution(ChallengeSolution) [async verify_solution] / provide_key = AuthOk(VerifyingKey),
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

async fn register_key(db: &crate::db::DatabasePool, pubkey_bytes: &[u8]) -> Result<(), Error> {
    let mut conn = db.get().await.map_err(|e| {
        error!(error = ?e, "Database pool error");
        Error::DatabasePoolUnavailable
    })?;

    diesel::insert_into(schema::useragent_client::table)
        .values((
            schema::useragent_client::public_key.eq(pubkey_bytes.to_vec()),
            schema::useragent_client::nonce.eq(1),
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

    async fn verify_solution(
        &self,
        ChallengeContext { challenge, key }: &ChallengeContext,
        ChallengeSolution { solution }: &ChallengeSolution,
    ) -> Result<bool, Self::Error> {
        let formatted_challenge =
            arbiter_proto::format_challenge(challenge.nonce, &challenge.pubkey);

        let signature = solution.as_slice().try_into().map_err(|_| {
            error!(?solution, "Invalid signature length");
            Error::InvalidChallengeSolution
        })?;

        let valid = key.verify_strict(&formatted_challenge, &signature).is_ok();

        Ok(valid)
    }

    async fn prepare_challenge(
        &mut self,
        ChallengeRequest { pubkey }: ChallengeRequest,
    ) -> Result<ChallengeContext, Self::Error> {
        let nonce = create_nonce(&self.conn.db, pubkey.as_bytes()).await?;

        let challenge = AuthChallenge {
            pubkey: pubkey.as_bytes().to_vec(),
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
                error!(?pubkey, "Failed to consume bootstrap token: {e}");
                Error::BootstrapperActorUnreachable
            })?;

        if !token_ok {
            error!(?pubkey, "Invalid bootstrap token provided");
            return Err(Error::InvalidBootstrapToken);
        }

        register_key(&self.conn.db, pubkey.as_bytes()).await?;

        Ok(true)
    }

    fn provide_key_bootstrap(
        &mut self,
        event_data: BootstrapAuthRequest,
    ) -> Result<VerifyingKey, Self::Error> {
        Ok(event_data.pubkey)
    }

    fn provide_key(
        &mut self,
        state_data: &ChallengeContext,
        _: ChallengeSolution,
    ) -> Result<VerifyingKey, Self::Error> {
        Ok(state_data.key)
    }
}
