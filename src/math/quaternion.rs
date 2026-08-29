use nalgebra::{Quaternion, Unit, UnitQuaternion, Vector3};
use ndarray::{Array1, Array2};
use numpy::{IntoPyArray, PyArray1, PyArray2};

use pyo3::prelude::*;

use crate::exception::QuaternionError;

type Scalar = f64;

/// The unit-quaternion, representing a rotation
#[pyclass(
    module = "janim_backend.math",
    name = "Quaternion",
    frozen,
    skip_from_py_object
)]
pub struct PyQuaternion {
    rot: UnitQuaternion<Scalar>,
}

#[pymethods]
impl PyQuaternion {
    /// Constructs the quaternion `w + x*i + y*j + z*k` in `(w, x, y, z)` and normalize it to unit-quaternion
    #[staticmethod]
    fn from_wxyz(w: Scalar, x: Scalar, y: Scalar, z: Scalar) -> PyResult<Self> {
        if w == 0.0 && x == 0.0 && y == 0.0 && z == 0.0 {
            return Err(QuaternionError::new_err(t!("Zero quaternion is invalid.")));
        }
        let q = Quaternion::new(w, x, y, z);
        let rot = UnitQuaternion::new_normalize(q);
        Ok(Self { rot })
    }

    /// Constructs the quaternion `w + x*i + y*j + z*k` in `(x, y, z, w)` and normalize it to unit-quaternion
    #[staticmethod]
    fn from_xyzw(x: Scalar, y: Scalar, z: Scalar, w: Scalar) -> PyResult<Self> {
        Self::from_wxyz(w, x, y, z)
    }

    /// Constructs unit-quaternion by `axis` and `angle` representing the rotation
    ///
    /// If the `axis` is a zero vector, returns the identity quaternion
    #[staticmethod]
    fn from_angle_axis(angle: Scalar, axis: (Scalar, Scalar, Scalar)) -> Self {
        let rot = if axis.0 == 0.0 && axis.1 == 0.0 && axis.2 == 0.0 {
            UnitQuaternion::identity()
        } else {
            let axis = Unit::new_normalize(Vector3::new(axis.0, axis.1, axis.2));
            UnitQuaternion::from_axis_angle(&axis, angle)
        };
        Self { rot }
    }

    fn __mul__(&self, rhs: &Self) -> Self {
        Self {
            rot: self.rot * rhs.rot,
        }
    }

    fn conjugate(&self) -> Self {
        Self {
            rot: self.rot.conjugate(),
        }
    }

    #[getter]
    fn xyzw(&self) -> (Scalar, Scalar, Scalar, Scalar) {
        (self.rot.i, self.rot.j, self.rot.k, self.rot.w)
    }

    #[getter]
    fn wxyz(&self) -> (Scalar, Scalar, Scalar, Scalar) {
        (self.rot.w, self.rot.i, self.rot.j, self.rot.k)
    }

    #[getter]
    fn x(&self) -> Scalar {
        self.rot.i
    }

    #[getter]
    fn y(&self) -> Scalar {
        self.rot.j
    }

    #[getter]
    fn z(&self) -> Scalar {
        self.rot.k
    }

    #[getter]
    fn w(&self) -> Scalar {
        self.rot.w
    }

    #[getter]
    fn angle(&self) -> Scalar {
        self.rot.angle()
    }

    #[getter]
    fn axis<'py>(&self, py: Python<'py>) -> Bound<'py, PyArray1<Scalar>> {
        let axis = self.rot.axis();
        // Use `OUT` as default axis when it is invalid
        let axis = axis.map_or([0.0, 0.0, 1.0], |ax| [ax.x, ax.y, ax.z]);

        Array1::from_vec(axis.to_vec()).into_pyarray(py)
    }

    #[getter]
    fn angle_axis<'py>(&self, py: Python<'py>) -> (Scalar, Bound<'py, PyArray1<Scalar>>) {
        (self.angle(), self.axis(py))
    }

    #[getter]
    fn rotation_matrix<'py>(&self, py: Python<'py>) -> Bound<'py, PyArray2<Scalar>> {
        let rot = self.rot.to_rotation_matrix();
        let matrix = rot.matrix();

        Array2::from_shape_fn((3, 3), |(row, column)| matrix[(row, column)]).into_pyarray(py)
    }

    fn rotate_vector<'py>(
        &self,
        py: Python<'py>,
        vec: (Scalar, Scalar, Scalar),
    ) -> Bound<'py, PyArray1<Scalar>> {
        let rotated = self
            .rot
            .transform_vector(&Vector3::new(vec.0, vec.1, vec.2));

        Array1::from_vec(vec![rotated.x, rotated.y, rotated.z]).into_pyarray(py)
    }
}
