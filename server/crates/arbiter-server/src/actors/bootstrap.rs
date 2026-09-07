use crate::db::{self, DatabasePool, schema};
use arbiter_proto::{BOOTSTRAP_PATH, home_path};

use diesel::QueryDsl;
use diesel_async::RunQueryDsl;
use kameo::{Actor, messages};
use rand::{
    distr::{Alphanumeric, SampleString as _},
    make_rng,
    rngs::StdRng,
};
use std::path::Path;
use subtle::ConstantTimeEq as _;
use thiserror::Error;

const TOKEN_LENGTH: usize = 64;

pub async fn generate_token(home: &Path) -> Result<String, std::io::Error> {
    let mut rng: StdRng = make_rng();

    // `Alphanumeric` samples raw `u8` ASCII codes, not `char`s -- `SampleString::sample_string`
    // is `rand`'s own documented way to turn that into an actual TOKEN_LENGTH-character string
    // (see the "Passwords" example on `Alphanumeric`'s docs). A prior version of this function
    // called `.to_string()` on the sampled `u8` directly, which stringifies the numeric byte
    // value (e.g. `65` instead of `'A'`) rather than the character it represents, silently
    // producing a variable-length, all-decimal-digit string instead of a real token.
    let token = Alphanumeric.sample_string(&mut rng, TOKEN_LENGTH);

    tokio::fs::write(home.join(BOOTSTRAP_PATH), token.as_str()).await?;

    Ok(token)
}

/// A token file is only trustworthy if it looks like something `generate_token` could have
/// produced. Anything else -- empty (a crash between the file's truncate and write), foreign
/// content, or a trailing newline added by an editor -- must not be adopted as a live
/// credential: an empty file would make every empty-string token verify, and mismatched
/// content would silently lock out every operator holding the real, already-printed token.
#[must_use]
fn is_valid_token(candidate: &str) -> bool {
    candidate.len() == TOKEN_LENGTH && candidate.chars().all(|c| c.is_ascii_alphanumeric())
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] db::PoolError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database query error: {0}")]
    Query(#[from] diesel::result::Error),
}

#[derive(Actor)]
pub struct Bootstrapper {
    token: Option<String>,
}

impl Bootstrapper {
    /// Production constructor: resolves the real `~/.arbiter` directory and delegates.
    pub async fn new(db: &DatabasePool) -> Result<Self, Error> {
        let home = home_path()?;
        Self::new_in(db, &home).await
    }

    /// Carries all of `new`'s logic, parameterized on the directory the token file lives in.
    /// `new` resolves the real home directory and calls this; tests call it directly with a
    /// throwaway temp directory so they never touch the real `~/.arbiter/bootstrap_token`.
    pub async fn new_in(db: &DatabasePool, home: &Path) -> Result<Self, Error> {
        let mut conn = db.get().await?;

        let bootstrapped: bool = schema::arbiter_settings::table
            .select(schema::arbiter_settings::root_key_id)
            .first::<Option<i32>>(&mut conn)
            .await?
            .is_some();

        if bootstrapped {
            return Ok(Self { token: None });
        }

        let any_operator_registered: bool = schema::operator_identity::table
            .count()
            .get_result::<i64>(&mut conn)
            .await?
            > 0;

        if !any_operator_registered {
            // Nobody has used the current token yet, so there is nothing to preserve across a
            // restart: generate a fresh one, exactly as on a first run. Reusing an old file
            // here would let a token survive a database reset, silently reviving trust in
            // whoever still held it.
            return Ok(Self {
                token: Some(generate_token(home).await?),
            });
        }

        // At least one operator has already registered with the current token: every other
        // declared operator still needs that same token, including across a restart, so an
        // existing file is reused rather than replaced -- but only if it still looks like a
        // real token. A truncated, foreign, or corrupted file must not become a live
        // credential (see `is_valid_token`).
        let path = home.join(BOOTSTRAP_PATH);
        let token = match tokio::fs::read_to_string(&path).await {
            Ok(existing) if is_valid_token(&existing) => existing,
            Ok(_) => {
                // Replacing the file invalidates whatever token the already-registered
                // operators were handed, so it must not happen quietly. The content itself is
                // a credential and stays out of the log; the path is enough to act on.
                tracing::warn!(
                    ?path,
                    "Bootstrap token file is not a well-formed token; replacing it"
                );
                generate_token(home).await?
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => generate_token(home).await?,
            Err(err) => return Err(Error::Io(err)),
        };

        Ok(Self { token: Some(token) })
    }

    /// Drops the token from memory. Called once the vault is bootstrapped: from then on,
    /// operators are added through governance rather than through the token.
    pub(crate) fn forget_token(&mut self) {
        self.token = None;
    }

    #[must_use]
    fn is_correct_token(&self, token: &str) -> bool {
        self.token.as_ref().is_some_and(|expected| {
            let expected_bytes = expected.as_bytes();
            let token_bytes = token.as_bytes();

            let choice = expected_bytes.ct_eq(token_bytes);
            bool::from(choice)
        })
    }
}

#[messages]
impl Bootstrapper {
    /// Checks the token without retiring it: every operator in a declared committee
    /// authenticates with the same token during bootstrap.
    #[message]
    #[must_use]
    pub fn verify_token(&self, token: String) -> bool {
        self.is_correct_token(&token)
    }
}

impl kameo::prelude::Message<crate::actors::vault::events::Bootstrapped> for Bootstrapper {
    type Reply = ();

    async fn handle(
        &mut self,
        _msg: crate::actors::vault::events::Bootstrapped,
        _ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.forget_token();
    }
}

#[messages]
impl Bootstrapper {
    #[message]
    pub fn get_token(&self) -> Option<String> {
        self.token.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::{ExpressionMethods as _, insert_into, update};

    /// A multi-operator committee registers every member with the same token, so verifying it
    /// must not consume it. Only a completed bootstrap retires the token.
    #[tokio::test]
    async fn token_verifies_repeatedly_until_bootstrap_completes() {
        let mut bootstrapper = Bootstrapper {
            token: Some("test-token".to_owned()),
        };

        assert!(bootstrapper.verify_token("test-token".to_owned()));
        assert!(bootstrapper.verify_token("test-token".to_owned()));
        assert!(!bootstrapper.verify_token("wrong-token".to_owned()));

        bootstrapper.forget_token();

        assert!(!bootstrapper.verify_token("test-token".to_owned()));
        assert!(bootstrapper.get_token().is_none());
    }

    /// Once the vault is bootstrapped, `Bootstrapper::new_in` must return with no token -- and
    /// it must do so from `arbiter_settings.root_key_id` alone, taking the early return before
    /// `home` is ever consulted. Uses `new_in` with a throwaway temp directory (never the
    /// production `new`, which unconditionally resolves the real home directory before this
    /// method even runs) so this test cannot touch the real filesystem regardless of outcome.
    #[tokio::test]
    async fn new_returns_no_token_once_the_vault_is_bootstrapped() {
        let db = db::create_test_pool().await;
        let mut conn = db.get().await.unwrap();

        let root_key_history_id: i32 = insert_into(schema::root_key_history::table)
            .values(&db::models::NewRootKeyHistory {
                ciphertext: vec![0u8; 32],
                tag: vec![0u8; 16],
                root_key_encryption_nonce: vec![0u8; 24],
                data_encryption_nonce: vec![0u8; 24],
                schema_version: 1,
                salt: vec![0u8; 16],
            })
            .returning(schema::root_key_history::id)
            .get_result(&mut conn)
            .await
            .unwrap();

        update(schema::arbiter_settings::table)
            .set(schema::arbiter_settings::root_key_id.eq(root_key_history_id))
            .execute(&mut conn)
            .await
            .unwrap();
        drop(conn);

        let home = tempfile::tempdir().unwrap();
        let bootstrapper = Bootstrapper::new_in(&db, home.path()).await.unwrap();

        assert!(bootstrapper.get_token().is_none());
    }

    /// The file-reuse path (an operator has already registered, so bootstrap is unfinished)
    /// must not adopt a corrupted token file as a live credential: it must reject it and
    /// generate a fresh one instead. This is the Critical from the review, now reachable
    /// safely because `new_in` takes a throwaway temp directory instead of the real home.
    #[tokio::test]
    async fn new_in_rejects_and_replaces_a_corrupted_token_file() {
        let db = db::create_test_pool().await;
        let mut conn = db.get().await.unwrap();

        // At least one operator must have registered, or `new_in` would regenerate
        // unconditionally regardless of the file (Important 2) and never exercise validation.
        insert_into(schema::operator_identity::table)
            .values(schema::operator_identity::public_key.eq(vec![0u8; 32]))
            .execute(&mut conn)
            .await
            .unwrap();
        drop(conn);

        let home = tempfile::tempdir().unwrap();
        tokio::fs::write(home.path().join(BOOTSTRAP_PATH), "")
            .await
            .unwrap();

        let bootstrapper = Bootstrapper::new_in(&db, home.path()).await.unwrap();

        let token = bootstrapper
            .get_token()
            .expect("a fresh token must be generated");
        assert!(is_valid_token(&token));

        // The replacement must also have landed on disk, not just in memory, so a restart
        // reads back the same (now valid) token rather than the corrupted one again.
        let on_disk = tokio::fs::read_to_string(home.path().join(BOOTSTRAP_PATH))
            .await
            .unwrap();
        assert_eq!(on_disk, token);
    }

    /// The second defect the task exists to fix: a restart between the first registration and
    /// the completed bootstrap must not invalidate the token the other declared operators were
    /// already given. With an operator registered and a well-formed file on disk, `new_in` has
    /// to hand back exactly what it read instead of generating a replacement.
    #[tokio::test]
    async fn new_in_reuses_a_valid_token_file_across_a_restart() {
        let db = db::create_test_pool().await;
        let mut conn = db.get().await.unwrap();

        // Without a registered operator, `new_in` regenerates unconditionally (Important 2 of
        // the round-2 review) and never reaches the reuse path this test is about.
        insert_into(schema::operator_identity::table)
            .values(schema::operator_identity::public_key.eq(vec![0u8; 32]))
            .execute(&mut conn)
            .await
            .unwrap();
        drop(conn);

        let home = tempfile::tempdir().unwrap();
        // Stands in for the token a previous run wrote and printed to the console.
        let handed_out = "Zq7Z2rXaB90kLmNpQwErTyUiOpAsDfGhJkLzXcVbNmQwErTyUiOpAsDfGhJkLzXc";
        assert!(is_valid_token(handed_out));
        tokio::fs::write(home.path().join(BOOTSTRAP_PATH), handed_out)
            .await
            .unwrap();

        let bootstrapper = Bootstrapper::new_in(&db, home.path()).await.unwrap();

        assert_eq!(bootstrapper.get_token().as_deref(), Some(handed_out));
    }

    /// An empty file must not be adopted as a live credential: `is_correct_token`'s
    /// `is_some_and` would enter its closure for `Some(String::new())`, and comparing two empty
    /// byte slices is true, so an unauthenticated `Some("")` from the wire would otherwise
    /// verify. Also pins the other corrupted-content shapes `is_valid_token` must reject.
    #[test]
    fn is_valid_token_rejects_anything_that_is_not_a_real_token() {
        assert!(!is_valid_token(""));
        assert!(!is_valid_token("too-short"));
        assert!(!is_valid_token(&"a".repeat(TOKEN_LENGTH - 1)));
        assert!(!is_valid_token(&"a".repeat(TOKEN_LENGTH + 1)));
        // A trailing newline (e.g. from an editor) must not be silently accepted either.
        assert!(!is_valid_token(&format!("{}\n", "a".repeat(TOKEN_LENGTH))));
        assert!(!is_valid_token(&"!".repeat(TOKEN_LENGTH)));

        assert!(is_valid_token(&"a".repeat(TOKEN_LENGTH)));
    }
}
