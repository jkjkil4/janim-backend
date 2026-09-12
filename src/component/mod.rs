mod cmpt_attrs;

use pyo3::prelude::*;

#[pymodule]
pub mod component {
    #[pymodule_export]
    use super::cmpt_attrs::{CmptCore, CmptField, CmptFieldDescriptor};
}
