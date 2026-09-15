use std::collections::HashMap;

use pyo3::{PyTraverseError, PyVisit, prelude::*};

use crate::relation::handle::RelationHandle;
use crate::relation::registry::FlagHandle;

struct BindStateInner {
    decl_cls: Py<PyAny>,
    at_item: Py<PyAny>,
    key: Py<PyAny>,

    /// at_item._rel_handle
    rel_handle: Py<RelationHandle>,
}

#[pyclass(module = "janim_backend.component")]
pub struct BindState {
    /// This struct helps `__clear__`, it should always be `Some` unless during GC
    inner: Option<BindStateInner>,

    flag_0: usize,
    computed_caches: HashMap<usize, (Py<FlagHandle>, Py<PyAny>)>,
}

#[pymethods]
impl BindState {
    #[new]
    pub fn new(
        py: Python<'_>,
        decl_cls: Py<PyAny>,
        at_item: Py<PyAny>,
        key: Py<PyAny>,
    ) -> PyResult<Self> {
        let rel_handle: Py<RelationHandle> = at_item
            .getattr(py, crate::attr_names::ITEM_RELATION__REL_HANDLE)?
            .extract(py)?;
        let flag_0 = rel_handle
            .borrow(py)
            .registry(py)
            .borrow()
            .indexize_key(key.extract(py)?);
        Ok(Self {
            inner: Some(BindStateInner {
                decl_cls,
                at_item,
                key,
                rel_handle,
            }),
            flag_0,
            computed_caches: HashMap::new(),
        })
    }

    #[getter]
    fn decl_cls(&self, py: Python<'_>) -> Py<PyAny> {
        self.inner.as_ref().unwrap().decl_cls.clone_ref(py)
    }

    #[getter]
    fn at_item(&self, py: Python<'_>) -> Py<PyAny> {
        self.inner.as_ref().unwrap().at_item.clone_ref(py)
    }

    #[getter]
    fn key(&self, py: Python<'_>) -> Py<PyAny> {
        self.inner.as_ref().unwrap().key.clone_ref(py)
    }

    fn get_computed_for(
        &self,
        py: Python<'_>,
        flag_handle: Bound<'_, FlagHandle>,
        on_expired: Py<PyAny>,
    ) -> PyResult<Py<PyAny>> {
        let inner = self.inner.as_ref().unwrap();
        if !inner
            .rel_handle
            .borrow(py)
            .get_computed_for(py, self.flag_0, flag_handle.clone())
        {
            return Ok(on_expired);
        }

        Ok(self
            .computed_caches
            .get(&flag_handle.borrow().flag_1())
            .map(|(_, data)| data.clone_ref(py))
            .unwrap_or(on_expired))
    }

    fn mark_computed_for(
        &mut self,
        py: Python<'_>,
        flag_handle: Bound<'_, FlagHandle>,
        data: Py<PyAny>,
    ) {
        let flag_1 = flag_handle.borrow().flag_1();
        self.computed_caches
            .insert(flag_1, (flag_handle.clone().unbind(), data));
        self.inner
            .as_ref()
            .unwrap()
            .rel_handle
            .borrow(py)
            .mark_computed_for(py, self.flag_0, flag_handle);
    }

    fn reset_computed_for(
        &self,
        py: Python<'_>,
        flag_handle: Bound<'_, FlagHandle>,
    ) -> PyResult<()> {
        self.inner
            .as_ref()
            .unwrap()
            .rel_handle
            .borrow(py)
            .reset_computed_for(py, self.flag_0, flag_handle)
    }

    fn reset_computed_for_func(&self, py: Python<'_>, func: Py<PyAny>) -> PyResult<()> {
        let flag_handle = func
            .getattr(py, crate::attr_names::CMPT_LAZY_METHOD___FLAG_HANDLE)?
            .extract(py)?;
        self.reset_computed_for(py, flag_handle)
    }

    fn reset_computed_for_list(
        &self,
        py: Python<'_>,
        flag_handles: Vec<Bound<'_, FlagHandle>>,
    ) -> PyResult<()> {
        for flag_handle in flag_handles {
            self.reset_computed_for(py, flag_handle)?;
        }
        Ok(())
    }

    fn reset_computed_for_all(&mut self, py: Python<'_>) -> PyResult<()> {
        let flag_handles = self
            .computed_caches
            .values()
            .map(|(flag_handle, _)| flag_handle.clone_ref(py))
            .collect::<Vec<_>>();
        let inner = self.inner.as_ref().unwrap();
        for flag_handle in flag_handles {
            inner.rel_handle.borrow(py).reset_computed_for(
                py,
                self.flag_0,
                flag_handle.clone_ref(py).into_bound(py),
            )?;
        }
        self.computed_caches.clear();
        Ok(())
    }

    fn __traverse__(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        if let Some(inner) = &self.inner {
            visit.call(&inner.decl_cls)?;
            visit.call(&inner.at_item)?;
            visit.call(&inner.key)?;
            visit.call(&inner.rel_handle)?;
        }
        for (_, cache) in self.computed_caches.values() {
            visit.call(cache)?;
        }
        Ok(())
    }

    fn __clear__(&mut self) -> PyResult<()> {
        self.inner = None;
        self.computed_caches.clear();
        Ok(())
    }
}
