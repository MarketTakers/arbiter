use argon2::password_hash::Salt as ArgonSalt;
use rand::{
    Rng as _, SeedableRng,
    rngs::{StdRng, SysRng},
};

pub const ROOT_KEY_TAG: &[u8] = "arbiter/seal/v1".as_bytes();
pub const TAG: &[u8] = "arbiter/private-key/v1".as_bytes();

pub const NONCE_LENGTH: usize = 24;

#[derive(Default)]
pub struct Nonce(pub [u8; NONCE_LENGTH]);
impl Nonce {
    pub fn increment(&mut self) {
        for i in (0..self.0.len()).rev() {
            if self.0[i] == 0xFF {
                self.0[i] = 0;
            } else {
                self.0[i] += 1;
                break;
            }
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}
impl<'a> TryFrom<&'a [u8]> for Nonce {
    type Error = ();

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        if value.len() != NONCE_LENGTH {
            return Err(());
        }
        let mut nonce = [0u8; NONCE_LENGTH];
        nonce.copy_from_slice(value);
        Ok(Self(nonce))
    }
}

pub type Salt = [u8; ArgonSalt::RECOMMENDED_LENGTH];

pub fn generate_salt() -> Salt {
    let mut salt = Salt::default();
    #[allow(
        clippy::unwrap_used,
        reason = "Rng failure is unrecoverable and should panic"
    )]
    let mut rng = StdRng::try_from_rng(&mut SysRng).unwrap();
    rng.fill_bytes(&mut salt);
    salt
}

#[cfg(test)]
mod tests {
    use std::ops::Deref as _;

    use super::*;
    use crate::crypto::derive_key;
    use arbiter_crypto::safecell::{SafeCell, SafeCellHandle as _};

    #[test]
    fn derive_seal_key_deterministic() {
        static PASSWORD: &[u8] = b"password";
        let password = SafeCell::new(PASSWORD.to_vec());
        let password2 = SafeCell::new(PASSWORD.to_vec());
        let salt = generate_salt();

        let mut key1 = derive_key(password, &salt);
        let mut key2 = derive_key(password2, &salt);

        let key1_reader = key1.0.read();
        let key2_reader = key2.0.read();

        assert_eq!(key1_reader.deref(), key2_reader.deref());
    }

    #[test]
    fn successful_derive() {
        static PASSWORD: &[u8] = b"password";
        let password = SafeCell::new(PASSWORD.to_vec());
        let salt = generate_salt();

        let mut key = derive_key(password, &salt);
        let key_reader = key.0.read();
        let key_ref = key_reader.deref();

        assert_ne!(key_ref.as_slice(), &[0u8; 32][..]);
    }

    #[test]
    // We should fuzz this
    fn test_nonce_increment() {
        let mut nonce = Nonce([0u8; NONCE_LENGTH]);
        nonce.increment();

        assert_eq!(
            nonce.0,
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1
            ]
        );
    }
}
