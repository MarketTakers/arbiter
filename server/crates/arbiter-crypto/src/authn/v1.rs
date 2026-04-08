use std::hash::Hash;

use base64::{Engine as _, prelude::BASE64_STANDARD};
use ml_dsa::{
    EncodedVerifyingKey, Error, KeyGen, MlDsa87, Seed, Signature as MlDsaSignature,
    SigningKey as MlDsaSigningKey, VerifyingKey as MlDsaVerifyingKey, signature::Keypair as _,
};

pub static CLIENT_CONTEXT: &[u8] = b"arbiter_client";
pub static USERAGENT_CONTEXT: &[u8] = b"arbiter_user_agent";

pub fn format_challenge(nonce: i32, pubkey: &[u8]) -> Vec<u8> {
    let concat_form = format!("{}:{}", nonce, BASE64_STANDARD.encode(pubkey));
    concat_form.into_bytes()
}

pub type KeyParams = MlDsa87;

#[derive(Clone, Debug, PartialEq)]
pub struct PublicKey(Box<MlDsaVerifyingKey<KeyParams>>);

impl Hash for PublicKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.to_bytes().hash(state);
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Signature(Box<MlDsaSignature<KeyParams>>);

#[derive(Debug)]
pub struct SigningKey(Box<MlDsaSigningKey<KeyParams>>);

impl PublicKey {
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.encode().to_vec()
    }

    pub fn verify(&self, nonce: i32, context: &[u8], signature: &Signature) -> bool {
        self.0.verify_with_context(
            &format_challenge(nonce, &self.to_bytes()),
            context,
            &signature.0,
        )
    }
}

impl Signature {
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.encode().to_vec()
    }
}

impl SigningKey {
    pub fn generate() -> Self {
        Self(Box::new(KeyParams::key_gen(&mut rand::rng())))
    }

    pub fn from_seed(seed: [u8; 32]) -> Self {
        Self(Box::new(KeyParams::from_seed(&Seed::from(seed))))
    }

    pub fn to_seed(&self) -> [u8; 32] {
        self.0.to_seed().into()
    }

    pub fn public_key(&self) -> PublicKey {
        self.0.verifying_key().into()
    }

    pub fn sign_message(&self, message: &[u8], context: &[u8]) -> Result<Signature, Error> {
        self.0
            .signing_key()
            .sign_deterministic(message, context)
            .map(Into::into)
    }

    pub fn sign_challenge(&self, nonce: i32, context: &[u8]) -> Result<Signature, Error> {
        self.sign_message(
            &format_challenge(nonce, &self.public_key().to_bytes()),
            context,
        )
    }
}

impl From<MlDsaVerifyingKey<KeyParams>> for PublicKey {
    fn from(value: MlDsaVerifyingKey<KeyParams>) -> Self {
        Self(Box::new(value))
    }
}

impl From<MlDsaSignature<KeyParams>> for Signature {
    fn from(value: MlDsaSignature<KeyParams>) -> Self {
        Self(Box::new(value))
    }
}

impl From<MlDsaSigningKey<KeyParams>> for SigningKey {
    fn from(value: MlDsaSigningKey<KeyParams>) -> Self {
        Self(Box::new(value))
    }
}

impl TryFrom<Vec<u8>> for PublicKey {
    type Error = ();

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(value.as_slice())
    }
}

impl TryFrom<&'_ [u8]> for PublicKey {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let encoded = EncodedVerifyingKey::<KeyParams>::try_from(value).map_err(|_| ())?;
        Ok(Self(Box::new(MlDsaVerifyingKey::decode(&encoded))))
    }
}

impl TryFrom<Vec<u8>> for Signature {
    type Error = ();

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(value.as_slice())
    }
}

impl TryFrom<&'_ [u8]> for Signature {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        MlDsaSignature::try_from(value)
            .map(|sig| Self(Box::new(sig)))
            .map_err(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use ml_dsa::{KeyGen, MlDsa87, signature::Keypair as _};

    use super::{CLIENT_CONTEXT, PublicKey, Signature, SigningKey, USERAGENT_CONTEXT};

    #[test]
    fn public_key_round_trip_decodes() {
        let key = MlDsa87::key_gen(&mut rand::rng());
        let encoded = PublicKey::from(key.verifying_key()).to_bytes();

        let decoded = PublicKey::try_from(encoded.as_slice()).expect("public key should decode");

        assert_eq!(decoded, PublicKey::from(key.verifying_key()));
    }

    #[test]
    fn signature_round_trip_decodes() {
        let key = SigningKey::generate();
        let signature = key
            .sign_message(b"challenge", CLIENT_CONTEXT)
            .expect("signature should be created");

        let decoded =
            Signature::try_from(signature.to_bytes().as_slice()).expect("signature should decode");

        assert_eq!(decoded, signature);
    }

    #[test]
    fn challenge_verification_uses_context_and_canonical_key_bytes() {
        let key = SigningKey::generate();
        let public_key = key.public_key();
        let nonce = 17;
        let signature = key
            .sign_challenge(nonce, CLIENT_CONTEXT)
            .expect("signature should be created");

        assert!(public_key.verify(nonce, CLIENT_CONTEXT, &signature));
        assert!(!public_key.verify(nonce, USERAGENT_CONTEXT, &signature));
    }

    #[test]
    fn signing_key_round_trip_seed_preserves_public_key_and_signing() {
        let original = SigningKey::generate();
        let restored = SigningKey::from_seed(original.to_seed());

        assert_eq!(restored.public_key(), original.public_key());

        let signature = restored
            .sign_challenge(9, CLIENT_CONTEXT)
            .expect("signature should be created");

        assert!(restored.public_key().verify(9, CLIENT_CONTEXT, &signature));
    }
}
