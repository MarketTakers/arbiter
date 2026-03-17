use arbiter_proto::{
    proto::{
        self,
        evm::{
            EtherTransferSettings as ProtoEtherTransferSettings, EvmError as ProtoEvmError,
            EvmGrantCreateRequest, EvmGrantCreateResponse, EvmGrantDeleteRequest,
            EvmGrantDeleteResponse, EvmGrantList, EvmGrantListResponse, GrantEntry,
            SharedSettings as ProtoSharedSettings, SpecificGrant as ProtoSpecificGrant,
            TokenTransferSettings as ProtoTokenTransferSettings,
            VolumeRateLimit as ProtoVolumeRateLimit, WalletCreateResponse, WalletEntry, WalletList,
            WalletListResponse, evm_grant_create_response::Result as EvmGrantCreateResult,
            evm_grant_delete_response::Result as EvmGrantDeleteResult,
            evm_grant_list_response::Result as EvmGrantListResult,
            specific_grant::Grant as ProtoSpecificGrantType,
            wallet_create_response::Result as WalletCreateResult,
            wallet_list_response::Result as WalletListResult,
        },
        user_agent::{
            AuthChallenge as ProtoAuthChallenge, AuthChallengeRequest as ProtoAuthChallengeRequest,
            AuthChallengeSolution as ProtoAuthChallengeSolution, AuthResult as ProtoAuthResult,
            BootstrapEncryptedKey as ProtoBootstrapEncryptedKey,
            BootstrapResult as ProtoBootstrapResult, ClientConnectionCancel,
            ClientConnectionRequest, ClientConnectionResponse, KeyType as ProtoKeyType,
            UnsealEncryptedKey as ProtoUnsealEncryptedKey, UnsealResult as ProtoUnsealResult,
            UnsealStart, UnsealStartResponse, UserAgentRequest, UserAgentResponse,
            VaultState as ProtoVaultState, user_agent_request::Payload as UserAgentRequestPayload,
            user_agent_response::Payload as UserAgentResponsePayload,
        },
    },
    transport::{Bi, Error as TransportError, Receiver, Sender, grpc::GrpcBi},
};
use async_trait::async_trait;
use tonic::{Status, Streaming};
use tracing::{info, warn};

use crate::{
    actors::user_agent::{
        self, AuthPublicKey, OutOfBand as DomainResponse, UserAgentConnection, auth,
    },
    db::models::KeyType,
    evm::policies::{
        Grant, SharedGrantSettings, SpecificGrant, TransactionRateLimit, VolumeRateLimit,
        ether_transfer, token_transfers,
    },
};
use alloy::primitives::{Address, U256};
use chrono::{DateTime, TimeZone, Utc};

pub struct AuthTransportAdapter<'a>(&'a mut GrpcBi<UserAgentRequest, UserAgentResponse>);

#[async_trait]
impl Sender<Result<auth::Outbound, auth::Error>> for AuthTransportAdapter<'_> {
    async fn send(
        &mut self,
        item: Result<auth::Outbound, auth::Error>,
    ) -> Result<(), TransportError> {
        use auth::{Error, Outbound};
        let response = match item {
            Ok(Outbound::AuthChallenge { nonce }) => Ok(UserAgentResponsePayload::AuthChallenge(
                ProtoAuthChallenge { nonce },
            )),
            Ok(Outbound::AuthSuccess) => Ok(UserAgentResponsePayload::AuthResult(
                ProtoAuthResult::Success.into(),
            )),

            Err(Error::UnregisteredPublicKey) => Ok(UserAgentResponsePayload::AuthResult(
                ProtoAuthResult::InvalidKey.into(),
            )),
            Err(Error::InvalidChallengeSolution) => Ok(UserAgentResponsePayload::AuthResult(
                ProtoAuthResult::InvalidSignature.into(),
            )),
            Err(Error::InvalidBootstrapToken) => Ok(UserAgentResponsePayload::BootstrapResult(
                ProtoAuthResult::TokenInvalid.into(),
            )),
            Err(Error::Internal { details }) => Err(Status::internal(details)),
            Err(Error::Transport) => Err(Status::unavailable("transport error")),
        };
        self.0
            .send(response.map(|r| UserAgentResponse { payload: Some(r) }))
            .await
    }
}

#[async_trait]
impl Receiver<auth::Inbound> for AuthTransportAdapter<'_> {
    async fn recv(&mut self) -> Option<auth::Inbound> {
        let Ok(UserAgentRequest {
            payload: Some(payload),
        }) = self.0.recv().await?
        else {
            warn!(
                event = "received request with empty payload",
                "grpc.useragent.auth_adapter"
            );
            return None;
        };

        match payload {
            UserAgentRequestPayload::AuthChallengeRequest(ProtoAuthChallengeRequest {
                pubkey,
                bootstrap_token,
                key_type,
            }) => {
                let Ok(key_type) = ProtoKeyType::try_from(key_type) else {
                    warn!(
                        event = "received request with invalid key type",
                        "grpc.useragent.auth_adapter"
                    );
                    return None;
                };
                let key_type = match key_type {
                    ProtoKeyType::Ed25519 => KeyType::Ed25519,
                    ProtoKeyType::EcdsaSecp256k1 => KeyType::EcdsaSecp256k1,
                    ProtoKeyType::Rsa => KeyType::Rsa,
                    ProtoKeyType::Unspecified => {
                        warn!(
                            event = "received request with unspecified key type",
                            "grpc.useragent.auth_adapter"
                        );
                        return None;
                    }
                };
                let Ok(pubkey) = AuthPublicKey::try_from((key_type, pubkey)) else {
                    warn!(
                        event = "received request with invalid public key",
                        "grpc.useragent.auth_adapter"
                    );
                    return None;
                };

                Some(auth::Inbound::AuthChallengeRequest {
                    pubkey,
                    bootstrap_token,
                })
            }
            UserAgentRequestPayload::AuthChallengeSolution(ProtoAuthChallengeSolution {
                signature,
            }) => Some(auth::Inbound::AuthChallengeSolution { signature }),
            _ => None, // Ignore other request types for this adapter
        }
    }
}
impl Bi<auth::Inbound, Result<auth::Outbound, auth::Error>> for AuthTransportAdapter<'_> {}

pub async fn start(
    conn: &mut UserAgentConnection,
    bi: &mut GrpcBi<UserAgentRequest, UserAgentResponse>,
) -> Result<AuthPublicKey, auth::Error> {
    let mut transport = AuthTransportAdapter(bi);
    auth::authenticate(conn, transport).await
}
