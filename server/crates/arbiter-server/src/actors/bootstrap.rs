use crate::{
    actors::vault::events,
    db::{self, schema},
};
use arbiter_crypto::safecell::{SafeCell, SafeCellHandle as _};
use arbiter_proto::{BOOTSTRAP_PATH, home_path};

use diesel::QueryDsl;
use diesel_async::RunQueryDsl;
use kameo::{
    Actor,
    actor::ActorRef,
    messages,
    prelude::{Context, Message},
};
use kameo_actors::message_bus::{MessageBus, Register};
use rand::{RngExt, distr::Alphanumeric, rngs::SysRng};
use rand_core::UnwrapErr;
use std::path::{Path, PathBuf};
use subtle::ConstantTimeEq as _;
use tracing::warn;

const TOKEN_LENGTH: usize = 64;

async fn write_token_file(path: &Path, content: &str) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(path, content.as_bytes()).await?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        tokio::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).await?;
    }
    Ok(())
}

async fn remove_token_file(path: &Path) {
    match tokio::fs::remove_file(path).await {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => warn!(?error, ?path, "Failed to remove bootstrap token file"),
    }
}

async fn generate_token(path: &Path) -> Result<SafeCell<[u8; TOKEN_LENGTH]>, std::io::Error> {
    let mut cell = SafeCell::new([0u8; TOKEN_LENGTH]);
    {
        let mut buf = cell.write();
        for (slot, byte) in buf
            .iter_mut()
            .zip(UnwrapErr(SysRng).sample_iter(Alphanumeric))
        {
            *slot = byte;
        }
    }
    let token = cell.read_inline(|buf| String::from_utf8_lossy(buf.as_ref()).into_owned());
    write_token_file(path, &token).await?;
    Ok(cell)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] db::PoolError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Database query error: {0}")]
    Query(#[from] diesel::result::Error),
}

/// Custodian of the one-time bootstrap token.
///
/// The token authorises registering operator identities before the vault
/// exists. A Shamir committee needs every member registered before it can be
/// declared, so the token stays valid across several registrations and is
/// retired by the `Bootstrapped` event rather than by first use, whichever
/// bootstrap path fired it.
pub struct Bootstrapper {
    token: Option<SafeCell<[u8; TOKEN_LENGTH]>>,
    token_path: Option<PathBuf>,
    events: ActorRef<MessageBus>,
}

impl Actor for Bootstrapper {
    type Args = Self;
    type Error = std::convert::Infallible;

    async fn on_start(args: Self::Args, actor_ref: ActorRef<Self>) -> Result<Self, Self::Error> {
        let _ = args
            .events
            .tell(Register(actor_ref.recipient::<events::Bootstrapped>()))
            .await;
        Ok(args)
    }
}

impl Bootstrapper {
    pub async fn new(db: &db::DatabasePool, events: ActorRef<MessageBus>) -> Result<Self, Error> {
        let path = home_path()?.join(BOOTSTRAP_PATH);
        let mut conn = db.get().await?;

        let bootstrapped = schema::arbiter_settings::table
            .select(schema::arbiter_settings::root_key_id)
            .first::<Option<i32>>(&mut conn)
            .await?
            .is_some();
        if bootstrapped {
            // A token file can outlive the bootstrap that made it obsolete:
            // the daemon may have been killed before the event was handled.
            remove_token_file(&path).await;
            return Ok(Self {
                token: None,
                token_path: None,
                events,
            });
        }

        let registered = diesel::select(diesel::dsl::exists(
            schema::operator_identity::table.select(schema::operator_identity::id),
        ))
        .get_result::<bool>(&mut conn)
        .await?;

        let token = if registered {
            match tokio::fs::read_to_string(&path).await {
                Ok(existing)
                    if existing.len() == TOKEN_LENGTH
                        && existing.chars().all(|c| c.is_ascii_alphanumeric()) =>
                {
                    let mut cell = SafeCell::new([0u8; TOKEN_LENGTH]);
                    cell.write().copy_from_slice(existing.as_bytes());
                    cell
                }
                Ok(_) | Err(_) => generate_token(&path).await?,
            }
        } else {
            generate_token(&path).await?
        };

        Ok(Self {
            token: Some(token),
            token_path: Some(path),
            events,
        })
    }

    fn is_correct_token(&mut self, token: &[u8]) -> bool {
        self.token.as_mut().is_some_and(|expected| {
            expected.read_inline(|bytes| bool::from(bytes.as_ref().ct_eq(token)))
        })
    }

    async fn forget(&mut self) {
        self.token = None;
        if let Some(path) = self.token_path.take() {
            remove_token_file(&path).await;
        }
    }
}

impl Message<events::Bootstrapped> for Bootstrapper {
    type Reply = ();

    async fn handle(
        &mut self,
        _: events::Bootstrapped,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.forget().await;
    }
}

#[messages]
impl Bootstrapper {
    #[message]
    pub fn verify_token(&mut self, token: Vec<u8>) -> bool {
        self.is_correct_token(&token)
    }

    #[message]
    pub fn get_token(&mut self) -> Option<String> {
        self.token
            .as_mut()
            .map(|cell| cell.read_inline(|buf| String::from_utf8_lossy(buf.as_ref()).into_owned()))
    }
}
