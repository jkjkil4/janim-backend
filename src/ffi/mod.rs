mod gl;

use pyo3::prelude::*;

#[pymodule]
pub mod ffi {
    #[pymodule_export]
    use super::gl::Gl;
}
