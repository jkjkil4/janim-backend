mod quaternion;

use pyo3::prelude::*;

#[pymodule]
pub mod math {
    #[pymodule_export]
    use super::quaternion::PyQuaternion;
}
