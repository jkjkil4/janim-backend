use pyo3::exceptions::PyValueError;
use pyo3::{prelude::*, types::PyWeakrefReference};

use crate::{exception::LifetimeError, utils::upgrade_ref};

use super::iter::{RelationBitsetIterator, RelationVecIterator};
use super::{NodeIndex, RelationRegistry};

#[pyclass(module = "janim_backend.relation", weakref, skip_from_py_object)]
pub(super) struct RelationHandle {
    /// Reference to the registry
    registry: Py<RelationRegistry>,
    /// Index in the regsitry
    index: NodeIndex,
    /// The object that we must ensure has the same lifetime with
    related_obj: Py<PyWeakrefReference>,
}

impl RelationHandle {
    pub(super) fn new(
        registry: Py<RelationRegistry>,
        index: NodeIndex,
        related_obj: &Bound<'_, PyAny>,
    ) -> PyResult<Self> {
        Ok(Self {
            registry,
            index,
            related_obj: PyWeakrefReference::new(related_obj)?.unbind(),
        })
    }

    pub(super) fn index(&self) -> NodeIndex {
        self.index
    }

    pub(super) fn get_ref<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        upgrade_ref(py, &self.related_obj)
            .ok_or_else(|| LifetimeError::new_err(t!("`RelationHandle` lifetime mismatch")))
    }

    pub(super) fn index_and_wref(&self) -> (NodeIndex, &Py<PyWeakrefReference>) {
        (self.index, &self.related_obj)
    }

    pub(super) fn index_and_ref<'py>(
        &self,
        py: Python<'py>,
    ) -> PyResult<(NodeIndex, Bound<'py, PyAny>)> {
        Ok((self.index, self.get_ref(py)?))
    }
}

#[pymethods]
impl RelationHandle {
    /// Add child objects
    fn add(
        &self,
        py: Python<'_>,
        children: Vec<Bound<'_, RelationHandle>>,
        prepend: bool,
    ) -> PyResult<()> {
        self.registry
            .borrow_mut(py)
            .add_children_to(py, self, children, prepend)
    }

    /// Insert child objects
    fn insert(
        &self,
        py: Python<'_>,
        index: usize,
        children: Vec<Bound<'_, RelationHandle>>,
    ) -> PyResult<()> {
        self.registry
            .borrow_mut(py)
            .insert_children_to(py, self, index, children)
    }

    /// Remove child objects
    fn remove(&self, py: Python<'_>, children: Vec<Bound<'_, RelationHandle>>) -> PyResult<()> {
        self.registry
            .borrow_mut(py)
            .remove_children_from(self, children)
    }

    /// Reindex children by the `indices`, e.g.
    ///
    /// ```plain
    /// indices = [3, 4, 1, 2]
    /// children => [children[3], children[4], children[1], children[2]]
    /// ```
    fn reindex(&self, py: Python<'_>, indices: Vec<usize>) -> PyResult<()> {
        self.registry
            .borrow_mut(py)
            .reindex_children_of(py, self, indices)
    }

    /// Clear parent objects
    fn clear_parents(&self, py: Python<'_>) -> PyResult<()> {
        self.registry.borrow_mut(py).clear_parents_of(py, self)
    }

    /// Clear child objects
    fn clear_children(&self, py: Python<'_>) -> PyResult<()> {
        self.registry.borrow_mut(py).clear_children_of(self)
    }

    /// Resolve parent objects
    fn parents<'py>(&self, py: Python<'py>) -> Vec<Bound<'py, PyAny>> {
        self.registry
            .borrow(py)
            .node(self.index)
            .resolve_parents(py)
    }

    /// Resolve child objects
    fn children(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        self.registry
            .borrow(py)
            .node(self.index)
            .resolve_children(py)
    }

    /// Resolve ancestor objects (unordered)
    fn walk_ancestor_set(&self, py: Python<'_>) -> PyResult<RelationBitsetIterator> {
        let index = self.index;

        let iter = self.registry.borrow(py).ancestor_set(index)?.into_iter();
        Ok(RelationBitsetIterator {
            registry: self.registry.clone_ref(py),
            iter,
        })
    }

    /// Resolve descendant objects (unordered)
    fn walk_descendant_set(&self, py: Python<'_>) -> PyResult<RelationBitsetIterator> {
        let index = self.index;

        let iter = self.registry.borrow(py).descendant_set(index)?.into_iter();
        Ok(RelationBitsetIterator {
            registry: self.registry.clone_ref(py),
            iter,
        })
    }

    /// Resolve ancestor objects (DFS ordered)
    fn walk_ancestor_dfs(&self, py: Python<'_>) -> PyResult<RelationVecIterator> {
        let index = self.index;

        Ok(RelationVecIterator::new(
            self.registry.clone_ref(py),
            self.registry.borrow(py).ancestor_dfs(index)?,
        ))
    }

    /// Resolve descendant objects (DFS ordered)
    fn walk_descendant_dfs(&self, py: Python<'_>) -> PyResult<RelationVecIterator> {
        let index = self.index;

        Ok(RelationVecIterator::new(
            self.registry.clone_ref(py),
            self.registry.borrow(py).descendant_dfs(index)?,
        ))
    }

    /// Get the count of child objects
    fn len(&self, py: Python<'_>) -> usize {
        self.registry.borrow(py).node(self.index).len()
    }

    /// Check whether `obj` is in the child objects
    fn contains(&self, py: Python<'_>, obj: Bound<'_, PyAny>) -> bool {
        self.registry
            .borrow(py)
            .node(self.index)
            .iter_child_refs()
            .any(|x| x.is(&obj))
    }

    /// Get the index of `obj` in the child objects
    fn index_of(&self, py: Python<'_>, obj: Bound<'_, PyAny>) -> PyResult<usize> {
        self.registry
            .borrow(py)
            .node(self.index)
            .iter_child_refs()
            .position(|x| x.is(&obj))
            .ok_or_else(|| {
                PyValueError::new_err(format!(
                    "{} not in list",
                    obj.repr()
                        .map_or_else(|_| "The object".into(), |v| v.to_string())
                ))
            })
    }

    /// Check for whether the flag is set
    fn has_flag(&self, py: Python<'_>, flag: String) -> bool {
        self.registry.borrow_mut(py).node_has_flag(self.index, flag)
    }

    /// Set the flag state
    fn set_flag(
        &self,
        py: Python<'_>,
        flag: String,
        state: bool,
        recurse_up: bool,
        recurse_down: bool,
    ) -> PyResult<()> {
        self.registry.borrow_mut(py).node_set_flag(
            self.index,
            flag,
            state,
            recurse_up,
            recurse_down,
        )
    }
}
