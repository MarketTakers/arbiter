use syn::{DeriveInput, parse_macro_input};

mod hashable;
mod utils;

#[proc_macro_derive(Hashable)]
pub fn derive_hashable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    hashable::derive(&input).into()
}
