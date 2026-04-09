use argon2::password_hash::Salt as ArgonSalt;

use rand::{
    Rng as _, SeedableRng,
    rngs::{StdRng, SysRng},
};

pub const ROOT_KEY_TAG: &[u8] = b"arbiter/seal/v1";
pub const TAG: &[u8] = b"arbiter/private-key/v1";

pub const NONCE_LENGTH: usize = 24;

#[derive(Default)]
pub struct Nonce(pub [u8; NONCE_LENGTH]);
impl Nonce {
    pub fn increment(&mut self) {
        for i in (0..self.0.len()).rev() {
            if let Some(byte) = self.0.get_mut(i) {
                if *byte == 0xFF {
                    *byte = 0;
                } else {
                    *byte += 1;
                    break;
                }
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
    let mut rng =
        StdRng::try_from_rng(&mut SysRng).expect("Rng failure is unrecoverable and should panic");
    rng.fill_bytes(&mut salt);
    salt
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::derive_key;
    use arbiter_crypto::safecell::{SafeCell, SafeCellHandle as _};

    #[test]
    pub fn derive_seal_key_deterministic() {
        static PASSWORD: &[u8] = b"password";
        let password = SafeCell::new(PASSWORD.to_vec());
        let password2 = SafeCell::new(PASSWORD.to_vec());
        let salt = generate_salt();

        let mut key1 = derive_key(password, &salt);
        let mut key2 = derive_key(password2, &salt);

        let key1_reader = key1.0.read();
        let key2_reader = key2.0.read();

        assert_eq!(&*key1_reader, &*key2_reader);
    }

    #[test]
    pub fn successful_derive() {
        static PASSWORD: &[u8] = b"password";
        let password = SafeCell::new(PASSWORD.to_vec());
        let salt = generate_salt();

        let mut key = derive_key(password, &salt);
        let key_reader = key.0.read();

        assert_ne!(key_reader.as_slice(), &[0u8; 32][..]);
    }

    #[test]
    // We should fuzz this
    pub fn nonce_increment() {
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
