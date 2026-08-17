mod bitset;
mod cache;
mod handle;
mod iter;
mod registry_flags;
mod registry_nodes;

type NodeIndex = usize;

use std::{cell::RefCell, collections::HashMap};

use pyo3::prelude::*;

use bitset::OffsetBitSet;
use handle::RelationHandle;
use registry_flags::FlagHandle;
use registry_nodes::Node;

#[pymodule]
pub mod relation {
    #[pymodule_export]
    use super::RelationRegistry;
    #[pymodule_export]
    use super::handle::RelationHandle;
    #[pymodule_export]
    use super::iter::{RelationBitsetIterator, RelationVecIterator};
    #[pymodule_export]
    use super::registry_flags::FlagHandle;
}

/// Used for create DAG relations between objects
///
/// Assumption 1: The object's lifetime is the same as its `handle`,
/// i.e. `handle` initialize & deconstruct together with the python object
///
/// Example:
/// ```python
/// class Item:
///     def __init__(self):
///         self.__rel_handle = registry.reg_create()
/// ```
///
/// These situations could break the assumption:
/// - Replace `__rel_handle` by another value
/// - Set `__rel_handle` to other variables outside of the related `Item` instance
///
/// Assumption 2: All iterators (`RelationBitsetIterator` & `RelationVecIterator`) are used immedately,
/// i.e. their lifetime is short, not stored for longer use.
///
/// Because `RelationRegistry` may be re-entered from python,
/// so we wrap every member inside `RefCell<>` to provide interior mutability for members,
/// so that we do not conflict on the borrow checker on `RelationRegistry` self
#[pyclass(module = "janim_backend.relation", unsendable, skip_from_py_object)]
struct RelationRegistry {
    offset: usize,
    nodes: RefCell<Vec<Node>>,

    computed_flags: RefCell<HashMap<(usize, usize), OffsetBitSet>>, // TODO: trim invalid node indices
    indexize_mapping: RefCell<HashMap<String, usize>>,
}

#[pymethods]
impl RelationRegistry {
    #[new]
    fn new() -> Self {
        Self {
            offset: 0,
            nodes: Default::default(),
            computed_flags: Default::default(),
            indexize_mapping: Default::default(),
        }
    }

    /// Create a new relation node
    ///
    /// You should make sure that the returned `RelationHandle` has the same lifetime as the `related_obj`
    fn create<'py>(
        slf: Bound<'_, Self>,
        py: Python<'py>,
        related_obj: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, RelationHandle>> {
        let index = slf.borrow().nodes.borrow().len();
        let handle = Py::new(
            py,
            RelationHandle::new(py, slf.clone().unbind(), index, &related_obj)?,
        )?
        .into_bound(py);

        let node = Node::new(&handle)?;
        slf.borrow().nodes.borrow_mut().push(node);

        Ok(handle)
    }

    /// Clean the leading invalid-nodes and the bitsets
    fn cleanup(&mut self, py: Python<'_>) {
        for set in &mut self.computed_flags.borrow_mut().values_mut() {
            set.cleanup();
        }

        let leading = self
            .nodes
            .borrow()
            .iter()
            .take_while(|node| !node.alive(py))
            .count();
        self.nodes.borrow_mut().drain(..leading);
        self.offset += leading;
    }

    /// Create a `FlagHandle`
    fn create_flag(
        &self,
        py: Python<'_>,
        key: &str,
        recurse_up: bool,
        recurse_down: bool,
    ) -> PyResult<Py<FlagHandle>> {
        let flag_handle = FlagHandle::new(self.indexize_key(key), recurse_up, recurse_down);
        Py::new(py, flag_handle)
    }

    /// Indexize a `str` to an corresponding `id`
    fn indexize_key(&self, key: &str) -> usize {
        let mut mapping = self.indexize_mapping.borrow_mut();
        match mapping.get(key) {
            Some(value) => *value,
            None => {
                let id = mapping.len();
                *mapping.entry(key.to_owned()).or_insert(id)
            }
        }
    }
}
