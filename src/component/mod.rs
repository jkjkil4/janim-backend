mod attrs;
mod attrs_storage;

use pyo3::prelude::*;

#[pymodule]
pub mod component {
    #[pymodule_export]
    use super::attrs::{CmptField, CmptFieldDescriptor};
    #[pymodule_export]
    use super::attrs_storage::AttrsStorage;
}
