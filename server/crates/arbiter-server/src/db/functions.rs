//! Typed bindings for the SQLite scalar functions used in Diesel expressions.

use diesel::sql_types::{Integer, Text};

diesel::define_sql_function! {
    /// SQLite `unixepoch(modifier)` -- seconds since the Unix epoch.
    ///
    /// Declared so timestamp comparisons are built by the query DSL instead of by
    /// `format!`-ing a SQL fragment: the argument becomes a bind parameter and the
    /// result type is checked against the column it is compared with.
    fn unixepoch(modifier: Text) -> Integer;
}
