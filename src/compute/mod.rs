mod bounding_box;
mod clip_box;

use pyo3::prelude::*;

#[pymodule]
pub mod compute {
    #[pymodule_export]
    use super::bounding_box::compute_bounding_box;
    #[pymodule_export]
    use super::clip_box::compute_mapped_clip_box_in_glcoord;
}
