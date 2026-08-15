use pyo3::{prelude::*, types::PyWeakrefReference};

#[inline]
pub fn upgrade_ref<'py>(
    py: Python<'py>,
    wref: &Py<PyWeakrefReference>,
) -> Option<Bound<'py, PyAny>> {
    wref.clone_ref(py).into_bound(py).upgrade()
}
