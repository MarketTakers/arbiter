use ml_dsa::{
    EncodedVerifyingKey, MlDsa87, Signature as MlDsaSignature, VerifyingKey as MlDsaVerifyingKey,
};

pub type KeyParams = MlDsa87;

#[derive(Clone, Debug, PartialEq)]
pub struct PublicKey(Box<MlDsaVerifyingKey<KeyParams>>);

#[derive(Clone, Debug, PartialEq)]
pub struct Signature(Box<MlDsaSignature<KeyParams>>);

impl PublicKey {
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.encode().to_vec()
    }

    pub fn verify(&self, nonce: i32, context: &[u8], signature: &Signature) -> bool {
        self.0.verify_with_context(&format_challenge(nonce, self), context, &signature.0)
    }
}

impl Signature {
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.encode().to_vec()
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

impl TryFrom<&'_ [u8]> for PublicKey {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let encoded = EncodedVerifyingKey::<KeyParams>::try_from(value).map_err(|_| ())?;
        Ok(Self(Box::new(MlDsaVerifyingKey::decode(&encoded))))
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

pub fn format_challenge(nonce: i32, pubkey: &PublicKey) -> Vec<u8> {
    arbiter_proto::format_challenge(nonce, &pubkey.to_bytes())
}

#[cfg(test)]
mod tests {
    use ml_dsa::{KeyGen, MlDsa87, signature::Keypair as _};

    use super::{PublicKey, Signature};

    #[test]
    fn public_key_round_trip_decodes() {
        let key = MlDsa87::key_gen(&mut rand::rng());
        let encoded = PublicKey::from(key.verifying_key()).to_bytes();

        let decoded = PublicKey::try_from(encoded.as_slice()).expect("public key should decode");

        assert_eq!(decoded, PublicKey::from(key.verifying_key()));
    }

    #[test]
    fn signature_round_trip_decodes() {
        let key = MlDsa87::key_gen(&mut rand::rng());
        let challenge = b"challenge";
        let signature = key
            .signing_key()
            .sign_deterministic(challenge, arbiter_proto::CLIENT_CONTEXT)
            .expect("signature should be created");

        let decoded = Signature::try_from(signature.encode().to_vec().as_slice()).expect("signature should decode");

        assert_eq!(decoded, Signature::from(signature));
    }

    #[test]
    fn challenge_verification_uses_context_and_canonical_key_bytes() {
        let key = MlDsa87::key_gen(&mut rand::rng());
        let public_key = PublicKey::from(key.verifying_key());
        let nonce = 17;
        let challenge =
            arbiter_proto::format_challenge(nonce, &public_key.to_bytes());
        let signature = key
            .signing_key()
            .sign_deterministic(&challenge, arbiter_proto::CLIENT_CONTEXT)
            .expect("signature should be created")
            .into();

        assert!(public_key.verify(nonce, arbiter_proto::CLIENT_CONTEXT, &signature));
        assert!(!public_key.verify(nonce, arbiter_proto::USERAGENT_CONTEXT, &signature));
    }
}
