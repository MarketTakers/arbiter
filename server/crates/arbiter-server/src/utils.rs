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

/// Renders an error together with its full `source` chain as `outer: inner: root`.
///
/// Error variants in this crate deliberately keep `Display` terse so that no
/// internal detail can leak across the gRPC boundary. That same terseness would
/// hide the cause in the logs, so use this for `tracing` fields, never in a
/// wire payload.
pub fn error_chain(err: &dyn core::error::Error) -> String {
    let mut out = err.to_string();
    let mut current = err.source();
    while let Some(source) = current {
        out.push_str(": ");
        out.push_str(&source.to_string());
        current = source.source();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::error_chain;

    #[derive(Debug, thiserror::Error)]
    #[error("root")]
    struct Root;

    #[derive(Debug, thiserror::Error)]
    #[error("middle")]
    struct Middle(#[source] Root);

    #[derive(Debug, thiserror::Error)]
    #[error("outer")]
    struct Outer(#[source] Middle);

    #[test]
    fn walks_the_whole_source_chain() {
        assert_eq!(error_chain(&Root), "root", "a leaf error renders alone");
        assert_eq!(
            error_chain(&Outer(Middle(Root))),
            "outer: middle: root",
            "every source link must appear, in order"
        );
    }
}
