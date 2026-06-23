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
