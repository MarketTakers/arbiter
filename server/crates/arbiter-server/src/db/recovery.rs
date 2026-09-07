//! Whether the recovery committee is awake.
//!
//! §3.6: a wake-up request opens a dispute window; recovery powers only become active once
//! that window has elapsed without cancellation. Both the proposal manager (for voting) and
//! the vault coordinator (for unsealing) gate on this, so the rule lives in one place.

use crate::db::{functions::unixepoch, schema};

use diesel::{
    ExpressionMethods as _, QueryDsl as _,
    dsl::{exists, select},
};
use diesel_async::RunQueryDsl;

/// Recovery operators stay asleep for this long after a wake-up is requested, so the other
/// operators have time to dispute it (§3.6).
pub const WAKEUP_DELAY_SECS: i32 = 14 * 24 * 60 * 60;

/// True when an uncancelled wake-up request is older than the dispute window.
pub async fn is_active(
    conn: &mut crate::db::DatabaseConnection,
) -> Result<bool, diesel::result::Error> {
    select(exists(
        schema::recovery_wakeup_request::table
            .filter(schema::recovery_wakeup_request::cancelled_at.is_null())
            .filter(
                schema::recovery_wakeup_request::requested_at
                    .le(unixepoch("now") - WAKEUP_DELAY_SECS),
            ),
    ))
    .get_result(conn)
    .await
}

#[cfg(test)]
mod tests {
    use super::{WAKEUP_DELAY_SECS, is_active};
    use crate::db::{self, schema};

    use diesel::{ExpressionMethods as _, insert_into};
    use diesel_async::RunQueryDsl;

    /// `recovery_wakeup_request.requested_by` references `operator_identity(id)`, and pooled
    /// connections enforce foreign keys, so every wake-up row needs a real identity behind it.
    async fn insert_operator(pool: &db::DatabasePool) -> i32 {
        let mut conn = pool.get().await.unwrap();
        insert_into(schema::operator_identity::table)
            .values(schema::operator_identity::public_key.eq(vec![7u8; 32]))
            .returning(schema::operator_identity::id)
            .get_result(&mut conn)
            .await
            .unwrap()
    }

    /// Pins `.filter(requested_at.le(...))`: a wake-up requested moments ago must not be
    /// active yet, even though nothing has cancelled it. Deleting that filter turns this
    /// assertion false without touching any other test in the suite.
    #[tokio::test]
    async fn a_recent_wakeup_is_not_yet_active() {
        let pool = db::create_test_pool().await;
        let operator_id = insert_operator(&pool).await;
        let mut conn = pool.get().await.unwrap();

        diesel::sql_query(format!(
            "INSERT INTO recovery_wakeup_request (requested_by, requested_at) \
             VALUES ({operator_id}, unixepoch('now'))"
        ))
        .execute(&mut conn)
        .await
        .unwrap();

        assert!(
            !is_active(&mut conn).await.unwrap(),
            "a wake-up requested moments ago must still be asleep"
        );
    }

    /// Pins `.filter(cancelled_at.is_null())`: a cancelled wake-up must not count towards
    /// activity even once its original request has outlived the dispute window. Deleting
    /// that filter turns this assertion false without touching any other test in the suite.
    #[tokio::test]
    async fn a_cancelled_wakeup_is_not_active_even_past_the_window() {
        let pool = db::create_test_pool().await;
        let operator_id = insert_operator(&pool).await;
        let mut conn = pool.get().await.unwrap();

        diesel::sql_query(format!(
            "INSERT INTO recovery_wakeup_request \
             (requested_by, requested_at, cancelled_by, cancelled_at) \
             VALUES ({operator_id}, unixepoch('now') - {WAKEUP_DELAY_SECS} - 1, \
             {operator_id}, unixepoch('now'))"
        ))
        .execute(&mut conn)
        .await
        .unwrap();

        assert!(
            !is_active(&mut conn).await.unwrap(),
            "a cancelled wake-up must not activate recovery even past the window"
        );
    }
}
