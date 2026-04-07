use arbiter_proto::transport::{Bi, Error, Receiver, Sender};
use arbiter_server::{
    actors::keyholder::KeyHolder,
    db::{self, schema},
};
use arbiter_crypto::{authn::{self, format_challenge, CLIENT_CONTEXT}, safecell::{SafeCell, SafeCellHandle as _}};

use async_trait::async_trait;
use diesel::QueryDsl;
use diesel_async::RunQueryDsl;
use tokio::sync::mpsc;

#[allow(dead_code)]
pub async fn bootstrapped_keyholder(db: &db::DatabasePool) -> KeyHolder {
    let mut actor = KeyHolder::new(db.clone()).await.unwrap();
    actor
        .bootstrap(SafeCell::new(b"test-seal-key".to_vec()))
        .await
        .unwrap();
    actor
}

#[allow(dead_code)]
pub async fn root_key_history_id(db: &db::DatabasePool) -> i32 {
    let mut conn = db.get().await.unwrap();
    let id = schema::arbiter_settings::table
        .select(schema::arbiter_settings::root_key_id)
        .first::<Option<i32>>(&mut conn)
        .await
        .unwrap();
    id.expect("root_key_id should be set after bootstrap")
}

#[allow(dead_code)]
pub struct ChannelTransport<T, Y> {
    receiver: mpsc::Receiver<T>,
    sender: mpsc::Sender<Y>,
}

impl<T, Y> ChannelTransport<T, Y> {
    #[allow(dead_code)]
    pub fn new() -> (Self, ChannelTransport<Y, T>) {
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
