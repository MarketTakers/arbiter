#![allow(
    dead_code,
    reason = "Common test utilities that may not be used in every test"
)]
use arbiter_proto::transport::{Bi, Error, Receiver, Sender};
use arbiter_server::{
    actors::{GlobalActors, vault::Vault},
    db::{self, schema},
};

use async_trait::async_trait;
use diesel::QueryDsl;
use diesel_async::RunQueryDsl;
use tokio::sync::mpsc;

pub(crate) async fn bootstrapped_vault(db: &db::DatabasePool) -> Vault {
    let mut actor = Vault::new(db.clone(), GlobalActors::spawn_message_bus())
        .await
        .unwrap();
    actor
        .bootstrap(arbiter_server::crypto::KeyCell::from([0u8; 32]))
        .await
        .unwrap();
    actor
}

/// Spawns a full `GlobalActors` for a test, backing `Bootstrapper`'s token file with a
/// throwaway temp directory rather than the real `~/.arbiter` -- a test must never be able to
/// reach, let alone write to, the developer's real bootstrap token file.
pub(crate) async fn spawn_actors(db: db::DatabasePool) -> GlobalActors {
    let home = tempfile::tempdir().expect("failed to create a temp home directory for a test");
    GlobalActors::spawn_in(db, home.path())
        .await
        .expect("failed to spawn GlobalActors for a test")
}

/// Retries `probe` until it yields a value, then returns it.
///
/// Effects that travel over the message bus are not visible the moment the publishing call
/// returns: `Publish` only enqueues to the bus's mailbox, which then delivers to each
/// subscriber's mailbox in turn. A test observing such an effect waits for it instead of
/// reading straight after the call that triggered it.
pub(crate) async fn eventually<T, F, Fut>(what: &str, mut probe: F) -> T
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Option<T>>,
{
    for _ in 0..100 {
        if let Some(value) = probe().await {
            return value;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    panic!("{what} did not happen within 2s");
}

pub(crate) async fn root_key_history_id(db: &db::DatabasePool) -> i32 {
    let mut conn = db.get().await.unwrap();
    let id = schema::arbiter_settings::table
        .select(schema::arbiter_settings::root_key_id)
        .first::<Option<i32>>(&mut conn)
        .await
        .unwrap();
    id.expect("root_key_id should be set after bootstrap")
}

pub(crate) struct ChannelTransport<T, Y> {
    receiver: mpsc::Receiver<T>,
    sender: mpsc::Sender<Y>,
}

impl<T, Y> ChannelTransport<T, Y> {
    pub(crate) fn new() -> (Self, ChannelTransport<Y, T>) {
        let (tx1, rx1) = mpsc::channel(10);
        let (tx2, rx2) = mpsc::channel(10);
        (
            Self {
                receiver: rx1,
                sender: tx2,
            },
            ChannelTransport {
                receiver: rx2,
                sender: tx1,
            },
        )
    }
}

#[async_trait]
impl<T, Y> Sender<Y> for ChannelTransport<T, Y>
where
    T: Send + Sync + 'static,
    Y: Send + Sync + 'static,
{
    async fn send(&mut self, item: Y) -> Result<(), Error> {
        self.sender
            .send(item)
            .await
            .map_err(|_| Error::ChannelClosed)
    }
}

#[async_trait]
impl<T, Y> Receiver<T> for ChannelTransport<T, Y>
where
    T: Send + Sync + 'static,
    Y: Send + Sync + 'static,
{
    async fn recv(&mut self) -> Option<T> {
        self.receiver.recv().await
    }
}

impl<T, Y> Bi<T, Y> for ChannelTransport<T, Y>
where
    T: Send + Sync + 'static,
    Y: Send + Sync + 'static,
{
}
