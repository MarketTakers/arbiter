use crate::utils::{HASHABLE_TRAIT_PATH, HMAC_DIGEST_PATH};

use proc_macro2::{Span, TokenStream, TokenTree};
use quote::quote;
use syn::{DataStruct, DeriveInput, Fields, Generics, Index, parse_quote, spanned::Spanned};

pub(crate) fn derive(input: &DeriveInput) -> TokenStream {
    match &input.data {
        syn::Data::Struct(struct_data) => hashable_struct(input, struct_data),
        syn::Data::Enum(_) => {
            syn::Error::new_spanned(input, "Hashable can currently be derived only for structs")
                .to_compile_error()
        }
        syn::Data::Union(_) => {
            syn::Error::new_spanned(input, "Hashable cannot be derived for unions")
                .to_compile_error()
        }
    }
}

fn hashable_struct(input: &DeriveInput, struct_data: &DataStruct) -> TokenStream {
    let ident = &input.ident;
    let hashable_trait = HASHABLE_TRAIT_PATH.to_path();
    let hmac_digest = HMAC_DIGEST_PATH.to_path();
    let generics = add_hashable_bounds(input.generics.clone(), &hashable_trait);
    let field_accesses = collect_field_accesses(struct_data);
    let hash_calls = build_hash_calls(&field_accesses, &hashable_trait);

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        #[automatically_derived]
        impl #impl_generics #hashable_trait for #ident #ty_generics #where_clause {
            fn hash<H: #hmac_digest>(&self, hasher: &mut H) {
                #(#hash_calls)*
            }
        }
    }
}

fn add_hashable_bounds(mut generics: Generics, hashable_trait: &syn::Path) -> Generics {
    for type_param in generics.type_params_mut() {
        type_param.bounds.push(parse_quote!(#hashable_trait));
    }

    generics
}

struct FieldAccess {
    access: TokenStream,
    span: Span,
}

fn collect_field_accesses(struct_data: &DataStruct) -> Vec<FieldAccess> {
    match &struct_data.fields {
        Fields::Named(fields) => {
            // Keep deterministic alphabetical order for named fields.
            // Do not remove this sort, because it keeps hash output stable regardless of source order.
            let mut named_fields = fields
                .named
                .iter()
                .map(|field| {
                    let name = field
                        .ident
                        .as_ref()
                        .expect("Fields::Named(fields) must have names")
                        .clone();
                    (name.to_string(), name)
                })
                .collect::<Vec<_>>();

            named_fields.sort_by(|a, b| a.0.cmp(&b.0));

            named_fields
                .into_iter()
                .map(|(_, name)| FieldAccess {
                    access: quote! { #name },
                    span: name.span(),
                })
                .collect()
        }
        Fields::Unnamed(fields) => fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(i, field)| FieldAccess {
                access: {
                    let index = Index::from(i);
                    quote! { #index }
                },
                span: field.ty.span(),
            })
            .collect(),
        Fields::Unit => Vec::new(),
    }
}

fn build_hash_calls(
    field_accesses: &[FieldAccess],
    hashable_trait: &syn::Path,
) -> Vec<TokenStream> {
    field_accesses
        .iter()
        .map(|field| {
            let access = &field.access;
            let call = quote! {
                #hashable_trait::hash(&self.#access, hasher);
            };

            respan(call, field.span)
        })
        .collect()
}

/// Recursively set span on all tokens, including interpolated ones.
fn respan(tokens: TokenStream, span: Span) -> TokenStream {
    tokens
        .into_iter()
        .map(|tt| match tt {
            TokenTree::Group(g) => {
                let mut new = proc_macro2::Group::new(g.delimiter(), respan(g.stream(), span));
                new.set_span(span);
                TokenTree::Group(new)
            }
            mut other => {
                other.set_span(span);
                other
            }
        })
        .collect()
}
