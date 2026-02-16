use std::ops::Deref as _;

use argon2::{Algorithm, Argon2, password_hash::Salt as ArgonSalt};
use chacha20poly1305::{
    AeadInPlace, Key, KeyInit as _, XChaCha20Poly1305, XNonce,
    aead::{AeadMut, Error, Payload},
};
use memsafe::MemSafe;
use rand::{
    Rng as _, SeedableRng,
    rngs::{StdRng, SysRng},
};

pub const ROOT_KEY_TAG: &[u8] = "arbiter/seal/v1".as_bytes();
pub const TAG: &[u8] = "arbiter/private-key/v1".as_bytes();

pub const NONCE_LENGTH: usize = 24;

#[derive(Default)]
pub struct Nonce([u8; NONCE_LENGTH]);
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
        nonce.copy_from_slice(&value);
        Ok(Self(nonce))
    }
}

pub struct KeyCell(pub(super) MemSafe<Key>);
impl From<MemSafe<Key>> for KeyCell {
    fn from(value: MemSafe<Key>) -> Self {
        Self(value)
    }
}
impl TryFrom<MemSafe<Vec<u8>>> for KeyCell {
    type Error = ();

    fn try_from(mut value: MemSafe<Vec<u8>>) -> Result<Self, Self::Error> {
        let value = value.read().unwrap();
        if value.len() != size_of::<Key>() {
            return Err(());
        }
        let mut cell = MemSafe::new(Key::default()).unwrap();
        {
            let mut cell_write = cell.write().unwrap();
            let cell_slice: &mut [u8] = cell_write.as_mut();
            cell_slice.copy_from_slice(&value);
        }
        Ok(Self(cell))
    }
}

impl KeyCell {
    pub fn new_secure_random() -> Self {
        let mut key = MemSafe::new(Key::default()).unwrap();
        {
            let mut key_buffer = key.write().unwrap();
            let key_buffer: &mut [u8] = key_buffer.as_mut();

            let mut rng = StdRng::try_from_rng(&mut SysRng).unwrap();
            rng.fill_bytes(key_buffer);
        }

        key.into()
    }

    pub fn into_inner(self) -> MemSafe<Key> {
        self.0
    }

    pub fn encrypt_in_place(
        &mut self,
        nonce: &Nonce,
        associated_data: &[u8],
        mut buffer: impl AsMut<Vec<u8>>,
    ) -> Result<(), Error> {
        let key_reader = self.0.read().unwrap();
        let key_ref = key_reader.deref();
        let cipher = XChaCha20Poly1305::new(key_ref);
        let nonce = XNonce::from_slice(nonce.0.as_ref());
        let buffer = buffer.as_mut();
        cipher.encrypt_in_place(nonce, associated_data, buffer)
    }
    pub fn decrypt_in_place(
        &mut self,
        nonce: &Nonce,
        associated_data: &[u8],
        buffer: &mut MemSafe<Vec<u8>>,
    ) -> Result<(), Error> {
        let key_reader = self.0.read().unwrap();
        let key_ref = key_reader.deref();
        let cipher = XChaCha20Poly1305::new(key_ref);
        let nonce = XNonce::from_slice(nonce.0.as_ref());
        let mut buffer = buffer.write().unwrap();
        let buffer: &mut Vec<u8> = buffer.as_mut();
        cipher.decrypt_in_place(nonce, associated_data, buffer)
    }

    pub fn encrypt(
        &mut self,
        nonce: &Nonce,
        associated_data: &[u8],
        plaintext: impl AsRef<[u8]>,
    ) -> Result<Vec<u8>, Error> {
        let key_reader = self.0.read().unwrap();
        let key_ref = key_reader.deref();
        let mut cipher = XChaCha20Poly1305::new(key_ref);
        let nonce = XNonce::from_slice(nonce.0.as_ref());


        let ciphertext = cipher.encrypt(
            &nonce,
            Payload {
                msg: plaintext.as_ref(),
                aad: associated_data,
            },
        )?;
        Ok(ciphertext)
    }
}

pub type Salt = [u8; ArgonSalt::RECOMMENDED_LENGTH];

pub(super) fn generate_salt() -> Salt {
    let mut salt = Salt::default();
    let mut rng = StdRng::try_from_rng(&mut SysRng).unwrap();
    rng.fill_bytes(&mut salt);
    salt
}

/// User password might be of different length, have not enough entropy, etc...
/// Derive a fixed-length key from the password using Argon2id, which is designed for password hashing and key derivation.
pub(super) fn derive_seal_key(mut password: MemSafe<Vec<u8>>, salt: &Salt) -> KeyCell {
    let params = argon2::Params::new(262_144, 3, 4, None).unwrap();
    let hasher = Argon2::new(Algorithm::Argon2id, argon2::Version::V0x13, params);
    let mut key = MemSafe::new(Key::default()).unwrap();
    {
        let password_source = password.read().unwrap();
        let mut key_buffer = key.write().unwrap();
        let key_buffer: &mut [u8] = key_buffer.as_mut();

        hasher
            .hash_password_into(password_source.deref(), salt, key_buffer)
            .unwrap();
    }

    key.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use memsafe::MemSafe;

    #[test]
    pub fn derive_seal_key_deterministic() {
        static PASSWORD: &[u8] = b"password";
        let password = MemSafe::new(PASSWORD.to_vec()).unwrap();
        let password2 = MemSafe::new(PASSWORD.to_vec()).unwrap();
        let salt = generate_salt();

        let mut key1 = derive_seal_key(password, &salt);
        let mut key2 = derive_seal_key(password2, &salt);

        let key1_reader = key1.0.read().unwrap();
        let key2_reader = key2.0.read().unwrap();

        assert_eq!(key1_reader.deref(), key2_reader.deref());
    }

    #[test]
    pub fn successful_derive() {
        static PASSWORD: &[u8] = b"password";
        let password = MemSafe::new(PASSWORD.to_vec()).unwrap();
        let salt = generate_salt();

        let mut key = derive_seal_key(password, &salt);
        let key_reader = key.0.read().unwrap();
        let key_ref = key_reader.deref();

        assert_ne!(key_ref.as_slice(), &[0u8; 32][..]);
    }

    #[test]
    pub fn encrypt_decrypt() {
        static PASSWORD: &[u8] = b"password";
        let password = MemSafe::new(PASSWORD.to_vec()).unwrap();
        let salt = generate_salt();

        let mut key = derive_seal_key(password, &salt);
        let nonce = Nonce(*b"unique nonce 123 1231233"); // 24 bytes for XChaCha20Poly1305
        let associated_data = b"associated data";
        let mut buffer = b"secret data".to_vec();

        key.encrypt_in_place(&nonce, associated_data, &mut buffer)
            .unwrap();
        assert_ne!(buffer, b"secret data");

        let mut buffer = MemSafe::new(buffer).unwrap();

        key.decrypt_in_place(&nonce, associated_data, &mut buffer)
            .unwrap();

        let buffer = buffer.read().unwrap();
        assert_eq!(*buffer, b"secret data");
    }

    #[test]
    // We should fuzz this
    pub fn test_nonce_increment() {
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
