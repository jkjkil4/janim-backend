mod attrs;
mod attrs_storage;
mod bind;
mod cmpts;
mod cmpts_storage;

use pyo3::prelude::*;

#[pymodule]
pub mod component {
    #[pymodule_export]
    use super::attrs::{AttrField, AttrFieldDescriptor};
    #[pymodule_export]
    use super::attrs_storage::AttrsStorage;
    #[pymodule_export]
    use super::bind::BindState;
    #[pymodule_export]
    use super::cmpts::{CmptField, CmptInfo};
    #[pymodule_export]
    use super::cmpts_storage::CmptsStorage;
}
