#![allow(
    clippy::duplicated_attributes,
    reason = "restructed's #[view] causes false positives"
)]
use crate::db::schema::{
    self, aead_encrypted, arbiter_settings, evm_basic_grant, evm_ether_transfer_grant,
    evm_ether_transfer_grant_target, evm_ether_transfer_limit, evm_token_transfer_grant,
    evm_token_transfer_log, evm_token_transfer_volume_limit, evm_transaction_log, evm_wallet,
    integrity_envelope, root_key_history, tls_history,
};

use diesel::{prelude::*, sqlite::Sqlite};
use restructed::Models;

pub mod types {
    use chrono::{DateTime, Utc};
    use diesel::{
        backend::Backend,
        deserialize::{FromSql, FromSqlRow},
        expression::AsExpression,
        serialize::{IsNull, ToSql},
        sql_types::{Integer, Text},
        sqlite::{Sqlite, SqliteType},
    };
    use strum::{Display, EnumDiscriminants, EnumString, IntoStaticStr};

    #[derive(Debug, FromSqlRow, AsExpression, Clone)]
    #[diesel(sql_type = Integer)]
    #[repr(transparent)] // hint compiler to optimize the wrapper struct away
    pub struct SqliteTimestamp(pub DateTime<Utc>);
    impl SqliteTimestamp {
        pub fn now() -> Self {
            Self(Utc::now())
        }
    }

    impl From<DateTime<Utc>> for SqliteTimestamp {
        fn from(dt: DateTime<Utc>) -> Self {
            Self(dt)
        }
    }
    impl From<SqliteTimestamp> for DateTime<Utc> {
        fn from(ts: SqliteTimestamp) -> Self {
            ts.0
        }
    }

    impl ToSql<Integer, Sqlite> for SqliteTimestamp {
        fn to_sql<'b>(
            &'b self,
            out: &mut diesel::serialize::Output<'b, '_, Sqlite>,
        ) -> diesel::serialize::Result {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::as_conversions,
                reason = "fixme! #84; this will break up in 2038 :3"
            )]
            let unix_timestamp = self.0.timestamp() as i32;
            out.set_value(unix_timestamp);
            Ok(IsNull::No)
        }
    }

    impl FromSql<Integer, Sqlite> for SqliteTimestamp {
        fn from_sql(
            mut bytes: <Sqlite as Backend>::RawValue<'_>,
        ) -> diesel::deserialize::Result<Self> {
            let Some(SqliteType::Long) = bytes.value_type() else {
                return Err(format!(
                    "Expected Integer type for SqliteTimestamp, got {:?}",
                    bytes.value_type()
                )
                .into());
            };

            let unix_timestamp = bytes.read_long();
            let datetime =
                DateTime::from_timestamp(unix_timestamp, 0).ok_or("Timestamp is out of bounds")?;

            Ok(Self(datetime))
        }
    }

    macro_rules! declare_id {
        ($name:ident) => {
            #[derive(Debug, FromSqlRow, AsExpression, Clone, Hash, Copy, PartialEq, Eq)]
            #[diesel(sql_type = Integer)]
            #[repr(transparent)] // hint compiler to optimize the wrapper struct away
            pub struct $name(i32);

            impl $name {
                pub const fn to_raw(self) -> i32 {
                    self.0
                }
                pub const fn from_raw(raw: i32) -> Self {
                    Self(raw)
                }
            }

            impl FromSql<Integer, Sqlite> for $name {
                fn from_sql(
                    bytes: <Sqlite as diesel::backend::Backend>::RawValue<'_>,
                ) -> diesel::deserialize::Result<Self> {
                    FromSql::<Integer, Sqlite>::from_sql(bytes).map(Self)
                }
            }
            impl ToSql<Integer, Sqlite> for $name {
                fn to_sql<'b>(
                    &'b self,
                    out: &mut diesel::serialize::Output<'b, '_, Sqlite>,
                ) -> diesel::serialize::Result {
                    ToSql::<Integer, Sqlite>::to_sql(&self.0, out)
                }
            }
        };
    }

    declare_id!(ChainId);

    #[expect(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::as_conversions,
        reason = "safe because chain_id is stored as i32 but is guaranteed to be a valid ChainId by the API when creating grants"
    )]
    const _: () = {
        impl From<ChainId> for alloy::primitives::ChainId {
            fn from(chain_id: ChainId) -> Self {
                chain_id.0 as Self
            }
        }
        impl From<alloy::primitives::ChainId> for ChainId {
            fn from(chain_id: alloy::primitives::ChainId) -> Self {
                Self(chain_id as _)
            }
        }
    };

    declare_id!(OperatorId);
    declare_id!(OperatorIdentityId);
    declare_id!(AeadEncryptedId);
    declare_id!(RootKeyHistoryId);
    declare_id!(TlsHistoryId);
    declare_id!(EvmWalletId);
    declare_id!(ClientId);

    #[derive(Debug, Clone, PartialEq, Eq, AsExpression, FromSqlRow)]
    #[diesel(sql_type = Text)]
    pub enum ProposalStatus {
        Pending,
        Approved,
        Rejected,
    }

    impl ToSql<Text, Sqlite> for ProposalStatus {
        fn to_sql<'b>(
            &'b self,
            out: &mut diesel::serialize::Output<'b, '_, Sqlite>,
        ) -> diesel::serialize::Result {
            let s: &str = match self {
                Self::Pending => "pending",
                Self::Approved => "approved",
                Self::Rejected => "rejected",
            };
            <str as ToSql<Text, Sqlite>>::to_sql(s, out)
        }
    }

    impl FromSql<Text, Sqlite> for ProposalStatus {
        fn from_sql(bytes: <Sqlite as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
            let s = <String as FromSql<Text, Sqlite>>::from_sql(bytes)?;
            match s.as_str() {
                "pending" => Ok(Self::Pending),
                "approved" => Ok(Self::Approved),
                "rejected" => Ok(Self::Rejected),
                other => Err(format!("Unknown proposal status: {other}").into()),
            }
        }
    }

    /// A governance proposal and the parameters it carries.
    #[derive(Debug, Clone, EnumDiscriminants)]
    #[strum_discriminants(
        name(ProposalKindTag),
        vis(pub),
        derive(Display, EnumString, IntoStaticStr, AsExpression, FromSqlRow),
        diesel(sql_type = Text),
        strum(serialize_all = "snake_case")
    )]
    pub enum ProposalKind {
        ApproveSdkClient {
            client_id: i32,
        },
        GrantWalletAccess {
            wallet_id: i32,
            client_id: i32,
        },
        ReplaceOperator {
            old_operator_id: i32,
            new_pubkey: Vec<u8>,
        },
        TriggerRekey,
        ApprovePersistentGrant {
            payload_bytes: Vec<u8>,
        },
        ApproveOneOffTransaction {
            payload_bytes: Vec<u8>,
        },
    }

    impl ProposalKind {
        pub fn encode_payload(&self) -> Vec<u8> {
            match self {
                Self::ApproveSdkClient { client_id } => client_id.to_be_bytes().to_vec(),
                Self::GrantWalletAccess {
                    wallet_id,
                    client_id,
                } => {
                    let mut buf = Vec::with_capacity(8);
                    buf.extend_from_slice(&wallet_id.to_be_bytes());
                    buf.extend_from_slice(&client_id.to_be_bytes());
                    buf
                }
                Self::ReplaceOperator {
                    old_operator_id,
                    new_pubkey,
                } => {
                    let len = u32::try_from(new_pubkey.len()).expect("pubkey len fits in u32");
                    let mut buf = Vec::with_capacity(4 + 4 + new_pubkey.len());
                    buf.extend_from_slice(&old_operator_id.to_be_bytes());
                    buf.extend_from_slice(&len.to_be_bytes());
                    buf.extend_from_slice(new_pubkey);
                    buf
                }
                Self::TriggerRekey => vec![],
                Self::ApprovePersistentGrant { payload_bytes }
                | Self::ApproveOneOffTransaction { payload_bytes } => payload_bytes.clone(),
            }
        }

        /// Key-rotation proposals require every operator to approve (§3.3).
        pub fn decode(tag: ProposalKindTag, payload: &[u8]) -> Result<Self, String> {
            match tag {
                ProposalKindTag::ApproveSdkClient => {
                    let bytes = <[u8; 4]>::try_from(payload)
                        .map_err(|_| "invalid payload for approve_sdk_client".to_owned())?;
                    Ok(Self::ApproveSdkClient {
                        client_id: i32::from_be_bytes(bytes),
                    })
                }
                ProposalKindTag::GrantWalletAccess => {
                    let bytes = <[u8; 8]>::try_from(payload)
                        .map_err(|_| "invalid payload for grant_wallet_access".to_owned())?;
                    Ok(Self::GrantWalletAccess {
                        wallet_id: i32::from_be_bytes(bytes[..4].try_into().unwrap()),
                        client_id: i32::from_be_bytes(bytes[4..].try_into().unwrap()),
                    })
                }
                ProposalKindTag::ReplaceOperator => {
                    let (id_bytes, rest) = payload
                        .split_first_chunk::<4>()
                        .ok_or_else(|| "replace_operator payload too short".to_owned())?;
                    let old_operator_id = i32::from_be_bytes(*id_bytes);
                    let (len_bytes, rest) = rest
                        .split_first_chunk::<4>()
                        .ok_or_else(|| "replace_operator payload too short".to_owned())?;
                    let len = u32::from_be_bytes(*len_bytes);
                    let len = usize::try_from(len).unwrap_or(usize::MAX);
                    let new_pubkey = rest
                        .get(..len)
                        .ok_or_else(|| "replace_operator payload truncated".to_owned())?
                        .to_vec();
                    Ok(Self::ReplaceOperator {
                        old_operator_id,
                        new_pubkey,
                    })
                }
                ProposalKindTag::TriggerRekey => Ok(Self::TriggerRekey),
                ProposalKindTag::ApprovePersistentGrant => Ok(Self::ApprovePersistentGrant {
                    payload_bytes: payload.to_vec(),
                }),
                ProposalKindTag::ApproveOneOffTransaction => Ok(Self::ApproveOneOffTransaction {
                    payload_bytes: payload.to_vec(),
                }),
            }
        }
    }

    impl ProposalKindTag {
        /// Key-rotation proposals require every operator to approve (§3.3).
        #[must_use]
        pub const fn requires_full_quorum(self) -> bool {
            matches!(self, Self::ReplaceOperator | Self::TriggerRekey)
        }
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
        fn from_sql(
            bytes: <Sqlite as Backend>::RawValue<'_>,
        ) -> diesel::deserialize::Result<Self> {
            let s = <String as FromSql<Text, Sqlite>>::from_sql(bytes)?;
            s.parse()
                .map_err(|_| format!("Unknown proposal kind: {s}").into())
        }
    }
}
pub use types::*;

#[derive(Models, Queryable, Debug, Insertable, Selectable)]
#[view(
    NewAeadEncrypted,
    derive(Insertable),
    omit(id),
    attributes_with = "deriveless"
)]
#[diesel(table_name = aead_encrypted, check_for_backend(Sqlite))]
pub struct AeadEncrypted {
    pub id: AeadEncryptedId,
    pub ciphertext: Vec<u8>,
    pub tag: Vec<u8>,
    pub current_nonce: Vec<u8>,
    pub schema_version: i32,
    pub associated_root_key_id: RootKeyHistoryId,
    pub created_at: SqliteTimestamp,
}

#[derive(Models, Queryable, Debug, Insertable, Selectable)]
#[diesel(table_name = root_key_history, check_for_backend(Sqlite))]
#[view(
    NewRootKeyHistory,
    derive(Insertable),
    omit(id),
    attributes_with = "deriveless"
)]
pub struct RootKeyHistory {
    pub id: RootKeyHistoryId,
    pub ciphertext: Vec<u8>,
    pub tag: Vec<u8>,
    pub root_key_encryption_nonce: Vec<u8>,
    pub data_encryption_nonce: Vec<u8>,
    pub schema_version: i32,
    pub salt: Vec<u8>,
}

#[derive(Models, Queryable, Debug, Insertable, Selectable)]
#[diesel(table_name = tls_history, check_for_backend(Sqlite))]
#[view(
    NewTlsHistory,
    derive(Insertable),
    omit(id, created_at),
    attributes_with = "deriveless"
)]
pub struct TlsHistory {
    pub id: TlsHistoryId,
    pub cert: String,
    pub cert_key: String, // PEM Encoded private key
    pub ca_cert: String,  // PEM Encoded certificate for cert signing
    pub ca_key: String,   // PEM Encoded public key for cert signing
    pub created_at: SqliteTimestamp,
}

#[derive(Queryable, Debug, Insertable, Selectable)]
#[diesel(table_name = arbiter_settings, check_for_backend(Sqlite))]
pub struct ArbiterSettings {
    pub id: i32,
    pub root_key_id: Option<i32>, // references root_key_history.id
    pub tls_id: Option<i32>,      // references tls_history.id
}

#[derive(Models, Queryable, Debug, Insertable, Selectable)]
#[diesel(table_name = evm_wallet, check_for_backend(Sqlite))]
#[view(
    NewEvmWallet,
    derive(Insertable),
    omit(id, created_at),
    attributes_with = "deriveless"
)]
pub struct EvmWallet {
    pub id: EvmWalletId,
    pub address: Vec<u8>,
    pub aead_encrypted_id: i32,
    pub created_at: SqliteTimestamp,
}

#[derive(Models, Queryable, Debug, Insertable, Selectable, Clone)]
#[diesel(table_name = schema::evm_wallet_access, check_for_backend(Sqlite))]
#[view(
    NewEvmWalletAccess,
    derive(Insertable),
    omit(id, created_at),
    attributes_with = "deriveless"
)]
#[view(
    CoreEvmWalletAccess,
    derive(Insertable),
    omit(created_at),
    attributes_with = "deriveless"
)]
pub struct EvmWalletAccess {
    pub id: i32,
    pub wallet_id: EvmWalletId,
    pub client_id: i32,
    pub created_at: SqliteTimestamp,
}

#[derive(Models, Queryable, Debug, Insertable, Selectable)]
#[diesel(table_name = schema::client_metadata, check_for_backend(Sqlite))]
pub struct ProgramClientMetadata {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub created_at: SqliteTimestamp,
}

#[derive(Models, Queryable, Debug, Insertable, Selectable)]
#[diesel(table_name = schema::client_metadata_history, check_for_backend(Sqlite))]
pub struct ProgramClientMetadataHistory {
    pub id: i32,
    pub metadata_id: i32,
    pub client_id: i32,
    pub created_at: SqliteTimestamp,
}

#[derive(Models, Queryable, Debug, Insertable, Selectable)]
#[diesel(table_name = schema::program_client, check_for_backend(Sqlite))]
pub struct ProgramClient {
    pub id: ClientId,
    pub public_key: Vec<u8>,
    pub metadata_id: i32,
    pub created_at: SqliteTimestamp,
    pub updated_at: SqliteTimestamp,
}

#[derive(Queryable, Debug)]
#[diesel(table_name = schema::operator_client, check_for_backend(Sqlite))]
pub struct OperatorClient {
    pub id: OperatorIdentityId,
    pub public_key: Vec<u8>,
    pub created_at: SqliteTimestamp,
    pub updated_at: SqliteTimestamp,
}

#[derive(Queryable, Debug)]
#[diesel(table_name = schema::operator, check_for_backend(Sqlite))]
pub struct Operator {
    pub id: OperatorId,
    pub share: Vec<u8>,
    pub share_nonce: Vec<u8>,
    pub share_salt: Vec<u8>,
    pub created_at: SqliteTimestamp,
    pub updated_at: SqliteTimestamp,
}

#[derive(Models, Queryable, Debug, Insertable, Selectable)]
#[diesel(table_name = evm_ether_transfer_limit, check_for_backend(Sqlite))]
#[view(
    NewEvmEtherTransferLimit,
    derive(Insertable),
    omit(id, created_at),
    attributes_with = "deriveless"
)]
pub struct EvmEtherTransferLimit {
    pub id: i32,
    pub window_secs: i32,
    pub max_volume: Vec<u8>,
}

#[derive(Models, Queryable, Debug, Insertable, Selectable)]
#[diesel(table_name = evm_basic_grant, check_for_backend(Sqlite))]
#[view(
    NewEvmBasicGrant,
    derive(Insertable),
    omit(id, created_at),
    attributes_with = "deriveless"
)]
pub struct EvmBasicGrant {
    pub id: i32,
    pub wallet_access_id: i32, // references evm_wallet_access.id
    pub chain_id: ChainId,
    pub valid_from: Option<SqliteTimestamp>,
    pub valid_until: Option<SqliteTimestamp>,
    pub max_gas_fee_per_gas: Option<Vec<u8>>,
    pub max_priority_fee_per_gas: Option<Vec<u8>>,
    pub rate_limit_count: Option<i32>,
    pub rate_limit_window_secs: Option<i32>,
    pub revoked_at: Option<SqliteTimestamp>,
    pub created_at: SqliteTimestamp,
}

#[derive(Models, Queryable, Debug, Insertable, Selectable)]
#[diesel(table_name = evm_transaction_log, check_for_backend(Sqlite))]
#[view(
    NewEvmTransactionLog,
    derive(Insertable),
    omit(id),
    attributes_with = "deriveless"
)]
pub struct EvmTransactionLog {
    pub id: i32,
    pub grant_id: i32,
    pub wallet_access_id: i32,
    pub chain_id: ChainId,
    pub eth_value: Vec<u8>,
    pub signed_at: SqliteTimestamp,
}

#[derive(Models, Queryable, Debug, Insertable, Selectable)]
#[diesel(table_name = evm_ether_transfer_grant, check_for_backend(Sqlite))]
#[view(
    NewEvmEtherTransferGrant,
    derive(Insertable),
    omit(id),
    attributes_with = "deriveless"
)]
pub struct EvmEtherTransferGrant {
    pub id: i32,
    pub basic_grant_id: i32,
    pub limit_id: i32, // references evm_ether_transfer_limit.id
}

#[derive(Models, Queryable, Debug, Insertable, Selectable)]
#[diesel(table_name = evm_ether_transfer_grant_target, check_for_backend(Sqlite))]
#[view(
    NewEvmEtherTransferGrantTarget,
    derive(Insertable),
    omit(id),
    attributes_with = "deriveless"
)]
pub struct EvmEtherTransferGrantTarget {
    pub id: i32,
    pub grant_id: i32,
    pub address: Vec<u8>,
}

#[derive(Models, Queryable, Debug, Insertable, Selectable)]
#[diesel(table_name = evm_token_transfer_grant, check_for_backend(Sqlite))]
#[view(
    NewEvmTokenTransferGrant,
    derive(Insertable),
    omit(id),
    attributes_with = "deriveless"
)]
pub struct EvmTokenTransferGrant {
    pub id: i32,
    pub basic_grant_id: i32,
    pub token_contract: Vec<u8>,
    pub receiver: Option<Vec<u8>>,
}

#[derive(Models, Queryable, Debug, Insertable, Selectable)]
#[diesel(table_name = evm_token_transfer_volume_limit, check_for_backend(Sqlite))]
#[view(
    NewEvmTokenTransferVolumeLimit,
    derive(Insertable),
    omit(id),
    attributes_with = "deriveless"
)]
pub struct EvmTokenTransferVolumeLimit {
    pub id: i32,
    pub grant_id: i32,
    pub window_secs: i32,
    pub max_volume: Vec<u8>,
}

#[derive(Models, Queryable, Debug, Insertable, Selectable)]
#[diesel(table_name = evm_token_transfer_log, check_for_backend(Sqlite))]
#[view(
    NewEvmTokenTransferLog,
    derive(Insertable),
    omit(id, created_at),
    attributes_with = "deriveless"
)]
pub struct EvmTokenTransferLog {
    pub id: i32,
    pub grant_id: i32,
    pub log_id: i32,
    pub chain_id: ChainId,
    pub token_contract: Vec<u8>,
    pub recipient_address: Vec<u8>,
    pub value: Vec<u8>,
    pub created_at: SqliteTimestamp,
}

#[derive(Models, Queryable, Debug, Insertable, Selectable)]
#[diesel(table_name = integrity_envelope, check_for_backend(Sqlite))]
#[view(
    NewIntegrityEnvelope,
    derive(Insertable),
    omit(id, signed_at, created_at),
    attributes_with = "deriveless"
)]
pub struct IntegrityEnvelope {
    pub id: i32,
    pub entity_kind: String,
    pub entity_id: Vec<u8>,
    pub payload_version: i32,
    pub key_version: RootKeyHistoryId,
    pub mac: Vec<u8>,
    pub signed_at: SqliteTimestamp,
    pub created_at: SqliteTimestamp,
}

#[derive(Debug, Queryable, Selectable, Identifiable)]
#[diesel(table_name = schema::proposal, check_for_backend(Sqlite))]
pub struct Proposal {
    pub id: i32,
    pub kind: ProposalKindTag,
    pub payload: Vec<u8>,
    pub initiator_id: i32,
    pub created_at: SqliteTimestamp,
    pub expires_at: SqliteTimestamp,
    pub status: ProposalStatus,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = schema::proposal, check_for_backend(Sqlite))]
pub struct NewProposal {
    pub kind: ProposalKindTag,
    pub payload: Vec<u8>,
    pub initiator_id: i32,
    // status defaults to 'pending' at the DB layer
    pub expires_at: SqliteTimestamp,
}

#[derive(Debug, Queryable, Selectable, Identifiable)]
#[diesel(table_name = schema::proposal_vote, check_for_backend(Sqlite))]
pub struct ProposalVote {
    pub id: i32,
    pub proposal_id: i32,
    pub operator_id: i32,
    pub approve: bool,
    pub signature: Vec<u8>,
    pub voted_at: SqliteTimestamp,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = schema::proposal_vote, check_for_backend(Sqlite))]
pub struct NewProposalVote {
    pub proposal_id: i32,
    pub operator_id: i32,
    pub approve: bool,
    pub signature: Vec<u8>,
}


#[derive(Debug, Insertable)]
#[diesel(table_name = schema::proposal_result, check_for_backend(Sqlite))]
pub struct NewProposalResult {
    pub proposal_id: i32,
    pub data: Vec<u8>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = schema::recovery_proposal_vote, check_for_backend(Sqlite))]
pub struct NewRecoveryProposalVote {
    pub proposal_id: i32,
    pub recovery_operator_id: i32,
    pub approve: bool,
    pub signature: Vec<u8>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = schema::recovery_wakeup_request, check_for_backend(Sqlite))]
pub struct NewRecoveryWakeupRequest {
    pub requested_by: i32,
}