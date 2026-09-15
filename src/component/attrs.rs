use std::{
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering},
};

use pyo3::{
    IntoPyObjectExt,
    exceptions::PyRuntimeError,
    prelude::*,
    types::{PyBool, PyFloat, PyInt, PyList, PyNone},
    {PyTraverseError, PyVisit},
};

use super::attrs_storage::AttrsStorage;

// -----------------------------------------------------
// Python Interface: AttrField & AttrFieldDescriptor
// -----------------------------------------------------

static NEXT_FIELD_ID: AtomicUsize = AtomicUsize::new(0);

#[inline]
fn next_id() -> usize {
    NEXT_FIELD_ID.fetch_add(1, Ordering::Relaxed)
}

#[pyclass(module = "janim_backend.component")]
pub struct AttrField {
    id: usize,
    default: FieldData,
}

impl AttrField {
    #[inline]
    fn new(default: FieldData) -> Self {
        Self {
            id: next_id(),
            default,
        }
    }
}

#[allow(non_snake_case)]
#[pymethods]
impl AttrField {
    #[staticmethod]
    fn Int(default: i32) -> Self {
        Self::new(FieldData::Int(default))
    }

    #[staticmethod]
    fn Float(default: f64) -> Self {
        Self::new(FieldData::Float(default))
    }

    #[staticmethod]
    fn Bool(default: bool) -> Self {
        Self::new(FieldData::Bool(default))
    }

    #[staticmethod]
    fn NDArray(py: Python<'_>, default: Py<PyAny>, setter: NDArraySetter) -> PyResult<Self> {
        let default = setter.call1(py, (default,))?;
        Ok(Self::new(FieldData::NDArray(default, setter)))
    }

    #[staticmethod]
    fn OwnedObject(py: Python<'_>) -> PyResult<Self> {
        Ok(Self::new(FieldData::OwnedObject(
            PyNone::get(py).into_py_any(py)?,
        )))
    }

    #[staticmethod]
    fn DirectObject(py: Python<'_>, copyer: DirectObjectCopyer) -> PyResult<Self> {
        Ok(Self::new(FieldData::DirectObject(
            PyNone::get(py).into_py_any(py)?,
            copyer,
        )))
    }
}

#[pyclass]
pub struct AttrFieldDescriptor {
    field_id: usize,
    modified_callback: Option<Py<PyAny>>,
}

#[pymethods]
impl AttrFieldDescriptor {
    #[new]
    fn new(field: Bound<'_, AttrField>) -> Self {
        Self {
            field_id: field.borrow().id,
            modified_callback: None,
        }
    }

    fn on_modified<'py>(&mut self, callback: Bound<'py, PyAny>) -> Bound<'py, PyAny> {
        self.modified_callback = Some(callback.clone().unbind());
        callback
    }

    /// Behaves like:
    ///
    /// ```python
    /// def __get__(self, obj, owner):
    ///     if obj is None:
    ///         return self
    ///     return obj.attrs_inst.get(self.field_id)
    /// ```
    fn __get__<'py>(
        slf: Bound<'py, Self>,
        py: Python<'py>,
        obj: Option<Bound<'py, AttrsStorage>>,
        _owner: Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let field_id = slf.borrow().field_id;
        let result = match obj {
            Some(obj) => obj.borrow().inst().get(py, field_id)?,
            None => slf.into_any(),
        };
        Ok(result)
    }

    /// Behaves like:
    ///
    /// ```python
    /// def __set__(self, obj, value, /) -> None:
    ///     obj.attrs_inst.set(self.field_id, value)
    /// ```
    fn __set__(
        &self,
        py: Python<'_>,
        obj: Bound<'_, AttrsStorage>,
        value: Bound<'_, PyAny>,
    ) -> PyResult<()> {
        obj.borrow_mut().inst_mut().set(py, self.field_id, value)?;

        if let Some(f) = &self.modified_callback {
            f.call1(py, (obj.into_any(),))?;
        }

        Ok(())
    }
}

// -----------------------------------------------------
// Internal Implementation: AttrsInstance & FieldData
// -----------------------------------------------------
pub(crate) struct AttrsInstance {
    pub(crate) datas: HashMap<usize, FieldData>,
    modified: bool,
}

impl AttrsInstance {
    pub fn new(py: Python<'_>, fields: Bound<'_, PyList>) -> PyResult<Self> {
        let datas = fields
            .iter()
            .map(|x| {
                let field: PyRef<'_, AttrField> = x.extract()?;
                Ok((field.id, field.default.clone_data(py)?))
            })
            .collect::<PyResult<_>>()?;

        Ok(Self {
            datas,
            modified: true,
        })
    }

    #[inline]
    fn data(&self, field_id: usize) -> PyResult<&FieldData> {
        self.datas
            .get(&field_id)
            .ok_or_else(|| PyRuntimeError::new_err("Incompatible component data"))
    }

    #[inline]
    fn data_mut(&mut self, field_id: usize) -> PyResult<&mut FieldData> {
        self.datas
            .get_mut(&field_id)
            .ok_or_else(|| PyRuntimeError::new_err("Incompatible component data"))
    }

    fn set(&mut self, py: Python<'_>, field_id: usize, value: Bound<'_, PyAny>) -> PyResult<()> {
        self.modified = true;
        self.data_mut(field_id)?.set_pyany(py, value)
    }

    fn get<'py>(&self, py: Python<'py>, field_id: usize) -> PyResult<Bound<'py, PyAny>> {
        self.data(field_id)?.get_pyany(py)
    }

    pub fn copy(&self, py: Python<'_>) -> PyResult<Self> {
        let copied_datas = self
            .datas
            .iter()
            .map(|(id, data)| Ok((*id, data.clone_data(py)?)))
            .collect::<PyResult<_>>()?;
        Ok(Self {
            datas: copied_datas,
            modified: true,
        })
    }

    pub fn become_from(&mut self, py: Python<'_>, other: &Self) -> PyResult<()> {
        for (id, other_data) in &other.datas {
            let data = self.data_mut(*id)?;
            if matches!(data, FieldData::OwnedObject(_)) {
                continue;
            }
            *data = other_data.clone_data(py)?;
        }
        self.modified = true;
        Ok(())
    }

    pub fn take_modified(&mut self) -> bool {
        let modified = self.modified;
        self.modified = false;
        modified
    }
}

type NDArraySetter = Py<PyAny>;
type DirectObjectCopyer = Option<Py<PyAny>>;

pub(crate) enum FieldData {
    Int(i32),
    Float(f64),
    Bool(bool),
    NDArray(Py<PyAny>, NDArraySetter),
    OwnedObject(Py<PyAny>),
    DirectObject(Py<PyAny>, DirectObjectCopyer),
}

impl FieldData {
    pub(crate) fn traverse(&self, visit: &PyVisit<'_>) -> Result<(), PyTraverseError> {
        match self {
            Self::NDArray(current, setter) => {
                visit.call(current)?;
                visit.call(setter)?;
            }
            Self::OwnedObject(current) => visit.call(current)?,
            Self::DirectObject(current, copyer) => {
                visit.call(current)?;
                visit.call(copyer)?;
            }
            Self::Int(_) | Self::Float(_) | Self::Bool(_) => {}
        }
        Ok(())
    }

    fn set_pyany(&mut self, py: Python<'_>, value: Bound<'_, PyAny>) -> PyResult<()> {
        match self {
            Self::Int(current) => *current = value.extract::<i32>()?,
            Self::Float(current) => *current = value.extract::<f64>()?,
            Self::Bool(current) => *current = value.extract::<bool>()?,

            Self::NDArray(current, setter) => {
                *current = setter.call1(py, (value,))?;
            }
            Self::OwnedObject(current) => {
                *current = value.unbind();
            }
            Self::DirectObject(current, _copyer) => *current = value.unbind(),
        }

        Ok(())
    }

    fn get_pyany<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let result = match self {
            Self::Int(value) => PyInt::new(py, *value).into_any(),
            Self::Float(value) => PyFloat::new(py, *value).into_any(),
            Self::Bool(value) => PyBool::new(py, *value).to_owned().into_any(),

            Self::NDArray(current, _setter) => current.clone_ref(py).into_bound(py),
            Self::OwnedObject(current) => current.clone_ref(py).into_bound(py),
            Self::DirectObject(current, _copyer) => current.clone_ref(py).into_bound(py),
        };
        Ok(result)
    }

    fn clone_data(&self, py: Python<'_>) -> PyResult<Self> {
        let result = match self {
            Self::Int(current) => Self::Int(*current),
            Self::Float(current) => Self::Float(*current),
            Self::Bool(current) => Self::Bool(*current),

            Self::NDArray(current, setter) => {
                Self::NDArray(current.clone_ref(py), setter.clone_ref(py))
            }
            Self::OwnedObject(_current) => Self::OwnedObject(PyNone::get(py).into_py_any(py)?),
            Self::DirectObject(current, copyer) => {
                let copied = match copyer {
                    Some(f) if !current.is_none(py) => f.call1(py, (current,))?,
                    _ => current.clone_ref(py),
                };
                Self::DirectObject(copied, copyer.as_ref().map(|f| f.clone_ref(py)))
            }
        };
        Ok(result)
    }
}
