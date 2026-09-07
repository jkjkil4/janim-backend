use ndarray::{Array2, ArrayView2, Axis};
use numpy::{IntoPyArray, PyArray2, PyReadonlyArray2};
use pyo3::prelude::*;

use crate::exception::JAnimBackendException;

fn map_points_impl(
    points: ArrayView2<'_, f32>,
    matrix: ArrayView2<'_, f32>,
    z_offset: f32,
    include_depth: bool,
) -> PyResult<Array2<f32>> {
    if points.ncols() != 3 {
        return Err(JAnimBackendException::new_err(format!(
            "points must have shape (N, 3), got ({}, {})",
            points.nrows(),
            points.ncols()
        )));
    }

    let output_width = if include_depth { 3 } else { 2 };
    let mut mapped = Array2::zeros((points.nrows(), output_width));

    for (point, mut output) in points.axis_iter(Axis(0)).zip(mapped.axis_iter_mut(Axis(0))) {
        let x = point[0];
        let y = point[1];
        let z = point[2] - z_offset;

        let clip_x = x * matrix[[0, 0]] + y * matrix[[0, 1]] + z * matrix[[0, 2]] + matrix[[0, 3]];
        let clip_y = x * matrix[[1, 0]] + y * matrix[[1, 1]] + z * matrix[[1, 2]] + matrix[[1, 3]];
        let clip_z = x * matrix[[2, 0]] + y * matrix[[2, 1]] + z * matrix[[2, 2]] + matrix[[2, 3]];
        let clip_w = x * matrix[[3, 0]] + y * matrix[[3, 1]] + z * matrix[[3, 2]] + matrix[[3, 3]];

        output[0] = clip_x / clip_w;
        output[1] = clip_y / clip_w;
        if include_depth {
            output[2] = clip_z / clip_w;
        }
    }

    Ok(mapped)
}

/// Maps Nx3 points into the camera's GL clip-space coordinates.
///
/// Returns an `(N, 2)` array containing the mapped x and y coordinates.
#[pyfunction]
pub fn map_points<'py>(
    py: Python<'py>,
    points: PyReadonlyArray2<'_, f32>,
    proj_view_matrix: PyReadonlyArray2<'_, f32>,
) -> PyResult<Bound<'py, PyArray2<f32>>> {
    map_points_impl(points.as_array(), proj_view_matrix.as_array(), 0.0, false)
        .map(|mapped| mapped.into_pyarray(py))
}

/// Maps Nx3 points into the camera's GL clip-space coordinates, including depth.
///
/// Returns an `(N, 3)` array containing the mapped x, y, and z coordinates.
#[pyfunction]
pub fn map_points_with_depth<'py>(
    py: Python<'py>,
    points: PyReadonlyArray2<'_, f32>,
    proj_view_matrix: PyReadonlyArray2<'_, f32>,
) -> PyResult<Bound<'py, PyArray2<f32>>> {
    map_points_impl(points.as_array(), proj_view_matrix.as_array(), 0.0, true)
        .map(|mapped| mapped.into_pyarray(py))
}

/// Maps Nx3 points into the default camera's GL clip-space coordinates.
///
/// Returns an `(N, 2)` array containing the mapped x and y coordinates.
#[pyfunction]
pub fn map_fixed_in_frame_points<'py>(
    py: Python<'py>,
    points: PyReadonlyArray2<'_, f32>,
    fixed_distance_from_plane: f32,
    proj_matrix: PyReadonlyArray2<'_, f32>,
) -> PyResult<Bound<'py, PyArray2<f32>>> {
    map_points_impl(
        points.as_array(),
        proj_matrix.as_array(),
        fixed_distance_from_plane,
        false,
    )
    .map(|mapped| mapped.into_pyarray(py))
}

/// Maps Nx3 points into the default camera's GL clip-space coordinates, including depth.
///
/// Returns an `(N, 3)` array containing the mapped x, y, and z coordinates.
#[pyfunction]
pub fn map_fixed_in_frame_points_with_depth<'py>(
    py: Python<'py>,
    points: PyReadonlyArray2<'_, f32>,
    fixed_distance_from_plane: f32,
    proj_matrix: PyReadonlyArray2<'_, f32>,
) -> PyResult<Bound<'py, PyArray2<f32>>> {
    map_points_impl(
        points.as_array(),
        proj_matrix.as_array(),
        fixed_distance_from_plane,
        true,
    )
    .map(|mapped| mapped.into_pyarray(py))
}
