use ndarray::Array2;
use numpy::{IntoPyArray, PyArray2, PyReadonlyArray2};
use pyo3::prelude::*;

/// Compute the bounding box of Nx3 `points`.
///
/// Returns the 3x3 NumPy array, represents `[bottom-left, center, top-right]` points.
#[pyfunction]
pub fn compute_bounding_box(
    points: PyReadonlyArray2<'_, f32>,
    py: Python<'_>,
) -> Py<PyArray2<f32>> {
    let points = points.as_array();

    if points.nrows() == 0 {
        return PyArray2::zeros(py, [3, 3], false).unbind();
    }

    let mut mins = [f32::INFINITY; 3];
    let mut maxs = [f32::NEG_INFINITY; 3];

    for row in points.rows() {
        for i in 0..3 {
            let value = row[i];

            if !value.is_nan() {
                mins[i] = mins[i].min(value);
                maxs[i] = maxs[i].max(value);
            }
        }
    }

    Array2::from_shape_vec(
        (3, 3),
        vec![
            mins[0],
            mins[1],
            mins[2],
            //
            (mins[0] + maxs[0]) / 2.0,
            (mins[1] + maxs[1]) / 2.0,
            (mins[2] + maxs[2]) / 2.0,
            //
            maxs[0],
            maxs[1],
            maxs[2],
        ],
    )
    .unwrap()
    .into_pyarray(py)
    .unbind()
}
