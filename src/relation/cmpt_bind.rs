use std::collections::HashMap;

use super::handle::RelationHandle;
use pyo3::exceptions::PyKeyError;
use pyo3::prelude::*;
use pyo3::types::PyList;
use pyresolver::PyResolver;

use super::registry::FlagHandle;
use super::registry::RelationRegistry;

#[derive(PyResolver)]
#[pyresolver(module = "janim_backend.relation")]
pub struct BindInfo {
    decl_cls: Py<PyAny>,
    at_item: Py<PyAny>,
    key: String,
}

struct Bind {
    info: BindInfo,
    /// The indexized `key`
    flag_0: usize,
    /// The relation handle in `ItemRelation`
    handle: Py<RelationHandle>,
    /// Caches of the computed methods, key is `flag_1` (`usize`)
    computed_caches: HashMap<usize, (Py<FlagHandle>, Py<PyAny>)>,
}

/// The motivation to define this struct in Rust is that
/// the attr access speed in Python is slow,
/// and the methods uses frequently in JAnim,
/// we want to accelerate it.
///
/// In advance of fix the attr access in compile-time, we can speed up a little bit.
#[pyclass(module = "janim_backend.relation", skip_from_py_object)]
pub struct BinderHandle {
    registry: Py<RelationRegistry>,
    bind: Option<Bind>,
}

impl BinderHandle {
    pub fn new(registry: Py<RelationRegistry>) -> Self {
        Self {
            registry,
            bind: None,
        }
    }
}

#[pymethods]
impl BinderHandle {
    /// bind to the item based on the provided `BindInfo`.
    fn bind_to(&mut self, py: Python<'_>, info: Bound<'_, PyBindInfo>) -> PyResult<()> {
        let info = info.borrow_mut().take()?;
        let flag_0 = self.registry.borrow(py).indexize_key(&info.key);
        let handle = info
            .at_item
            .getattr(py, crate::attr_names::ITEM_RELATION__REL_HANDLE)?
            .cast_bound::<RelationHandle>(py)?
            .clone()
            .unbind();

        self.bind = Some(Bind {
            info,
            flag_0,
            handle,
            computed_caches: Default::default(),
        });
        Ok(())
    }

    /// Unbind from the item.
    fn unbind(&mut self) {
        self.bind = None;
    }

    fn is_binded(&self) -> bool {
        self.bind.is_some()
    }

    #[getter]
    fn decl_cls(&self, py: Python<'_>) -> Option<Py<PyAny>> {
        self.bind
            .as_ref()
            .map(|bind| bind.info.decl_cls.clone_ref(py))
    }

    #[getter]
    fn at_item(&self, py: Python<'_>) -> Option<Py<PyAny>> {
        self.bind
            .as_ref()
            .map(|bind| bind.info.at_item.clone_ref(py))
    }

    #[getter]
    fn key(&self) -> Option<String> {
        self.bind.as_ref().map(|bind| bind.info.key.clone())
    }

    fn unwrap_decl_cls(&self, py: Python<'_>) -> Py<PyAny> {
        self.bind.as_ref().unwrap().info.decl_cls.clone_ref(py)
    }

    fn unwrap_at_item(&self, py: Python<'_>) -> Py<PyAny> {
        self.bind.as_ref().unwrap().info.at_item.clone_ref(py)
    }

    fn unwrap_key(&self) -> String {
        self.bind.as_ref().unwrap().info.key.clone()
    }

    /// Get the cached data for method, `None` indicates the cache does not exist.
    ///
    /// Always return `None` if the bind is not set.
    fn get_computed_for(
        &self,
        py: Python<'_>,
        flag_handle: Bound<'_, FlagHandle>,
    ) -> PyResult<Option<&Py<PyAny>>> {
        let Some(bind) = &self.bind else {
            return Ok(None);
        };

        let has_flag = bind
            .handle
            .borrow(py)
            .get_computed_for(py, bind.flag_0, &flag_handle);
        if !has_flag {
            return Ok(None);
        }

        let flag_1 = flag_handle.borrow().flag_1();
        let Some(cache) = bind.computed_caches.get(&flag_1) else {
            return Err(PyKeyError::new_err("Access to cache which is not recorded"));
        };
        Ok(Some(&cache.1))
    }

    /// Set the computed state to `true`, considering the recursion.
    ///
    /// The `data` will be stored, and available at `get_computed_for`.
    ///
    /// Has no effect if the bind is not set.
    fn mark_computed_for(
        &mut self,
        py: Python<'_>,
        flag_handle: Bound<'_, FlagHandle>,
        data: Py<PyAny>,
    ) {
        let Some(bind) = &mut self.bind else {
            return;
        };

        let flag_1 = flag_handle.borrow().flag_1();
        bind.computed_caches
            .insert(flag_1, (flag_handle.clone().unbind(), data));

        bind.handle
            .borrow(py)
            .mark_computed_for(py, bind.flag_0, &flag_handle);
    }

    /// Reset the computed state to `false`, without considering the recursion.
    ///
    /// The data set by `mark_computed_for` will be expired.
    ///
    /// Has no effect if the bind is not set.
    fn reset_computed_for(
        &self,
        py: Python<'_>,
        flag_handle: &Bound<'_, FlagHandle>,
    ) -> PyResult<()> {
        let Some(bind) = &self.bind else {
            return Ok(());
        };
        bind.handle
            .borrow(py)
            .reset_computed_for(py, bind.flag_0, flag_handle)
    }

    /// Reset the computed state of the `func` to `false`, without considering the recursion.
    ///
    /// The data set by `mark_computed_for` will be expired.
    ///
    /// Has no effect if the bind is not set.
    fn reset_computed_for_func(&self, py: Python<'_>, func: Bound<'_, PyAny>) -> PyResult<()> {
        let attr = func.getattr(super::relation::FLAG_HANDLE_NAME)?;
        let flag_handle = attr.cast::<FlagHandle>()?;
        self.reset_computed_for(py, flag_handle)
    }

    /// Reset the computed states in the list to `false`, without considering the recursion.
    ///
    /// The related datas will be expired.
    ///
    /// Has no effect if the bind is not set.
    fn reset_computed_for_list(&self, py: Python<'_>, handles: Bound<'_, PyList>) -> PyResult<()> {
        let Some(bind) = &self.bind else {
            return Ok(());
        };
        bind.handle
            .borrow(py)
            .reset_computed_for_list(py, bind.flag_0, handles)
    }

    /// Reset the previously computed states to `false`, without considering the recursion.
    ///
    /// All datas will be expired.
    ///
    /// Has no effect if the bind is not set.
    fn reset_computed_for_all(&mut self, py: Python<'_>) -> PyResult<()> {
        let Some(bind) = &mut self.bind else {
            return Ok(());
        };
        for (flag_handle, _) in bind.computed_caches.values() {
            bind.handle
                .borrow(py)
                .reset_computed_for(py, bind.flag_0, flag_handle.bind(py))?;
        }
        bind.computed_caches.clear();
        Ok(())
    }
}
