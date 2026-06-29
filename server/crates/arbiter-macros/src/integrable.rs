use crate::utils::INTEGRABLE_TRAIT_PATH;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, LitInt, LitStr};

struct IntegrableAttr {
    kind: String,
    version: i32,
}

impl IntegrableAttr {
    fn from_attrs(attrs: &[syn::Attribute], span: proc_macro2::Span) -> Result<Self, syn::Error> {
        let mut kind: Option<String> = None;
        let mut version: i32 = 1;

        for attr in attrs {
            if !attr.path().is_ident("integrable") {
                continue;
            }
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("kind") {
                    let lit: LitStr = meta.value()?.parse()?;
                    kind = Some(lit.value());
                } else if meta.path.is_ident("version") {
                    let lit: LitInt = meta.value()?.parse()?;
                    version = lit.base10_parse()?;
                } else {
                    return Err(meta.error("unknown key; expected `kind` or `version`"));
                }
                Ok(())
            })?;
        }

        let kind = kind.ok_or_else(|| {
            syn::Error::new(span, "#[integrable(kind = \"...\")] is required")
        })?;

        Ok(Self { kind, version })
    }
}

pub(crate) fn derive(input: &DeriveInput) -> TokenStream {
    let integrable_trait = INTEGRABLE_TRAIT_PATH.to_path();
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let attr = match IntegrableAttr::from_attrs(&input.attrs, proc_macro2::Span::call_site()) {
        Ok(a) => a,
        Err(e) => return e.to_compile_error(),
    };

    let kind = attr.kind;
    let version = attr.version;

    quote! {
        #[automatically_derived]
        impl #impl_generics #integrable_trait for #ident #ty_generics #where_clause {
            const KIND: &'static str = #kind;
            const VERSION: i32 = #version;
        }
    }
}
