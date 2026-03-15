use terrors::OneOf;

// Wire boundary type — what would go into a proto response
#[derive(Debug)]
pub enum ProtoError {
    NotRegistered,
    InvalidSignature,
    Internal(String),
}

// Internal terrors types
pub struct NotRegistered;
pub struct InvalidSignature;
pub struct Internal(pub String);

impl From<NotRegistered> for ProtoError {
    fn from(_: NotRegistered) -> Self {
        ProtoError::NotRegistered
    }
}

impl From<InvalidSignature> for ProtoError {
    fn from(_: InvalidSignature) -> Self {
        ProtoError::InvalidSignature
    }
}

impl From<Internal> for ProtoError {
    fn from(e: Internal) -> Self {
        ProtoError::Internal(e.0)
    }
}

// Converts the narrowed remainder after handling NotRegistered
impl From<OneOf<(InvalidSignature, Internal)>> for ProtoError {
    fn from(e: OneOf<(InvalidSignature, Internal)>) -> Self {
        match e.narrow::<InvalidSignature, _>() {
            Ok(_) => ProtoError::InvalidSignature,
            Err(e) => {
                let Internal(msg) = e.take();
                ProtoError::Internal(msg)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_registered_converts_to_proto() {
        let e: ProtoError = NotRegistered.into();
        assert!(matches!(e, ProtoError::NotRegistered));
    }

    #[test]
    fn invalid_signature_converts_to_proto() {
        let e: ProtoError = InvalidSignature.into();
        assert!(matches!(e, ProtoError::InvalidSignature));
    }

    #[test]
    fn internal_converts_to_proto() {
        let e: ProtoError = Internal("boom".into()).into();
        assert!(matches!(e, ProtoError::Internal(msg) if msg == "boom"));
    }

    #[test]
    fn one_of_remainder_converts_to_proto_invalid_signature() {
        use terrors::OneOf;
        let e: OneOf<(InvalidSignature, Internal)> = OneOf::new(InvalidSignature);
        let proto = ProtoError::from(e);
        assert!(matches!(proto, ProtoError::InvalidSignature));
    }

    #[test]
    fn one_of_remainder_converts_to_proto_internal() {
        use terrors::OneOf;
        let e: OneOf<(InvalidSignature, Internal)> = OneOf::new(Internal("db fail".into()));
        let proto = ProtoError::from(e);
        assert!(matches!(proto, ProtoError::Internal(msg) if msg == "db fail"));
    }
}
