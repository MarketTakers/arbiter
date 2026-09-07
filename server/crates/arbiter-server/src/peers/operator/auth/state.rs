use super::{
    super::{Credentials, OperatorConnection},
    Error,
};
use crate::{
    actors::bootstrap::VerifyToken,
    db::{
        DatabasePool,
        schema::{arbiter_settings, operator_identity},
    },
    peers::operator::auth::Outbound,
};
use arbiter_crypto::authn::{self, AuthChallenge, SigningContext};
use arbiter_proto::transport::Bi;

use diesel::{ExpressionMethods as _, OptionalExtension as _, QueryDsl};
use diesel_async::{AsyncConnection as _, RunQueryDsl};
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

    conn.transaction(async move |conn| {
        // The database is authoritative on whether bootstrap has completed: `Vault::bootstrap`
        // commits `root_key_id` before it publishes `events::Bootstrapped`, and `Bootstrapper`
        // only learns of that two mailbox hops later. Re-checking it here, in the same
        // transaction as the insert, closes that window deterministically instead of trusting
        // a token that verified against `Bootstrapper`'s possibly-stale in-memory state.
        let already_bootstrapped: bool = arbiter_settings::table
            .select(arbiter_settings::root_key_id)
            .first::<Option<i32>>(&mut *conn)
            .await?
            .is_some();

        if already_bootstrapped {
            error!("Bootstrap token used to register after the vault was already bootstrapped");
            return Err(Error::InvalidBootstrapToken);
        }

        let id: i32 = diesel::insert_into(operator_identity::table)
            .values((operator_identity::public_key.eq(pubkey_bytes),))
            .returning(operator_identity::id)
            .get_result(&mut *conn)
            .await?;

        Ok(id)
    })
    .await
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

        // Resolve client id: bootstrap (verify token, then register) or lookup
        let id = match bootstrap_token {
            Some(token) => {
                let token_ok: bool = self
                    .conn
                    .actors
                    .bootstrapper
                    .ask(VerifyToken {
                        token: token.clone(),
                    })
                    .await
                    .map_err(|e| {
                        error!(?e, "Failed to verify bootstrap token");
                        Error::internal("Failed to verify bootstrap token")
                    })?;

                if !token_ok {
                    error!("Invalid bootstrap token provided");
                    self.transport
                        .send(Err(Error::InvalidBootstrapToken))
                        .await
                        .map_err(|_| Error::Transport)?;
                    return Err(Error::InvalidBootstrapToken);
                }

                match register_key(&self.conn.db, pubkey).await {
                    Ok(id) => id,
                    // `register_key` refuses a token that verified here but lost the race
                    // against bootstrap. Reported to the peer exactly like the refusal above,
                    // so that operator sees a protocol error rather than a handshake that
                    // stops with nothing on the wire.
                    Err(Error::InvalidBootstrapToken) => {
                        self.transport
                            .send(Err(Error::InvalidBootstrapToken))
                            .await
                            .map_err(|_| Error::Transport)?;
                        return Err(Error::InvalidBootstrapToken);
                    }
                    Err(err) => return Err(err),
                }
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
