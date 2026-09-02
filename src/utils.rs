use pyo3::{prelude::*, types::PyWeakrefReference};

#[inline]
pub fn upgrade_ref<'py>(
    py: Python<'py>,
    wref: &Py<PyWeakrefReference>,
) -> Option<Bound<'py, PyAny>> {
    wref.clone_ref(py).into_bound(py).upgrade()
}

/// Format the `bytes` into `Bytes` / `KB` / `MB`
pub fn format_bytes(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} Bytes", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", (bytes as f64) / 1024.0)
    } else {
        format!("{:.1} MB", (bytes as f64) / (1024.0 * 1024.0))
    }
}
