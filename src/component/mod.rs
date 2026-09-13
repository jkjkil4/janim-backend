mod attrs;
mod core;

use pyo3::prelude::*;

#[pymodule]
pub mod component {
    #[pymodule_export]
    use super::attrs::{CmptField, CmptFieldDescriptor};
    #[pymodule_export]
    use super::core::CmptCore;
}
