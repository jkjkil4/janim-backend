use pyo3::{
    PyTraverseError, PyVisit,
    exceptions::PyTypeError,
    prelude::*,
    types::{PyDict, PyList, PyTuple},
};

use crate::component::bind::BindState;

use super::attrs::AttrsInstance;

/// The base class of `Component` in Python
#[pyclass(module = "janim_backend.component", subclass)]
pub struct AttrsStorage {
    pub(crate) attrs_inst: Option<AttrsInstance>,

    #[pyo3(get)]
    _bind: Option<Py<BindState>>,
}

impl AttrsStorage {
    #[inline]
    pub(crate) fn inst(&self) -> &AttrsInstance {
        self.attrs_inst.as_ref().unwrap()
    }

    #[inline]
    pub(crate) fn inst_mut(&mut self) -> &mut AttrsInstance {
        self.attrs_inst.as_mut().unwrap()
    }
}

#[pymethods]
impl AttrsStorage {
    #[new]
    #[pyo3(signature = (*_args, **_kwargs))]
    fn new(_args: &Bound<'_, PyTuple>, _kwargs: Option<&Bound<'_, PyDict>>) -> Self {
        Self {
            attrs_inst: None,
            _bind: None,
        }
    }

    fn _init_attrs(&mut self, py: Python<'_>, fields: Bound<'_, PyList>) -> PyResult<()> {
        self.attrs_inst = Some(AttrsInstance::new(py, fields)?);
        Ok(())
    }

    #[inline]
    pub(crate) fn bind(&mut self, bind_state: Py<BindState>) {
        self._bind = Some(bind_state);
    }

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

    /// Behaves like:
    ///
    /// ```python
    /// def copy(self) -> Self:
    ///     cls = self.__class__
    ///     cmpt_copy = cls.__new__(cls)
    ///     cmpt_copy.attrs_inst = self.attrs_inst.copy()
    ///     return cmpt_copy
    /// ```
    pub fn copy<'py>(slf: Bound<'py, Self>, py: Python<'py>) -> PyResult<Bound<'py, Self>> {
        AttrsStorage::__copy__(slf, py)
    }

    /// Updates fields from `other`, preserving owned objects already held by `self`.
    pub fn _become(&mut self, py: Python<'_>, other: Bound<'_, Self>) -> PyResult<()> {
        self.inst_mut().become_from(py, other.borrow().inst())?;
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

    // GC compatibility
    fn __traverse__(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        visit.call(&self._bind)?;
        if let Some(attrs_inst) = &self.attrs_inst {
            for data in attrs_inst.datas.values() {
                data.traverse(&visit)?;
            }
        }
        Ok(())
    }
    fn __clear__(&mut self) -> PyResult<()> {
        self._bind = None;
        if let Some(attrs_inst) = &mut self.attrs_inst {
            attrs_inst.datas.clear();
        }
        Ok(())
    }
}
