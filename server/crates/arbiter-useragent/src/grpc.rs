use arbiter_proto::{
    proto::{
        arbiter_service_client::ArbiterServiceClient,
        user_agent::{UserAgentRequest, UserAgentResponse},
    },
    transport::{IdentityRecvConverter, IdentitySendConverter, grpc},
    url::ArbiterUrl,
};
use kameo::actor::{ActorRef, Spawn};

use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use tonic::transport::ClientTlsConfig;

use super::{SigningKeyEnum, UserAgentActor};

#[derive(Debug, thiserror::Error)]
pub enum ConnectError {
    #[error("Could establish connection")]
    Connection(#[from] tonic::transport::Error),

    #[error("Invalid server URI")]
    InvalidUri(#[from] http::uri::InvalidUri),

    #[error("Invalid CA certificate")]
    InvalidCaCert(#[from] webpki::Error),

    #[error("gRPC error")]
    Grpc(#[from] tonic::Status),
}

pub type UserAgentGrpc = ActorRef<
    UserAgentActor<
        grpc::GrpcAdapter<
            IdentityRecvConverter<UserAgentResponse>,
            IdentitySendConverter<UserAgentRequest>,
        >,
    >,
>;
pub async fn connect_grpc(
    url: ArbiterUrl,
    key: SigningKeyEnum,
) -> Result<UserAgentGrpc, ConnectError> {
    let bootstrap_token = url.bootstrap_token.clone();
    let anchor = webpki::anchor_from_trusted_cert(&url.ca_cert)?.to_owned();
    let tls = ClientTlsConfig::new().trust_anchor(anchor);

    // TODO: if `host` is localhost, we need to verify server's process authenticity
    let channel = tonic::transport::Channel::from_shared(format!("{}:{}", url.host, url.port))?
        .tls_config(tls)?
        .connect()
        .await?;

    let mut client = ArbiterServiceClient::new(channel);
    let (tx, rx) = mpsc::channel(16);
    let bistream = client.user_agent(ReceiverStream::new(rx)).await?;
    let bistream = bistream.into_inner();

    let adapter = grpc::GrpcAdapter::new(
        tx,
        bistream,
        IdentityRecvConverter::new(),
        IdentitySendConverter::new(),
    );

    let actor = UserAgentActor::spawn(UserAgentActor::new(key, bootstrap_token, adapter));

    Ok(actor)
}
