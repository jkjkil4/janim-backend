mod bounding_box;
mod clip_box;
mod map_points;
mod pixelchar;

use pyo3::prelude::*;

#[pymodule]
pub mod compute {
    #[pymodule_export]
    use super::bounding_box::compute_bounding_box;
    #[pymodule_export]
    use super::clip_box::compute_mapped_clip_box_in_glcoord;
    #[pymodule_export]
    use super::map_points::{
        map_fixed_in_frame_points, map_fixed_in_frame_points_with_depth, map_points,
        map_points_with_depth,
    };
    #[pymodule_export]
    use super::pixelchar::compute_pixelchar_uniform_bytes;
}
