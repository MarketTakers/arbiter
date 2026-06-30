use crate::hashing::Hashable;

/// Marks a struct as a participant in the database integrity system.
///
/// Implementors are protected by an HMAC-SHA256 MAC stored in the
/// `integrity_envelope` table. The MAC is computed over:
///
/// ```text
/// HMAC-SHA256(key, len(KIND) || KIND || len(entity_id) || entity_id || VERSION || SHA256(Hashable))
/// ```
///
/// Both `KIND` and `VERSION` act as domain separators — they prevent a valid
/// MAC for one entity type or schema version from being accepted for another.
///
/// # Deriving
///
/// Use `#[derive(Integrable)]` with the `#[integrable(kind = "...")]` attribute.
/// `VERSION` is computed automatically as an FNV-1a hash of the struct's field
/// names and types, so it changes whenever the schema changes without any manual
/// bookkeeping.
///
/// ```rust,ignore
/// #[derive(Hashable, Integrable)]
/// #[integrable(kind = "operator_credentials")]
/// pub struct OperatorCredentials {
///     pub pubkey: PublicKey,
/// }
/// ```
///
/// # Upgrading schema
///
/// When fields are added, removed, or reordered, `VERSION` changes automatically.
/// Existing MAC records in the database will return [`PayloadVersionMismatch`] on
/// verification — this is the signal to re-sign all rows for this `KIND` as part
/// of a migration.
///
/// [`PayloadVersionMismatch`]: crate::integrity::Integrable
pub trait Integrable: Hashable {
    /// Stable name of this entity type as stored in `integrity_envelope.entity_kind`.
    ///
    /// Must be a valid schema name: starts with a letter, contains only `[a-zA-Z0-9_]`,
    /// and must be globally unique across all `Integrable` types in the system.
    const KIND: &'static str;

    /// FNV-1a hash of the struct's field names and types at the time the derive
    /// macro ran. Changes automatically when the schema changes, invalidating
    /// existing MACs and signalling that a migration is required.
    const VERSION: i32;
}
