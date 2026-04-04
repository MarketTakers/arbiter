use crate::{crypto::KeyCell, safe_cell::SafeCellHandle as _};
use chacha20poly1305::Key;
use hmac::Mac as _;

pub const USERAGENT_INTEGRITY_DERIVE_TAG: &[u8] = "arbiter/useragent/integrity-key/v1".as_bytes();
pub const USERAGENT_INTEGRITY_TAG: &[u8] = "arbiter/useragent/pubkey-entry/v1".as_bytes();

/// Computes an integrity tag for a specific domain and payload shape.
pub fn compute_integrity_tag<'a, I>(
    integrity_key: &mut KeyCell,
    purpose_tag: &[u8],
    data_parts: I,
) -> [u8; 32]
where
    I: IntoIterator<Item = &'a [u8]>,
{
    type HmacSha256 = hmac::Hmac<sha2::Sha256>;

    let mut output_tag = [0u8; 32];
    integrity_key.0.read_inline(|integrity_key_bytes: &Key| {
        let mut mac = <HmacSha256 as hmac::Mac>::new_from_slice(integrity_key_bytes.as_ref())
            .expect("HMAC key initialization must not fail for 32-byte key");
        mac.update(purpose_tag);
        for data_part in data_parts {
            mac.update(data_part);
        }
        output_tag.copy_from_slice(&mac.finalize().into_bytes());
    });

    output_tag
}

#[cfg(test)]
mod tests {
    use crate::{
        crypto::{derive_key, encryption::v1::generate_salt},
        safe_cell::{SafeCell, SafeCellHandle as _},
    };

    use super::{USERAGENT_INTEGRITY_TAG, compute_integrity_tag};

    #[test]
    pub fn integrity_tag_deterministic() {
        let salt = generate_salt();
        let mut integrity_key = derive_key(SafeCell::new(b"password".to_vec()), &salt);
        let key_type = 1i32.to_be_bytes();
        let t1 = compute_integrity_tag(
            &mut integrity_key,
            USERAGENT_INTEGRITY_TAG,
            [key_type.as_slice(), b"pubkey".as_ref()],
        );
        let t2 = compute_integrity_tag(
            &mut integrity_key,
            USERAGENT_INTEGRITY_TAG,
            [key_type.as_slice(), b"pubkey".as_ref()],
        );
        assert_eq!(t1, t2);
    }

    #[test]
    pub fn integrity_tag_changes_with_payload() {
        let salt = generate_salt();
        let mut integrity_key = derive_key(SafeCell::new(b"password".to_vec()), &salt);
        let key_type_1 = 1i32.to_be_bytes();
        let key_type_2 = 2i32.to_be_bytes();
        let t1 = compute_integrity_tag(
            &mut integrity_key,
            USERAGENT_INTEGRITY_TAG,
            [key_type_1.as_slice(), b"pubkey".as_ref()],
        );
        let t2 = compute_integrity_tag(
            &mut integrity_key,
            USERAGENT_INTEGRITY_TAG,
            [key_type_2.as_slice(), b"pubkey".as_ref()],
        );
        assert_ne!(t1, t2);
    }
}
