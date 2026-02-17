use arbiter_server::{
    actors::keyholder::KeyHolder,
    db::{self, schema},
};
use diesel::QueryDsl;
use diesel_async::RunQueryDsl;
use memsafe::MemSafe;

#[allow(dead_code)]
pub async fn bootstrapped_keyholder(db: &db::DatabasePool) -> KeyHolder {
    let mut actor = KeyHolder::new(db.clone()).await.unwrap();
    actor
        .bootstrap(MemSafe::new(b"test-seal-key".to_vec()).unwrap())
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
