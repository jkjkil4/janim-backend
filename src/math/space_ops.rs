use ndarray::Array1;
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1, PyUntypedArrayMethods};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

const DOWN: [f32; 3] = [0.0, -1.0, 0.0];
const OUT: [f32; 3] = [0.0, 0.0, 1.0];

fn dot(v1: [f32; 3], v2: [f32; 3]) -> f32 {
    v1[0] * v2[0] + v1[1] * v2[1] + v1[2] * v2[2]
}

fn cross(v1: [f32; 3], v2: [f32; 3]) -> [f32; 3] {
    [
        v1[1] * v2[2] - v1[2] * v2[1],
        v1[2] * v2[0] - v1[0] * v2[2],
        v1[0] * v2[1] - v1[1] * v2[0],
    ]
}

fn vec3_div(v: [f32; 3], divisor: f32) -> [f32; 3] {
    [v[0] / divisor, v[1] / divisor, v[2] / divisor]
}

fn vec3_unit_normal(v1: [f32; 3], v2: [f32; 3], tol: f32) -> [f32; 3] {
    let v1_norm = dot(v1, v1).sqrt();
    let v2_norm = dot(v2, v2).sqrt();

    let u = match (v1_norm == 0.0, v2_norm == 0.0) {
        (false, false) => {
            // Normal case: v1 and v2 are both non-null
            let cp = cross(v1, v2);
            let cp_norm = dot(cp, cp).sqrt();
            if cp_norm > tol {
                return vec3_div(cp, cp_norm);
            }
            // Otherwise, v1 and v2 were aligned, use v1 as the main vector
            v1
        }
        (true, false) => vec3_div(v2, v2_norm),
        (false, true) => vec3_div(v1, v1_norm),
        (true, true) => {
            // We consider zero-vector is also too aligned to the Z-axis, just return DOWN
            return DOWN;
        }
    };

    // If u is too aligned to the Z-axis, just return DOWN
    if u[0].abs() < tol && u[1].abs() < tol {
        return DOWN;
    }

    // Otherwise we use the vector that in the plane u shares with the Z-axis
    let cp = cross(cross(u, OUT), u);
    let cp_norm = dot(cp, cp).sqrt();
    vec3_div(cp, cp_norm)
}

/// Compute the Euclidean norm, or length, of an iterable vector.
#[pyfunction]
pub fn get_norm(vect: &Bound<'_, PyAny>) -> PyResult<f64> {
    let mut squared_norm = 0.0;
    for item in vect.try_iter()? {
        let value: f64 = item?.extract()?;
        squared_norm += value * value;
    }
    Ok(squared_norm.sqrt())
}

/// Compute a unit vector normal to the plane defined by two vectors.
///
/// - For non-aligned vectors, it returns their normalized cross product.
///
/// - For aligned vectors, it constructs a normal in the plane shared by the
/// direction and the positive z-axis.
///
/// - If the direction is too close to the z-axis, `DOWN` is used as a stable fallback.
#[pyfunction]
pub fn get_unit_normal<'py>(
    py: Python<'py>,
    v1: PyReadonlyArray1<'_, f32>,
    v2: PyReadonlyArray1<'_, f32>,
    tol: f32,
) -> PyResult<Bound<'py, PyArray1<f32>>> {
    if v1.len() != 3 || v2.len() != 3 {
        return Err(PyValueError::new_err("v1 and v2 must have shape (3,)"));
    }

    let v1 = v1.as_array();
    let v2 = v2.as_array();
    let v1 = [v1[0], v1[1], v1[2]];
    let v2 = [v2[0], v2[1], v2[2]];

    Ok(Array1::from_vec(vec3_unit_normal(v1, v2, tol).to_vec()).into_pyarray(py))
}
