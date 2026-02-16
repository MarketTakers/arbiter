use arbiter_server::{
    actors::keyholder::KeyHolder,
    db::{self, models::ArbiterSetting, schema},
};
use diesel::{QueryDsl, insert_into};
use diesel_async::RunQueryDsl;
use memsafe::MemSafe;

pub async fn seed_settings(pool: &db::DatabasePool) {
    let mut conn = pool.get().await.unwrap();
    insert_into(schema::arbiter_settings::table)
        .values(&ArbiterSetting {
            id: 1,
            root_key_id: None,
            cert_key: vec![],
            cert: vec![],
        })
        .execute(&mut conn)
        .await
        .unwrap();
}

#[allow(dead_code)]
pub async fn bootstrapped_keyholder(db: &db::DatabasePool) -> KeyHolder {
    seed_settings(db).await;
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
