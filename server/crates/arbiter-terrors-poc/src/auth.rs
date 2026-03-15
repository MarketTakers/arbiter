use crate::errors::{InternalError1, InternalError2, InvalidSignature, NotRegistered};
use terrors::OneOf;

use crate::errors::ProtoError;

// Each sub-call's error type already implements DrainInto<ProtoError>, so we convert
// directly to ProtoError without broaden — no turbofish needed anywhere.
//
// Call chain:
//   load_config()  → OneOf<(InternalError2,)>          → ProtoError::from
//   get_nonce()    → OneOf<(InternalError1, InternalError2)> → ProtoError::from
//   verify_sig()   → OneOf<(InvalidSignature,)>         → ProtoError::from
pub fn process_request(id: u32, sig: &str) -> Result<String, ProtoError> {
    if id == 0 {
        return Err(ProtoError::NotRegistered);
    }

    let config = load_config(id).map_err(ProtoError::from)?;
    let nonce = crate::db::get_nonce(id).map_err(ProtoError::from)?;
    verify_signature(nonce, sig).map_err(ProtoError::from)?;

    Ok(format!("config={config} nonce={nonce} sig={sig}"))
}

// Simulates loading a config value.
// id=97 triggers InternalError2 ("config read failed").
fn load_config(id: u32) -> Result<String, OneOf<(InternalError2,)>> {
    if id == 97 {
        return Err(OneOf::new(InternalError2("config read failed".to_owned())));
    }
    Ok(format!("cfg-{id}"))
}

pub fn verify_signature(_nonce: u32, sig: &str) -> Result<(), OneOf<(InvalidSignature,)>> {
    if sig != "ok" {
        return Err(OneOf::new(InvalidSignature));
    }
    Ok(())
}

type AuthError = OneOf<(
    NotRegistered,
    InvalidSignature,
    InternalError1,
    InternalError2,
)>;

pub fn authenticate(id: u32, sig: &str) -> Result<u32, AuthError> {
    if id == 0 {
        return Err(OneOf::new(NotRegistered));
    }

    // Return type AuthError lets the compiler infer the broaden target.
    let nonce = crate::db::get_nonce(id).map_err(OneOf::broaden)?;
    verify_signature(nonce, sig).map_err(OneOf::broaden)?;

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
    fn authenticate_internal_error1() {
        let err = authenticate(99, "ok").unwrap_err();
        assert!(err.narrow::<crate::errors::InternalError1, _>().is_ok());
    }

    #[test]
    fn authenticate_internal_error2() {
        let err = authenticate(98, "ok").unwrap_err();
        assert!(err.narrow::<crate::errors::InternalError2, _>().is_ok());
    }

    #[test]
    fn process_request_success() {
        let result = process_request(1, "ok").unwrap();
        assert!(result.contains("nonce=42"));
    }

    #[test]
    fn process_request_not_registered() {
        let err = process_request(0, "ok").unwrap_err();
        assert!(matches!(err, crate::errors::ProtoError::NotRegistered));
    }

    #[test]
    fn process_request_invalid_signature() {
        let err = process_request(1, "bad").unwrap_err();
        assert!(matches!(err, crate::errors::ProtoError::InvalidSignature));
    }

    #[test]
    fn process_request_internal_from_config() {
        // id=97 → load_config returns InternalError2
        let err = process_request(97, "ok").unwrap_err();
        assert!(
            matches!(err, crate::errors::ProtoError::Internal(ref msg) if msg == "config read failed")
        );
    }

    #[test]
    fn process_request_internal_from_db() {
        // id=99 → get_nonce returns InternalError1
        let err = process_request(99, "ok").unwrap_err();
        assert!(
            matches!(err, crate::errors::ProtoError::Internal(ref msg) if msg == "db pool unavailable")
        );
    }
}
