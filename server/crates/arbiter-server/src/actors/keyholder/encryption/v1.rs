use std::ops::Deref as _;

use argon2::{Algorithm, Argon2, password_hash::Salt as ArgonSalt};
use chacha20poly1305::{
    AeadInPlace, Key, KeyInit as _, XChaCha20Poly1305, XNonce,
    aead::{AeadMut, Error, Payload},
};
use hmac::Mac as _;
use rand::{
    Rng as _, SeedableRng,
    rngs::{StdRng, SysRng},
};

use crate::safe_cell::{SafeCell, SafeCellHandle as _};

pub const ROOT_KEY_TAG: &[u8] = "arbiter/seal/v1".as_bytes();
pub const TAG: &[u8] = "arbiter/private-key/v1".as_bytes();
pub const USERAGENT_INTEGRITY_DERIVE_TAG: &[u8] = "arbiter/useragent/integrity-key/v1".as_bytes();
pub const USERAGENT_INTEGRITY_TAG: &[u8] = "arbiter/useragent/pubkey-entry/v1".as_bytes();

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
        nonce.copy_from_slice(value);
        Ok(Self(nonce))
    }
}

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

/// User password might be of different length, have not enough entropy, etc...
/// Derive a fixed-length key from the password using Argon2id, which is designed for password hashing and key derivation.
pub fn derive_seal_key(mut password: SafeCell<Vec<u8>>, salt: &Salt) -> KeyCell {
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

/// Derives a dedicated key used only for user-agent pubkey integrity tags.
pub fn derive_useragent_integrity_key(seal_key: &mut KeyCell) -> KeyCell {
    type HmacSha256 = hmac::Hmac<sha2::Sha256>;

    let mut derived = SafeCell::new(Key::default());
    seal_key.0.read_inline(|seal_key_bytes| {
        let mut mac = <HmacSha256 as hmac::Mac>::new_from_slice(seal_key_bytes.as_ref())
            .expect("HMAC key initialization must not fail for 32-byte key");
        mac.update(USERAGENT_INTEGRITY_DERIVE_TAG);
        let output = mac.finalize().into_bytes();

        let mut writer = derived.write();
        let writer: &mut [u8] = writer.as_mut();
        writer.copy_from_slice(&output);
    });

    derived.into()
}

/// Computes an integrity tag for a user-agent pubkey DB entry.
pub fn compute_useragent_pubkey_integrity_tag(
    integrity_key: &mut KeyCell,
    key_type_discriminant: i32,
    public_key: &[u8],
) -> [u8; 32] {
    type HmacSha256 = hmac::Hmac<sha2::Sha256>;

    let mut tag = [0u8; 32];
    integrity_key.0.read_inline(|integrity_key_bytes| {
        let mut mac = <HmacSha256 as hmac::Mac>::new_from_slice(integrity_key_bytes.as_ref())
            .expect("HMAC key initialization must not fail for 32-byte key");
        mac.update(USERAGENT_INTEGRITY_TAG);
        mac.update(&key_type_discriminant.to_be_bytes());
        mac.update(public_key);
        tag.copy_from_slice(&mac.finalize().into_bytes());
    });

    tag
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::safe_cell::SafeCell;

    #[test]
    pub fn derive_seal_key_deterministic() {
        static PASSWORD: &[u8] = b"password";
        let password = SafeCell::new(PASSWORD.to_vec());
        let password2 = SafeCell::new(PASSWORD.to_vec());
        let salt = generate_salt();

        let mut key1 = derive_seal_key(password, &salt);
        let mut key2 = derive_seal_key(password2, &salt);

        let key1_reader = key1.0.read();
        let key2_reader = key2.0.read();

        assert_eq!(key1_reader.deref(), key2_reader.deref());
    }

    #[test]
    pub fn successful_derive() {
        static PASSWORD: &[u8] = b"password";
        let password = SafeCell::new(PASSWORD.to_vec());
        let salt = generate_salt();

        let mut key = derive_seal_key(password, &salt);
        let key_reader = key.0.read();
        let key_ref = key_reader.deref();

        assert_ne!(key_ref.as_slice(), &[0u8; 32][..]);
    }

    #[test]
    pub fn encrypt_decrypt() {
        static PASSWORD: &[u8] = b"password";
        let password = SafeCell::new(PASSWORD.to_vec());
        let salt = generate_salt();

        let mut key = derive_seal_key(password, &salt);
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

    #[test]
    pub fn useragent_integrity_tag_deterministic() {
        let salt = generate_salt();
        let mut seal_key = derive_seal_key(SafeCell::new(b"password".to_vec()), &salt);
        let mut integrity_key = derive_useragent_integrity_key(&mut seal_key);
        let t1 = compute_useragent_pubkey_integrity_tag(&mut integrity_key, 1, b"pubkey");
        let t2 = compute_useragent_pubkey_integrity_tag(&mut integrity_key, 1, b"pubkey");
        assert_eq!(t1, t2);
    }

    #[test]
    pub fn useragent_integrity_tag_changes_with_key_type() {
        let salt = generate_salt();
        let mut seal_key = derive_seal_key(SafeCell::new(b"password".to_vec()), &salt);
        let mut integrity_key = derive_useragent_integrity_key(&mut seal_key);
        let t1 = compute_useragent_pubkey_integrity_tag(&mut integrity_key, 1, b"pubkey");
        let t2 = compute_useragent_pubkey_integrity_tag(&mut integrity_key, 2, b"pubkey");
        assert_ne!(t1, t2);
    }
}
