use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DataStruct, Fields, Ident, Type};

use crate::utils::{is_resolver, resolver_expr};

pub fn wrapper_methods_for_struct(name: &Ident, data: DataStruct) -> syn::Result<TokenStream2> {
    let Fields::Named(fields) = data.fields else {
        return Err(syn::Error::new_spanned(
            name,
            "PyResolver only supports named fields",
        ));
    };

    let mut parsed_fields = ParsedFields::default();

    for field in fields.named {
        let field_ident = field.ident.unwrap();
        let field_ty = field.ty;
        if is_resolver(&field.attrs) {
            parsed_fields.collect_resolver_field(field_ident, field_ty);
        } else {
            parsed_fields.collect_normal_field(field_ident, field_ty);
        }
    }

    let ParsedFields { args, init_fields } = parsed_fields;

    Ok(quote! {
        #[new]
        fn new(
            #(#args),*
        ) -> pyo3::PyResult<Self> {
            Ok(Self {
                data: Some(#name {
                    #(#init_fields),*
                })
            })
        }
    })
}

#[derive(Default)]
struct ParsedFields {
    args: Vec<TokenStream2>,
    init_fields: Vec<TokenStream2>,
}

impl ParsedFields {
    pub fn collect_normal_field(&mut self, field_ident: Ident, field_ty: Type) {
        self.args.push(quote! {
            #field_ident: #field_ty
        });
        self.init_fields.push(quote! {
            #field_ident
        });
    }

    pub fn collect_resolver_field(&mut self, field_ident: Ident, field_ty: Type) {
        let (arg_ty, init_expr) = resolver_expr(&field_ident, &field_ty);
        self.args.push(quote! {
            #field_ident: #arg_ty
        });
        self.init_fields.push(quote! {
            #field_ident: #init_expr
        });
    }
}
