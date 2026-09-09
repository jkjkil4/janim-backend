mod quaternion;
mod space_ops;

use pyo3::prelude::*;

#[pymodule]
pub mod math {
    #[pymodule_export]
    use super::quaternion::PyQuaternion;

    #[pymodule_export]
    use super::space_ops::{get_norm, get_unit_normal};
}
