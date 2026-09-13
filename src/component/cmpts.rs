use std::{
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering},
};

use pyo3::{
    IntoPyObjectExt,
    exceptions::PyRuntimeError,
    prelude::*,
    types::{PyDict, PyTuple, PyType},
};

use super::attrs_storage::AttrsStorage;
use super::cmpts_storage::CmptsStorage;

// -----------------------------------------------------
// Python Interface: CmptField & CmptInfo
// -----------------------------------------------------

#[pyclass(module = "janim_backend.component")]
pub struct CmptField {
    pub(crate) info: Py<CmptInfo>,
    pub(crate) decl_cls: Py<PyAny>,
}

#[pymethods]
impl CmptField {
    #[new]
    fn new(info: Py<CmptInfo>, decl_cls: Py<PyAny>) -> Self {
        Self { info, decl_cls }
    }

    #[getter]
    fn info(&self, py: Python<'_>) -> Py<CmptInfo> {
        self.info.clone_ref(py)
    }

    #[setter]
    fn set_info(&mut self, info: Py<CmptInfo>) {
        self.info = info;
    }
}

static NEXT_CMPT_INFO_ID: AtomicUsize = AtomicUsize::new(0);

#[inline]
fn next_id() -> usize {
    NEXT_CMPT_INFO_ID.fetch_add(1, Ordering::Relaxed)
}

#[pyclass(module = "janim_backend.component", subclass)]
pub struct CmptInfo {
    cmpt_field_id: usize,

    #[pyo3(get)]
    cls: Py<PyAny>,
    pub(crate) last_attrs_cls: Py<PyAny>,

    args: Py<PyTuple>,
    kwargs: Option<Py<PyDict>>,
}

#[pymethods]
impl CmptInfo {
    #[new]
    #[pyo3(signature = (cls, *args, **kwargs))]
    fn new(
        cls: Bound<'_, PyAny>,
        args: &Bound<'_, PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Self> {
        let cls = cls.getattr("__origin__").unwrap_or(cls);
        let last_attrs_cls = cls.getattr(crate::attr_names::COMPONENT__LAST_ATTRS_CLS)?;
        Ok(Self {
            cmpt_field_id: next_id(),
            cls: cls.unbind(),
            last_attrs_cls: last_attrs_cls.unbind(),
            args: args.clone().unbind(),
            kwargs: kwargs.map(|x| x.clone().unbind()),
        })
    }

    fn create(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        self.cls
            .call(py, &self.args, self.kwargs.as_ref().map(|x| x.bind(py)))
    }

    /// Behaves like:
    ///
    /// ```python
    /// def __get__(self, obj, owner):
    ///     if obj is None:
    ///         return self
    ///     return obj.cmpts_inst.get(self.cmpt_field_id)
    /// ```
    fn __get__<'py>(
        slf: Bound<'py, Self>,
        py: Python<'py>,
        obj: Option<Bound<'py, CmptsStorage>>,
        _owner: Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let cmpt_field_id = slf.borrow().cmpt_field_id;
        let result = match obj {
            Some(obj) => obj
                .borrow()
                .cmpts_inst
                .as_ref()
                .unwrap()
                .get(py, cmpt_field_id)?
                .into_bound_py_any(py)?,
            None => slf.into_any(),
        };
        Ok(result)
    }

    fn __set__(&self, _obj: Bound<'_, AttrsStorage>, _value: Bound<'_, PyAny>) -> PyResult<()> {
        Err(PyRuntimeError::new_err(
            "component cannot be assigned directly",
        ))
    }

    /// Be compatible with the generic subscripting
    #[classmethod]
    fn __class_getitem__<'py>(
        cls: &Bound<'py, PyType>,
        _item: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyType>> {
        Ok(cls.clone())
    }
}

// -----------------------------------------------------
// Internal Implementation: CmptsInstance
// -----------------------------------------------------

pub type CmptValues = (String, Py<CmptField>, Py<AttrsStorage>);

pub(crate) struct CmptsInstance {
    pub(crate) cmpts: HashMap<usize, CmptValues>,
}

impl CmptsInstance {
    pub fn new(py: Python<'_>, fields: Bound<'_, PyDict>) -> PyResult<Self> {
        let cmpts = fields
            .iter()
            .map(|(k, v)| {
                let key = k.extract::<String>()?;

                let field: Py<CmptField> = v.extract()?;
                let field_ref = field.borrow(py);
                let info = field_ref.info.borrow(py);

                let id = info.cmpt_field_id;
                let cmpt: Py<AttrsStorage> = info.create(py)?.extract(py)?;

                drop(info);
                drop(field_ref);
                Ok((id, (key, field, cmpt)))
            })
            .collect::<PyResult<_>>()?;

        Ok(Self { cmpts })
    }

    #[inline]
    fn cmpt(&self, cmpt_field_id: usize) -> PyResult<&CmptValues> {
        self.cmpts
            .get(&cmpt_field_id)
            .ok_or_else(|| PyRuntimeError::new_err("Incompatible component"))
    }

    fn get(&self, py: Python<'_>, cmpt_field_id: usize) -> PyResult<Py<AttrsStorage>> {
        Ok(self.cmpt(cmpt_field_id)?.2.clone_ref(py))
    }

    pub fn copy(&self, py: Python<'_>) -> PyResult<Self> {
        let copied_cmpts = self
            .cmpts
            .iter()
            .map(|(id, (key, field, cmpt))| {
                let copied = AttrsStorage::copy(cmpt.clone_ref(py).into_bound(py), py)?;
                Ok((*id, (key.clone(), field.clone_ref(py), copied.unbind())))
            })
            .collect::<PyResult<_>>()?;

        Ok(Self {
            cmpts: copied_cmpts,
        })
    }
}
