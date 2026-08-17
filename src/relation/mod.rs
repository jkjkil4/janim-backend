mod bitset;
mod cache;
mod handle;
mod iter;
mod registry_flags;
mod registry_nodes;

type NodeIndex = usize;

use std::collections::HashMap;

use pyo3::prelude::*;

use bitset::OffsetBitSet;
use handle::RelationHandle;
use registry_nodes::Node;

#[pymodule]
pub mod relation {
    #[pymodule_export]
    use super::RelationRegistry;
    #[pymodule_export]
    use super::handle::RelationHandle;
    #[pymodule_export]
    use super::iter::{RelationBitsetIterator, RelationVecIterator};
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
#[pyclass(module = "janim_backend.relation", unsendable, skip_from_py_object)]
struct RelationRegistry {
    offset: usize,
    nodes: Vec<Node>,
    flags: HashMap<String, OffsetBitSet>,
}

#[pymethods]
impl RelationRegistry {
    #[new]
    fn new() -> Self {
        Self {
            offset: 0,
            nodes: Vec::new(),
            flags: HashMap::new(),
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
        let index = slf.borrow().nodes.len();
        let handle = Py::new(
            py,
            RelationHandle::new(py, slf.clone().unbind(), index, &related_obj)?,
        )?
        .into_bound(py);

        let node = Node::new(&handle)?;
        slf.borrow_mut().nodes.push(node);

        Ok(handle)
    }

    /// Clean the leading invalid-nodes and the bitsets
    fn cleanup(&mut self, py: Python<'_>) {
        for set in &mut self.flags.values_mut() {
            set.cleanup();
        }

        let leading = self.nodes.iter().take_while(|node| !node.alive(py)).count();
        self.nodes.drain(..leading);
        self.offset += leading;
    }
}
