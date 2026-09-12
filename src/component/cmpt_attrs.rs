use std::{
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering},
};

use pyo3::{
    IntoPyObjectExt,
    exceptions::PyRuntimeError,
    prelude::*,
    types::{PyBool, PyDict, PyFloat, PyInt, PyList, PyNone, PyTuple},
};

// -----------------------------------------------------
// Python Interface: CmptCore & CmptField & CmptFieldDescriptor
// -----------------------------------------------------

#[pyclass(module = "janim_backend.component", subclass)]
pub struct CmptCore {
    attrs_inst: Option<CmptAttrsInstance>,
}

#[pymethods]
impl CmptCore {
    #[new]
    #[pyo3(signature = (*_args, **_kwargs))]
    fn new(_args: &Bound<'_, PyTuple>, _kwargs: Option<&Bound<'_, PyDict>>) -> Self {
        Self { attrs_inst: None }
    }

    fn _init_attrs(&mut self, py: Python<'_>, fields: Bound<'_, PyList>) -> PyResult<()> {
        self.attrs_inst = Some(CmptAttrsInstance::new(py, fields)?);
        Ok(())
    }

    /// Behaves like:
    ///
    /// ```python
    /// def copy(self) -> Self:
    ///     cls = self.__class__
    ///     cmpt_copy = cls.__new__(cls)
    ///     cmpt_copy.attrs_inst = self.attrs_inst.copy()
    ///     return cmpt_copy
    /// ```
    fn _copy_attrs_inst_to(&self, py: Python<'_>, cmpt_copy: Bound<'_, Self>) -> PyResult<()> {
        // let cls = slf.get_type();
        // let cmpt_copy: Bound<'py, Self> = cls.clone().call_method1("__new__", (cls,))?.extract()?;
        // cmpt_copy.borrow_mut().attrs_inst =
        //     Some(slf.borrow().attrs_inst.as_ref().unwrap().copy(py)?);
        // Ok(cmpt_copy)
        cmpt_copy.borrow_mut().attrs_inst = Some(self.attrs_inst.as_ref().unwrap().copy(py)?);
        Ok(())
    }

    /// Behaves like:
    ///
    /// ```python
    /// def _become(self, other) -> None:
    ///     self.attrs_inst = other.attrs_inst.copy()
    /// ```
    fn _become(&mut self, py: Python<'_>, other: Bound<'_, Self>) -> PyResult<()> {
        self.attrs_inst = Some(other.borrow().attrs_inst.as_ref().unwrap().copy(py)?);
        Ok(())
    }

    /// Behaves like:
    ///
    /// ```python
    /// def take_modified(self) -> bool:
    ///     return self.attrs_inst.take_modified()
    /// ```
    fn take_modified(&mut self) -> bool {
        self.attrs_inst.as_mut().unwrap().take_modified()
    }
}

static NEXT_FIELD_ID: AtomicUsize = AtomicUsize::new(0);

#[inline]
fn next_id() -> usize {
    NEXT_FIELD_ID.fetch_add(1, Ordering::Relaxed)
}

#[pyclass(module = "janim_backend.component")]
pub struct CmptField {
    id: usize,
    default: FieldData,
}

impl CmptField {
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
impl CmptField {
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

    // #[staticmethod]
    // fn Custom(
    //     py: Python<'_>,
    //     default: Py<PyAny>,
    //     setter: CustomSetter,
    //     getter: CustomGetter,
    //     copyer: CustomCopyer,
    // ) -> PyResult<Self> {
    //     let default = match setter.as_ref() {
    //         Some(f) => f.call1(py, (default,))?,
    //         None => default,
    //     };
    //     Ok(Self::new(FieldData::Custom(
    //         default, setter, getter, copyer,
    //     )))
    // }

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
pub struct CmptFieldDescriptor {
    field_id: usize,
    modified_callback: Option<Py<PyAny>>,
}

#[pymethods]
impl CmptFieldDescriptor {
    #[new]
    fn new(field: Bound<'_, CmptField>) -> Self {
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
        obj: Option<Bound<'py, CmptCore>>,
        _owner: Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let field_id = slf.borrow().field_id;
        let result = match obj {
            Some(obj) => obj
                .borrow()
                .attrs_inst
                .as_ref()
                .unwrap()
                .get(py, field_id)?,
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
        obj: Bound<'_, CmptCore>,
        value: Bound<'_, PyAny>,
    ) -> PyResult<()> {
        obj.borrow_mut()
            .attrs_inst
            .as_mut()
            .unwrap()
            .set(py, self.field_id, value)?;

        if let Some(f) = &self.modified_callback {
            f.call1(py, (obj.into_any(),))?;
        }

        Ok(())
    }
}

// -----------------------------------------------------
// Internal Implementation: CmptAttrsInstance & FieldData
// -----------------------------------------------------
struct CmptAttrsInstance {
    datas: HashMap<usize, FieldData>,
    modified: bool,
}

impl CmptAttrsInstance {
    fn new(py: Python<'_>, fields: Bound<'_, PyList>) -> PyResult<Self> {
        let datas = fields
            .iter()
            .map(|x| {
                let field: PyRef<'_, CmptField> = x.extract()?;
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

    fn copy<'py>(&self, py: Python<'py>) -> PyResult<Self> {
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

    fn take_modified(&mut self) -> bool {
        let modified = self.modified;
        self.modified = false;
        modified
    }
}

type NDArraySetter = Py<PyAny>;
type DirectObjectCopyer = Option<Py<PyAny>>;

enum FieldData {
    Int(i32),
    Float(f64),
    Bool(bool),
    NDArray(Py<PyAny>, NDArraySetter),
    OwnedObject(Py<PyAny>),
    DirectObject(Py<PyAny>, DirectObjectCopyer),
}

impl FieldData {
    fn set_pyany(&mut self, py: Python<'_>, value: Bound<'_, PyAny>) -> PyResult<()> {
        match self {
            Self::Int(current) => *current = value.extract::<i32>()?,
            Self::Float(current) => *current = value.extract::<f64>()?,
            Self::Bool(current) => *current = value.extract::<bool>()?,

            // Self::Custom(current, setter, _, _) => {
            //     *current = match setter {
            //         Some(f) => f.call1(py, (value,))?,
            //         None => value.unbind(),
            //     }
            // }
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

            // Self::Custom(value, _, getter, _) => match getter {
            //     Some(f) => f.call1(py, (value,))?.into_bound(py),
            //     None => value.bind(py).clone(),
            // },
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

            // Self::Custom(current, setter, getter, copyer) => {
            //     let copied = match copyer {
            //         Some(f) if !current.is_none(py) => f.call1(py, (current,))?,
            //         _ => current.clone_ref(py),
            //     };
            //     Self::Custom(
            //         copied,
            //         setter.as_ref().map(|f| f.clone_ref(py)),
            //         getter.as_ref().map(|f| f.clone_ref(py)),
            //         copyer.as_ref().map(|f| f.clone_ref(py)),
            //     )
            // }
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
