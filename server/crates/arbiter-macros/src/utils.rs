pub struct ToPath(pub &'static str);

impl ToPath {
    pub fn to_path(&self) -> syn::Path {
        syn::parse_str(self.0).expect("Invalid path")
    }
}

macro_rules! ensure_path {
    ($path:path) => {{
        #[cfg(test)]
        #[expect(unused_imports)]
        use $path as _;
        ToPath(stringify!($path))
    }};
}

pub const HASHABLE_TRAIT_PATH: ToPath = ensure_path!(::arbiter_crypto::hashing::Hashable);
pub const HMAC_DIGEST_PATH: ToPath = ensure_path!(::arbiter_crypto::hashing::Digest);
