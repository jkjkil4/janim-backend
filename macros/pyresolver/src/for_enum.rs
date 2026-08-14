use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{DataEnum, Ident, Type};

use crate::utils::{is_resolver, resolver_expr};

pub fn impl_wrapper_for_enum(
    name: &Ident,
    wrapper_ident: &Ident,
    data: DataEnum,
) -> syn::Result<TokenStream2> {
    let mut initializers = Vec::new();

    for variant in data.variants {
        let mut parsed_fields = ParsedFields::default();

        for field in variant.fields {
            let field_ty = field.ty;
            if is_resolver(&field.attrs) {
                parsed_fields.collect_resolver_field(field_ty);
            } else {
                parsed_fields.collect_normal_field(field_ty);
            }
        }

        let ParsedFields { args, init_fields } = parsed_fields;

        let variant_ident = variant.ident;
        let variant_name = variant_ident.to_string();
        let init_ident = format_ident!("new_{}", variant_name);

        initializers.push(quote! {
            #[staticmethod]
            #[pyo3(name = #variant_name)]
            fn #init_ident(
                #(#args),*
            ) -> pyo3::PyResult<Self> {
                Ok(Self {
                    data: Some(#name::#variant_ident(
                        #(#init_fields),*
                    ))
                })
            }
        })
    }

    Ok(quote! {
        #[pyo3::prelude::pymethods]
        impl #wrapper_ident {
            #(#initializers)*
        }
    })
}

#[derive(Default)]
struct ParsedFields {
    args: Vec<TokenStream2>,
    init_fields: Vec<TokenStream2>,
}

impl ParsedFields {
    fn next_ident(&self) -> Ident {
        format_ident!("_{}", &self.args.len())
    }
    pub fn collect_normal_field(&mut self, field_ty: Type) {
        let ident = self.next_ident();
        self.args.push(quote! {
            #ident: #field_ty
        });
        self.init_fields.push(quote! {
            #ident
        });
    }

    pub fn collect_resolver_field(&mut self, field_ty: Type) {
        let ident = self.next_ident();
        let (arg_ty, init_expr) = resolver_expr(&ident, &field_ty);
        self.args.push(quote! {
            #ident: #arg_ty
        });
        self.init_fields.push(quote! {
            #init_expr
        });
    }
}
