mod flags;
mod nodes;
mod recursive_cache;

use std::fmt::Write;
use std::{cell::RefCell, collections::HashMap};

use pyo3::prelude::*;

pub use flags::FlagHandle;
pub use nodes::{CutType, ResolveResult};

use super::bitset::OffsetBitSet;
use super::handle::RelationHandle;
use nodes::{Node, Nodes};

pub type NodeIndex = usize;

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
pub struct RelationRegistry {
    nodes: RefCell<Nodes>,

    computed_flags: RefCell<HashMap<(usize, usize), OffsetBitSet>>,
    indexize_mapping: RefCell<HashMap<String, usize>>,
}

#[pymethods]
impl RelationRegistry {
    #[new]
    fn new() -> Self {
        Self {
            nodes: RefCell::new(Nodes::new()),
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
        slf.borrow().nodes.borrow_mut().push(|index| {
            let handle = Py::new(
                py,
                RelationHandle::new(py, slf.clone().unbind(), index, &related_obj)?,
            )?
            .into_bound(py);

            let node = Node::new(&handle)?;

            Ok((node, handle))
        })
    }

    /// Cut a new chunk for the nodes registry,
    /// allowing `cleanup` to remove preceding dead chunks.
    fn cut_nodes_chunk(&self, cut_result: CutType) {
        self.nodes.borrow_mut().cut(cut_result);
    }

    /// Clean the leading invalid-nodes and the bitsets
    fn cleanup(&mut self, py: Python<'_>) {
        // Since `OffsetBitset` is small enough,
        // we don't plan to apply the same chunking optimization used by `Nodes` to `computed_flags`.
        for set in &mut self.computed_flags.borrow_mut().values_mut() {
            set.cleanup();
        }

        self.nodes.borrow_mut().cleanup(py);
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

    /// Get statistics string of member data structures,
    /// used for debugging in Python
    fn printable_statistics(&self, py: Python<'_>) -> String {
        let mut s = String::new();

        // self.nodes

        self.nodes.borrow().printable_statistics(py, &mut s);
        writeln!(s).unwrap();

        // self.computed_flags

        let mapping = self.indexize_mapping.borrow();
        let to_key = |index: usize| {
            mapping
                .iter()
                .find(|(_, v)| **v == index)
                .map(|(k, _)| k.clone())
                .unwrap_or_else(|| index.to_string())
        };

        writeln!(s, "Computed flags:").unwrap();
        for ((flag_0, flag_1), set) in self.computed_flags.borrow().iter() {
            let (start, end) = set.range();

            let entry = format!("\"{}.{}\"", to_key(*flag_0), to_key(*flag_1));

            let (range, appendix) = if start == end {
                (String::from("word-range: (Empty)"), String::default())
            } else {
                let bytes = (end - start) * 8;
                (
                    format!("word-range: [{}, {})", start, end),
                    if bytes < 1024 {
                        format!("@ {} Bytes", bytes)
                    } else {
                        format!("@ {:.1} KB", (bytes as f64) / 1024.0)
                    },
                )
            };

            writeln!(s, "- {:<34} {:<10} {}", entry, range, appendix).unwrap();
        }

        s
    }
}
