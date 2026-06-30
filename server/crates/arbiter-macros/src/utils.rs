pub(crate) struct ToPath(pub &'static str);

impl ToPath {
    pub(crate) fn to_path(&self) -> syn::Path {
        syn::parse_str(self.0).expect("Invalid path")
    }
}

macro_rules! ensure_path {
    ($path:path as $name:ident) => {
        const _: () = {
            #[cfg(test)]
            #[expect(
                unused_imports,
                reason = "Ensures the path is valid and will cause a compile error if not"
            )]
            use $path as _;
        };
        pub(crate) const $name: ToPath = ToPath(stringify!($path));
    };
}

ensure_path!(::arbiter_crypto::hashing::Hashable as HASHABLE_TRAIT_PATH);
ensure_path!(::arbiter_crypto::hashing::Digest as HMAC_DIGEST_PATH);
ensure_path!(::arbiter_crypto::integrity::Integrable as INTEGRABLE_TRAIT_PATH);

/// Returns named struct fields sorted alphabetically by name.
/// Both `Hashable` and `Integrable` derive macros must iterate fields in the
/// same deterministic order so that source-level reordering never changes
/// either the runtime hash or the compile-time VERSION.
pub(crate) fn sorted_named_fields(fields: &syn::FieldsNamed) -> Vec<&syn::Field> {
    let mut v: Vec<&syn::Field> = fields.named.iter().collect();
    v.sort_by_key(|f| f.ident.as_ref().unwrap().to_string());
    v
}
