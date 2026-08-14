use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident, parse_macro_input};

/// ```rust
/// #[pyclass]
/// #[derive(PyDataClassNew)]
/// struct X {
///     field1: Ty1,
///     ...
/// }
/// ```
///
/// this proc-macro generates the corresponding `X::new` python-method, likes:
///
/// ```rust
/// #[pymethods]
/// impl X {
///     #[new]
///     fn new(field1: Ty1, ...) -> Self {
///         Self { field1, ... }
///     }
/// }
/// ```
#[proc_macro_derive(PyDataClassNew)]
pub fn py_dataclass_new(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match impl_py_dataclass_new(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn impl_py_dataclass_new(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name = &input.ident;

    let Data::Struct(data) = &input.data else {
        return Err(syn::Error::new_spanned(
            struct_name,
            "PyDataClassNew only supports structs",
        ));
    };
    let Fields::Named(fields) = &data.fields else {
        return Err(syn::Error::new_spanned(
            struct_name,
            "PyDataClassNew only supports named fields",
        ));
    };

    let field_names: Vec<&Ident> = fields
        .named
        .iter()
        .map(|field| field.ident.as_ref().unwrap())
        .collect();

    let field_types = fields.named.iter().map(|field| &field.ty);

    Ok(quote! {
        #[pymethods]
        impl #struct_name {
            #[new]
            fn new(
                #(
                    #field_names: #field_types
                ),*
            ) -> Self {
                Self {
                    #(
                        #field_names,
                    )*
                }
            }
        }
    })
}
