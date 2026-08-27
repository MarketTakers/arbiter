//! Governed actions and the parameters they carry.
//!
//! Laid out the way [`crate::evm::policies::Policy`] is: a unit type per kind, its
//! parameters as an associated `Settings`, and the persistence for those parameters
//! implemented next to them. Everything downstream is generic over [`Proposal`], so a
//! new kind is a new module plus one arm in each dispatcher -- nothing else in the
//! codebase has to learn about it.

use crate::db::{DatabaseConnection, models::ProposalId};
use diesel::{
    QueryResult,
    backend::Backend,
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    serialize::ToSql,
    sql_types::Text,
    sqlite::Sqlite,
};
use strum::{Display, EnumDiscriminants, EnumString, IntoStaticStr};

pub mod approve_sdk_client;
pub mod grant_wallet_access;
pub mod one_off_transaction;
pub mod persistent_grant;
pub mod replace_operator;
pub mod trigger_rekey;

pub use approve_sdk_client::ApproveSdkClient;
pub use grant_wallet_access::GrantWalletAccess;
pub use one_off_transaction::OneOffTransaction;
pub use persistent_grant::PersistentGrant;
pub use replace_operator::ReplaceOperator;
pub use trigger_rekey::TriggerRekey;

/// A governed action that owns the child table holding its parameters.
pub trait Proposal: Sized {
    /// The value stored in `proposal.kind` for this action.
    const KIND: ProposalKindTag;

    /// Parameters the action is voted on with.
    type Settings: Send + Sync + 'static;

    /// Writes the child row carrying `settings`.
    fn insert(
        proposal_id: ProposalId,
        settings: &Self::Settings,
        conn: &mut DatabaseConnection,
    ) -> impl Future<Output = QueryResult<()>> + Send;

    /// Reads the child row back. A missing row surfaces as [`diesel::result::Error::NotFound`],
    /// which is what a proposal without its parameters is.
    fn load(
        proposal_id: ProposalId,
        conn: &mut DatabaseConnection,
    ) -> impl Future<Output = QueryResult<Self::Settings>> + Send;
}

/// Parameters of a proposal, in the one shape that can cross the actor boundary.
///
/// Every variant holds the `Settings` of the matching [`Proposal`] implementation, so
/// the two cannot drift.
#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(
    name(ProposalKindTag),
    vis(pub),
    derive(Display, EnumString, IntoStaticStr, AsExpression, FromSqlRow),
    diesel(sql_type = Text),
    strum(serialize_all = "snake_case")
)]
pub enum ProposalKind {
    ApproveSdkClient(approve_sdk_client::Settings),
    GrantWalletAccess(grant_wallet_access::Settings),
    ReplaceOperator(replace_operator::Settings),
    TriggerRekey,
    ApprovePersistentGrant(Box<persistent_grant::Settings>),
    ApproveOneOffTransaction(Box<one_off_transaction::Settings>),
}

impl ProposalKindTag {
    /// Key-rotation proposals require every operator to approve (§3.3).
    #[must_use]
    pub const fn requires_full_quorum(self) -> bool {
        matches!(self, Self::ReplaceOperator | Self::TriggerRekey)
    }
}

/// Pins every implementation to the variant it is dispatched from. Without this a
/// mistyped `KIND` would compile and only show up as a proposal stored under the
/// wrong `proposal.kind`.
const _: () = {
    assert!(
        matches!(ApproveSdkClient::KIND, ProposalKindTag::ApproveSdkClient),
        "ApproveSdkClient::KIND must be ProposalKindTag::ApproveSdkClient"
    );
    assert!(
        matches!(GrantWalletAccess::KIND, ProposalKindTag::GrantWalletAccess),
        "GrantWalletAccess::KIND must be ProposalKindTag::GrantWalletAccess"
    );
    assert!(
        matches!(ReplaceOperator::KIND, ProposalKindTag::ReplaceOperator),
        "ReplaceOperator::KIND must be ProposalKindTag::ReplaceOperator"
    );
    assert!(
        matches!(TriggerRekey::KIND, ProposalKindTag::TriggerRekey),
        "TriggerRekey::KIND must be ProposalKindTag::TriggerRekey"
    );
    assert!(
        matches!(
            PersistentGrant::KIND,
            ProposalKindTag::ApprovePersistentGrant
        ),
        "PersistentGrant::KIND must be ProposalKindTag::ApprovePersistentGrant"
    );
    assert!(
        matches!(
            OneOffTransaction::KIND,
            ProposalKindTag::ApproveOneOffTransaction
        ),
        "OneOffTransaction::KIND must be ProposalKindTag::ApproveOneOffTransaction"
    );
};

/// Writes the child row carrying this proposal's parameters.
///
/// The only place the create path has to know every kind; each arm hands straight off
/// to the implementation that owns the table.
pub async fn insert_kind(
    conn: &mut DatabaseConnection,
    proposal_id: ProposalId,
    kind: &ProposalKind,
) -> QueryResult<()> {
    match kind {
        ProposalKind::ApproveSdkClient(s) => ApproveSdkClient::insert(proposal_id, s, conn).await,
        ProposalKind::GrantWalletAccess(s) => GrantWalletAccess::insert(proposal_id, s, conn).await,
        ProposalKind::ReplaceOperator(s) => ReplaceOperator::insert(proposal_id, s, conn).await,
        ProposalKind::TriggerRekey => TriggerRekey::insert(proposal_id, &(), conn).await,
        ProposalKind::ApprovePersistentGrant(s) => {
            PersistentGrant::insert(proposal_id, s, conn).await
        }
        ProposalKind::ApproveOneOffTransaction(s) => {
            OneOffTransaction::insert(proposal_id, s, conn).await
        }
    }
}

/// Reads the parameters back for a `proposal.kind` that is only known at runtime.
pub async fn load_kind(
    conn: &mut DatabaseConnection,
    proposal_id: ProposalId,
    tag: ProposalKindTag,
) -> QueryResult<ProposalKind> {
    Ok(match tag {
        ProposalKindTag::ApproveSdkClient => {
            ProposalKind::ApproveSdkClient(ApproveSdkClient::load(proposal_id, conn).await?)
        }
        ProposalKindTag::GrantWalletAccess => {
            ProposalKind::GrantWalletAccess(GrantWalletAccess::load(proposal_id, conn).await?)
        }
        ProposalKindTag::ReplaceOperator => {
            ProposalKind::ReplaceOperator(ReplaceOperator::load(proposal_id, conn).await?)
        }
        ProposalKindTag::TriggerRekey => {
            TriggerRekey::load(proposal_id, conn).await?;
            ProposalKind::TriggerRekey
        }
        ProposalKindTag::ApprovePersistentGrant => ProposalKind::ApprovePersistentGrant(Box::new(
            PersistentGrant::load(proposal_id, conn).await?,
        )),
        ProposalKindTag::ApproveOneOffTransaction => ProposalKind::ApproveOneOffTransaction(
            Box::new(OneOffTransaction::load(proposal_id, conn).await?),
        ),
    })
}

impl ToSql<Text, Sqlite> for ProposalKindTag {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Sqlite>,
    ) -> diesel::serialize::Result {
        <str as ToSql<Text, Sqlite>>::to_sql(<&'static str>::from(*self), out)
    }
}

impl FromSql<Text, Sqlite> for ProposalKindTag {
    fn from_sql(bytes: <Sqlite as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let s = <String as FromSql<Text, Sqlite>>::from_sql(bytes)?;
        s.parse()
            .map_err(|_| format!("Unknown proposal kind: {s}").into())
    }
}

/// SQLite has no unsigned integers; the column is `BigInt`, so a value that does not
/// round-trip is a corrupt row rather than something to silently wrap.
pub(crate) fn as_i64(value: u64) -> QueryResult<i64> {
    i64::try_from(value).map_err(|_| diesel::result::Error::SerializationError(Box::new(Overflow)))
}

pub(crate) fn as_u64(value: i64) -> QueryResult<u64> {
    u64::try_from(value)
        .map_err(|_| diesel::result::Error::DeserializationError(Box::new(Overflow)))
}

pub(crate) fn fixed_bytes<const N: usize>(
    bytes: &[u8],
    column: &'static str,
) -> QueryResult<[u8; N]> {
    <[u8; N]>::try_from(bytes)
        .map_err(|_| diesel::result::Error::DeserializationError(Box::new(WrongLength(column))))
}

/// Reads a fixed-width column into an array, labelling failures with the column it came
/// from.
///
/// The label is taken from the field itself, so renaming a column cannot leave a stale
/// name behind in the error -- which is the whole reason this is a macro and not a
/// second argument.
///
/// - `fixed!(row.column)` for a `Vec<u8>` field
/// - `fixed!(opt row.column)` for a `Option<Vec<u8>>` one
/// - `fixed!(binding)` for a local
macro_rules! fixed {
    (opt $src:ident.$field:ident) => {
        $src.$field
            .as_deref()
            .map(|value| $crate::db::proposal::fixed_bytes(value, stringify!($field)))
            .transpose()
    };
    ($src:ident.$field:ident) => {
        $crate::db::proposal::fixed_bytes(&$src.$field, stringify!($field))
    };
    ($binding:ident) => {
        $crate::db::proposal::fixed_bytes(&$binding, stringify!($binding))
    };
}
pub(crate) use fixed;

#[derive(Debug, thiserror::Error)]
#[error("value does not fit a SQLite integer")]
struct Overflow;

#[derive(Debug, thiserror::Error)]
#[error("column {0} has the wrong byte length")]
struct WrongLength(&'static str);
