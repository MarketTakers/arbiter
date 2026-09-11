use arbiter_crypto::safecell::{SafeCell, SafeCellHandle as _};
use rand_core::CryptoRng;
use vsss_rs::Gf256;

use crate::crypto::KeyCell;

/// GF(256) addresses shares by a non-zero byte, so no committee can exceed 255.
pub const MAX_COMMITTEE_SIZE: usize = 255;

/// Errors returned by Shamir split/combine operations.
#[derive(Debug, thiserror::Error)]
pub enum ShamirError {
    #[error("failed to split key: {0}")]
    Split(String),
    #[error("failed to combine shares: {0}")]
    Combine(String),
}

/// Return the required threshold for a Shamir share pool of `committee_size`.
///
/// A pool of two is rejected: a majority of two is two, which gives each holder
/// a veto over every unseal without giving either one recovery. That rejects no
/// supported committee, because a two-operator vault must carry at least one
/// recovery share and so never splits into a pool of two -- see
/// `docs/ARCHITECTURE.md` 3.9.
#[expect(
    clippy::integer_division,
    reason = "majority thresholds use integer arithmetic"
)]
#[must_use]
pub const fn shamir_threshold(committee_size: usize) -> Option<usize> {
    match committee_size {
        0 | 2 => None,
        size if size > MAX_COMMITTEE_SIZE => None,
        1 => Some(1),
        size => Some(size / 2 + 1),
    }
}

/// Split a seal key into `total` shares, `threshold` of which reconstruct it.
pub fn split_key(
    threshold: usize,
    total: usize,
    key: &mut KeyCell,
    rng: impl CryptoRng,
) -> Result<Vec<SafeCell<Vec<u8>>>, ShamirError> {
    if total == 0 || threshold == 0 || threshold > total || total == 2 || total > MAX_COMMITTEE_SIZE
    {
        return Err(ShamirError::Split(
            "unsupported committee parameters".to_owned(),
        ));
    }

    // Nothing to interpolate when one share suffices.
    if threshold == 1 {
        return Ok(key.0.read_inline(|key| {
            std::iter::repeat_with(|| SafeCell::new(key.as_slice().to_vec()))
                .take(total)
                .collect()
        }));
    }

    key.0.read_inline(|key| {
        let key: &[u8; 32] = key
            .as_slice()
            .try_into()
            .map_err(|_| ShamirError::Split("unexpected seal key length".to_owned()))?;

        Gf256::split_array(threshold, total, key, rng)
            .map(|shares| shares.into_iter().map(SafeCell::new).collect())
            .map_err(|error| ShamirError::Split(format!("{error:?}")))
    })
}

/// Combine shares back into the seal key.
///
/// `threshold` comes from storage rather than from the shapes of the shares:
/// a one-of-one committee stores the key verbatim, and telling that apart by
/// share length alone would misread any Shamir share that happened to be key
/// sized.
pub fn combine_shares(
    threshold: usize,
    shares: &mut [SafeCell<Vec<u8>>],
) -> Result<KeyCell, ShamirError> {
    if threshold == 0 {
        return Err(ShamirError::Combine("threshold is zero".to_owned()));
    }
    if shares.len() < threshold {
        return Err(ShamirError::Combine(
            "not enough shares supplied".to_owned(),
        ));
    }

    // Mirror of the one-of-one case in [`split_key`]: the share is the key.
    if threshold == 1 {
        let share = shares
            .first_mut()
            .ok_or_else(|| ShamirError::Combine("no shares supplied".to_owned()))?;
        return reconstructed_key(share.read_inline(|share| SafeCell::new(share.clone())));
    }

    let mut gathered = SafeCell::new(Vec::with_capacity(shares.len()));
    for share in shares.iter_mut() {
        share.read_inline(|share| {
            gathered.write_inline(|gathered| gathered.push(share.clone()));
        });
    }

    let combined = gathered.read_inline(|gathered| {
        Gf256::combine_array(gathered.as_slice())
            .map(SafeCell::new)
            .map_err(|error| ShamirError::Combine(format!("{error:?}")))
    })?;

    reconstructed_key(combined)
}

fn reconstructed_key(bytes: SafeCell<Vec<u8>>) -> Result<KeyCell, ShamirError> {
    KeyCell::try_from(bytes)
        .map_err(|()| ShamirError::Combine("unexpected reconstructed key length".to_owned()))
}

#[cfg(test)]
mod tests {
    use super::{MAX_COMMITTEE_SIZE, combine_shares, shamir_threshold, split_key};
    use crate::crypto::KeyCell;
    use arbiter_crypto::safecell::{SafeCell, SafeCellHandle as _};
    use rand::rngs::SysRng;
    use rand_core::UnwrapErr;
    use rstest::rstest;

    fn key_bytes(mut key: KeyCell) -> [u8; 32] {
        key.0.read_inline(|key| {
            let mut bytes = [0u8; 32];
            bytes.copy_from_slice(key.as_slice());
            bytes
        })
    }

    fn select(shares: &mut [SafeCell<Vec<u8>>], indexes: &[usize]) -> Vec<SafeCell<Vec<u8>>> {
        indexes
            .iter()
            .filter_map(|index| {
                shares
                    .get_mut(*index)
                    .map(|share| share.read_inline(|share| SafeCell::new(share.clone())))
            })
            .collect()
    }

    #[rstest]
    #[case(&[0, 1])]
    #[case(&[0, 2])]
    #[case(&[1, 2])]
    fn threshold_shares_reconstruct_fixed_key(#[case] indexes: &[usize]) {
        let expected = [9_u8; 32];
        let mut key = KeyCell::from(expected);
        let rng = UnwrapErr(SysRng);
        let mut shares = split_key(2, 3, &mut key, rng).expect("split should succeed");
        let mut selected = select(&mut shares, indexes);
        let combined = combine_shares(2, &mut selected).expect("combine should succeed");
        assert_eq!(key_bytes(combined), expected);
    }

    #[test]
    fn one_of_one_round_trips_a_fixed_size_key() {
        let expected = [7_u8; 32];
        let mut key = KeyCell::from(expected);
        let rng = UnwrapErr(SysRng);
        let mut shares = split_key(1, 1, &mut key, rng).expect("split should succeed");
        let combined = combine_shares(1, &mut shares).expect("combine should succeed");
        assert_eq!(key_bytes(combined), expected);
    }

    #[test]
    fn fewer_shares_than_threshold_is_rejected() {
        let mut key = KeyCell::from([3_u8; 32]);
        let rng = UnwrapErr(SysRng);
        let mut shares = split_key(3, 5, &mut key, rng).expect("split should succeed");
        let mut selected = select(&mut shares, &[0, 1]);
        assert!(
            combine_shares(3, &mut selected).is_err(),
            "two of three shares must not reconstruct the key"
        );
    }

    #[rstest]
    #[case(0, None)]
    #[case(1, Some(1))]
    #[case(2, None)]
    #[case(3, Some(2))]
    #[case(4, Some(3))]
    #[case(MAX_COMMITTEE_SIZE, Some(128))]
    #[case(MAX_COMMITTEE_SIZE + 1, None)]
    fn committee_threshold_is_a_majority(
        #[case] committee_size: usize,
        #[case] expected: Option<usize>,
    ) {
        assert_eq!(shamir_threshold(committee_size), expected);
    }

    #[test]
    fn oversized_committee_is_rejected_by_split() {
        let mut key = KeyCell::from([1_u8; 32]);
        let rng = UnwrapErr(SysRng);
        assert!(
            split_key(129, MAX_COMMITTEE_SIZE + 1, &mut key, rng).is_err(),
            "committees above the GF(256) share limit must be rejected"
        );
    }

    #[test]
    fn two_operator_committee_is_explicitly_unsupported() {
        let mut key = KeyCell::from([7_u8; 32]);
        let rng = UnwrapErr(SysRng);
        assert!(
            split_key(2, 2, &mut key, rng).is_err(),
            "two-operator committees must be rejected"
        );
    }
}
