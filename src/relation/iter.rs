use std::rc::Rc;

use pyo3::prelude::*;

use super::bitset::OffsetBitSetIter;
use super::{NodeIndex, RelationRegistry};

#[pyclass(module = "janim_backend.relation", unsendable, skip_from_py_object)]
pub(super) struct RelationBitsetIterator {
    pub registry: Py<RelationRegistry>,
    pub iter: OffsetBitSetIter,
}

#[pymethods]
impl RelationBitsetIterator {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__<'py>(mut slf: PyRefMut<Self>, py: Python<'py>) -> Option<Bound<'py, PyAny>> {
        let index = slf.iter.next()?;

        let registry = slf.registry.bind(py).borrow();
        registry.node(index).resolve_self(py)
    }
}

#[pyclass(module = "janim_backend.relation", unsendable, skip_from_py_object)]
pub(super) struct RelationVecIterator {
    registry: Py<RelationRegistry>,
    vec: Rc<Vec<NodeIndex>>,
    current: usize,
}

impl RelationVecIterator {
    pub(super) fn new(registry: Py<RelationRegistry>, vec: Rc<Vec<NodeIndex>>) -> Self {
        Self {
            registry,
            vec,
            current: 0,
        }
    }
}

#[pymethods]
impl RelationVecIterator {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__<'py>(mut slf: PyRefMut<Self>, py: Python<'py>) -> Option<Bound<'py, PyAny>> {
        if slf.current == slf.vec.len() {
            return None;
        }

        let index = slf.vec[slf.current];
        slf.current += 1;

        let registry = slf.registry.bind(py).borrow();
        registry.node(index).resolve_self(py)
    }
}
