use terrors::OneOf;

// Wire boundary type — what would go into a proto response
#[derive(Debug)]
pub enum ProtoError {
    NotRegistered,
    InvalidSignature,
    Internal(String), // Or Box<dyn Error>, who cares?
}

// Internal terrors types
#[derive(Debug)]
pub struct NotRegistered;
#[derive(Debug)]
pub struct InvalidSignature;
#[derive(Debug)]
pub struct InternalError1(pub String);
#[derive(Debug)]
pub struct InternalError2(pub String);

// Errors can be scattered across the codebase as long as they implement Into<ProtoError>
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

impl From<InternalError1> for ProtoError {
    fn from(e: InternalError1) -> Self {
        ProtoError::Internal(e.0)
    }
}
impl From<InternalError2> for ProtoError {
    fn from(e: InternalError2) -> Self {
        ProtoError::Internal(e.0)
    }
}

/// Private helper trait for converting from OneOf<T...> where each T can be converted
/// into the target type `O` by recursively narrowing until a match is found.
/// 
/// IDK why this isn't already in terrors.
trait DrainInto<O>: terrors::TypeSet + Sized {
    fn drain(e: OneOf<Self>) -> O;
}

macro_rules! impl_drain_into {
    ($head:ident) => {
        impl<$head, O> DrainInto<O> for ($head,)
        where
            $head: Into<O> + 'static,
        {
            fn drain(e: OneOf<($head,)>) -> O {
                e.take().into()
            }
        }
    };
    ($head:ident, $($tail:ident),+) => {
        impl<$head, $($tail),+, O> DrainInto<O> for ($head, $($tail),+)
        where
            $head: Into<O> + 'static,
            ($($tail,)+): DrainInto<O>,
        {
            fn drain(e: OneOf<($head, $($tail),+)>) -> O {
                match e.narrow::<$head, _>() {
                    Ok(h) => h.into(),
                    Err(rest) => <($($tail,)+)>::drain(rest),
                }
            }
        }
        impl_drain_into!($($tail),+);
    };
}

// Generates impls for all tuple sizes from 1 up to 7 (restricted by terrors internal impl).
// Each invocation produces one impl then recurses on the tail.
impl_drain_into!(A, B, C, D, E, F, G, H, I);

// Blanket From impl: body delegates to the recursive drain.
impl<E: DrainInto<ProtoError>> From<OneOf<E>> for ProtoError {
    fn from(e: OneOf<E>) -> Self {
        E::drain(e)
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
        let e: ProtoError = InternalError1("boom".into()).into();
        assert!(matches!(e, ProtoError::Internal(msg) if msg == "boom"));
    }

    #[test]
    fn one_of_remainder_converts_to_proto_invalid_signature() {
        use terrors::OneOf;
        let e: OneOf<(InvalidSignature, InternalError1)> = OneOf::new(InvalidSignature);
        let proto = ProtoError::from(e);
        assert!(matches!(proto, ProtoError::InvalidSignature));
    }

    #[test]
    fn one_of_remainder_converts_to_proto_internal() {
        use terrors::OneOf;
        let e: OneOf<(InvalidSignature, InternalError1)> =
            OneOf::new(InternalError1("db fail".into()));
        let proto = ProtoError::from(e);
        assert!(matches!(proto, ProtoError::Internal(msg) if msg == "db fail"));
    }
}
