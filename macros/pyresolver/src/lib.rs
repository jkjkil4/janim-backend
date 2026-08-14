use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Ident, LitStr, parse_macro_input};

mod for_enum;
mod for_struct;
mod utils;

use crate::for_enum::impl_wrapper_for_enum;
use crate::for_struct::impl_wrapper_for_struct;

/// ```rust
/// #[derive(PyResolver)]
/// #[py_resolver(module = "...")]
/// struct X {
///     // .. fields
/// }
/// ```
///
/// this proc-macro generates:
/// - The corresponding `PyX` python class
/// - `X::resolve`, which takes the data inside `PyX`, return a owned `X`
///
/// Example:
///
/// ```rust
/// fn test(x: Bound<'_, PyX>) -> PyResult<()> {
///     let resolved = X::resolve(x);
///     // ...
/// }
/// ```
#[proc_macro_derive(PyResolver, attributes(pyresolver))]
pub fn py_resolver(input: TokenStream) -> TokenStream {
    match impl_py_resolver(parse_macro_input!(input as DeriveInput)) {
        Ok(tokens) => tokens.into(),

        Err(err) => err.to_compile_error().into(),
    }
}

fn impl_py_resolver(input: DeriveInput) -> syn::Result<TokenStream2> {
    let name = input.ident;

    let mut module_name = None;

    for attr in &input.attrs {
        if !attr.path().is_ident("pyresolver") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("module") {
                let value: LitStr = meta.value()?.parse()?;
                module_name = Some(value.value());
            } else {
                return Err(meta.error("unknown pyresolver attribute"));
            }

            Ok(())
        })?;
    }

    let class_name = name.to_string();
    let wrapper_ident = format_ident!("Py{}", class_name);

    let module_name = module_name
        .ok_or_else(|| syn::Error::new_spanned(&name, "missing #[pyresolver(module = \"...\")]"))?;

    let code_impl_resolve = impl_resolve(&name, &wrapper_ident);
    let code_decl_wrapper = decl_wrapper(&name, &wrapper_ident, &class_name, &module_name);

    let code_impl_wrapper = match input.data {
        Data::Struct(data) => impl_wrapper_for_struct(&name, &wrapper_ident, data),
        Data::Enum(data) => impl_wrapper_for_enum(&name, &wrapper_ident, data),
        _ => {
            return Err(syn::Error::new_spanned(
                name,
                "Not a PyResolver supported structure",
            ));
        }
    }?;

    Ok(quote! {
        #code_impl_resolve

        #code_decl_wrapper

        #code_impl_wrapper
    })
}

fn impl_resolve(name: &Ident, wrapper_ident: &Ident) -> TokenStream2 {
    quote! {
        impl #name {
            pub fn resolve(
                object: pyo3::Bound<'_, #wrapper_ident>
            ) -> pyo3::PyResult<Self> {
                Self::resolve_any(object.into_any())
            }

            pub fn resolve_any(
                object: pyo3::Bound<'_, pyo3::PyAny>
            ) -> pyo3::PyResult<Self> {
                use pyo3::types::PyAnyMethods;
                let mut value: pyo3::PyRefMut<#wrapper_ident> = object.extract()?;
                match value.data.take() {
                    Some(data) => Ok(data),
                    None => Err(pyo3::exceptions::PyException::new_err("Data already taken"))
                }
            }
        }
    }
}

fn decl_wrapper(
    name: &Ident,
    wrapper_ident: &Ident,
    class_name: &String,
    module_name: &String,
) -> TokenStream2 {
    quote! {
        #[pyo3::prelude::pyclass(
            name = #class_name,
            module = #module_name,
            skip_from_py_object
        )]
        pub struct #wrapper_ident {
            data: Option<#name>,
        }
    }
}
