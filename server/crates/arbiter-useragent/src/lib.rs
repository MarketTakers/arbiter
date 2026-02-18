use ed25519_dalek::SigningKey;
use kameo::Actor;
use tonic::transport::CertificateDer;

struct Storage {
    pub identity: SigningKey,
    pub server_ca_cert: CertificateDer<'static>,
}

#[derive(Actor)]
pub struct UserAgent {

}