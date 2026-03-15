use terrors::OneOf;
use crate::errors::Internal;

// Simulates fetching a nonce from a database.
// id=99 is a sentinel that triggers an Internal error.
pub fn get_nonce(id: u32) -> Result<u32, OneOf<(Internal,)>> {
    if id == 99 {
        return Err(OneOf::new(Internal("db pool unavailable".into())));
    }
    Ok(42)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_nonce_returns_nonce_for_valid_id() {
        assert_eq!(get_nonce(1).unwrap(), 42);
    }

    #[test]
    fn get_nonce_returns_internal_error_for_sentinel() {
        let err = get_nonce(99).unwrap_err();
        let internal = err.take::<crate::errors::Internal>();
        assert_eq!(internal.0, "db pool unavailable");
    }
}
