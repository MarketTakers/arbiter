use crate::{
    actors::GlobalActors,
    crypto::integrity::{self, Integrable},
    db,
    peers::client::ClientProfile,
};
use arbiter_crypto::authn;

use arbiter_proto::transport::{Bi, Sender};
pub use auth::authenticate;
use kameo::actor::{ActorRef, Spawn as _};
pub use session::UserAgentSession;
use tokio::sync::oneshot;
use tracing::warn;
use vault_gate::VaultGate;

use crate::crypto::integrity::hashing::Hashable;

pub mod auth;
pub mod session;
pub mod vault_gate;

#[derive(Debug, Clone, Hash)]
pub struct Credentials {
    pub id: i32,
    pub pubkey: authn::PublicKey,
}
impl Hashable for Credentials {
    fn hash<H: sha2::Digest>(&self, hasher: &mut H) {
        self.id.hash(hasher);
        self.pubkey.hash(hasher);
    }
}

#[derive(Debug, Clone)]
pub struct AuthCredentials {
    pub creds: Credentials,
    // denotes new nonce, not current
    pub new_nonce: i32,
}

impl Hashable for authn::PublicKey {
    fn hash<H: sha2::Digest>(&self, hasher: &mut H) {
        hasher.update(self.to_bytes());
    }
}

impl Hashable for AuthCredentials {
    fn hash<H: sha2::Digest>(&self, hasher: &mut H) {
        self.creds.hash(hasher);
        self.new_nonce.hash(hasher);
    }
}

impl Integrable for AuthCredentials {
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
    #[error("internal: {0}")]
    Internal(String),
}

impl From<auth::Error> for Error {
    fn from(err: auth::Error) -> Self {
        Self::Auth(err)
    }
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
    let auth_creds = authenticate(props, &mut transport).await?;

    let creds = if integrity::is_signing_available(&props.actors.vault)
        .await
        .unwrap_or(false)
    {
        auth_creds.creds
    } else {
        run_vault_gate(props, &mut transport, auth_creds).await?
    };

    Ok(UserAgentSession::spawn(UserAgentSession::new(
        props.clone(),
        creds,
        oob_sender,
    )))
}

async fn run_vault_gate<T>(
    props: &UserAgentConnection,
    transport: &mut T,
    auth_creds: AuthCredentials,
) -> Result<Credentials, Error>
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
