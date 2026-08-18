use numpy::{PyReadonlyArray1, PyReadonlyArray2};
use pyo3::prelude::*;
use pyo3::types::PyBytes;

/// Computes the mapped clip-space bounding box of a 3D axis-aligned
/// bounding box.
///
/// The eight corners of the bounding box are mapped to GL coordinates,
/// and the resulting 2D bounding box is expanded by `buff`.
///
/// The coordinates are clamped to the GL clip range `[-1, 1]`.
///
/// # Arguments
///
/// * `mins` - The minimum `[x, y, z]` coordinates of the bounding box. Must have shape `(3,)`.
///
/// * `maxs` - The maximum `[x, y, z]` coordinates of the bounding box. Must have shape `(3,)`.
///
/// * `proj_view_matrix` - The 4x4 projection-view matrix used when `fixed_in_frame` is false. Must have shape `(4, 4)`.
///
/// * `proj_matrix` - The 4x4 projection matrix used when `fixed_in_frame` is true. Must have shape `(4, 4)`.
///
/// * `fixed_in_frame` - Whether the bounding box should be mapped using the fixed-in-frame projection.
///
/// * `fixed_distance_from_plane` - The distance to subtract from the z coordinate when `fixed_in_frame` is true.
///
/// * `frame_radius` - Containing the x and y frame radii. Must have shape `(2,)`.
///
/// * `buff` - The amount by which to expand the resulting bounding box, expressed in the same units as `frame_radius`.
///
/// # Returns
///
/// A Python `bytes` object containing eight little-endian `f32` values:
///
/// ```text
/// [
///     min_x, min_y,
///     min_x, max_y,
///     max_x, min_y,
///     max_x, max_y,
/// ]
/// ```
#[allow(clippy::too_many_arguments)]
#[pyfunction]
pub fn compute_mapped_clip_box_in_glcoord<'py>(
    py: Python<'py>,
    mins: PyReadonlyArray1<'_, f32>,
    maxs: PyReadonlyArray1<'_, f32>,
    proj_view_matrix: PyReadonlyArray2<'_, f32>,
    proj_matrix: PyReadonlyArray2<'_, f32>,
    fixed_in_frame: bool,
    fixed_distance_from_plane: f32,
    frame_radius: PyReadonlyArray1<'_, f32>,
    buff: f32,
) -> PyResult<Bound<'py, PyBytes>> {
    let mins = mins.as_slice()?;
    let maxs = maxs.as_slice()?;
    let frame_radius = frame_radius.as_slice()?;

    let frame_radius_x = frame_radius[0];
    let frame_radius_y = frame_radius[1];

    let (matrix, z_offset) = if fixed_in_frame {
        (proj_matrix.as_array(), fixed_distance_from_plane)
    } else {
        (proj_view_matrix.as_array(), 0.0)
    };

    let mut clip_min_x = f32::INFINITY;
    let mut clip_min_y = f32::INFINITY;
    let mut clip_max_x = f32::NEG_INFINITY;
    let mut clip_max_y = f32::NEG_INFINITY;

    // Enumerate all eight corners of the axis-aligned bounding box.
    for &x in &[mins[0], maxs[0]] {
        for &y in &[mins[1], maxs[1]] {
            for &original_z in &[mins[2], maxs[2]] {
                let z = original_z - z_offset;

                // Equivalent to:
                //
                // aligned @ matrix.T
                //
                // where aligned = [x, y, z, 1].
                let clip_x =
                    x * matrix[[0, 0]] + y * matrix[[0, 1]] + z * matrix[[0, 2]] + matrix[[0, 3]];

                let clip_y =
                    x * matrix[[1, 0]] + y * matrix[[1, 1]] + z * matrix[[1, 2]] + matrix[[1, 3]];

                let clip_w =
                    x * matrix[[3, 0]] + y * matrix[[3, 1]] + z * matrix[[3, 2]] + matrix[[3, 3]];

                let mapped_x = clip_x / clip_w;
                let mapped_y = clip_y / clip_w;

                clip_min_x = clip_min_x.min(mapped_x);
                clip_min_y = clip_min_y.min(mapped_y);
                clip_max_x = clip_max_x.max(mapped_x);
                clip_max_y = clip_max_y.max(mapped_y);
            }
        }
    }

    let buff_x = buff / frame_radius_x;
    let buff_y = buff / frame_radius_y;

    let min_x = (clip_min_x - buff_x).clamp(-1.0, 1.0);
    let min_y = (clip_min_y - buff_y).clamp(-1.0, 1.0);
    let max_x = (clip_max_x + buff_x).clamp(-1.0, 1.0);
    let max_y = (clip_max_y + buff_y).clamp(-1.0, 1.0);

    let clip_box = [min_x, min_y, min_x, max_y, max_x, min_y, max_x, max_y];

    let bytes = bytemuck::cast_slice(&clip_box);

    Ok(PyBytes::new(py, bytes))
}
