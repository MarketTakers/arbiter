use std::ops::Deref as _;

use argon2::{Algorithm, Argon2};
use chacha20poly1305::{
    AeadInPlace, Key, KeyInit as _, XChaCha20Poly1305, XNonce,
    aead::{AeadMut, Error, Payload},
};
use rand::{
    Rng as _, SeedableRng as _,
    rngs::{StdRng, SysRng},
};

use crate::safe_cell::{SafeCell, SafeCellHandle as _};

pub mod encryption;
pub mod integrity;

use encryption::v1::{Nonce, Salt};

pub struct KeyCell(pub SafeCell<Key>);
impl From<SafeCell<Key>> for KeyCell {
    fn from(value: SafeCell<Key>) -> Self {
        Self(value)
    }
}
impl TryFrom<SafeCell<Vec<u8>>> for KeyCell {
    type Error = ();

    fn try_from(mut value: SafeCell<Vec<u8>>) -> Result<Self, Self::Error> {
        let value = value.read();
        if value.len() != size_of::<Key>() {
            return Err(());
        }
        let cell = SafeCell::new_inline(|cell_write: &mut Key| {
            cell_write.copy_from_slice(&value);
        });
        Ok(Self(cell))
    }
}

impl KeyCell {
    pub fn new_secure_random() -> Self {
        let key = SafeCell::new_inline(|key_buffer: &mut Key| {
            #[allow(
                clippy::unwrap_used,
                reason = "Rng failure is unrecoverable and should panic"
            )]
            let mut rng = StdRng::try_from_rng(&mut SysRng).unwrap();
            rng.fill_bytes(key_buffer);
        });

        key.into()
    }

    pub fn encrypt_in_place(
        &mut self,
        nonce: &Nonce,
        associated_data: &[u8],
        mut buffer: impl AsMut<Vec<u8>>,
    ) -> Result<(), Error> {
        let key_reader = self.0.read();
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
        buffer: &mut SafeCell<Vec<u8>>,
    ) -> Result<(), Error> {
        let key_reader = self.0.read();
        let key_ref = key_reader.deref();
        let cipher = XChaCha20Poly1305::new(key_ref);
        let nonce = XNonce::from_slice(nonce.0.as_ref());
        let mut buffer = buffer.write();
        let buffer: &mut Vec<u8> = buffer.as_mut();
        cipher.decrypt_in_place(nonce, associated_data, buffer)
    }

    pub fn encrypt(
        &mut self,
        nonce: &Nonce,
        associated_data: &[u8],
        plaintext: impl AsRef<[u8]>,
    ) -> Result<Vec<u8>, Error> {
        let key_reader = self.0.read();
        let key_ref = key_reader.deref();
        let mut cipher = XChaCha20Poly1305::new(key_ref);
        let nonce = XNonce::from_slice(nonce.0.as_ref());

        let ciphertext = cipher.encrypt(
            nonce,
            Payload {
                msg: plaintext.as_ref(),
                aad: associated_data,
            },
        )?;
        Ok(ciphertext)
    }
}

/// User password might be of different length, have not enough entropy, etc...
/// Derive a fixed-length key from the password using Argon2id, which is designed for password hashing and key derivation.
pub fn derive_key(mut password: SafeCell<Vec<u8>>, salt: &Salt) -> KeyCell {
    #[allow(clippy::unwrap_used)]
    let params = argon2::Params::new(262_144, 3, 4, None).unwrap();
    let hasher = Argon2::new(Algorithm::Argon2id, argon2::Version::V0x13, params);
    let mut key = SafeCell::new(Key::default());
    password.read_inline(|password_source| {
        let mut key_buffer = key.write();
        let key_buffer: &mut [u8] = key_buffer.as_mut();

        #[allow(
            clippy::unwrap_used,
            reason = "Better fail completely than return a weak key"
        )]
        hasher
            .hash_password_into(password_source.deref(), salt, key_buffer)
            .unwrap();
    });

    key.into()
}

#[cfg(test)]
mod tests {
    use super::{
        derive_key,
        encryption::v1::{Nonce, generate_salt},
    };
    use crate::safe_cell::{SafeCell, SafeCellHandle as _};

    #[test]
    pub fn encrypt_decrypt() {
        static PASSWORD: &[u8] = b"password";
        let password = SafeCell::new(PASSWORD.to_vec());
        let salt = generate_salt();

        let mut key = derive_key(password, &salt);
        let nonce = Nonce(*b"unique nonce 123 1231233"); // 24 bytes for XChaCha20Poly1305
        let associated_data = b"associated data";
        let mut buffer = b"secret data".to_vec();

        key.encrypt_in_place(&nonce, associated_data, &mut buffer)
            .unwrap();
        assert_ne!(buffer, b"secret data");

        let mut buffer = SafeCell::new(buffer);

        key.decrypt_in_place(&nonce, associated_data, &mut buffer)
            .unwrap();

        let buffer = buffer.read();
        assert_eq!(*buffer, b"secret data");
    }
}
