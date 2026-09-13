use pyo3::{
    PyTraverseError, PyVisit,
    prelude::*,
    types::{PyDict, PyList, PyTuple},
};

use crate::component::attrs_storage::AttrsStorage;

use super::cmpts::{CmptValues, CmptsInstance};

#[pyclass(module = "janim_backend.component", subclass)]
pub struct CmptsStorage {
    pub(crate) cmpts_inst: Option<CmptsInstance>,
}

impl CmptsStorage {
    #[inline]
    fn inst(&self) -> &CmptsInstance {
        self.cmpts_inst.as_ref().unwrap()
    }
}

impl CmptsStorage {
    fn iter_common_components<'a>(
        inst1: &'a CmptsInstance,
        inst2: &'a CmptsInstance,
    ) -> impl Iterator<Item = (&'a usize, &'a CmptValues, &'a CmptValues)> + 'a {
        inst1.cmpts.iter().filter_map(|(id, value)| {
            inst2
                .cmpts
                .get(id)
                .map(|source_value| (id, value, source_value))
        })
    }
}

#[pymethods]
impl CmptsStorage {
    #[new]
    #[pyo3(signature = (*_args, **_kwargs))]
    fn new(_args: &Bound<'_, PyTuple>, _kwargs: Option<&Bound<'_, PyDict>>) -> Self {
        Self { cmpts_inst: None }
    }

    fn _init_cmpts(&mut self, py: Python<'_>, fields: Bound<'_, PyDict>) -> PyResult<()> {
        self.cmpts_inst = Some(CmptsInstance::new(py, fields)?);
        Ok(())
    }

    fn get_component(&self, py: Python<'_>, name: String) -> PyResult<Option<Py<AttrsStorage>>> {
        let result = self
            .inst()
            .cmpts
            .values()
            .find(|&x| x.0 == name)
            .map(|x| x.2.clone_ref(py));
        Ok(result)
    }

    fn get_components<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
        let components = PyList::empty(py);
        for (_, _, cmpt) in self.inst().cmpts.values() {
            components.append(cmpt.clone_ref(py).into_bound(py))?;
        }
        Ok(components)
    }

    fn get_common_components<'py>(
        &self,
        py: Python<'py>,
        other: Bound<'_, Self>,
    ) -> PyResult<Bound<'py, PyList>> {
        let components = PyList::empty(py);
        let other = other.borrow();

        for (_, values, other_values) in Self::iter_common_components(self.inst(), other.inst()) {
            components.append((
                values.0.as_str(),
                values.2.clone_ref(py),
                other_values.2.clone_ref(py),
            ))?;
        }

        Ok(components)
    }

    fn _copy_cmpts_to(&self, py: Python<'_>, target: Bound<'_, Self>) -> PyResult<()> {
        target.borrow_mut().cmpts_inst = Some(self.inst().copy(py)?);
        Ok(())
    }

    fn _become_cmpts_from(&self, py: Python<'_>, source: Bound<'_, Self>) -> PyResult<()> {
        let source = source.borrow();
        for (_, values, source_values) in Self::iter_common_components(self.inst(), source.inst()) {
            let cmpt = &values.2;
            let source_cmpt = &source_values.2;
            cmpt.borrow_mut(py)
                ._become(py, source_cmpt.clone_ref(py).into_bound(py))?;
        }
        Ok(())
    }

    fn _bind_cmpts(&self, py: Python<'_>, callback: Bound<'_, PyAny>) -> PyResult<()> {
        for (key, field, cmpt) in self.inst().cmpts.values() {
            let decl_cls = &field.borrow(py).decl_cls;
            callback.call1((cmpt, decl_cls, key))?;
        }

        Ok(())
    }

    fn _take_cmpts_modified(&self, py: Python<'_>) -> PyResult<bool> {
        let mut flag = false;
        for (_, _, cmpt) in self.inst().cmpts.values() {
            if cmpt.borrow_mut(py)._take_modified() {
                flag = true;
            }
        }
        Ok(flag)
    }

    fn __traverse__(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        if let Some(cmpts_inst) = &self.cmpts_inst {
            for (_, field, cmpt) in cmpts_inst.cmpts.values() {
                visit.call(field)?;
                visit.call(cmpt)?;
            }
        }
        Ok(())
    }

    fn __clear__(&mut self) -> PyResult<()> {
        self.cmpts_inst = None;
        Ok(())
    }
}
