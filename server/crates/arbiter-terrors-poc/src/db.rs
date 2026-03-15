use crate::errors::{InternalError1, InternalError2};
use terrors::OneOf;

// Simulates fetching a nonce from a database.
// id=99 → InternalError1 (pool unavailable)
// id=98 → InternalError2 (query timeout)
pub fn get_nonce(id: u32) -> Result<u32, OneOf<(InternalError1, InternalError2)>> {
    match id {
        99 => Err(OneOf::new(InternalError1("db pool unavailable".to_owned()))),
        98 => Err(OneOf::new(InternalError2("query timeout".to_owned()))),
        _ => Ok(42),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_nonce_returns_nonce_for_valid_id() {
        assert_eq!(get_nonce(1).unwrap(), 42);
    }

    #[test]
    fn get_nonce_returns_internal_error1_for_sentinel() {
        let err = get_nonce(99).unwrap_err();
        let internal = err.narrow::<crate::errors::InternalError1, _>().unwrap();
        assert_eq!(internal.0, "db pool unavailable");
    }

    #[test]
    fn get_nonce_returns_internal_error2_for_sentinel() {
        let err = get_nonce(98).unwrap_err();
        let e = err.narrow::<crate::errors::InternalError1, _>().unwrap_err();
        let internal = e.take::<crate::errors::InternalError2>();
        assert_eq!(internal.0, "query timeout");
    }
}
