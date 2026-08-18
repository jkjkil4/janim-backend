use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, LitStr, parse_macro_input};

mod for_enum;
mod for_struct;
mod utils;

use crate::for_enum::wrapper_methods_for_enum;
use crate::for_struct::wrapper_methods_for_struct;

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
/// - `PyX::take`, which takes the data inside `PyX`, return a owned `X`
///
/// Example:
///
/// ```rust
/// fn test(x: Bound<'_, PyX>) -> PyResult<()> {
///     let data = x.borrow_mut().take();
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

    let code_wrapper_methods = match input.data {
        Data::Struct(data) => wrapper_methods_for_struct(&name, data),
        Data::Enum(data) => wrapper_methods_for_enum(&name, data),
        _ => {
            return Err(syn::Error::new_spanned(
                name,
                "Not a PyResolver supported structure",
            ));
        }
    }?;

    Ok(quote! {
        #[pyo3::prelude::pyclass(
            name = #class_name,
            module = #module_name,
            skip_from_py_object
        )]
        pub struct #wrapper_ident {
            data: Option<#name>,
        }

        #[pyo3::prelude::pymethods]
        impl #wrapper_ident {
            #code_wrapper_methods
        }

        impl #wrapper_ident {
            pub fn take(&mut self) -> pyo3::PyResult<#name> {
                match self.data.take() {
                    Some(data) => Ok(data),
                    None => Err(pyo3::exceptions::PyException::new_err("Data already taken")),
                }
            }
        }
    })
}
