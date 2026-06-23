use anyhow::anyhow;
use arbiter_crypto::authn::{self, AuthChallenge, USERAGENT_CONTEXT};
use flutter_rust_bridge::frb;

#[frb(opaque)]
pub struct MldsaKey(authn::SigningKey);

impl MldsaKey {
    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        let bytes: [u8; 32] = bytes
            .try_into()
            .map_err(|_| anyhow!("Invalid key length"))?;
        Ok(Self(authn::SigningKey::from_seed(bytes)))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_seed().to_vec()
    }

    pub fn sign(&self, message: &[u8]) -> anyhow::Result<Vec<u8>> {
        Ok(self.0.sign_message(message, USERAGENT_CONTEXT)?.to_bytes())
    }

    pub fn generate() -> Self {
        Self(authn::SigningKey::generate())
    }

    pub fn get_public_key(&self) -> Vec<u8> {
        self.0.public_key().to_bytes().to_vec()
    }
}

pub fn format_challenge(random: Vec<u8>, timestamp: i64) -> Result<Vec<u8>, String> {
    let challenge = AuthChallenge::from_parts(&random, timestamp)
        .map_err(|_| "Invalid nonce length".to_string())?;

    Ok(challenge.format())
}
