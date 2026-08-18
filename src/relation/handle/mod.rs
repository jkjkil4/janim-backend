mod iter;

use pyo3::types::PyList;
use pyo3::{prelude::*, types::PyWeakrefReference};

use crate::exception::BorrowMutError;
use crate::{exception::LifetimeError, utils::upgrade_ref};

use super::registry;
use registry::{FlagHandle, RelationRegistry};

use super::NodeIndex;
pub use iter::{RelationBitsetIterator, RelationVecIterator};

#[pyclass(module = "janim_backend.relation", weakref, skip_from_py_object)]
pub(super) struct RelationHandle {
    /// Reference to the registry
    registry: Py<RelationRegistry>,
    /// Index in the regsitry
    index: NodeIndex,

    /// The object that we must ensure has the same lifetime with
    related_obj: Py<PyWeakrefReference>,
    /// We must not store strong reference to python objects in the registry, so we put the reference here
    parents: Py<PyList>,
    /// We must not store strong reference to python objects in the registry, so we put the reference here
    children: Py<PyList>,
}

impl RelationHandle {
    pub(super) fn new(
        py: Python<'_>,
        registry: Py<RelationRegistry>,
        index: NodeIndex,
        related_obj: &Bound<'_, PyAny>,
    ) -> PyResult<Self> {
        Ok(Self {
            registry,
            index,
            related_obj: PyWeakrefReference::new(related_obj)?.unbind(),
            parents: PyList::empty(py).unbind(),
            children: PyList::empty(py).unbind(),
        })
    }

    pub(super) fn index(&self) -> NodeIndex {
        self.index
    }

    pub(super) fn obj_ref<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        upgrade_ref(py, &self.related_obj)
            .ok_or_else(|| LifetimeError::new_err(t!("`RelationHandle` lifetime mismatch")))
    }

    pub(super) fn index_and_ref<'py>(
        &self,
        py: Python<'py>,
    ) -> PyResult<(NodeIndex, Bound<'py, PyAny>)> {
        Ok((self.index, self.obj_ref(py)?))
    }

    /// Used for borrow mutable [RelationRegistry] when modifying children
    fn mut_children<'py>(&'py self, py: Python<'py>) -> PyResult<PyRef<'py, RelationRegistry>> {
        if self.registry.try_borrow_mut(py).is_err() {
            Err(BorrowMutError::new_err(t!(
                "Cannot modify children while a modification is already in progress"
            )))
        } else {
            Ok(self.registry.borrow(py))
        }
    }
}

#[pymethods]
impl RelationHandle {
    /// Get the reference to `parents` list
    pub(super) fn parents_ref<'py>(&self, py: Python<'py>) -> &Bound<'py, PyList> {
        self.parents.bind(py)
    }

    /// Get the reference to `children` list
    pub(super) fn children_ref<'py>(&self, py: Python<'py>) -> &Bound<'py, PyList> {
        self.children.bind(py)
    }

    /// Add child objects
    fn add(
        &self,
        py: Python<'_>,
        new_children: Vec<Bound<'_, RelationHandle>>,
        prepend: bool,
    ) -> PyResult<()> {
        self.mut_children(py)?
            .add_children_to(py, self, new_children, prepend)
    }

    /// Insert child objects
    fn insert(
        &self,
        py: Python<'_>,
        index: usize,
        children: Vec<Bound<'_, RelationHandle>>,
    ) -> PyResult<()> {
        self.mut_children(py)?
            .insert_children_to(py, self, index, children)
    }

    /// Remove child objects
    fn remove(&self, py: Python<'_>, children: Vec<Bound<'_, RelationHandle>>) -> PyResult<()> {
        self.mut_children(py)?
            .remove_children_from(py, self, children)
    }

    /// Manually trigger [RelationRegistry::children_changed]
    fn emit_children_changed(&self, py: Python<'_>) -> PyResult<()> {
        self.mut_children(py)?.children_changed(py, self.index)
    }

    /// Clear parent objects
    fn clear_parents(&self, py: Python<'_>) -> PyResult<()> {
        self.mut_children(py)?.clear_parents_of(py, self)
    }

    /// Clear child objects
    fn clear_children(&self, py: Python<'_>) -> PyResult<()> {
        self.mut_children(py)?.clear_children_of(py, self)
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

    /// Check for whether the flag of `index` is set
    pub(super) fn get_computed_for(
        &self,
        py: Python<'_>,
        flag_0: usize,
        flag_handle: Bound<'_, FlagHandle>,
    ) -> bool {
        self.registry
            .borrow(py)
            .node_get_computed_for(self.index, flag_0, flag_handle)
    }

    /// Set the computed state to `true`, considering the recursion
    pub(super) fn mark_computed_for(
        &self,
        py: Python<'_>,
        flag_0: usize,
        flag_handle: Bound<'_, FlagHandle>,
    ) {
        self.registry
            .borrow(py)
            .node_mark_computed_for(self.index, flag_0, flag_handle);
    }

    /// Reset the computed state to `false`, without considering the recursion
    pub(super) fn reset_computed_for(
        &self,
        py: Python<'_>,
        flag_0: usize,
        flag_handle: Bound<'_, FlagHandle>,
    ) -> PyResult<()> {
        self.registry
            .borrow(py)
            .node_reset_computed_for(self.index, flag_0, flag_handle.borrow())
    }

    /// Reset the computed states in the list to `false`, without considering the recursion
    pub(super) fn reset_computed_for_list(
        &self,
        py: Python<'_>,
        flag_0: usize,
        handles: Bound<'_, PyList>,
    ) -> PyResult<()> {
        let registry = self.registry.borrow(py);
        for any in handles.iter() {
            let flag_handle: PyRef<FlagHandle> = any.extract()?;
            registry.node_reset_computed_for(self.index, flag_0, flag_handle)?;
        }
        Ok(())
    }
}
