use std::ops::Deref;

struct DeferClosure<F: FnOnce()> {
    f: Option<F>,
}

impl<F: FnOnce()> Drop for DeferClosure<F> {
    fn drop(&mut self) {
        if let Some(f) = self.f.take() {
            f();
        }
    }
}

// Run some code when a scope is exited, similar to Go's defer statement
pub fn defer<F: FnOnce()>(f: F) -> impl Drop + Sized {
    DeferClosure { f: Some(f) }
}

/// A trait for casting between two transparently wrapped types with identical memory layouts.
///
/// [`ReinterpretWrapper`] enables zero-cost conversions between two types (`Self` and `Counterpart`)
/// that wrap the same underlying data but differ in how that data is presented. Both types must
/// transparently wrap the same "deref target" and provide bidirectional `AsRef` conversions.
pub trait ReinterpretWrapper<Counterpart>
where
    Self: Deref<Target = Self::Inner> + AsRef<Counterpart>,
    Counterpart: Deref<Target = Self::Inner> + AsRef<Self>,
{
    /// The shared target type that both `Self` and `Counterpart` transparently wrap.
    type Inner;
    /// Reinterprets `Self` as `Counterpart`.
    fn reinterpret(self) -> Counterpart;
}
