use crate::{
    actors::{
        GlobalActors,
        vault::{GetState, Vault},
    },
    crypto::integrity::{self, AttestationStatus, Integrable},
    db::{self, DatabaseError, DatabasePool},
    peers::client::ClientProfile,
};
use arbiter_crypto::authn;
use arbiter_macros::Hashable;
use arbiter_proto::transport::{Bi, Sender};
use vault_gate::VaultGate;

use kameo::actor::{ActorRef, Spawn as _};
use tokio::sync::oneshot;
use tracing::{error, warn};

pub use auth::authenticate;
pub use session::UserAgentSession;

pub mod auth;
pub mod session;
pub mod vault_gate;

#[derive(Debug, Clone, Hashable)]
pub struct Credentials {
    pub id: i32,
    pub pubkey: authn::PublicKey,
}

impl Integrable for Credentials {
    const KIND: &'static str = "useragent_credentials";
}

// Messages, sent by user agent to connection client without having a request
#[derive(Debug)]
pub enum OutOfBand {
    ClientConnectionRequest { profile: ClientProfile },
    ClientConnectionCancel { pubkey: authn::PublicKey },
}

#[derive(Clone)]
pub struct UserAgentConnection {
    pub(crate) db: db::DatabasePool,
    pub(crate) actors: GlobalActors,
}

impl UserAgentConnection {
    pub fn new(db: db::DatabasePool, actors: GlobalActors) -> Self {
        Self { db, actors }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("authentication failed: {0:?}")]
    Auth(auth::Error),
    #[error("vault gate failed: {0}")]
    VaultGate(#[from] vault_gate::Error),
    #[error("transport closed unexpectedly")]
    Transport,
    #[error("database error: {0}")]
    Database(DatabaseError),
    #[error("internal: {0}")]
    Internal(String),
}

impl From<auth::Error> for Error {
    fn from(err: auth::Error) -> Self {
        Self::Auth(err)
    }
}

async fn verify_integrity(
    db: &DatabasePool,
    vault: &ActorRef<Vault>,
    credentials: &Credentials,
) -> Result<(), Error> {
    let mut conn = db
        .get()
        .await
        .map_err(|_| Error::Internal("DB unavailable".into()))?;
    match integrity::verify_entity(&mut conn, vault, credentials, credentials.id).await {
        Ok(AttestationStatus::Attested) => Ok(()),
        Ok(AttestationStatus::Unavailable) => {
            Err(Error::Internal("Vault sealed during promotion".into()))
        }
        Err(e) => {
            error!(?e, "Integrity verification failed during unseal promotion");
            Err(Error::Internal("Integrity check failed".into()))
        }
    }
}

async fn should_run_gate(vault: &ActorRef<Vault>) -> Result<bool, Error> {
    let vault_state = vault
        .ask(GetState {})
        .await
        .map_err(|_| Error::Internal("Failed to contact the vault".into()))?;

    Ok(!matches!(
        vault_state,
        crate::actors::vault::VaultState::Unsealed
    ))
}

async fn run_vault_gate<T>(
    props: &UserAgentConnection,
    transport: &mut T,
    auth_creds: Credentials,
) -> Result<(), Error>
where
    T: Bi<vault_gate::Inbound, Result<vault_gate::Outbound, vault_gate::Error>> + Send + ?Sized,
{
    let (promotion_tx, mut promotion_rx) = oneshot::channel();
    let gate = VaultGate::spawn(VaultGate::new(
        auth_creds,
        props.actors.clone(),
        props.db.clone(),
        promotion_tx,
    ));

    let result = loop {
        tokio::select! {
            promotion = &mut promotion_rx => {
                break match promotion {
                    Ok(Ok(creds)) => Ok(creds),
                    Ok(Err(err)) => Err(Error::VaultGate(err)),
                    Err(_) => Err(Error::Internal(
                        "vault gate promotion channel closed".into(),
                    )),
                };
            }

            inbound = transport.recv() => {
                let Some(inbound) = inbound else {
                    break Err(Error::Transport);
                };

                match gate.ask(inbound).await {
                    Ok(outbound) => {
                        if transport.send(Ok(outbound)).await.is_err() {
                            break Err(Error::Transport);
                        }
                    }
                    Err(err) => {
                        warn!(?err, "VaultGate failed to handle message");
                        break Err(Error::Internal(format!(
                            "vault gate ask failed: {err:?}"
                        )));
                    }
                }
            }
        }
    };

    gate.kill();
    result
}

pub async fn start<T>(
    props: &mut UserAgentConnection,
    mut transport: T,
    oob_sender: Box<dyn Sender<OutOfBand>>,
) -> Result<ActorRef<UserAgentSession>, Error>
where
    T: Bi<auth::Inbound, Result<auth::Outbound, auth::Error>> + Send,
    T: Bi<vault_gate::Inbound, Result<vault_gate::Outbound, vault_gate::Error>> + Send,
{
    let creds = authenticate(props, &mut transport).await?;

    // should run vault gate only if sealed / unbootstrapped
    if should_run_gate(&props.actors.vault).await? {
        run_vault_gate(props, &mut transport, creds.clone()).await?;
    }

    // checking the integrity
    verify_integrity(&props.db, &props.actors.vault, &creds).await?;

    Ok(UserAgentSession::spawn(UserAgentSession::new(
        props.clone(),
        oob_sender,
    )))
}
