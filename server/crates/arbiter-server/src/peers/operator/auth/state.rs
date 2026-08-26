use super::{
    super::{Credentials, OperatorConnection},
    Error,
};
use crate::{
    actors::bootstrap::ConsumeToken,
    db::{DatabasePool, schema::operator_identity},
    peers::operator::auth::Outbound,
};
use arbiter_crypto::authn::{self, AuthChallenge, SigningContext};
use arbiter_proto::transport::Bi;

use diesel::{ExpressionMethods as _, OptionalExtension as _, QueryDsl};
use diesel_async::RunQueryDsl;
use tracing::error;

pub(crate) struct ChallengeRequest {
    pub(crate) pubkey: authn::PublicKey,
    pub(crate) bootstrap_token: Option<String>,
}

pub struct ChallengeContext {
    pub challenge: AuthChallenge,
    pub pubkey: authn::PublicKey,
    pub bootstrap_token: Option<String>,
}

pub(crate) struct ChallengeSolution {
    pub(crate) solution: Vec<u8>,
}

smlang::statemachine!(
    name: Auth,
    custom_error: true,
    transitions: {
        *Init + AuthRequest(ChallengeRequest) / async prepare_challenge = SentChallenge(ChallengeContext),
        SentChallenge(ChallengeContext) + ReceivedSolution(ChallengeSolution) / async verify_solution = AuthOk(Credentials),
    }
);

async fn get_client_id(db: &DatabasePool, pubkey: &authn::PublicKey) -> Result<Option<i32>, Error> {
    let mut conn = db.get().await.map_err(|e| {
        error!(error = ?e, "Database pool error");
        Error::internal("Database unavailable")
    })?;

    operator_identity::table
        .filter(operator_identity::public_key.eq(pubkey.to_bytes()))
        .select(operator_identity::id)
        .first::<i32>(&mut conn)
        .await
        .optional()
        .map_err(|e| {
            error!(error = ?e, "Database error");
            Error::internal("Database operation failed")
        })
}

async fn register_key(db: &DatabasePool, pubkey: &authn::PublicKey) -> Result<i32, Error> {
    let pubkey_bytes = pubkey.to_bytes();
    let mut conn = db.get().await.map_err(|e| {
        error!(error = ?e, "Database pool error");
        Error::internal("Database unavailable")
    })?;

    let id: i32 = diesel::insert_into(operator_identity::table)
        .values((operator_identity::public_key.eq(pubkey_bytes),))
        .returning(operator_identity::id)
        .get_result(&mut conn)
        .await
        .map_err(|e| {
            error!(error = ?e, "Database error");
            Error::internal("Database operation failed")
        })?;

    Ok(id)
}

pub(super) struct AuthContext<'a, T: ?Sized> {
    pub(super) conn: &'a mut OperatorConnection,
    pub(super) transport: &'a mut T,
}

impl<'a, T: ?Sized> AuthContext<'a, T> {
    pub(super) const fn new(conn: &'a mut OperatorConnection, transport: &'a mut T) -> Self {
        Self { conn, transport }
    }
}

impl<T> AuthStateMachineContext for AuthContext<'_, T>
where
    T: Bi<super::Inbound, Result<Outbound, Error>> + Send + ?Sized,
{
    type Error = Error;

    async fn prepare_challenge(
        &mut self,
        ChallengeRequest {
            pubkey,
            bootstrap_token,
        }: ChallengeRequest,
    ) -> Result<ChallengeContext, Self::Error> {
        // Verify pubkey is registered (unless bootstrapping)
        if bootstrap_token.is_none() {
            let id = get_client_id(&self.conn.db, &pubkey).await?;
            if id.is_none() {
                return Err(Error::UnregisteredPublicKey);
            }
        }

        let challenge = AuthChallenge::generate(&mut rand::rng());

        self.transport
            .send(Ok(Outbound::AuthChallenge {
                challenge: challenge.clone(),
            }))
            .await
            .map_err(|e| {
                error!(?e, "Failed to send auth challenge");
                Error::Transport
            })?;

        Ok(ChallengeContext {
            challenge,
            pubkey,
            bootstrap_token,
        })
    }

    async fn verify_solution(
        &mut self,
        ChallengeContext {
            challenge,
            pubkey,
            bootstrap_token,
        }: &ChallengeContext,
        ChallengeSolution { solution }: ChallengeSolution,
    ) -> Result<Credentials, Self::Error> {
        let signature = authn::Signature::try_from(solution.as_slice()).map_err(|()| {
            error!("Failed to decode signature in challenge solution");
            Error::InvalidChallengeSolution
        })?;

        let valid = pubkey.verify(challenge, SigningContext::Operator, &signature);

        if !valid {
            self.transport
                .send(Err(Error::InvalidChallengeSolution))
                .await
                .map_err(|_| Error::Transport)?;
            return Err(Error::InvalidChallengeSolution);
        }

        // Resolve client id: bootstrap (consume token + register) or lookup
        let id = match bootstrap_token {
            Some(token) => {
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
                    self.transport
                        .send(Err(Error::InvalidBootstrapToken))
                        .await
                        .map_err(|_| Error::Transport)?;
                    return Err(Error::InvalidBootstrapToken);
                }

                register_key(&self.conn.db, pubkey).await?
            }
            None => get_client_id(&self.conn.db, pubkey)
                .await?
                .ok_or(Error::UnregisteredPublicKey)?,
        };

        self.transport
            .send(Ok(Outbound::AuthSuccess))
            .await
            .map_err(|_| Error::Transport)?;

        Ok(Credentials {
            id,
            pubkey: pubkey.clone(),
        })
    }
}
