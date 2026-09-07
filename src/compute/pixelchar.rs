use numpy::{PyReadonlyArray1, PyReadonlyArray2};
use pyo3::prelude::*;
use pyo3::types::PyBytes;

/// Computes the uniform data for a pixel character from three mapped points.
///
/// Returns `(orig_bytes, mat_bytes)`, containing the origin and the 2x2
/// character matrix as little-endian `f32` bytes.
#[pyfunction]
pub fn compute_pixelchar_uniform_bytes<'py>(
    py: Python<'py>,
    mapped: PyReadonlyArray2<'_, f32>,
    frame_radius: PyReadonlyArray1<'_, f32>,
) -> PyResult<(Bound<'py, PyBytes>, Bound<'py, PyBytes>)> {
    let mapped = mapped.as_array();
    let frame_radius = frame_radius.as_slice()?;

    let orig = [
        mapped[[0, 0]] * frame_radius[0],
        mapped[[0, 1]] * frame_radius[1],
    ];
    let right = [
        mapped[[1, 0]] * frame_radius[0],
        mapped[[1, 1]] * frame_radius[1],
    ];
    let up = [
        mapped[[2, 0]] * frame_radius[0],
        mapped[[2, 1]] * frame_radius[1],
    ];
    let mat = [
        right[0] - orig[0],
        right[1] - orig[1],
        up[0] - orig[0],
        up[1] - orig[1],
    ];

    Ok((
        PyBytes::new(py, bytemuck::cast_slice(&orig)),
        PyBytes::new(py, bytemuck::cast_slice(&mat)),
    ))
}
