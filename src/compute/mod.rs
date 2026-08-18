mod clip_box;

use pyo3::prelude::*;

#[pymodule]
pub mod compute {
    #[pymodule_export]
    use super::clip_box::compute_mapped_clip_box_in_glcoord;
}
