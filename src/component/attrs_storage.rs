use pyo3::{
    exceptions::PyTypeError,
    prelude::*,
    types::{PyDict, PyList, PyTuple},
};

use super::attrs::AttrsInstance;

#[pyclass(module = "janim_backend.component", subclass)]
pub struct AttrsStorage {
    pub(crate) attrs_inst: Option<AttrsInstance>,
}

impl AttrsStorage {
    #[inline]
    fn inst(&self) -> &AttrsInstance {
        self.attrs_inst.as_ref().unwrap()
    }
}

#[pymethods]
impl AttrsStorage {
    #[new]
    #[pyo3(signature = (*_args, **_kwargs))]
    fn new(_args: &Bound<'_, PyTuple>, _kwargs: Option<&Bound<'_, PyDict>>) -> Self {
        Self { attrs_inst: None }
    }

    fn _init_attrs(&mut self, py: Python<'_>, fields: Bound<'_, PyList>) -> PyResult<()> {
        self.attrs_inst = Some(AttrsInstance::new(py, fields)?);
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
    fn __copy__<'py>(slf: Bound<'py, Self>, py: Python<'py>) -> PyResult<Bound<'py, Self>> {
        // SAFETY:
        //
        // - `ptr` comes from `slf.get_type_ptr()`, so it points to a valid
        //   `PyTypeObject` and lifetime matches.
        //
        // - `tp_new` is the `tp_new` slot of `ptr` and `#[new]` will be automatically called by it,
        //   so the returned pointer is a new owned reference suitable for `Bound::from_owned_ptr`
        let obj = unsafe {
            let ptr = slf.get_type_ptr();

            let tp_new = (*ptr)
                .tp_new
                .ok_or_else(|| PyTypeError::new_err("type has no tp_new"))?;

            let obj = tp_new(ptr, PyTuple::empty(py).as_ptr(), std::ptr::null_mut());

            if obj.is_null() {
                return Err(PyErr::fetch(py));
            }

            Bound::from_owned_ptr(py, obj)
        };

        let cmpt_copy: Bound<'_, Self> = obj.extract()?;
        cmpt_copy.borrow_mut().attrs_inst = Some(slf.borrow().inst().copy(py)?);

        Ok(cmpt_copy)
    }

    pub fn copy<'py>(slf: Bound<'py, Self>, py: Python<'py>) -> PyResult<Bound<'py, Self>> {
        AttrsStorage::__copy__(slf, py)
    }

    /// Behaves like:
    ///
    /// ```python
    /// def _become(self, other) -> None:
    ///     self.attrs_inst = other.attrs_inst.copy()
    /// ```
    pub fn _become(&mut self, py: Python<'_>, other: Bound<'_, Self>) -> PyResult<()> {
        self.attrs_inst = Some(other.borrow().inst().copy(py)?);
        Ok(())
    }

    /// Behaves like:
    ///
    /// ```python
    /// def _take_modified(self) -> bool:
    ///     return self.attrs_inst.take_modified()
    /// ```
    pub fn _take_modified(&mut self) -> bool {
        self.attrs_inst.as_mut().unwrap().take_modified()
    }
}
