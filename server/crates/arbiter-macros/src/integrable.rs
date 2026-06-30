use crate::utils::INTEGRABLE_TRAIT_PATH;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, LitStr, spanned::Spanned as _};

struct IntegrableAttr {
    kind: String,
}

impl IntegrableAttr {
    fn from_attrs(
        attrs: &[syn::Attribute],
        ident_span: proc_macro2::Span,
    ) -> Result<Self, syn::Error> {
        let mut kind: Option<String> = None;
        let mut found = false;

        for attr in attrs {
            if !attr.path().is_ident("integrable") {
                continue;
            }
            if found {
                return Err(syn::Error::new(attr.span(), "duplicate #[integrable] attribute"));
            }
            found = true;
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("kind") {
                    let lit: LitStr = meta.value()?.parse()?;
                    let v = lit.value();
                    if v.is_empty() {
                        return Err(syn::Error::new(lit.span(), "kind must not be empty"));
                    }
                    if !is_valid_kind(&v) {
                        return Err(syn::Error::new(
                            lit.span(),
                            "kind must be a valid schema name: start with a letter, contain only [a-zA-Z0-9_]",
                        ));
                    }
                    kind = Some(v);
                } else {
                    return Err(meta.error("unknown key; expected `kind`"));
                }
                Ok(())
            })?;
        }

        let kind = kind.ok_or_else(|| {
            syn::Error::new(ident_span, "#[integrable(kind = \"...\")] is required")
        })?;

        Ok(Self { kind })
    }
}

fn is_valid_kind(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn fnv1a(data: &[u8], mut hash: u32) -> u32 {
    const FNV_PRIME: u32 = 16_777_619;
    for &b in data {
        hash ^= u32::from(b);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

// Hashes field names and types using the same alphabetical sort order as Hashable,
// so that source-level field reordering never changes VERSION.
fn compute_version(fields: &syn::Fields) -> i32 {
    const FNV_OFFSET: u32 = 2_166_136_261;
    let mut hash = FNV_OFFSET;

    match fields {
        syn::Fields::Named(named) => {
            for field in crate::utils::sorted_named_fields(named) {
                let name = field.ident.as_ref().unwrap().to_string();
                let ty = &field.ty;
                hash = fnv1a(name.as_bytes(), hash);
                hash = fnv1a(quote!(#ty).to_string().as_bytes(), hash);
            }
        }
        syn::Fields::Unnamed(unnamed) => {
            for (i, field) in unnamed.unnamed.iter().enumerate() {
                let ty = &field.ty;
                hash = fnv1a(i.to_string().as_bytes(), hash);
                hash = fnv1a(quote!(#ty).to_string().as_bytes(), hash);
            }
        }
        syn::Fields::Unit => {}
    }

    // Clear sign bit to guarantee a positive i32; substitute 0 → 1.
    let v = (hash >> 1).cast_signed();
    if v == 0 { 1 } else { v }
}

pub(crate) fn derive(input: &DeriveInput) -> TokenStream {
    let syn::Data::Struct(ref data) = input.data else {
        return syn::Error::new(
            input.ident.span(),
            "#[derive(Integrable)] is only supported on structs",
        )
        .to_compile_error();
    };

    let integrable_trait = INTEGRABLE_TRAIT_PATH.to_path();
    let hashable_trait = crate::utils::HASHABLE_TRAIT_PATH.to_path();
    let ident = &input.ident;

    let mut generics = input.generics.clone();
    for type_param in generics.type_params_mut() {
        type_param.bounds.push(syn::parse_quote!(#hashable_trait));
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let attr = match IntegrableAttr::from_attrs(&input.attrs, input.ident.span()) {
        Ok(a) => a,
        Err(e) => return e.to_compile_error(),
    };

    let kind = attr.kind;
    let version = compute_version(&data.fields);

    quote! {
        #[automatically_derived]
        impl #impl_generics #integrable_trait for #ident #ty_generics #where_clause {
            const KIND: &'static str = #kind;
            const VERSION: i32 = #version;
        }
    }
}
