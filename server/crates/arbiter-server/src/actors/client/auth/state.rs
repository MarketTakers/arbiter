use arbiter_proto::proto::client::{
    AuthChallenge, ClientResponse,
    client_response::Payload as ClientResponsePayload,
};
use diesel::{ExpressionMethods as _, OptionalExtension as _, QueryDsl, update};
use diesel_async::RunQueryDsl;
use ed25519_dalek::VerifyingKey;
use tracing::error;

use super::Error;
use crate::{actors::client::ConnectionProps, db::schema};

pub struct ChallengeRequest {
    pub pubkey: VerifyingKey,
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
                let current_nonce = schema::program_client::table
                    .filter(schema::program_client::public_key.eq(pubkey_bytes.to_vec()))
                    .select(schema::program_client::nonce)
                    .first::<i32>(conn)
                    .await?;

                update(schema::program_client::table)
                    .filter(schema::program_client::public_key.eq(pubkey_bytes.to_vec()))
                    .set(schema::program_client::nonce.eq(current_nonce + 1))
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

pub struct AuthContext<'a> {
    pub(super) conn: &'a mut ConnectionProps,
}

impl<'a> AuthContext<'a> {
    pub fn new(conn: &'a mut ConnectionProps) -> Self {
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
            .send(Ok(ClientResponse {
                payload: Some(ClientResponsePayload::AuthChallenge(challenge.clone())),
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

    fn provide_key(
        &mut self,
        state_data: &ChallengeContext,
        _: ChallengeSolution,
    ) -> Result<VerifyingKey, Self::Error> {
        Ok(state_data.key)
    }
}
