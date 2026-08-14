use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Attribute, Ident, Type};

pub fn is_resolver(attrs: &Vec<Attribute>) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident("pyresolver"))
}

pub fn resolver_expr(ident: &Ident, ty: &Type) -> (TokenStream2, TokenStream2) {
    if let Type::Path(type_path) = ty {
        let segment = type_path.path.segments.last().unwrap();

        match segment.ident.to_string().as_str() {
            "Vec" => {
                let inner_ty = match &segment.arguments {
                    syn::PathArguments::AngleBracketed(args) => match args.args.first().unwrap() {
                        syn::GenericArgument::Type(ty) => ty,
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                };
                let inner_py_ty = py_type_name(inner_ty);

                let arg_ty = quote! {
                    Vec<pyo3::Bound<'_, #inner_py_ty>>
                };
                let init_expr = quote! {
                    {
                        let mut result = Vec::new();
                        for obj in #ident {
                            let resolved = <#inner_ty>::resolve(obj)?;
                            result.push(resolved);
                        }
                        result
                    }
                };
                return (arg_ty, init_expr);
            }

            // TODO: HashMap
            _ => {}
        }
    }

    // default
    let py_ty = py_type_name(ty);

    let arg_ty = quote! { pyo3::Bound<'_, #py_ty> };
    let init_expr = quote! { <#ty>::resolve(#ident)? };

    (arg_ty, init_expr)
}

fn py_type_name(ty: &syn::Type) -> syn::Type {
    match ty {
        syn::Type::Path(path) if path.qself.is_none() => {
            let mut path = path.clone();

            let last = path.path.segments.last_mut().unwrap();
            let ident = &last.ident;

            last.ident = format_ident!("Py{}", ident);

            syn::Type::Path(path)
        }
        _ => panic!("unsupported type"),
    }
}
