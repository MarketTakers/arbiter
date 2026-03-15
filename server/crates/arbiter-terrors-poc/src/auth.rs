use terrors::OneOf;
use crate::errors::{Internal, InvalidSignature, NotRegistered};

pub fn verify_signature(_nonce: u32, sig: &str) -> Result<(), OneOf<(InvalidSignature,)>> {
    if sig != "ok" {
        return Err(OneOf::new(InvalidSignature));
    }
    Ok(())
}

pub fn authenticate(
    id: u32,
    sig: &str,
) -> Result<u32, OneOf<(NotRegistered, InvalidSignature, Internal)>> {
    if id == 0 {
        return Err(OneOf::new(NotRegistered));
    }

    let nonce = crate::db::get_nonce(id)
        .map_err(|e| e.broaden::<(NotRegistered, InvalidSignature, Internal), _>())?;
    verify_signature(nonce, sig)
        .map_err(|e| e.broaden::<(NotRegistered, InvalidSignature, Internal), _>())?;

    Ok(nonce)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_signature_ok() {
        assert!(verify_signature(42, "ok").is_ok());
    }

    #[test]
    fn verify_signature_bad() {
        let err = verify_signature(42, "bad").unwrap_err();
        assert!(err.narrow::<crate::errors::InvalidSignature, _>().is_ok());
    }

    #[test]
    fn authenticate_success() {
        assert_eq!(authenticate(1, "ok").unwrap(), 42);
    }

    #[test]
    fn authenticate_not_registered() {
        let err = authenticate(0, "ok").unwrap_err();
        assert!(err.narrow::<crate::errors::NotRegistered, _>().is_ok());
    }

    #[test]
    fn authenticate_invalid_signature() {
        let err = authenticate(1, "bad").unwrap_err();
        assert!(err.narrow::<crate::errors::InvalidSignature, _>().is_ok());
    }

    #[test]
    fn authenticate_internal_error() {
        let err = authenticate(99, "ok").unwrap_err();
        assert!(err.narrow::<crate::errors::Internal, _>().is_ok());
    }
}
